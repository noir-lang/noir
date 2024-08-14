struct Foo {
    field: Field,
    array: [Field; 3],
    slice: [Field],
}

fn main(
    dynamic_one: Field, // == 1
) {
    let foo_dynamic_slice = Foo { field: 0, array: [1, 2, 3], slice: &[dynamic_one] };
    assert_constant(foo_dynamic_slice);
}
