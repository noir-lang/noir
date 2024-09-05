fn main() {}

#[my_comptime_fn]
struct MyType<A, B, C> {
    field1: [A; 10],
    field2: (B, C),
}

#[mutate_struct_fields]
struct I32AndField {
    z: i8,
}

comptime fn my_comptime_fn(typ: StructDefinition) {
    let _ = typ.as_type();
    assert_eq(typ.generics().len(), 3);
    assert_eq(typ.fields().len(), 2);
}

comptime fn mutate_struct_fields(s: StructDefinition) {
    let fields = &[
        (quote[x], quote[i32].as_type()),
        (quote[y], quote[Field].as_type())
    ];
    s.set_fields(fields);
}
