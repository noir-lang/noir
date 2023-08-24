use dep::std::slice;
use dep::std;

fn main(x : Field, y : pub Field) {
    // Dynamic indexing works on slice with constant length
    // let mut slice = [0, 1, 2, 3, 4];

    // Need to implement flattening on a dynamic length
    // let mut slice = [];
    // for i in 0..5 {
    //     slice = slice.push_back(i);
    // }
    // let mut slice = [0, 1];
    // if x != y {
    //     slice = slice.push_back(y);
    // } else {
    //     slice = slice.push_back(x);
    // }
    // assert(slice.len() == 3);
    // assert(slice[2] == 10);


    // let mut slice = [0; 2];
    // assert(slice[0] == 0);
    // assert(slice[0] != 1);
    // slice[0] = x;
    // assert(slice[0] == x);

    // let slice_plus_10 = slice.push_back(y);
    // assert(slice_plus_10[2] == 10);
    // assert(slice_plus_10[2] != 8);
    // assert(slice_plus_10.len() == 3);

    // let mut new_slice = [];
    // for i in 0..5 {
    //     new_slice = new_slice.push_back(i);
    // }
    // assert(new_slice.len() == 5);

    // new_slice = new_slice.push_front(20);
    // assert(new_slice[0] == 20);
    // assert(new_slice.len() == 6);

    // let (popped_slice, last_elem) = new_slice.pop_back();
    // assert(last_elem == 4);
    // assert(popped_slice.len() == 5);

    // let (first_elem, rest_of_slice) = popped_slice.pop_front();
    // assert(first_elem == 20);
    // assert(rest_of_slice.len() == 4);

    // new_slice = rest_of_slice.insert(2, 100);
    // assert(new_slice[2] == 100);
    // assert(new_slice[4] == 3);
    // assert(new_slice.len() == 5);

    // let (remove_slice, removed_elem) = new_slice.remove(3);
    // assert(removed_elem == 2);
    // assert(remove_slice[3] == 3);
    // assert(remove_slice.len() == 4);

    // let append = [1, 2].append([3, 4, 5]);
    // assert(append.len() == 5);
    // assert(append[0] == 1);
    // assert(append[4] == 5);

    // regression_2083();
    // // The parameters to this function must come from witness values (inputs to main)
    // regression_merge_slices(x, y);
    regression_dynamic_slice_index(x - 1, x - 4);
}

fn regression_dynamic_slice_index(x: Field, y: Field) {
    let mut slice = [];
    for i in 0..5 {
        slice = slice.push_back(i);
    }
    assert(slice.len() == 5);
    
    dynamic_slice_index_set_if(slice, x, y);
    dynamic_slice_index_set_else(slice, x, y);
    dynamic_slice_index_set_nested_if_else_else(slice, x, y);
    dynamic_slice_index_set_nested_if_else_if(slice, x, y + 1);
    dynamic_slice_index_if(slice, x, y);
    dynamic_slice_index_else(slice, x, y);

    // dynamic_slice_merge_if(slice, x, y);
    // dynamic_slice_merge_else(slice, x, y);
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
fn dynamic_slice_index_if(mut slice: [Field], x: Field, y: Field) {
    if x as u32 < 10 {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
    } else {
        assert(slice[x] == 0);
    }
    assert(slice[4] == 2);
}

// This tests the case of missing a store instruction in the then branch
// of merging slices
fn dynamic_slice_index_else(mut slice: [Field], x: Field, y: Field) {
    if x as u32 > 10 {
        assert(slice[x] == 0);
    } else {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
    }
    assert(slice[4] == 2);
}

// TODO: This is still broken. Need to verify whether attaching 
// predicates to memory ops in ACIR solves the issue (PR #2400), but I am pretty
// sure that it is the cause. 
fn dynamic_slice_merge_if(mut slice: [Field], x: Field, y: Field) {
    if x as u32 < 10 {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
        slice = slice.push_back(10);
    } else {
        assert(slice[x] == 0);
    }

    assert(slice[4] == 2);
    assert(slice[slice.len() - 1] == 10);
}

// TODO: This is still broken. Need to verify whether attaching 
// predicates to memory ops in ACIR solves the issue (PR #2400), but I am pretty
// sure that it is the cause. 
fn dynamic_slice_merge_else(mut slice: [Field], x: Field, y: Field) {
    if x as u32 > 10 {
        assert(slice[x] == 0);
    } else {
        assert(slice[x] == 4);
        slice[x] = slice[x] - 2;
        slice = slice.push_back(10);
    }

    assert(slice[4] == 2);
    assert(slice[slice.len() - 1] == 10);
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
            }
            assert(slice[4] == 10);
        }
    } else {
        slice[x] = 0;
    }
    assert(slice[4] == 10);
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

// Ensure that slices of struct/tuple values work.
fn regression_2083() {
    let y = [(1, 2)];
    let y = y.push_back((3, 4)); // [(1, 2), (3, 4)]
    let y = y.push_back((5, 6)); // [(1, 2), (3, 4), (5, 6)]
    assert(y[2].1 == 6);

    let y = y.push_front((10, 11)); // [(10, 11), (1, 2), (3, 4), (5, 6)]
    let y = y.push_front((12, 13)); // [(12, 13), (10, 11), (1, 2), (3, 4), (5, 6)]

    assert(y[1].0 == 10);

    let y = y.insert(1, (55, 56)); // [(12, 13), (55, 56), (10, 11), (1, 2), (3, 4), (5, 6)]
    assert(y[0].1 == 13);
    assert(y[1].1 == 56);
    assert(y[2].0 == 10);

    let (y, x) = y.remove(2); // [(12, 13), (55, 56), (1, 2), (3, 4), (5, 6)]
    assert(y[2].0 == 1);
    assert(x.0 == 10);
    assert(x.1 == 11);

    let (x, y) = y.pop_front(); // [(55, 56), (1, 2), (3, 4), (5, 6)]
    assert(y[0].0 == 55);
    assert(x.0 == 12);
    assert(x.1 == 13);

    let (y, x) = y.pop_back(); // [(55, 56), (1, 2), (3, 4)]
    assert(y.len() == 3);
    assert(x.0 == 5);
    assert(x.1 == 6);
}

// The parameters to this function must come from witness values (inputs to main)
fn regression_merge_slices(x: Field, y: Field) {
    merge_slices_if(x, y);
    merge_slices_else(x);
}

fn merge_slices_if(x: Field, y: Field) {
    let slice = merge_slices_return(x, y);
    assert(slice[2] == 10);
    assert(slice.len() == 3);

    let slice = merge_slices_mutate(x, y);
    assert(slice[3] == 5);
    assert(slice.len() == 4);

    let slice = merge_slices_mutate_in_loop(x, y);
    assert(slice[6] == 4);
    assert(slice.len() == 7);
}

fn merge_slices_else(x: Field) {
    let slice = merge_slices_return(x, 5);
    assert(slice[0] == 0);
    assert(slice[1] == 0);
    assert(slice.len() == 2);

    let slice = merge_slices_mutate(x, 5);
    assert(slice[2] == 5);
    assert(slice.len() == 3);

    let slice = merge_slices_mutate_in_loop(x, 5);
    assert(slice[2] == 5);
    assert(slice.len() == 3);
}

// Test returning a merged slice without a mutation
fn merge_slices_return(x: Field, y: Field) -> [Field] {
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
fn merge_slices_mutate(x: Field, y: Field) -> [Field] {
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
fn merge_slices_mutate_in_loop(x: Field, y: Field) -> [Field] {
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
