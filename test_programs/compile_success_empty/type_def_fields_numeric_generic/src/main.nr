// Numeric values are valid in type-level positions, so `Quoted::as_type` accepts a numeric
// literal and yields a constant type. That constant can then fill `Foo`'s `Numeric(u32)`
// generic slot when applying `TypeDefinition::fields`.
pub struct Foo<let N: u32, T> {
    a: [T; N],
}

fn main() {
    comptime {
        let (foo_def, _) =
            quote { Foo<3, Field> }.as_type().as_data_type().unwrap();

        // Ensure `4` can be parsed as a type
        let four_t = quote { 4 }.as_type();
        let field_t = quote { Field }.as_type();

        let fields = foo_def.fields([four_t, field_t].as_vector());
        assert_eq(fields.len(), 1);
        assert_eq(fields[0].1, quote { [Field; 4] }.as_type());
    }
}
