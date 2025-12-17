#[foo([1, 2, 3], "hello", f"world")]
comptime fn foo(_: FunctionDefinition, list: [u8], _str1: CtString, _str2: CtString) -> Quoted {
    quote {
        fn bar() -> [u8] {
            $list
        }
    }
}

// Make sure coercion also works in varargs
#[baz("hello", "world")]
#[varargs]
comptime fn baz(_f: FunctionDefinition, args: [CtString]) {
    assert_eq(args.len(), 2);
}

fn main() {
    assert_eq(bar(), &[1, 2, 3])
}
