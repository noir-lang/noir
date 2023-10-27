struct Foo<N> {
    x : N,
}

// Expected: constant provided when a type was expected
type FooField = Foo<12>;