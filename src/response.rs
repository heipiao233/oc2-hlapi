use crate::call::ApiCall;
use crate::types::{DeviceDescriptor, MethodDescriptor};
use serde::de::{self, DeserializeOwned};
use serde::Deserialize;
use std::mem::MaybeUninit;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type", content = "data")]
pub enum Response<T: ApiCall> {
    #[serde(alias = "list", alias = "methods", rename = "result")]
    Response(T::Response),
    Error(String),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
pub struct List(pub Vec<DeviceDescriptor>);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
pub struct Methods(pub Vec<MethodDescriptor>);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Return<R>(pub R);

impl<'de, R: DeserializeOwned + 'static> Deserialize<'de> for Return<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Generates a zero-sized type 'from thin air'.
        // When invoking a method in the HLAPI that doesn't have a return value, one might expect
        // the response to look like `{"type": "result", "data": null}`, but in reality, it looks
        // like `{"type": "result"}`, and is missing the data field. With a derived Deserialize,
        // serde_json would complain about a missing `data` field, when it is actually supposed to
        // be missing. This only happens in the case of the void return type, which is represented
        // by zero-sized types (usually `()`) in Rust.
        fn zst<T>() -> T {
            assert!(std::mem::size_of::<T>() == 0, "`T` must be a ZST");

            // SAFETY: The check above ensures that T is a zero-sized type, and thus can be constructed by
            // reading a well-aligned pointer, even if that pointer doesn't point to anything valid.
            #[allow(clippy::uninit_assumed_init)]
            unsafe {
                MaybeUninit::uninit().assume_init()
            }
        }

        let opt: Option<R> = Deserialize::deserialize(deserializer)?;

        match opt {
            Some(r) => Ok(Return(r)),
            None if std::mem::size_of::<R>() == 0 => Ok(Return(zst())),
            // We actually do expect the `data` field if the return type is not actually zero-sized.
            // If there's no `data` field when it was expected, that means something went wrong, and
            // not just that the call didn't return anything.
            None => Err(de::Error::missing_field("data")),
        }
    }
}

impl<T: ApiCall> From<Response<T>> for Result<T::Response, String> {
    fn from(value: Response<T>) -> Self {
        match value {
            Response::Response(t) => Ok(t),
            // This branch should never get executed unless T is serialized to "null" in JSON. This
            // is the case with Option<T> and (), for example.
            Response::Error(err) => Err(err),
        }
    }
}
