contract Foo {
    fn double(x: field) -> pub field {
        x * 2
    }
    fn triple(x: field) -> pub field {
        x * 3
    }
    fn quadruple(x: field) -> pub field {
        x * 4
    }
    // Regression for issue #3344
    #[contract_library_method]
    fn foo(x: u8) -> u8 {
        x
    }
}
