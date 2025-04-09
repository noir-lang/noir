
    pub trait MagicNumber {
        fn from_magic_value() -> Self {
            Self::from_value()
        }
        fn from_value() -> Self;
    }

    impl MagicNumber for i32 {
        fn from_value() -> Self {
            0
        }
    }

    impl MagicNumber for i64 {
        fn from_value() -> Self {
            0
        }
    }

    fn main() {}
    