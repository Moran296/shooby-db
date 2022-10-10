#![allow(dead_code)]

use crate::errors::ShoobyError;
use crate::utils::*;

#[derive(Debug)]
pub(crate) enum ShoobyFieldType<'a> {
    Bool(bool),
    Int(i32),
    String(&'a mut [u8]),
    Blob(&'a mut [u8]),
}

pub const PERSISTENT: bool = true;
pub const NON_PERSISTENT: bool = false;

/// ShoobyField
/// This is the fields that are held for each item in the database
#[derive(Debug)]
pub struct ShoobyField {
    name: &'static str,
    data: ShoobyFieldType<'static>,
    range: Option<(i32, i32)>,
    persistent: bool,
    has_changed: bool,
}

impl ShoobyField {
    pub(crate) const fn new(
        name: &'static str,
        data: ShoobyFieldType<'static>,
        range: Option<(i32, i32)>,
        persistent: bool,
    ) -> Self {
        ShoobyField {
            name,
            data,
            range,
            persistent,
            has_changed: false,
        }
    }

    // ===================GETTERS==================

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

}