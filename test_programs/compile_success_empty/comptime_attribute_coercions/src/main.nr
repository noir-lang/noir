#[foo([1, 2, 3])]
comptime fn foo(_: FunctionDefinition, slice: [u8]) -> Quoted {
    quote {
        fn bar() -> [u8] {
            $slice
        }
    }
}

fn main() {
    assert_eq(bar(), &[1, 2, 3])
}
