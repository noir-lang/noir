
        trait Foo {
            let Bar: u32;
        }

        impl Foo for i32 {
            let Bar = 5;
        }

        impl Foo for i32 {
            let Bar = 6;
        }

        fn main() {}
    