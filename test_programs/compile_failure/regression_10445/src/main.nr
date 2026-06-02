#[foo]
comptime fn foo(_: FunctionDefinition) -> Quoted {
    let t = quote { [i32; 3] }.as_type();
    quote {
        struct Foo<$t> {
            x: $t,
        }
    }
}

fn main() {
    let _ = Foo::<i32> { x: [1, 2, 3] };
}
