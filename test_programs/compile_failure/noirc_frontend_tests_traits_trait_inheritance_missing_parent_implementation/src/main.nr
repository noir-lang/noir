
        pub trait Foo {}

        pub trait Bar: Foo {}

        pub struct Struct {}

        impl Bar for Struct {}

        fn main() {
        }
    