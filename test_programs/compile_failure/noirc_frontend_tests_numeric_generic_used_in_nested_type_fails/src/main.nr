
    pub struct Foo<let N: u32> {
        a: Field,
        b: Bar<N>,
    }
    pub struct Bar<let N: u32> {
        inner: N
    }
    