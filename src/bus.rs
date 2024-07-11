use crate::call::{ApiCall, ListCall, RpcCall};
use crate::device::RpcDevice;
use crate::error::{Error, Result};
use crate::response::{ListResponse, RpcResponse};
use arrayvec::ArrayVec;
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use serde::Serialize;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind as IoErrorKind, Read, Write};
use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use termios::Termios;

static NEXT_TOKEN: AtomicUsize = AtomicUsize::new(0);

/// The maximum size of one message that can be handled by the HLAPI in bytes. 4 KiB by default.
pub(crate) const MAX_MESSAGE_SIZE: usize = 4096;

/// The operating system's device bus. This is represented by a device file in Linux which acts as a
/// serial console to read and write HLAPI RPC messages.
#[derive(Clone, Debug)]
pub struct DeviceBus(Rc<Inner>);

impl DeviceBus {
    /// Creates a new device bus at the specified path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bus = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.as_ref())?;

        let fd = bus.as_raw_fd();
        let bus_token = Token(NEXT_TOKEN.fetch_add(1, Ordering::Relaxed));

        // Sets options to not echo back the input to the device bus, and immediately applies that
        // change. Without this, writing to the device bus will just hang the applicaton.
        // Taken from https://docs.rs/miku-rpc/0.1.4/src/miku_rpc/bus.rs.html#34-37
        let mut termios = Termios::from_fd(fd)?;
        termios::cfmakeraw(&mut termios);
        termios.c_lflag &= !termios::ECHO;
        termios::tcsetattr(fd, termios::TCSANOW, &termios)?;

        let poll = Poll::new()?;
        poll.registry()
            .register(&mut SourceFd(&fd), bus_token, Interest::READABLE)?;

        Ok(Self(Rc::new(Inner {
            bus,
            events: RefCell::new(Events::with_capacity(16)),
            poll: RefCell::new(poll),
            _not_send_sync: PhantomData,
        })))
    }

    /// Calls an RPC method. A convenience method for writing to the device bus and then reading an
    /// RPC value returned.
    pub fn call<T: ApiCall + Serialize>(&self, call: RpcCall<T>) -> Result<RpcResponse<T>> {
        self.write_message(call)?;
        self.read_message()
    }

    /// Finds a device or module by its RpcDevice identifier.
    pub fn find<D: RpcDevice>(&self) -> Result<Option<D>> {
        self.find_by_name(D::IDENTIFIER)
    }

    /// Finds a device or module by its name.
    pub fn find_by_name<D: RpcDevice>(&self, name: &str) -> Result<Option<D>> {
        let list_result = self.call::<ListCall>(RpcCall::list())?.into();

        let ListResponse(list) = match list_result {
            Ok(response) => response,
            Err(e) => return Err(e.into()),
        };

        let device = list
            .iter()
            .find(|&desc| {
                desc.type_names
                    .iter()
                    .any(|identifier| &**identifier == name)
            })
            .map(|desc| D::new(desc.device_id, self));

        Ok(device)
    }

    /// Writes an RPC message.
    pub fn write_message<T: ApiCall + Serialize>(&self, message: RpcCall<T>) -> Result<()> {
        let mut write_buffer = const { ArrayVec::<_, MAX_MESSAGE_SIZE>::new_const() };

        write_buffer
            .try_push(b'\0')
            .map_err(|_| Error::MessageLengthExceeded)?;

        serde_json::to_writer(&mut write_buffer, &message).map_err(Error::from)?;

        write_buffer
            .try_push(b'\0')
            .map_err(|_| Error::MessageLengthExceeded)?;

        (&self.0.bus)
            .write_all(write_buffer.as_slice())
            .map_err(Error::from)
    }

    /// Reads an RPC message.
    pub fn read_message<T: ApiCall>(&self) -> Result<RpcResponse<T>> {
        let mut read_buffer = const { ArrayVec::<_, MAX_MESSAGE_SIZE>::new_const() };
        let mut total_bytes = 0;

        loop {
            let bytes_read = self.read(&mut read_buffer)?;

            if bytes_read > 0 {
                total_bytes += bytes_read;

                if read_buffer.len() > 1 && read_buffer.last().is_some_and(|&byte| byte == b'\0') {
                    break;
                }
            } else {
                return Err(Error::ReadZero);
            }
        }

        // The message without the null bytes at the start and end.
        let msg_slice = &read_buffer[1..total_bytes - 1];

        let response = serde_json::from_slice(msg_slice).map_err(Error::from)?;

        Ok(response)
    }

    fn read(&self, buf: &mut ArrayVec<u8, MAX_MESSAGE_SIZE>) -> Result<usize> {
        {
            let mut poll = self.0.poll.borrow_mut();
            let mut events = self.0.events.borrow_mut();
            events.clear();

            while let Err(e) = poll.poll(&mut events, None) {
                if e.kind() != IoErrorKind::Interrupted {
                    return Err(e.into());
                }
            }
        }

        loop {
            let result = (&self.0.bus).read(buf);

            match result {
                Ok(n) => break Ok(n),
                Err(e) if e.kind() != IoErrorKind::Interrupted => {
                    break Err(e.into());
                }
                _ => continue,
            }
        }
    }
}

#[derive(Debug)]
struct Inner {
    bus: File,
    poll: RefCell<Poll>,
    events: RefCell<Events>,

    // Ensures that this struct is not Send or Sync
    _not_send_sync: PhantomData<*mut ()>,
}
