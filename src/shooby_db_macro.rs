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
        $name.set_num($value).unwrap();
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

            // ================= EMPTY STRUCT AS DEFAULTS =================

            pub struct EmptyObserver;
            impl ShoobyObserver for EmptyObserver {
                type ID = ID;

                fn update(&self, _field: &ShoobyField<Self::ID>) {}
            }

            pub struct EmptyStorage;
            impl ShoobyStorage for EmptyStorage {
                type ID = ID;
                fn save_raw(&self, _id: Self::ID, _data: &[u8]) -> Result<(), ShoobyError> {
                    Ok(())
                }
                fn load_raw(&mut self, _id: Self::ID, _data: &mut [u8]) -> Result<bool, ShoobyError> {
                    Ok(false)
                }
            }

            // ================= HELPER FUNCTIONS FOR TAKING DB =================
            ///helper function for taking DB with empty Storage and Observer. used mostly for testing
            pub fn take_db_with_empty_observer_and_storage() -> DB<EmptyObserver, EmptyStorage> {
                DB::take(None, None)
            }
            ///helper function for taiking DB with empty Storage. Used if no persistence is required.
            pub fn take_with_observer_only<T: ShoobyObserver<ID=ID>>(observer: Option<T>) -> DB<T, EmptyStorage> {
                DB::take(observer, None)
            }
            ///helper function for taiking DB with empty observer. Used if not oberserver is required.
            pub fn take_with_storage_only<T: ShoobyStorage<ID=ID>>(storage: Option<T>) -> DB<EmptyObserver, T> {
                DB::take(None, storage)
            }

            // ================= CONFIGURATION DB =================

            /// This is the main struct that holds the database
            /// A new struct will be generated for call to macro shooby_db!
            pub struct DB<Observer: ShoobyObserver<ID=ID> = EmptyObserver, Storage: ShoobyStorage<ID=ID> = EmptyStorage> {
                items: &'static mut [ShoobyField<ID>],
                observer: Option<Observer>,
                storage: Option<Storage>,
                // RWLock for the array / wrapper of the array
            }

            impl<Observer: ShoobyObserver<ID=ID>, Storage: ShoobyStorage<ID=ID>> DB<Observer, Storage> {
                /// takes the database from the static memory to the DB struct.
                /// paramters:
                ///    observer: an optional observer that will be notified on every change
                ///   storage: an optional storage that will be used to save and load the data to persistent storage
                /// returns: the DB struct
                ///
                /// if the DB is already taken, this function will panic
                /// if the take is called like this
                ///     `let db: NAME::FB = NAME::DB::take(None, None)`, the DB will be taken with empty observer and storage
                /// If one does not need observer or storage, the following functions can be used:
                ///     `let db: NAME::FB = NAME::take_db_with_empty_observer_and_storage()`
                ///     `let db: NAME::FB = NAME::take_with_observer_only(Some(observer))`
                ///    `let db: NAME::FB = NAME::take_with_storage_only(Some(storage))`
                pub fn take(observer: Option<Observer>, storage: Option<Storage>) -> Self {

                    // make sure we call this function only one time!
                    // TODO: limit the AtomicBool to only if feature std is enabled
                    static TAKEN: AtomicBool = AtomicBool::new(false);
                    let taken = TAKEN.fetch_or(true, Ordering::Relaxed);
                    if taken {
                        panic!("DB already taken");
                    }

                    //alloc all static data for strings and blobs
                    $( _shooby_static_alloc!($name, $var, $default, $range); )*

                    // creates the array of fields
                    static mut ITEMS: &'static mut [ShoobyField<ID>] = &mut [
                        $(_shooby_create_cfgs!($name, $var, $default, $range, $persistent), ) *
                    ];

                    // creates the DB struct with all data supplied
                    let mut s = Self {
                        items: unsafe { ITEMS },
                        observer,
                        storage,
                    };

                    // reset all fields to default
                    s.reset_to_default();

                    // reset all changed flags for all fields
                    s.reset_changed_flags();

                    s
                }

                /// This function reset all values to default and saves them to persistent storage if needed
                /// The function will NOT notify observer on changes
                pub fn factory_reset(&mut self) -> Result<(), ShoobyError> {
                    self.reset_to_default();
                    self.save_to_storage()
                }

                /// Get the DB name as string reference
                pub fn name(&self) -> &str {
                    stringify!($DB_NAME)
                }

                /// Get the DB array of fields to read from
                pub fn reader(&self) -> &[ShoobyField<ID>] {
                    self.items
                }

                /// Get the DB array of fields to write to inside a closure
                pub fn write_with<F>(&mut self, f: F) where F: FnOnce(&mut [ShoobyField<ID>]) {
                    f(self.items);
                    self.save_to_storage();
                    self.update_observer();
                }

                /// Perform an operation on the observer object if it exists
                pub fn observer<F>(&mut self, f: F) where F: FnOnce(Option<&mut Observer>) {
                    f(self.observer.as_mut());
                }

                //============PRIVATE FUNCTIONS================

                pub fn reset_to_default(&mut self) {
                    $(
                        let data = &mut self.items[ID::$name];
                        _shooby_assign_value!(data, $var, $default, $range);
                    )*
                }


                fn update_observer(&mut self) {
                    if let Some(observer) = self.observer.as_ref() {
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
                            loaded &= item.load(storage)?;
                        }
                    }

                    Ok(loaded)
                }

                fn save_to_storage(&self) -> Result<(), ShoobyError> {
                    if let Some(storage) = self.storage.as_ref() {
                        for item in self.items.as_ref() {
                            if (item.has_changed) {
                                item.save(storage)?;
                            }
                        }
                    }

                    Ok(())
                }

                fn reset_changed_flags(&mut self) {
                    for item in self.items.as_mut() {
                        item.has_changed = false;
                    }
                }

            }
        }
            // ================= CONFIGURATION DB END =================
        };

    }
