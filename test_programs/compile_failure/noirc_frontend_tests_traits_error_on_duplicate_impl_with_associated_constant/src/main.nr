
        trait Foo {
            let Bar: u32;
        }

        impl Foo for i32 {
            let Bar: u32 = 5;
        }

        impl Foo for i32 {
            let Bar: u32 = 6;
        }

        fn main() {}
    