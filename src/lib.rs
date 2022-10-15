pub mod errors;
pub mod multi_observers;
pub mod shooby_field;
pub mod traits;
mod utils;

#[macro_use]
mod shooby_db_macro;

pub(crate) use shooby_field::*;

pub use errors::*;
pub use multi_observers::MultiObserver;
pub use traits::*;

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
#[allow(dead_code)]
struct A {
    a: u32,
    b: u32,
}

#[cfg(test)]
mod tests {
    #![allow(unaligned_references)]
    use super::*;

    macro_rules! create_db_instance {
        ($name:ident) => {
            shooby_db!($name =>
                {NUM, Int, 15, Some((10, 100)), NON_PERSISTENT},
                {STRING, String, "default", 24, NON_PERSISTENT},
                {BOOLEAN, Bool, false, None, PERSISTENT},
                {BLOB, Blob, A {a: 5, b: 9} , std::mem::size_of::<A>(), PERSISTENT},
            );
        };
    }

    #[test]
    fn it_works() {
        create_db_instance!(TESTER);
        let db = TESTER::take_db_with_empty_observer_and_storage();
        let reader = db.reader();
        assert_eq!(reader[TESTER::ID::NUM].get_int::<f64>().unwrap(), 15.0);
        assert_eq!(reader[TESTER::ID::STRING].get_string().unwrap(), "default");
        assert_eq!(reader[TESTER::ID::BOOLEAN].get_bool().unwrap(), false);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().unwrap().a, 5);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().unwrap().b, 9);
    }

    #[test]
    #[should_panic]
    fn it_panics() {
        create_db_instance!(TESTER);
        let db_1 = TESTER::take_db_with_empty_observer_and_storage();
        let db_2 = TESTER::take_db_with_empty_observer_and_storage();
    }

    #[test]
    fn can_be_changed() {
        create_db_instance!(TESTER);
        let mut db = TESTER::take_db_with_empty_observer_and_storage();

        db.write_with(|writer| {
            assert_eq!(writer[TESTER::ID::NUM].set_int::<i8>(17i8).unwrap(), 15);
            assert_eq!(writer[TESTER::ID::BOOLEAN].set_bool(true).unwrap(), false);
            writer[TESTER::ID::STRING]
                .set_string("I LOVE JENNY")
                .unwrap();
            writer[TESTER::ID::BLOB]
                .set_blob(&A { a: 80, b: 90 })
                .unwrap();
        });

        let reader = db.reader();
        assert_eq!(reader[TESTER::ID::NUM].get_int::<i8>().unwrap(), 17);
        assert_eq!(
            reader[TESTER::ID::STRING].get_string().unwrap(),
            "I LOVE JENNY"
        );
        assert_eq!(reader[TESTER::ID::BOOLEAN].get_bool().unwrap(), true);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().unwrap().a, 80);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().unwrap().b, 90);
    }

    #[test]
    fn name_as_str() {
        create_db_instance!(TESTER);
        let mut db = TESTER::take_db_with_empty_observer_and_storage();

        assert_eq!(db.name(), "TESTER");
        assert_eq!(db.reader()[TESTER::ID::NUM].name(), "TESTER::ID::NUM");
        assert_eq!(db.reader()[TESTER::ID::BLOB].name(), "TESTER::ID::BLOB");
        assert_eq!(db.reader()[TESTER::ID::STRING].name(), "TESTER::ID::STRING");
        assert_eq!(
            db.reader()[TESTER::ID::BOOLEAN].name(),
            "TESTER::ID::BOOLEAN"
        );
    }

    #[test]
    fn observer() {
        use core::cell::Cell;
        struct TestObserver<'a> {
            num_changed: &'a Cell<bool>,
        }

        impl<'a> ShoobyObserver for TestObserver<'a> {
            type ID = TESTER::ID;
            fn update(&self, field: &ShoobyField<Self::ID>) {
                if field.id() == TESTER::ID::NUM {
                    self.num_changed.set(true);
                }
            }
        }

        let boolcell = Cell::new(false);

        let observer = TestObserver {
            num_changed: &boolcell,
        };

        create_db_instance!(TESTER);
        let mut db: TESTER::DB<TestObserver> = TESTER::take_with_observer_only(Some(observer));

        db.write_with(|writer| {
            writer[TESTER::ID::NUM].set_int(90).unwrap();
        });

        assert_eq!(boolcell.get(), true);
    }

    #[test]
    fn multi_observers_before_creating_db() {
        use core::cell::Cell;

        struct TestObserver<'a> {
            num_changed: &'a Cell<bool>,
        }

        impl<'a> ShoobyObserver for TestObserver<'a> {
            type ID = TESTER::ID;
            fn update(&self, field: &ShoobyField<Self::ID>) {
                if field.id() == TESTER::ID::NUM {
                    self.num_changed.set(true);
                }
            }
        }

        let bools: [Cell<bool>; 3] = [Cell::new(false), Cell::new(false), Cell::new(false)];
        let observer_0 = TestObserver {
            num_changed: &bools[0],
        };
        let observer_1 = TestObserver {
            num_changed: &bools[1],
        };
        let observer_2 = TestObserver {
            num_changed: &bools[2],
        };

        let mut multi_observer: MultiObserver<TESTER::ID, TestObserver, 3> = MultiObserver::new();
        multi_observer.add(observer_0).unwrap();
        multi_observer.add(observer_1).unwrap();
        multi_observer.add(observer_2).unwrap();

        create_db_instance!(TESTER);
        let mut db: TESTER::DB<MultiObserver<_, _, 3>> =
            TESTER::take_with_observer_only(Some(multi_observer));

        db.write_with(|writer| {
            writer[TESTER::ID::NUM].set_int(90).unwrap();
        });

        assert_eq!(bools[0].get(), true);
        assert_eq!(bools[1].get(), true);
        assert_eq!(bools[2].get(), true);
    }

    #[test]
    fn multi_observers_after_creating_db() {
        use core::cell::Cell;

        struct TestObserver<'a> {
            num_changed: &'a Cell<bool>,
        }

        impl<'a> ShoobyObserver for TestObserver<'a> {
            type ID = TESTER::ID;
            fn update(&self, field: &ShoobyField<Self::ID>) {
                if field.id() == TESTER::ID::NUM {
                    self.num_changed.set(true);
                }
            }
        }

        let bools: [Cell<bool>; 3] = [Cell::new(false), Cell::new(false), Cell::new(false)];
        let observer_0 = TestObserver {
            num_changed: &bools[0],
        };
        let observer_1 = TestObserver {
            num_changed: &bools[1],
        };
        let observer_2 = TestObserver {
            num_changed: &bools[2],
        };

        let multi_observer: MultiObserver<TESTER::ID, TestObserver, 3> = MultiObserver::new();
        create_db_instance!(TESTER);
        let mut db: TESTER::DB<MultiObserver<_, _, 3>, TESTER::EmptyStorage> =
            TESTER::DB::take(Some(multi_observer), None);

        db.observer(|observer| {
            if let Some(multi_observer) = observer {
                multi_observer.add(observer_0).unwrap();
                multi_observer.add(observer_1).unwrap();
                multi_observer.add(observer_2).unwrap();
            }
        });

        db.write_with(|writer| {
            writer[TESTER::ID::NUM].set_int(90).unwrap();
        });

        assert_eq!(bools[0].get(), true);
        assert_eq!(bools[1].get(), true);
        assert_eq!(bools[2].get(), true);
    }

    #[test]
    fn factory_reset() {
        create_db_instance!(TESTER);
        let mut db = TESTER::take_db_with_empty_observer_and_storage();

        //changing all values
        db.write_with(|writer| {
            assert_eq!(writer[TESTER::ID::NUM].set_int::<i8>(17i8).unwrap(), 15);
            assert_eq!(writer[TESTER::ID::BOOLEAN].set_bool(true).unwrap(), false);
            writer[TESTER::ID::STRING]
                .set_string("I LOVE JENNY")
                .unwrap();
            writer[TESTER::ID::BLOB]
                .set_blob(&A { a: 80, b: 90 })
                .unwrap();
        });

        //perform factory reset and read again all valued
        db.factory_reset().unwrap();
        let reader = db.reader();
        assert_eq!(reader[TESTER::ID::NUM].get_int::<i8>().unwrap(), 15);
        assert_eq!(reader[TESTER::ID::STRING].get_string().unwrap(), "default");
        assert_eq!(reader[TESTER::ID::BOOLEAN].get_bool().unwrap(), false);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().unwrap().a, 5);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().unwrap().b, 9);
    }
}
