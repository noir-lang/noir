contract Foo {
    #[abi(foo)]
    pub global foo: Field = 42;

    #[abi(bar)]
    pub struct Bar {
        inner: Field
    }
}
