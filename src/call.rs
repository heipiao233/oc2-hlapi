use crate::response::{InvokeResponse, ListResponse, MethodsResponse};
use erased_serde::Serialize as ErasedSerialize;
use serde::de::DeserializeOwned;
use serde::ser::SerializeStruct;
use serde::Serialize;
use std::marker::PhantomData;

pub trait ApiCall: sealed::Sealed {
    const KIND: &str;
    type Response: DeserializeOwned + 'static;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct RpcCall<T>(T);

impl<T: ApiCall + Serialize> RpcCall<T> {
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

impl<T: ApiCall + Serialize> Serialize for RpcCall<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let zst = std::mem::size_of::<T>() == 0;
        let len = if zst { 1 } else { 2 };

        let mut s = serializer.serialize_struct("RpcCall", len)?;

        s.serialize_field("type", T::KIND)?;

        if !zst {
            s.serialize_field("data", &self.0)?;
        }

        s.end()
    }
}

impl RpcCall<ListCall> {
    pub fn list() -> Self {
        Self(ListCall)
    }
}

impl RpcCall<MethodsCall> {
    pub fn methods(device_id: uuid::Uuid) -> Self {
        Self(MethodsCall { device_id })
    }
}

impl<'a, R: DeserializeOwned + 'static> RpcCall<InvokeCall<'a, R>> {
    pub fn invoke(
        device_id: uuid::Uuid,
        method_name: &'a str,
        parameters: &'a [&'a dyn ErasedSerialize],
    ) -> Self {
        Self(InvokeCall {
            device_id,
            name: method_name,
            parameters,
            _ret_value: PhantomData,
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RpcCallKind {
    List,
    Methods,
    Invoke,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize)]
pub struct ListCall;

impl ApiCall for ListCall {
    const KIND: &'static str = "list";
    type Response = ListResponse;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct MethodsCall {
    #[serde(rename = "deviceId")]
    pub device_id: uuid::Uuid,
}

impl ApiCall for MethodsCall {
    const KIND: &'static str = "methods";
    type Response = MethodsResponse;
}

#[derive(Copy, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvokeCall<'a, R> {
    pub device_id: uuid::Uuid,
    pub name: &'a str,
    pub parameters: &'a [&'a dyn ErasedSerialize],
    // Unused, but this is required for the API for RpcResponse to be the way I'd like it.
    _ret_value: PhantomData<fn() -> R>,
}

impl<R: DeserializeOwned + 'static> ApiCall for InvokeCall<'_, R> {
    const KIND: &'static str = "invoke";
    type Response = InvokeResponse<R>;
}

mod sealed {
    use serde::de::DeserializeOwned;

    use super::{InvokeCall, ListCall, MethodsCall};

    pub trait Sealed {}

    impl Sealed for ListCall {}
    impl Sealed for MethodsCall {}
    impl<R: DeserializeOwned + 'static> Sealed for InvokeCall<'_, R> {}
}
