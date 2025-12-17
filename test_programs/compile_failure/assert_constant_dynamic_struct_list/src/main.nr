struct Foo {
    field: Field,
    array: [Field; 3],
    list: [Field],
}

fn main(
    dynamic_one: Field, // == 1
) {
    let foo_dynamic_list = Foo { field: 0, array: [1, 2, 3], list: &[dynamic_one] };
    assert_constant(foo_dynamic_list);
}
