use std::meta::unquote;

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
    let fields = f.parameters().map(|(name, typ)| quote { $name: $typ, }).join(quote {});

    quote {
        struct Foo { $fields }
    impl Foo {
            fn assert_equal(self) {
                assert_eq(self.x as u32, self.y);
            }
        }
    }
}

// The following code proves that `Bar::<i32>` can be unquoted
// into a constructor position without losing its generic types.

pub struct Bar<T> {}

pub fn bar() {
    let _ = comptime {
        let x = quote { Bar::<i32> };
        let q = quote { $x {} };
        unquote!(q)
    };
}
