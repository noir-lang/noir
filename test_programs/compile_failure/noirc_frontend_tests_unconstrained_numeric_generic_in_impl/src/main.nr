
        pub struct Foo {}

        impl<let N: u32> Foo {}

        fn main() {
            let _ = Foo {};
        }
        