fn main() {}

#[my_comptime_fn]
struct MyType<A, B, C> {
    field1: [A; 10],
    field2: (B, C),
}

comptime fn my_comptime_fn(typ: StructDefinition) {
    let _ = typ.as_type();
    assert_eq(typ.generics().len(), 3);
    assert_eq(typ.fields().len(), 2);
}
