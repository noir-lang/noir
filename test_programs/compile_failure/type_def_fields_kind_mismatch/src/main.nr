// Foo's first generic is `Numeric(u32)`. Calling `fields` with a `Normal`-kinded
// argument (`bool`) in that slot produces a comptime error.
pub struct Foo<let N: u32, T> {
    a: [T; N],
}

fn main() {
    comptime {
        let (foo_def, _) =
            quote { Foo<3, Field> }.as_type().as_data_type().unwrap();

        let bool_t = quote { bool }.as_type();
        let field_t = quote { Field }.as_type();

        let _ = foo_def.fields([bool_t, field_t].as_vector());
    }
}
