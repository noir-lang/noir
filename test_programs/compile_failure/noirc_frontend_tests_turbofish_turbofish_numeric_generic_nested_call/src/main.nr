
    struct Foo<T> {
        a: T
    }

    impl<T> Foo<T> {
        pub fn static_method<let N: u32>() -> [u8; N] {
            [0; N]
        }

        pub fn impl_method<let N: u32>(self) -> [T; N] {
            [self.a; N]
        }
    }

    fn bar<let N: u32>() -> [u8; N] {
        let _ = Foo::static_method::<N>();
        let x: Foo<u8> = Foo { a: 0 };
        x.impl_method::<N>()
    }

    global M: u32 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    