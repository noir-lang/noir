use std::slice;

fn main(x: Field, y: pub Field) {
    let mut slice = &[0; 2];
    assert(slice[0] == 0);
    assert(slice[0] != 1);
    slice[0] = x;
    assert(slice[0] == x);

    let slice_plus_10 = slice.push_back(y);
    assert(slice_plus_10[2] == 10);
    assert(slice_plus_10[2] != 8);
    assert(slice_plus_10.len() == 3);

    let mut new_slice = &[];
    for i in 0..5 {
        new_slice = new_slice.push_back(i);
    }
    assert(new_slice.len() == 5);

    new_slice = new_slice.push_front(20);
    assert(new_slice[0] == 20);
    assert(new_slice.len() == 6);

    let (popped_slice, last_elem) = new_slice.pop_back();
    assert(last_elem == 4);
    assert(popped_slice.len() == 5);

    let (first_elem, rest_of_slice) = popped_slice.pop_front();
    assert(first_elem == 20);
    assert(rest_of_slice.len() == 4);

    new_slice = rest_of_slice.insert(2, 100);
    assert(new_slice[2] == 100);
    assert(new_slice[4] == 3);
    assert(new_slice.len() == 5);

    let (remove_slice, removed_elem) = new_slice.remove(3);
    assert(removed_elem == 2);
    assert(remove_slice[3] == 3);
    assert(remove_slice.len() == 4);

    let append = &[1, 2].append(&[3, 4, 5]);
    assert(append.len() == 5);
    assert(append[0] == 1);
    assert(append[4] == 5);

    let mapped = &[1, 2].map(|x| x + 1);
    assert_eq(mapped, &[2, 3]);

    assert_eq(&[1, 2, 3].fold(0, |acc, x| acc + x), 6);
    assert_eq(&[1, 2, 3].reduce(|acc, x| acc + x), 6);
    assert(&[2, 4, 6].all(|x| x > 0));
    assert(&[2, 4, 6].any(|x| x > 5));

    regression_2083();
    // The parameters to this function must come from witness values (inputs to main)
    regression_merge_slices(x, y);
    regression_2370();

    regression_4418(x);
    regression_slice_call_result(x, y);
    regression_4506();
}

// Ensure that slices of struct/tuple values work.
fn regression_2083() {
    let y = &[(1, 2)];
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
    assert(slice.len() == 3);
    assert(slice[2] == 10);

    let slice = merge_slices_mutate(x, y);
    assert(slice.len() == 4);
    assert(slice[3] == 5);

    let slice = merge_slices_mutate_in_loop(x, y);
    assert(slice.len() == 7);
    assert(slice[6] == 4);

    let slice = merge_slices_mutate_two_ifs(x, y);
    assert(slice.len() == 6);
    assert(slice[3] == 5);
    assert(slice[4] == 15);
    assert(slice[5] == 30);

    let slice = merge_slices_mutate_between_ifs(x, y);
    assert(slice.len() == 8);
    assert(slice[3] == 5);
    assert(slice[4] == 30);
    assert(slice[5] == 15);
    assert(slice[6] == 50);
    assert(slice[7] == 60);

    merge_slices_push_then_pop(x, y);

    let slice = merge_slices_push_then_insert(x, y);
    assert(slice.len() == 7);
    assert(slice[1] == 50);
    assert(slice[2] == 0);
    assert(slice[5] == 30);
    assert(slice[6] == 100);

    let slice = merge_slices_remove_between_ifs(x, y);
    assert(slice.len() == 5);
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
    let slice = &[0; 2];
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
    let mut slice = &[0; 2];
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
    let mut slice = &[0; 2];
    if x != y {
        for i in 0..5 {
            slice = slice.push_back(i as Field);
        }
    } else {
        slice = slice.push_back(x);
    }
    slice
}

fn merge_slices_mutate_two_ifs(x: Field, y: Field) -> [Field] {
    let mut slice = &[0; 2];
    if x != y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }

    if x == 20 {
        slice = slice.push_back(20);
    }

    slice = slice.push_back(15);
    slice = slice.push_back(30);

    slice
}

fn merge_slices_mutate_between_ifs(x: Field, y: Field) -> [Field] {
    let mut slice = &[0; 2];
    if x != y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }

    slice = slice.push_back(30);

    if x == 20 {
        slice = slice.push_back(20);
    }

    slice = slice.push_back(15);

    if x != 20 {
        slice = slice.push_back(50);
    }

    slice = slice.push_back(60);

    slice
}

fn merge_slices_push_then_pop(x: Field, y: Field) {
    let mut slice = &[0; 2];
    if x != y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }

    slice = slice.push_back(30);

    if x == 20 {
        slice = slice.push_back(20);
    }

    let (slice, elem) = slice.pop_back();
    assert(slice.len() == 4);
    assert(elem == 30);

    let (slice, elem) = slice.pop_back();
    assert(slice.len() == 3);
    assert(elem == x);
}

fn merge_slices_push_then_insert(x: Field, y: Field) -> [Field] {
    let mut slice = &[0; 2];
    if x != y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }

    slice = slice.push_back(30);

    if x == 20 {
        slice = slice.push_back(20);
        slice = slice.push_back(15);
    }

    slice = slice.insert(1, 50);
    // Test that we can use slice insert the same as slice push back
    slice = slice.insert(6, 100);

    slice
}

fn merge_slices_remove_between_ifs(x: Field, y: Field) -> [Field] {
    let mut slice = &[0; 2];
    if x != y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }

    let (mut slice, elem) = slice.remove(2);
    assert(elem == y);

    if x == 20 {
        slice = slice.push_back(20);
    }

    slice = slice.push_back(15);

    if x != 20 {
        slice = slice.push_back(50);
    }

    slice
}
// Previously, we'd get a type error when trying to assign an array of a different size to
// an existing array variable. Now, we infer the variable must be a slice.
fn regression_2370() {
    let mut slice = &[];
    slice = &[1, 2, 3];
}

fn regression_4418(x: Field) {
    let mut crash: [u8; 32] = x.to_be_bytes();

    if x != 0 {
        crash[0] = 10;
    }
}

fn regression_slice_call_result(x: Field, y: Field) {
    let mut slice = merge_slices_return(x, y);
    if x != 0 {
        slice = slice.push_back(5);
        slice = slice.push_back(10);
    } else {
        slice = slice.push_back(5);
    }
    assert(slice.len() == 5);
    assert(slice[0] == 0);
    assert(slice[1] == 0);
    assert(slice[2] == 10);
    assert(slice[3] == 5);
    assert(slice[4] == 10);
}

fn regression_4506() {
    let slice: [Field] = &[1, 2, 3];
    assert(slice == slice);
}
