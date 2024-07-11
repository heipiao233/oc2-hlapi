use crate::response;
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
pub struct Call<T>(T);

impl<T: ApiCall + Serialize> Call<T> {
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

impl<T: ApiCall + Serialize> Serialize for Call<T> {
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

impl Call<List> {
    pub fn list() -> Self {
        Self(List)
    }
}

impl Call<Methods> {
    pub fn methods(device_id: uuid::Uuid) -> Self {
        Self(Methods { device_id })
    }
}

impl<'a, R: DeserializeOwned + 'static> Call<Invoke<'a, R>> {
    pub fn invoke(
        device_id: uuid::Uuid,
        method_name: &'a str,
        parameters: &'a [&'a dyn ErasedSerialize],
    ) -> Self {
        Self(Invoke {
            device_id,
            name: method_name,
            parameters,
            _ret_value: PhantomData,
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize)]
pub struct List;

impl ApiCall for List {
    const KIND: &'static str = "list";
    type Response = response::List;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct Methods {
    #[serde(rename = "deviceId")]
    pub device_id: uuid::Uuid,
}

impl ApiCall for Methods {
    const KIND: &'static str = "methods";
    type Response = response::Methods;
}

#[derive(Copy, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoke<'a, R> {
    pub device_id: uuid::Uuid,
    pub name: &'a str,
    pub parameters: &'a [&'a dyn ErasedSerialize],
    // Unused, but this is required for the API for RpcResponse to be the way I'd like it.
    _ret_value: PhantomData<fn() -> R>,
}

impl<R: DeserializeOwned + 'static> ApiCall for Invoke<'_, R> {
    const KIND: &'static str = "invoke";
    type Response = response::Return<R>;
}

mod sealed {
    use serde::de::DeserializeOwned;

    use super::{Invoke, List, Methods};

    pub trait Sealed {}

    impl Sealed for List {}
    impl Sealed for Methods {}
    impl<R: DeserializeOwned + 'static> Sealed for Invoke<'_, R> {}
}
