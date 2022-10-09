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
                {NUM_2, UInt, 40, Some((10, 100)), NON_PERSISTENT},
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
        assert_eq!(reader[TESTER::ID::NUM].get_int::<f64>(), 15.0);
        assert_eq!(reader[TESTER::ID::NUM_2].get_uint::<u32>(), 40);
        assert_eq!(reader[TESTER::ID::STRING].get_string(), "default");
        assert_eq!(reader[TESTER::ID::BOOLEAN].get_bool(), false);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().a, 5);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().b, 9);
    }

    #[test]
    fn can_be_changed() {
        create_db_instance!(TESTER);
        let mut db = TESTER::DB::take();

        db.write_with(|writer| {
            writer[TESTER::ID::NUM].set_int::<i8>(17i8);
            writer[TESTER::ID::NUM_2].set_uint(17u32);
            writer[TESTER::ID::STRING].set_string("I LOVE JENNY");
            writer[TESTER::ID::BOOLEAN].set_bool(true);
            writer[TESTER::ID::BLOB].set_blob(&A { a: 80, b: 90 });
        });

        let reader = db.reader();

        let g: i8 = reader[TESTER::ID::NUM].get_int();
        assert_eq!(reader[TESTER::ID::NUM].get_int::<i8>(), 17);
        assert_eq!(reader[TESTER::ID::NUM_2].get_uint::<u32>(), 17);
        assert_eq!(reader[TESTER::ID::STRING].get_string(), "I LOVE JENNY");
        assert_eq!(reader[TESTER::ID::BOOLEAN].get_bool(), true);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().a, 80);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().b, 90);
    }
}
