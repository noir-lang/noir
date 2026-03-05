#[foo]
comptime fn foo(_: FunctionDefinition) -> Quoted {
    let t = quote { [i32; 3] }.as_type();
    quote {
        struct Bar<T> {
            x: T,
        }

        struct Foo {
            // See https://github.com/noir-lang/noir/pull/11410/changes#r2747886398
            x: Bar<$t>,
        }
    }
}

fn main() {
    let _ = Foo { x: Bar { x: [1, 2, 3] } };
}
