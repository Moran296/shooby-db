use crate::{errors::ShoobyError, ShoobyField};
use std::fmt::Debug;

pub trait ShoobyObserver {
    type ID;

    fn update(&self, field: &ShoobyField<Self::ID>);
}

pub trait ShoobyStorage {
    type ID;
    // TODO: add a generic way to give errors..

    fn save_raw(&self, id: Self::ID, data: &[u8]) -> Result<(), ShoobyError>;
    fn load_raw(&mut self, id: Self::ID, data: &mut [u8]) -> Result<bool, ShoobyError>;
}
