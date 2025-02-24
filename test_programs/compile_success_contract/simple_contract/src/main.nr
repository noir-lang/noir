contract Foo {
    fn double(x: Field) -> pub Field {
        x * 2
    }
    fn triple(x: Field) -> pub Field {
        x * 3
    }
    fn quadruple(x: Field) -> pub Field {
        x * 4
    }
    // Regression for issue #3344
    #[contract_library_method]
    pub fn foo(x: u8) -> u8 {
        x
    }
}
