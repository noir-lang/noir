
    pub struct NestedNumeric<let N: u32> {
        a: Field,
        b: InnerNumeric<N>
    }
    pub struct InnerNumeric<let N: u32> {
        inner: [u64; N],
    }

    fn main() { }
    