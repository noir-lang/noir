pub struct WeirdStruct<T, U> {
    a: T,
    b: U,
}

#[mangle_fn]
pub fn my_fn() -> [u8; 3] {
    [0; 3]
}

comptime fn mangle_fn(f: FunctionDefinition) {
    let return_type = f.return_type();

    // This relies on how types are displayed
    let generics = f"Field,{return_type}".quoted_contents();
    let new_return_type = quote { WeirdStruct<$generics>}.as_type();

    let new_body = quote {
        {
            WeirdStruct { a: 1, b: [0;3] }
        }
    }
        .as_expr()
        .unwrap();

    f.set_return_type(new_return_type);
    f.set_body(new_body);
}

fn main() {}
