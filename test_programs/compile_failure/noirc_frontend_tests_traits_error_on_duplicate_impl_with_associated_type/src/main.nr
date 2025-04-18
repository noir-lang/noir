
        trait Foo {
            type Bar;
        }

        impl Foo for i32 {
            type Bar = u32;
        }

        impl Foo for i32 {
            type Bar = u8;
        }

        fn main() {}
    