struct Struct {}

trait Trait<T> {
    fn method(self) -> T;
}

fn main() {
    let st = Struct {};
    assert_eq(st.method(), 1);
}

#[foo]
comptime fn foo(_: FunctionDefinition) -> Quoted {
    let tr = quote { Trait<Field> }.as_trait_constraint();
    let st = quote { Struct }.as_type();
    quote {
        impl $tr for $st {
            fn method(self) -> Field {
                1
            }
        }
    }
}

