fn main(x: Field) {
    // The parameters to this function must come directly from witness values (inputs to main).
    regression_dynamic_slice_index(x - 1, x - 4);
}

fn regression_dynamic_slice_index(x: Field, y: Field) {
    let mut slice = &[];
    for i in 0..5 {
        slice = slice.push_back(i as Field);
    }
    assert(slice.len() == 5);

    dynamic_slice_index_set_if(slice, x, y);
    dynamic_slice_index_set_else(slice, x, y);
    dynamic_slice_index_set_nested_if_else_else(slice, x, y);
    dynamic_slice_index_set_nested_if_else_if(slice, x, y + 1);
    dynamic_slice_index_if(slice, x);
    dynamic_array_index_if([0, 1, 2, 3, 4], x);
    dynamic_slice_index_else(slice, x);

    dynamic_slice_merge_if(slice, x);
    dynamic_slice_merge_else(slice, x);
    dynamic_slice_merge_two_ifs(slice, x);
    dynamic_slice_merge_mutate_between_ifs(slice, x, y);
    dynamic_slice_merge_push_then_pop(slice, x, y);
}

fn dynamic_slice_index_set_if(mut slice: [Field], x: Field, y: Field) {
    assert(slice[x] == 4);
    assert(slice[y] == 1);
    slice[y] = 0;
    assert(slice[x] == 4);
    assert(slice[1] == 0);
    if x as u32 < 10 {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
        slice[x - 1] = slice[x];
    } else {
        slice[x] = 0;
    }
    assert(slice[3] == 2);
    assert(slice[4] == 2);
}

fn dynamic_slice_index_set_else(mut slice: [Field], x: Field, y: Field) {
    assert(slice[x] == 4);
    assert(slice[y] == 1);
    slice[y] = 0;
    assert(slice[x] == 4);
    assert(slice[1] == 0);
    if x as u32 > 10 {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
        slice[x - 1] = slice[x];
    } else {
        slice[x] = 0;
    }
    assert(slice[4] == 0);
}
// This tests the case of missing a store instruction in the else branch
// of merging slices
fn dynamic_slice_index_if(mut slice: [Field], x: Field) {
    if x as u32 < 10 {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
    } else {
        assert(slice[x] == 0);
    }
    assert(slice[4] == 2);
}

fn dynamic_array_index_if(mut array: [Field; 5], x: Field) {
    if x as u32 < 10 {
        assert(array[x] == 4);
        array[x] = array[x] - 2;
    } else {
        assert(array[x] == 0);
    }
    assert(array[4] == 2);
}
// This tests the case of missing a store instruction in the then branch
// of merging slices
fn dynamic_slice_index_else(mut slice: [Field], x: Field) {
    if x as u32 > 10 {
        assert(slice[x] == 0);
    } else {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
    }
    assert(slice[4] == 2);
}

fn dynamic_slice_merge_if(mut slice: [Field], x: Field) {
    if x as u32 < 10 {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;

        slice = slice.push_back(10);
        // Having an array set here checks whether we appropriately
        // handle a slice length that is not yet resolving to a constant
        // during flattening
        slice[x] = 10;
        assert(slice[slice.len() - 1] == 10);
        assert(slice.len() == 6);

        slice[x] = 20;
        slice[x] = slice[x] + 10;

        slice = slice.push_front(11);
        assert(slice[0] == 11);
        assert(slice.len() == 7);
        assert(slice[5] == 30);

        slice = slice.push_front(12);
        assert(slice[0] == 12);
        assert(slice.len() == 8);
        assert(slice[6] == 30);

        let (popped_slice, last_elem) = slice.pop_back();
        assert(last_elem == 10);
        assert(popped_slice.len() == 7);

        let (first_elem, rest_of_slice) = popped_slice.pop_front();
        assert(first_elem == 12);
        assert(rest_of_slice.len() == 6);

        slice = rest_of_slice.insert(x as u32 - 2, 20);
        assert(slice[2] == 20);
        assert(slice[6] == 30);
        assert(slice.len() == 7);

        let (removed_slice, removed_elem) = slice.remove(x as u32 - 1);
        // The deconstructed tuple assigns to the slice but is not seen outside of the if statement
        // without a direct assignment
        slice = removed_slice;

        assert(removed_elem == 1);
        assert(slice.len() == 6);
    } else {
        assert(slice[x] == 0);
        slice = slice.push_back(20);
    }

    assert(slice.len() == 6);
    assert(slice[slice.len() - 1] == 30);
}

fn dynamic_slice_merge_else(mut slice: [Field], x: Field) {
    if x as u32 > 10 {
        assert(slice[x] == 0);
        slice[x] = 2;
    } else {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
        slice = slice.push_back(10);
    }
    assert(slice.len() == 6);
    assert(slice[slice.len() - 1] == 10);

    slice = slice.push_back(20);
    assert(slice.len() == 7);
    assert(slice[slice.len() - 1] == 20);
}

fn dynamic_slice_index_set_nested_if_else_else(mut slice: [Field], x: Field, y: Field) {
    assert(slice[x] == 4);
    assert(slice[y] == 1);
    slice[y] = 0;
    assert(slice[x] == 4);
    assert(slice[1] == 0);
    if x as u32 < 10 {
        slice[x] = slice[x] - 2;
        if y != 1 {
            slice[x] = slice[x] + 20;
        } else {
            if x == 5 {
                // We should not hit this case
                assert(slice[x] == 22);
            } else {
                slice[x] = 10;
                slice = slice.push_back(15);
                assert(slice.len() == 6);
            }
            assert(slice[4] == 10);
        }
    } else {
        slice[x] = 0;
    }
    assert(slice[4] == 10);
    assert(slice.len() == 6);
    assert(slice[slice.len() - 1] == 15);

    slice = slice.push_back(20);
    assert(slice.len() == 7);
    assert(slice[slice.len() - 1] == 20);
}

fn dynamic_slice_index_set_nested_if_else_if(mut slice: [Field], x: Field, y: Field) {
    assert(slice[x] == 4);
    assert(slice[y] == 2);
    slice[y] = 0;
    assert(slice[x] == 4);
    assert(slice[2] == 0);
    if x as u32 < 10 {
        slice[x] = slice[x] - 2;
        // TODO: this panics as we have a load for the slice in flattening
        if y == 1 {
            slice[x] = slice[x] + 20;
        } else {
            if x == 4 {
                slice[x] = 5;
            }
            assert(slice[4] == 5);
        }
    } else {
        slice[x] = 0;
    }
    assert(slice[4] == 5);
}

fn dynamic_slice_merge_two_ifs(mut slice: [Field], x: Field) {
    if x as u32 > 10 {
        assert(slice[x] == 0);
        slice[x] = 2;
    } else {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
        slice = slice.push_back(10);
    }

    assert(slice.len() == 6);
    assert(slice[slice.len() - 1] == 10);

    if x == 20 {
        slice = slice.push_back(20);
    }

    slice = slice.push_back(15);

    assert(slice.len() == 7);
    assert(slice[slice.len() - 1] == 15);

    slice = slice.push_back(20);
    assert(slice.len() == 8);
    assert(slice[slice.len() - 1] == 20);
}

fn dynamic_slice_merge_mutate_between_ifs(mut slice: [Field], x: Field, y: Field) {
    if x != y {
        slice[x] = 50;
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice[x] = slice[x] - 2;
        slice = slice.push_back(x);
    }

    slice = slice.push_back(30);
    assert(slice.len() == 8);

    if x == 20 {
        slice = slice.push_back(20);
    }

    slice = slice.push_back(15);

    if x != 20 {
        slice = slice.push_back(50);
    }

    slice = slice.push_back(60);
    assert(slice.len() == 11);
    assert(slice[x] == 50);
    assert(slice[slice.len() - 4] == 30);
    assert(slice[slice.len() - 3] == 15);
    assert(slice[slice.len() - 2] == 50);
    assert(slice[slice.len() - 1] == 60);
}

fn dynamic_slice_merge_push_then_pop(mut slice: [Field], x: Field, y: Field) {
    if x != y {
        slice[x] = 5;
        slice = slice.push_back(y);
        slice = slice.push_back(x);
        assert(slice.len() == 7);

        let (popped_slice, elem) = slice.pop_back();
        assert(slice.len() == 7);
        assert(elem == x);
        slice = popped_slice;
    } else {
        slice = slice.push_back(x);
    }

    slice = slice.push_back(30);
    assert(slice.len() == 7);

    if x == 20 {
        slice = slice.push_back(20);
    }

    let (slice, elem) = slice.pop_back();
    assert(elem == 30);

    let (_, elem) = slice.pop_back();
    assert(elem == y);
}

