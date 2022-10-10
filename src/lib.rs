pub mod errors;
pub mod shooby_field;
mod utils;

#[macro_use]
mod shooby_db_macro;

pub(crate) use shooby_field::*;

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
        let db = TESTER::DB::take();
        let reader = db.reader();
        assert_eq!(reader[TESTER::ID::NUM].get_int::<f64>().unwrap(), 15.0);
        assert_eq!(reader[TESTER::ID::STRING].get_string().unwrap(), "default");
        assert_eq!(reader[TESTER::ID::BOOLEAN].get_bool().unwrap(), false);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().unwrap().a, 5);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().unwrap().b, 9);
    }

    #[test]
    fn can_be_changed() {
        create_db_instance!(TESTER);
        let mut db = TESTER::DB::take();

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
}
