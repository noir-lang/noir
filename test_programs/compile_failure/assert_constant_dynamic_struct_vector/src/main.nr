struct Foo {
    field: Field,
    array: [Field; 3],
    vector: [Field],
}

fn main(
    dynamic_one: Field, // == 1
) {
    let foo_dynamic_vector = Foo { field: 0, array: [1, 2, 3], vector: #[dynamic_one] };
    assert_constant(foo_dynamic_vector);
}
