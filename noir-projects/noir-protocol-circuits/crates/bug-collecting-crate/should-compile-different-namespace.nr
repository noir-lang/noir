struct Foo<N> {
    x : N,
}

type FooField = Foo<Field>;
type Foou32 = Foo<u32>;

// These two impls should work since the `hash1` implementations
// are in different "namespaces".
impl FooField {
    fn hash1(self) {}
}
impl Foou32 {
    fn hash1(self) {}
}