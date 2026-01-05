struct Foo {
    field: Field,
    array: [Field; 3],
    vector: [Field],
}

fn main(
    dynamic_one: Field, // == 1
) {
    let foo_dynamic = Foo { field: dynamic_one, array: [1, 2, 3], vector: #[] };
    assert_constant(foo_dynamic);
}
