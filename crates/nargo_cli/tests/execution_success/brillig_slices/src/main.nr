use dep::std::slice;
unconstrained fn main(x: Field, y: Field) {
    let mut slice: [Field] = [y, x];
    assert_eq(slice.len(), 2);

    slice = slice.push_back(7);
    assert_eq(slice.len(), 3);
    assert_eq(slice[0], y);
    assert_eq(slice[1], x);
    assert_eq(slice[2], 7);

    // Array set on slice target
    slice[0] = x;
    slice[1] = y;
    slice[2] = 1;

    assert_eq(slice[0], x);
    assert_eq(slice[1], y);
    assert_eq(slice[2], 1);

    slice = push_front_to_slice(slice, 2);
    assert_eq(slice.len(), 4);
    assert_eq(slice[0], 2);
    assert_eq(slice[1], x);
    assert_eq(slice[2], y);
    assert_eq(slice[3], 1);

    let (item, popped_front_slice) = slice.pop_front();
    slice = popped_front_slice;
    assert_eq(item, 2);

    assert_eq(slice.len(), 3);
    assert_eq(slice[0], x);
    assert_eq(slice[1], y);
    assert_eq(slice[2], 1);

    let (popped_back_slice, another_item) = slice.pop_back();
    slice = popped_back_slice;
    assert_eq(another_item, 1);

    assert_eq(slice.len(), 2);
    assert_eq(slice[0], x);
    assert_eq(slice[1], y);

    slice = slice.insert(1, 2);
    assert_eq(slice.len(), 3);
    assert_eq(slice[0], x);
    assert_eq(slice[1], 2);
    assert_eq(slice[2], y);

    let (removed_slice, should_be_2) = slice.remove(1);
    slice = removed_slice;
    assert_eq(should_be_2, 2);

    assert_eq(slice.len(), 2);
    assert_eq(slice[0], x);
    assert_eq(slice[1], y);

    let (slice_with_only_x, should_be_y) = slice.remove(1);
    slice = slice_with_only_x;
    assert_eq(should_be_y, y);

    assert_eq(slice.len(), 1);
    assert_eq(slice[0], x);

    let (empty_slice, should_be_x) = slice.remove(0);
    assert_eq(should_be_x, x);
    assert_eq(empty_slice.len(), 0);

    regression_merge_slices(x, y);
}

// Tests slice passing to/from functions
unconstrained fn push_front_to_slice<T>(slice: [T], item: T) -> [T] {
    slice.push_front(item)
}

// The parameters to this function must come from witness values (inputs to main)
unconstrained fn regression_merge_slices(x: Field, y: Field) {
    merge_slices_if(x, y);
    merge_slices_else(x);
}

unconstrained fn merge_slices_if(x: Field, y: Field) {
    let slice = merge_slices_return(x, y);
    assert_eq(slice[2], 10);
    assert_eq(slice.len(), 3);

    let slice = merge_slices_mutate(x, y);
    assert_eq(slice[3], 5);
    assert_eq(slice.len(), 4);

    let slice = merge_slices_mutate_in_loop(x, y);
    assert_eq(slice[6], 4);
    assert_eq(slice.len(), 7);
}

unconstrained fn merge_slices_else(x: Field) {
    let slice = merge_slices_return(x, 5);
    assert_eq(slice[0], 0);
    assert_eq(slice[1], 0);
    assert_eq(slice.len(), 2);

    let slice = merge_slices_mutate(x, 5);
    assert_eq(slice[2], 5);
    assert_eq(slice.len(), 3);

    let slice = merge_slices_mutate_in_loop(x, 5);
    assert_eq(slice[2], 5);
    assert_eq(slice.len(), 3);
}

// Test returning a merged slice without a mutation
unconstrained fn merge_slices_return(x: Field, y: Field) -> [Field] {
    let slice = [0; 2];
    if x != y {
        if x != 20 {
            slice.push_back(y)
        } else {
            slice
        }
    } else {
        slice
    }
}

// Test mutating a slice inside of an if statement
unconstrained fn merge_slices_mutate(x: Field, y: Field) -> [Field] {
    let mut slice = [0; 2];
    if x != y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }
    slice
}

// Test mutating a slice inside of a loop in an if statement
unconstrained fn merge_slices_mutate_in_loop(x: Field, y: Field) -> [Field] {
    let mut slice = [0; 2];
    if x != y {
        for i in 0..5 {
            slice = slice.push_back(i);
        }
    } else {
        slice = slice.push_back(x);
    }
    slice
}
