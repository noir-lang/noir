    
fn static_assert<N>(predicate: bool, message: str<N>) {
    assert_constant(predicate);
    assert(predicate, message);
}

global GLOBAL_ONE = 1;
global GLOBAL_TWO = 2;
global GLOBAL_THREE = GLOBAL_ONE + GLOBAL_TWO;

// contents known at compile time
// length known at compile time
global GLOBAL_ARRAY_PAIR = [GLOBAL_ONE, GLOBAL_TWO];
global GLOBAL_SLICE_PAIR = &[GLOBAL_ONE, GLOBAL_TWO];

struct Foo {
    field: Field,
    array: [Field; 3],
    slice: [Field],
}

fn main(dynamic_one: Field, dynamic_two: Field) {

    // contents unknown at compile time
    // length known at compile time
    let dynamic_array_pair = [dynamic_one, dynamic_two];
    let dynamic_slice_pair = &[dynamic_one, dynamic_two];

    // length unknown at compile time
    let mut dynamic_built_slice_pair = &[];
    if dynamic_one == 1 {
        dynamic_built_slice_pair = dynamic_built_slice_pair.push_back(dynamic_one);
    }
    if dynamic_two == 2 {
        dynamic_built_slice_pair = dynamic_built_slice_pair.push_back(dynamic_two);
    }


    // works!
    // TODO: cleanup
    // assert(false);

    // works!
    // TODO: move to compile_success test
    // assert(true);

    // works! (already tested)
    // assert(1 == 2);

    // works!
    // TODO: move to compile_success test
    // assert(1 + 1 == 2);

    // works! (already tested)
    // assert(1 + 1 == 3);

    // works!
    // TODO: move to compile_success test
    // assert(3 == GLOBAL_THREE);

    // works!
    // TODO: move to compile_success test
    // assert(GLOBAL_ARRAY_PAIR.len() == 2);

    // works!
    // TODO: move to compile_success test
    // assert(GLOBAL_SLICE_PAIR.len() == 2);

    // works!
    // TODO: move to compile_success test
    // assert(dynamic_array_pair.len() == 2);

    // works!
    // TODO: move to compile_success test
    // assert(dynamic_slice_pair.len() == 2);

    // works! (i.e. passes b/c can't evaluate)
    // assert(dynamic_built_slice_pair.len() == 3);


    // assert_constant
    ////////////////////////////////////////////////

    // works! (already tested?)
    // assert_constant(true);

    // works! (already tested?)
    // assert_constant(false);

    // works!
    // TODO: move to own test
    // assert_constant(1 + 1 == 2);

    // works!
    // TODO: move to own test
    // assert_constant(1 + 1 == 3);

    // works! (already tested)
    // fails at compile time
    // assert_constant(dynamic_one);

    // works!
    // TODO: move to own test
    // assert_constant(dynamic_one + 1);

    // works!
    // TODO: move to own test
    // assert_constant(dynamic_one + 1 == 2);

    // works!
    // TODO: move to own test
    // assert_constant(dynamic_one + 1 == 3);

    // works!
    // TODO: move to own test
    // assert_constant((dynamic_one, 2));

    // works!
    // TODO: move to compile_success test
    // let foo = Foo { field: 0, array: [1, 2, 3], slice: &[] };
    // assert_constant(foo);

    // works!
    // TODO: move to own test
    // let foo_dynamic = Foo { field: dynamic_one, array: [1, 2, 3], slice: &[] };
    // assert_constant(foo_dynamic);

    // works!
    // TODO: move to compile_success test
    // static_assert([1, 2, dynamic_one].len() == 3, "hi");

    // works!
    // TODO: move to own test
    // assert_constant([dynamic_one]);

    // works!
    // TODO: move to own test
    // assert_constant(&[dynamic_one]);

    // works!
    // TODO: move to own test
    // let foo_dynamic_array = Foo { field: 0, array: [dynamic_one, 2, 3], slice: &[] };
    // assert_constant(foo_dynamic_array);

    // works!
    // TODO: move to own test
    // let foo_dynamic_slice = Foo { field: 0, array: [1, 2, 3], slice: &[dynamic_one] };
    // assert_constant(foo_dynamic_slice);

    // works!
    // TODO: move to own test
    // assert_constant([1, 2, dynamic_one].len() == 4);
    // assert([1, 2, dynamic_one].len() == 4);

    let x = 2;
    let y = 3;
    let xy = x + y;

    // TODO: needs assert_constant to handle complex values
    // (both currently passing)
    static_assert(xy == 5, "hi");
    static_assert(xy == 6, "hi !=");

}
