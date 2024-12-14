fn main() {
    let foo = Foo { x: 4, y: 4 };
    foo.assert_equal();
}

#[output_struct]
fn foo(x: Field, y: u32) -> u32 {
    x as u32 + y
}

// Given a function, wrap its parameters in a struct definition
comptime fn output_struct(f: FunctionDefinition) -> Quoted {
    let fields = f
        .parameters()
        .map(|param: (Quoted, Type)| {
            let name = param.0;
            let typ = param.1;
            quote { $name: $typ, }
        })
        .join(quote {});

    quote {
        struct Foo { $fields }
    impl Foo {
            fn assert_equal(self) {
                assert_eq(self.x as u32, self.y);
            }
        }
    }
}
