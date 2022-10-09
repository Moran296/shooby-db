#![allow(dead_code)]

use crate::utils::*;

#[derive(Debug)]
pub(crate) enum ShoobyFieldType<'a> {
    Bool(bool),
    Int(i32),
    UInt(u32),
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

    //======================SETTERS======================
    pub fn set_int(&mut self, new_val: i32) -> i32 {
        if let ShoobyFieldType::Int(ref mut data) = self.data {
            let old_value = *data;
            if let Some((min, max)) = self.range {
                if new_val < min || new_val > max {
                    println!("value {} is out of range {} - {}", new_val, min, max);
                    return old_value;
                }
            }

            if *data != new_val {
                *data = new_val;
                self.has_changed = true;
            }
            old_value
        } else {
            panic!("{} type is not int!", self.name);
        }
    }

    pub fn set_uint(&mut self, new_val: u32) -> u32 {
        if let ShoobyFieldType::UInt(ref mut data) = self.data {
            let old_value = *data;
            if let Some((min, max)) = self.range {
                if new_val < min as u32 || new_val > max as u32 {
                    println!("value {} is out of range {} - {}", new_val, min, max);
                    return old_value;
                }
            }

            if *data != new_val {
                *data = new_val;
                self.has_changed = true;
            }
            old_value
        } else {
            panic!("{} type is not int!", self.name);
        }
    }

    pub fn set_bool(&mut self, new_val: bool) -> bool {
        if let ShoobyFieldType::Bool(ref mut data) = self.data {
            let old_value = *data;
            if *data != new_val {
                *data = new_val;
                self.has_changed = true;
            }
            old_value
        } else {
            panic!("{} type is not bool!", self.name);
        }
    }

    pub fn set_string(&mut self, new_str: &str) {
        if let ShoobyFieldType::String(ref mut data) = self.data {
            assert!(data.len() >= new_str.len());
            if *data != new_str.as_bytes() {
                data[0..new_str.len()].copy_from_slice(new_str.as_bytes());
                data[new_str.len()..].fill(0);
                self.has_changed = true;
            }
        } else {
            panic!("{} type is not string!", self.name);
        }
    }

    pub fn set_blob<T: Sized>(&mut self, new_blob: &T) {
        if let ShoobyFieldType::Blob(ref mut data) = self.data {
            assert!(data.len() == std::mem::size_of::<T>());
            let new_blob_slice = unsafe { any_as_u8_slice(new_blob) };
            if *data != new_blob_slice {
                data.copy_from_slice(new_blob_slice);
                self.has_changed = true;
            }
        } else {
            panic!("{} type is not blob!", self.name);
        }
    }

    // ===================GETTERS==================

    pub fn get_int(&self) -> i32 {
        if let ShoobyFieldType::Int(val) = self.data {
            val
        } else {
            panic!("{} type is not int!", self.name);
        }
    }

    pub fn get_uint(&self) -> u32 {
        if let ShoobyFieldType::UInt(val) = self.data {
            val
        } else {
            panic!("{} type is not int!", self.name);
        }
    }

    pub fn get_bool(&self) -> bool {
        if let ShoobyFieldType::Bool(val) = self.data {
            val
        } else {
            panic!("{} type is not bool!", self.name);
        }
    }

    pub fn get_string(&self) -> &str {
        if let ShoobyFieldType::String(ref data) = self.data {
            //TODO: return error if invalid string
            str_from_u8_nul_utf8(data).unwrap()
        } else {
            panic!("{} type is not string!", self.name);
        }
    }

    pub fn get_blob<T: Sized>(&self) -> &T {
        if let ShoobyFieldType::Blob(ref data) = self.data {
            assert!(data.len() == std::mem::size_of::<T>());
            unsafe { u8_slice_as_any(data) }
        } else {
            panic!("{} type is not blob!", self.name);
        }
    }
}
