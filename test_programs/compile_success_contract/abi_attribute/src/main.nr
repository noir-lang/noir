contract Foo {
    #[abi(foo)]
    global foo: Field = 42;

    #[abi(bar)]
    struct Bar {
        inner: Field
    }
}