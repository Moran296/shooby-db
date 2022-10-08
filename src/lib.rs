#![allow(dead_code)]

// ======== TEST MOD ==========================

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

unsafe fn u8_slice_as_any<T: Sized>(p: &[u8]) -> &T {
    &*(p.as_ptr() as *const T)
}

macro_rules! _shooby_static_alloc {
    ($name:ident, String, $default:expr, $range:expr) => {
        static mut $name: [u8; $range] = [0; $range];
    };

    ($name:ident, Blob, $default:expr, $range:expr) => {
        static mut $name: [u8; $range] = [0; $range];
    };

    ($name:ident, $t:ident, $default:expr, $range:expr) => {};
}

macro_rules! _shooby_create_cfgs {
    ($name:ident, String, $default:expr, $range:expr, $persistent:path) => {

        ShoobyDbField {
            name: stringify!($name),
            // safety: this happens in take function, that can only happen once
            data: CfgType::String(unsafe { &mut $name }), 
            range: None,
            persistent: $persistent,
            has_changed: false,
        }
    };

    ($name:ident, Blob, $default:expr, $range:expr, $persistent:path) => {

        ShoobyDbField {
            name: stringify!($name),
            // safety: this happens in take function, that can only happen once
            data: CfgType::Blob(unsafe { &mut $name as &mut [u8; $range] }), 
            range: None,
            persistent: $persistent,
            has_changed: false,
        }
    };

    ($name:ident, $var:ident, $default:literal, $range:expr, $persistent:path) => {
        ShoobyDbField {
            name: stringify!($name),
            data: CfgType::$var($default),
            range: $range,
            persistent: $persistent,
            has_changed: false,
        }
    };
}

macro_rules! _shooby_assign_value {
    ($name:ident, Bool, $value:expr, $range:expr) => {
        $name.set_bool($value);
    };
    ($name:ident, Int, $value:expr, $range:expr) => {
        $name.set_number($value);
    };
    ($name:ident, String, $value:expr, $range:expr) => {
        $name.set_string($value);
    };
    ($name:ident, Blob, $value:expr, $range:expr) => {
        $name.set_blob(&$value);
    };

}

macro_rules! shooby_db {
    ($DB_NAME:ident => $({$name:ident, $var:ident, $default:expr, $range:expr, $persistent:path},)+ ) => {

        #[allow(non_camel_case_types, non_snake_case)]
        mod $DB_NAME  {

            use std::stringify;
            use super::*;
            use std::sync::atomic::AtomicBool;
            use std::sync::atomic::Ordering;

            // =============== CONFIGURATION ID =================

            pub enum ID {
                $($name,)*
            }

            /* TODO: might this be better to impl for DB instead of the array itself?
               Or for the return value for wrapper of reader lock?

               I will want to expose reader and writer different later, and just iterating the inner array is BAD!!
            */
            impl std::ops::Index <ID> for [crate::ShoobyDbField] {
                type Output = ShoobyDbField;

                fn index(&self, index: ID) -> &Self::Output {
                    & self[index as usize]
                }
            }

            impl std::ops::IndexMut <ID> for [crate::ShoobyDbField] {
                fn index_mut(&mut self, index: ID) -> &mut Self::Output {
                    &mut  self[index as usize ]
                }
            }

            // ================= CONFIGURATION ID END =================

            // ================= CONFIGURATION DB =================

            pub struct DB {
                items: &'static mut [crate::ShoobyDbField],
                // RWLock for the array / wrapper of the array
                // observer manager
                // persistent storage
            }

            impl DB {
                pub fn take(/*TODO: reset or load from memory*/) -> Self {

                    // make sure we call this function only one time!
                    // TODO: this works only if we have atomic bool, which maybe in no_std is not possible - check it
                    static TAKEN: AtomicBool = AtomicBool::new(false);
                    let taken = TAKEN.fetch_or(true, Ordering::Relaxed);
                    if taken {
                        panic!("DB already taken");
                    }

                    //alloc all static data for strings and maybe blobs later on
                    $( _shooby_static_alloc!($name, $var, $default, $range); )*

                    static mut ITEMS: &'static mut [crate::ShoobyDbField] = &mut [
                        $(_shooby_create_cfgs!($name, $var, $default, $range, $persistent), ) *
                    ];

                    let mut s = Self {
                        items: unsafe { ITEMS },
                    };

                    s.reset_to_default();
                    s
                }

                fn reset_to_default(&mut self) {
                    $(
                        let data = &mut self.items[ID::$name];
                        _shooby_assign_value!(data, $var, $default, $range);
                    )*
                }

                fn init(&mut self) {
                    // load from persistent storage
                }

                //TODO, give this in a RWlock as reader... maybe a wrapper for the array?
                pub fn reader<'a>(&'a self) -> &'a [crate::ShoobyDbField] {
                    self.items
                }

                pub fn write_with<F>(&mut self, f: F) where F: FnOnce(&mut [crate::ShoobyDbField]) {
                    // TODO writer lock
                    f(self.items)
                    // TODO write to persistent storage
                    // TODO release the lock and update observers
                }

            }
        }
            // ================= CONFIGURATION DB END =================
        };

    }

#[derive(Debug)]
enum CfgType<'a> {
    Bool(bool),
    Int(i32),
    String(&'a mut [u8]),
    Blob(&'a mut [u8]),
}

const PERSISTENT: bool = true;
const NON_PERSISTENT: bool = false;
/// ShoobyDbField
/// This is the fields that are held for each item in the database
#[derive(Debug)]
pub struct ShoobyDbField {
    name: &'static str,
    data: CfgType<'static>,
    range: Option<(i32, i32)>,
    persistent: bool,
    has_changed: bool,
}

impl ShoobyDbField {
    //======================SETTERS======================
    pub fn set_number(&mut self, new_val: i32) -> i32 {
        if let CfgType::Int(ref mut data) = self.data {
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

    pub fn set_bool(&mut self, new_val: bool) -> bool {
        if let CfgType::Bool(ref mut data) = self.data {
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
        if let CfgType::String(ref mut data) = self.data {
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
        if let CfgType::Blob(ref mut data) = self.data {
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

    pub fn get_number(&self) -> i32 {
        if let CfgType::Int(val) = self.data {
            val
        } else {
            panic!("{} type is not int!", self.name);
        }
    }
    
    pub fn get_bool(&self) -> bool {
        if let CfgType::Bool(val) = self.data {
            val
        } else {
            panic!("{} type is not bool!", self.name);
        }
    }

    pub fn get_string(&self) -> &str {
        if let CfgType::String(ref data) = self.data {
            unsafe { std::str::from_utf8_unchecked(data) }
        } else {
            panic!("{} type is not string!", self.name);
        }
    }

    pub fn get_blob<T: Sized>(&self) -> &T {
        if let CfgType::Blob(ref data) = self.data {
            assert!(data.len() == std::mem::size_of::<T>());
            unsafe { u8_slice_as_any(data) }
        } else {
            panic!("{} type is not blob!", self.name);
        }
    }

}

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
struct A {
    a: u32,
    b: u32,
}

shooby_db!(TESTER =>
    {FOO, Int, 15, Some((10, 20)), NON_PERSISTENT},
    {FAR, String, "default Something", 24, NON_PERSISTENT},
    {FAZ, Bool, false, None, PERSISTENT},
    {FOODDB, Blob, A {a: 5, b: 9} , std::mem::size_of::<A>(), PERSISTENT},
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut db = TESTER::DB::take();
        {
            let reader = db.reader();
            println!("number in FOO: {}", reader[TESTER::ID::FOO].get_number());
            println!("string in FAR: {}", reader[TESTER::ID::FAR ].get_string());
            println!("blob in FAODOSAD: {:?}", reader[TESTER::ID::FOODDB ].get_blob::<A>());
        }

        db.write_with(|writer| {
            writer[TESTER::ID::FOO ].set_number(17);
            writer[TESTER::ID::FAR ].set_string("I LOVE JENNY");
            writer[TESTER::ID::FOODDB ].set_blob(&A{a: 80, b: 90});

        });

        {
            let reader = db.reader();
            println!("number in FOO: {}", reader[TESTER::ID::FOO ].get_number());
            println!("string in FAR: {}", reader[TESTER::ID::FAR ].get_string());
            println!("blob in FAODOSAD: {:?}", reader[TESTER::ID::FOODDB ].get_blob::<A>());
        }

    }
}
