use std::marker::PhantomData;
use crate::asset::Asset;

#[derive(Clone, PartialEq)]
pub struct Handle<A> 
where
    A: Asset
{
    pub id: HandleId,
    marker: PhantomData<A>
}

#[derive(Hash, PartialEq, PartialOrd, Eq, Clone)]
pub enum HandleId {
    Id(u64),
}

impl HandleId {
    pub fn random() -> Self {
        HandleId::Id(29481)
    }

    pub fn default() -> Self {
        HandleId::Id(0)
    }

    pub fn new() {

    }
}

impl<T: Asset> From<HandleId> for Handle<T> {
    fn from(value: HandleId) -> Self {
        Self {
            id: value,
            marker: PhantomData
        }
    }
}

impl<T: Asset> From<Handle<T>> for HandleId {
    fn from(value: Handle<T>) -> Self {
        value.id
    }
}