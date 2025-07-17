
    pub trait MagicNumber {
        fn from_magic_value() -> Self {
            Self::from_value()
        }
        fn from_value() -> Self;
    }

    pub struct Foo {}

    impl MagicNumber for Foo {
        fn from_value() -> Self {
            Self {}
        }
    }

    pub struct Bar {}

    impl MagicNumber for Bar {
        fn from_value() -> Self {
            Self {}
        }
    }

    fn main() {}
    