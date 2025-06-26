#[foo([1, 2, 3], "hello", f"world")]
comptime fn foo(_: FunctionDefinition, slice: [u8], _str1: CtString, _str2: CtString) -> Quoted {
    quote {
        fn bar() -> [u8] {
            $slice
        }
    }
}

fn main() {
    assert_eq(bar(), &[1, 2, 3])
}
