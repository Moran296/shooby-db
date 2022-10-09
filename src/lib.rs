pub mod errors;
pub mod shooby_field;
mod utils;

#[macro_use]
mod shooby_db_macro;

pub(crate) use shooby_field::*;

#[derive(Debug, Copy, Clone)]
struct A {
    a: u32,
    b: u32,
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! create_db_instance {
        () => {
            shooby_db!(TESTER =>
                {NUM, Int, 15, Some((10, 100)), NON_PERSISTENT},
                {STRING, String, "default", 24, NON_PERSISTENT},
                {BOOLEAN, Bool, false, None, PERSISTENT},
                {BLOB, Blob, A {a: 5, b: 9} , std::mem::size_of::<A>(), PERSISTENT},
            );
        };
    }

    #[test]
    fn it_works() {

        create_db_instance!();
        let mut db = TESTER::DB::take();
            let reader = db.reader();
            assert_eq!(reader[TESTER::ID::NUM].get_number(), 15);
            assert_eq!(reader[TESTER::ID::STRING].get_string(), "default");
            assert_eq!(reader[TESTER::ID::BOOLEAN].get_bool(), false);
            assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().a, 5);
            assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().b, 9);
    }

    #[test]
    fn can_be_changed() {

        create_db_instance!();
        let mut db = TESTER::DB::take();

        db.write_with(|writer| {
            writer[TESTER::ID::NUM ].set_number(17);
            writer[TESTER::ID::STRING ].set_string("I LOVE JENNY");
            writer[TESTER::ID::BOOLEAN ].set_bool(true);
            writer[TESTER::ID::BLOB ].set_blob(&A{a: 80, b: 90});

        });

        let reader = db.reader();
        assert_eq!(reader[TESTER::ID::NUM].get_number(), 17);
        assert_eq!(reader[TESTER::ID::STRING].get_string(), "I LOVE JENNY");
        assert_eq!(reader[TESTER::ID::BOOLEAN].get_bool(), true);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().a, 80);
        assert_eq!(reader[TESTER::ID::BLOB].get_blob::<A>().b, 90);
    }

}
