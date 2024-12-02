use std::static_assert;

global GLOBAL_ONE: Field = 1;
global GLOBAL_TWO: Field = 2;
global GLOBAL_THREE: Field = GLOBAL_ONE + GLOBAL_TWO;

// contents known at compile time
// length known at compile time
global GLOBAL_ARRAY_PAIR: [Field; 2] = [GLOBAL_ONE, GLOBAL_TWO];
global GLOBAL_SLICE_PAIR: [Field] = &[GLOBAL_ONE, GLOBAL_TWO];

struct Foo {
    field: Field,
    array: [Field; 3],
    slice: [Field],
}

fn main(
    dynamic_one: Field, // == 1
    dynamic_two: Field, // == 2
) {
    // contents unknown at compile time
    // length known at compile time
    let dynamic_array_pair = [dynamic_one, dynamic_two];
    let dynamic_slice_pair = &[dynamic_one, dynamic_two];

    assert_constant(true);
    assert_constant(false);

    assert_constant(2 == 2);
    assert_constant(1 + 1 == 2);

    // assert_constant doesn't check for true
    assert_constant(1 + 1 == 3);

    let local_one = 1;
    let local_two = 2;
    let local_three = local_one + local_two;
    let local_array_pair = [local_one, local_two];
    let local_slice_pair = &[local_one, local_two];

    assert_constant(local_one);
    assert_constant(local_three);
    assert_constant(local_array_pair);
    assert_constant(local_slice_pair);

    assert_constant(GLOBAL_ONE);
    assert_constant(GLOBAL_THREE);
    assert_constant(GLOBAL_ARRAY_PAIR);
    assert_constant(GLOBAL_SLICE_PAIR);

    assert_constant([1, 2, dynamic_one].len() == 4);

    static_assert(dynamic_array_pair.len() == 2, "");
    static_assert(dynamic_slice_pair.len() == 2, "");

    let foo = Foo { field: 0, array: [1, 2, 3], slice: &[] };
    assert_constant(foo);
}
