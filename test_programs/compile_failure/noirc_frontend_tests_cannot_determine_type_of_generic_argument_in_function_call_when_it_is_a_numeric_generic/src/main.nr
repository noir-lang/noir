
    struct Foo<let N: u32> {
        array: [Field; N],
    }

    impl<let N: u32> Foo<N> {
        fn new() -> Self {
            Self { array: [0; N] }
        }
    }

    fn foo<let N: u32>() -> Foo<N> {
        Foo::new()
    }

    fn main() {
        let _ = foo();
    }
    