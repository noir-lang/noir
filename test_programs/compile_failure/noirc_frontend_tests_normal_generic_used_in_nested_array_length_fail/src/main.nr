
    pub struct Foo<N> {
        a: Field,
        b: Bar<N>,
    }
    pub struct Bar<let N: u32> {
        inner: [Field; N]
    }
    