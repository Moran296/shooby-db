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
        ShoobyField::new(
            ID::$name,
            // safety: this happens in take function, that can only happen once
            ShoobyFieldType::String(unsafe { &mut $name }),
            None,
            $persistent,
        )
    };

    ($name:ident, Blob, $default:expr, $range:expr, $persistent:path) => {
        ShoobyField::new(
            ID::$name,
            // safety: this happens in take function, that can only happen once
            ShoobyFieldType::Blob(unsafe { &mut $name as &mut [u8; $range] }),
            None,
            $persistent,
        )
    };

    ($name:ident, $var:ident, $default:literal, $range:expr, $persistent:path) => {
        ShoobyField::new(
            ID::$name,
            ShoobyFieldType::$var($default),
            $range,
            $persistent,
        )
    };
}

macro_rules! _shooby_assign_value {
    ($name:ident, Bool, $value:expr, $range:expr) => {
        $name.set_bool($value).unwrap();
    };
    ($name:ident, Int, $value:expr, $range:expr) => {
        $name.set_int($value).unwrap();
    };
    ($name:ident, String, $value:expr, $range:expr) => {
        $name.set_string($value).unwrap();
    };
    ($name:ident, Blob, $value:expr, $range:expr) => {
        $name.set_blob(&$value).unwrap();
    };
}

/// This is the main macro that creates the Database and the fields defined by the user
#[macro_export]
macro_rules! shooby_db {
    ($DB_NAME:ident => $({$name:ident, $var:ident, $default:expr, $range:expr, $persistent:path},)+ ) => {

        #[allow(non_camel_case_types, non_snake_case)]
        mod $DB_NAME  {

            use super::*;
            use std::sync::atomic::AtomicBool;
            use std::sync::atomic::Ordering;
            use std::fmt::{Formatter, Display, Result as FmtResult};

            // =============== CONFIGURATION ID ====================================
            // This is the ID of the configuration to index the db by
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum ID {
                $($name,)*
                FIELD_NUM
            }

            static _ID_AS_STR: [&str; ID::FIELD_NUM as usize] = [
                $(
                    concat!(stringify!($DB_NAME), "::ID::", stringify!($name)),
                )*
            ];

            impl AsRef<str> for ID {
                fn as_ref(&self) -> &str {
                    _ID_AS_STR[*self as usize]
                }
            }

            impl Display for ID {
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {

                    match self {
                        $(ID::$name => write!(f, "{}", _ID_AS_STR[*self as usize]),)*
                        ID::FIELD_NUM => write!(f, "FIELD_NUM"),
                    }
                }
            }

            impl std::ops::Index <ID> for [ShoobyField<ID>] {
                type Output = ShoobyField<ID>;

                fn index(&self, index: ID) -> &Self::Output {
                    & self[index as usize]
                }
            }

            impl std::ops::IndexMut <ID> for [ShoobyField<ID>] {
                fn index_mut(&mut self, index: ID) -> &mut Self::Output {
                    &mut  self[index as usize ]
                }
            }

            // ================= CONFIGURATION ID END =================

            // ================= CONFIGURATION DB =================

            pub struct DB<'a> {
                items: &'static mut [ShoobyField<ID>],
                observer: Option<&'a dyn ShoobyObserver<ID = ID>>,
                storage: Option<&'a mut dyn ShoobyStorage<ID = ID>>,
                // RWLock for the array / wrapper of the array
            }

            impl<'a> DB<'a> {
                pub (crate) fn take(/*TODO: reset or load from memory as param*/) -> Self {

                    // make sure we call this function only one time!
                    // TODO: limit the AtomicBool to only if feature std is enabled
                    static TAKEN: AtomicBool = AtomicBool::new(false);
                    let taken = TAKEN.fetch_or(true, Ordering::Relaxed);
                    if taken {
                        panic!("DB already taken");
                    }

                    //alloc all static data for strings and blobs
                    $( _shooby_static_alloc!($name, $var, $default, $range); )*

                    static mut ITEMS: &'static mut [ShoobyField<ID>] = &mut [
                        $(_shooby_create_cfgs!($name, $var, $default, $range, $persistent), ) *
                    ];

                    let mut s = Self {
                        items: unsafe { ITEMS },
                        observer: None,
                        storage: None,
                    };

                    s.reset_to_default();
                    s
                }

                pub fn set_observer(&mut self, observer: &'a dyn ShoobyObserver<ID = ID>) {
                    self.observer = Some(observer);
                }
                pub fn set_storage(&mut self, storage: &'a mut dyn ShoobyStorage<ID = ID>) {
                    self.storage = Some(storage);
                }

                pub (crate) fn reset_to_default(&mut self) {
                    $(
                        let data = &mut self.items[ID::$name];
                        _shooby_assign_value!(data, $var, $default, $range);
                    )*
                }

                pub fn name(&self) -> &str {
                    stringify!($DB_NAME)
                }

                //TODO, give this in a RWlock as reader... maybe a wrapper for the array?
                pub fn reader(&'a self) -> &'a [ShoobyField<ID>] {
                    self.items
                }

                pub fn write_with<F>(&mut self, f: F) where F: FnOnce(&mut [ShoobyField<ID>]) {
                    f(self.items);
                    self.save_to_storage();
                    self.update_observers();
                }

                fn update_observers(&mut self) {
                    if let Some(observer) = self.observer {
                        for item in self.items.as_mut() {
                            if (item.has_changed) {
                                observer.update(item);
                                item.has_changed = false;
                            }
                        }
                    }
                }

                fn load_from_storage(&mut self) -> Result<bool, ShoobyError> {
                    let mut loaded = false;

                    if let Some(storage) = self.storage.as_mut() {
                        for item in self.items.as_mut() {
                            loaded &= item.load(*storage)?;
                        }
                    }

                    Ok(loaded)
                }

                fn save_to_storage(&self) -> Result<(), ShoobyError> {
                    if let Some(storage) = self.storage.as_ref() {
                        for item in self.items.as_ref() {
                            if (item.has_changed) {
                                item.save(*storage)?;
                            }
                        }
                    }

                    Ok(())
                }

            }
        }
            // ================= CONFIGURATION DB END =================
        };

    }
