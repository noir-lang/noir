use std::static_assert;

global GLOBAL_ONE: Field = 1;
global GLOBAL_TWO: Field = 2;
global GLOBAL_THREE: Field = GLOBAL_ONE + GLOBAL_TWO;

// contents known at compile time
// length known at compile time
global GLOBAL_ARRAY_PAIR: [Field; 2] = [GLOBAL_ONE, GLOBAL_TWO];
global GLOBAL_SLICE_PAIR: [Field] = &[GLOBAL_ONE, GLOBAL_TWO];

pub struct Foo {
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

    static_assert(true, "");

    static_assert(1 + 1 == 2, "");

    let x = 2;
    let y = 3;
    let xy = x + y;
    static_assert(xy == 5, "");

    static_assert(3 == GLOBAL_THREE, "");

    static_assert(GLOBAL_ARRAY_PAIR.len() == 2, "");
    static_assert(GLOBAL_SLICE_PAIR.len() == 2, "");

    static_assert(dynamic_array_pair.len() == 2, "");
    static_assert(dynamic_slice_pair.len() == 2, "");

    assert_constant([1, 2, dynamic_one].len() == 4);
    static_assert([1, 2, dynamic_one].len() == 3, "");
}
