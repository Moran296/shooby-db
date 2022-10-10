#![allow(dead_code)]

use crate::utils::*;
use crate::{errors::ShoobyError, ShoobyStorage};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub(crate) enum ShoobyFieldType {
    Bool(bool),
    Int(i32),
    String(&'static mut [u8]),
    Blob(&'static mut [u8]),
}

impl Display for ShoobyFieldType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ShoobyFieldType::Bool(data) => write!(f, "Bool({})", data),
            ShoobyFieldType::Int(data) => write!(f, "Int({})", data),
            ShoobyFieldType::Blob(data) => write!(f, "Blob of size: {})", data.len()),
            ShoobyFieldType::String(data) => match str_from_u8_nul_utf8(data) {
                Ok(data) => write!(f, "String({})", data),
                Err(err) => write!(f, "String error({:?})", err),
            },
        }
    }
}

pub const PERSISTENT: bool = true;
pub const NON_PERSISTENT: bool = false;

/// ShoobyField
/// This is the fields that are held for each item in the database
#[derive(Debug)]
pub struct ShoobyField<ID> {
    id: ID,
    data: ShoobyFieldType,
    range: Option<(i32, i32)>,
    pub(crate) persistent: bool,
    pub(crate) has_changed: bool,
}

impl<ID: AsRef<str> + Copy> ShoobyField<ID> {
    pub(crate) const fn new(
        id: ID,
        data: ShoobyFieldType,
        range: Option<(i32, i32)>,
        persistent: bool,
    ) -> Self {
        ShoobyField {
            id,
            data,
            range,
            persistent,
            has_changed: false,
        }
    }

    // ===================GETTERS==================

    pub fn id(&self) -> ID {
        self.id
    }

    pub fn name(&self) -> &str {
        self.id.as_ref()
    }

    pub fn get_int<T: TryFrom<i32>>(&self) -> Result<T, ShoobyError> {
        if let ShoobyFieldType::Int(val) = self.data {
            val.try_into()
                .map_err(|_| ShoobyError::InvalidTypeConversion)
        } else {
            Err(ShoobyError::InvalidType)
        }
    }

    pub fn get_bool(&self) -> Result<bool, ShoobyError> {
        if let ShoobyFieldType::Bool(val) = self.data {
            Ok(val)
        } else {
            Err(ShoobyError::InvalidType)
        }
    }

    pub fn get_string(&self) -> Result<&str, ShoobyError> {
        if let ShoobyFieldType::String(ref data) = self.data {
            str_from_u8_nul_utf8(data).map_err(|_| ShoobyError::InvalidTypeConversion)
        } else {
            Err(ShoobyError::InvalidType)
        }
    }

    pub fn get_blob<T: Sized>(&self) -> Result<&T, ShoobyError> {
        if let ShoobyFieldType::Blob(ref data) = self.data {
            assert!(data.len() == std::mem::size_of::<T>());
            Ok(unsafe { u8_slice_as_any(data) })
        } else {
            Err(ShoobyError::InvalidType)
        }
    }

    //======================SETTERS======================
    pub fn set_int<T: TryInto<i32>>(&mut self, new_val: T) -> Result<i32, ShoobyError> {
        let value: i32 = new_val
            .try_into()
            .map_err(|_| ShoobyError::InvalidTypeConversion)?;

        if let ShoobyFieldType::Int(ref mut data) = self.data {
            let old_value = *data;
            if let Some((min, max)) = self.range {
                if value < min || value > max {
                    println!("value {} is out of bounds range {} - {}", value, min, max);
                    return Err(ShoobyError::OutOfBounds);
                }
            }

            if *data != value {
                *data = value;
                self.has_changed = true;
            }

            Ok(old_value)
        } else {
            Err(ShoobyError::InvalidType)
        }
    }

    pub fn set_bool(&mut self, new_val: bool) -> Result<bool, ShoobyError> {
        if let ShoobyFieldType::Bool(ref mut data) = self.data {
            let old_value = *data;
            if *data != new_val {
                *data = new_val;
                self.has_changed = true;
            }
            Ok(old_value)
        } else {
            Err(ShoobyError::InvalidType)
        }
    }

    pub fn set_string(&mut self, new_str: &str) -> Result<(), ShoobyError> {
        if let ShoobyFieldType::String(ref mut data) = self.data {
            if data.len() < new_str.len() {
                return Err(ShoobyError::OutOfBounds);
            }

            let old_str =
                str_from_u8_nul_utf8(data).unwrap_or_else(|_| panic!("Invalid UTF-8 as string"));

            if old_str != new_str {
                data[0..new_str.len()].copy_from_slice(new_str.as_bytes());
                data[new_str.len()..].fill(0);
                self.has_changed = true;
            }
            Ok(())
        } else {
            Err(ShoobyError::InvalidType)
        }
    }

    pub fn set_blob<T: Sized>(&mut self, new_blob: &T) -> Result<(), ShoobyError> {
        if let ShoobyFieldType::Blob(ref mut data) = self.data {
            assert!(data.len() == std::mem::size_of::<T>());
            let new_blob_slice = unsafe { any_as_u8_slice(new_blob) };
            if *data != new_blob_slice {
                data.copy_from_slice(new_blob_slice);
                self.has_changed = true;
            }

            Ok(())
        } else {
            Err(ShoobyError::InvalidType)
        }
    }

    //===============================PERSISTENCE===============================

    pub(crate) fn save<Storage: ShoobyStorage<ID = ID> + ?Sized>(&self, storage: &Storage) -> Result<(), ShoobyError> {
        if !self.persistent {
            return Ok(());
        }

        let data = match &self.data {
            ShoobyFieldType::Int(ref val) => unsafe { any_as_u8_slice(val) },
            ShoobyFieldType::Bool(ref val) => unsafe { any_as_u8_slice(val)},
            ShoobyFieldType::String(data) => data,
            ShoobyFieldType::Blob(data) => data,
        };

        storage.save_raw(self.id, data)
    }

    pub(crate) fn load<Storage: ShoobyStorage<ID = ID> + ?Sized>(&mut self, storage: &mut Storage) -> Result<bool, ShoobyError> {
        if !self.persistent {
            return Ok(false);
        }

        let res = match &mut self.data {
            ShoobyFieldType::Int(ref mut val) => {
                let mut data = [0; std::mem::size_of::<i32>()];
                let loaded = storage.load_raw(self.id, &mut data)?;
                if loaded {
                    *val = unsafe { *u8_slice_as_any(&data) };
                    true
                } else {
                    false
                }
            },
            ShoobyFieldType::Bool(ref mut val) => {
                let mut data = [0; std::mem::size_of::<bool>()];
                let loaded = storage.load_raw(self.id, &mut data)?;
                if loaded {
                    *val = unsafe { *u8_slice_as_any(&data) };
                    true
                } else {
                    false
                }
            }
            ShoobyFieldType::String(data) | ShoobyFieldType::Blob(data)  => {
                let loaded = storage.load_raw(self.id, *data)?;
                if loaded {
                    true
                } else {
                    false
                }
            }

        };

        Ok(res)
    }
}
