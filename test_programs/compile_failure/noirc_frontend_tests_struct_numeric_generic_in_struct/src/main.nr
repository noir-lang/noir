pub struct Foo {
    inner: u64,
}

pub struct Bar<let N: Foo> {}
