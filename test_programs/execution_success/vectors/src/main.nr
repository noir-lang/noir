
use std::collections::unbounded_vec::UnboundedVec;



fn main24(x: Field, y: pub Field) {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([0, 0]);
    assert(vector.get(0) == 0);
    assert(vector.get(0) != 1);
    vector.set(0, x);
    assert(vector.get(0) == x);

    vector.push_back(y);
    assert(vector.get(2) == 10);
    assert(vector.get(2) != 8);
    assert(vector.len() == 3);

    let mut new_vector: UnboundedVec<Field> = UnboundedVec::new();
    for i in 0..5 {
        new_vector.push_back(i as Field);
    }
    assert(new_vector.len() == 5);

    new_vector.push_front(20);
    assert(new_vector.get(0) == 20);
    assert(new_vector.len() == 6);

    let last_elem = new_vector.pop_back();
    assert(last_elem == 4);
    assert(new_vector.len() == 5);

    let first_elem = new_vector.pop_front();
    assert(first_elem == 20);
    assert(new_vector.len() == 4);

    new_vector.insert(2, 100);
    assert(new_vector.get(2) == 100);
    assert(new_vector.get(4) == 3);
    assert(new_vector.len() == 5);

    let removed_elem = new_vector.remove(3);
    assert(removed_elem == 2);
    assert(new_vector.get(3) == 3);
    assert(new_vector.len() == 4);

    let mut append: UnboundedVec<Field> = UnboundedVec::from_array([1, 2]);
    append.extend_from_array([3, 4, 5]);
    assert(append.len() == 5);
    assert(append.get(0) == 1);
    assert(append.get(4) == 5);

    // TODO: UnboundedVec doesn't have map/fold/reduce/all/any methods yet
    // let mapped = [1, 2].as_vector().map(|x| x + 1);
    // assert_eq(mapped, [2, 3].as_vector());
    // assert_eq([1, 2, 3].as_vector().fold(0, |acc, x| acc + x), 6);
    // assert_eq([1, 2, 3].as_vector().reduce(|acc, x| acc + x), 6);
    // assert([2, 4, 6].as_vector().all(|x| x > 0));
    // assert([2, 4, 6].as_vector().any(|x| x > 5));

    // TODO: Tuple support has issues with vector length calculations
    regression_2083();

    // The parameters to this function must come from witness values (inputs to main)
    // TODO: Complex control flow with vector merging has issues
    // regression_merge_vectors(x, y);  // TESTING: Test this alone

    regression_2370();

    regression_4418(x);

    // TODO: Control flow issues?
    regression_vector_call_result(x, y);  // TESTING: Comment this out

    regression_4506();

    // TODO: Unconstrained functions with vectors
    // regression_vector_to_brillig(x as u32 % 3);
}

// Ensure that vectors of struct/tuple values work.
// TODO: it currently fails
fn regression_2083() {
    let mut y: UnboundedVec<(Field, Field)> = UnboundedVec::from_array([(1, 2)]);
    y.push_back((3, 4)); // [(1, 2), (3, 4)]
    y.push_back((5, 6)); // [(1, 2), (3, 4), (5, 6)]
    assert(y.get(2).1 == 6);

    y.push_front((10, 11)); // [(10, 11), (1, 2), (3, 4), (5, 6)]
    y.push_front((12, 13)); // [(12, 13), (10, 11), (1, 2), (3, 4), (5, 6)]
    assert(y.get(1).0 == 10);

    y.insert(1, (55, 56)); // [(12, 13), (55, 56), (10, 11), (1, 2), (3, 4), (5, 6)]
    assert(y.get(0).1 == 13);
    assert(y.get(1).1 == 56);
    assert(y.get(2).0 == 10);

    let x = y.remove(2); // [(12, 13), (55, 56), (1, 2), (3, 4), (5, 6)]
    assert(y.get(2).0 == 1);
    assert(x.0 == 10);
    assert(x.1 == 11);

    let x = y.pop_front(); // [(55, 56), (1, 2), (3, 4), (5, 6)]
    assert(y.get(0).0 == 55);
    assert(x.0 == 12);
    assert(x.1 == 13);

    let x = y.pop_back(); // [(55, 56), (1, 2), (3, 4)]
    assert(y.len() == 3);
    assert(x.0 == 5);
    assert(x.1 == 6);
}

// The parameters to this function must come from witness values (inputs to main)
fn regression_merge_vectors(x: Field, y: Field) {
    merge_vectors_if(x, y);
    // merge_vectors_else(x);  // TESTING: Comment this out
}

fn merge_vectors_if(x: Field, y: Field) {
    let vector = merge_vectors_return(x, y);
    assert(vector.len() == 3);
    assert(vector.get(2) == 10);

    let vector = merge_vectors_mutate(x, y);
    assert(vector.len() == 4);
    assert(vector.get(3) == 5);

    let vector = merge_vectors_mutate_in_loop(x, y);
    assert(vector.len() == 7);
    assert(vector.get(6) == 4);

    let vector = merge_vectors_mutate_two_ifs(x, y);
    assert(vector.len() == 6);
    assert(vector.get(3) == 5);
    assert(vector.get(4) == 15);
    assert(vector.get(5) == 30);

    let vector = merge_vectors_mutate_between_ifs(x, y);
    assert(vector.len() == 8);
    assert(vector.get(3) == 5);
    assert(vector.get(4) == 30);
    assert(vector.get(5) == 15);
    assert(vector.get(6) == 50);
    assert(vector.get(7) == 60);

    merge_vectors_push_then_pop(x, y);

    // COMMENTED OUT: This uses insert() which may cause SSA issues
    // let vector = merge_vectors_push_then_insert(x, y);
    // assert(vector.len() == 7);
    // assert(vector.get(1) == 50);
    // assert(vector.get(2) == 0);
    // assert(vector.get(5) == 30);
    // assert(vector.get(6) == 100);

    // COMMENTED OUT: This uses remove() which may cause SSA issues
    // let vector = merge_vectors_remove_between_ifs(x, y);
    // assert(vector.len() == 5);
}

fn merge_vectors_else(x: Field) {
    let vector = merge_vectors_return(x, 5);
    assert(vector.get(0) == 0);
    assert(vector.get(1) == 0);
    assert(vector.len() == 2);

    let vector = merge_vectors_mutate(x, 5);
    assert(vector.get(2) == 5);
    assert(vector.len() == 3);

    let vector = merge_vectors_mutate_in_loop(x, 5);
    assert(vector.get(2) == 5);
    assert(vector.len() == 3);
}

// Test returning a merged vector without a mutation
fn merge_vectors_return(x: Field, y: Field) -> UnboundedVec<Field> {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([0, 0]);
    if x != y {
        if x != 20 {
            vector.push_back(y);
        }
    }
    vector
}

// Test mutating a vector inside of an if statement
fn merge_vectors_mutate(x: Field, y: Field) -> UnboundedVec<Field> {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([0, 0]);
    if x != y {
        vector.push_back(y);
        vector.push_back(x);
    } else {
        vector.push_back(x);
    }
    vector
}

// Test mutating a vector inside of a loop in an if statement
fn merge_vectors_mutate_in_loop(x: Field, y: Field) -> UnboundedVec<Field> {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([0, 0]);
    if x != y {
        for i in 0..5 {
            vector.push_back(i as Field);
        }
    } else {
        vector.push_back(x);
    }
    vector
}

fn merge_vectors_mutate_two_ifs(x: Field, y: Field) -> UnboundedVec<Field> {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([0, 0]);
    if x != y {
        vector.push_back(y);
        vector.push_back(x);
    } else {
        vector.push_back(x);
    }

    if x == 20 {
        vector.push_back(20);
    }

    vector.push_back(15);
    vector.push_back(30);

    vector
}

fn merge_vectors_mutate_between_ifs(x: Field, y: Field) -> UnboundedVec<Field> {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([0, 0]);
    if x != y {
        vector.push_back(y);
        vector.push_back(x);
    } else {
        vector.push_back(x);
    }

    vector.push_back(30);

    if x == 20 {
        vector.push_back(20);
    }

    vector.push_back(15);

    if x != 20 {
        vector.push_back(50);
    }

    vector.push_back(60);

    vector
}

fn merge_vectors_push_then_pop(x: Field, y: Field) {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([0, 0]);
    if x != y {
        vector.push_back(y);
        vector.push_back(x);
    } else {
        vector.push_back(x);
    }

    vector.push_back(30);

    if x == 20 {
        vector.push_back(20);
    }

    let elem = vector.pop_back();
    assert(vector.len() == 4);
    assert(elem == 30);

    let elem = vector.pop_back();
    assert(vector.len() == 3);
    assert(elem == x);
}

fn merge_vectors_push_then_insert(x: Field, y: Field) -> UnboundedVec<Field> {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([0, 0]);
    if x != y {
        vector.push_back(y);
        vector.push_back(x);
    } else {
        vector.push_back(x);
    }

    vector.push_back(30);

    if x == 20 {
        vector.push_back(20);
        vector.push_back(15);
    }

    vector.insert(1, 50);
    // Test that we can use vector insert the same as vector push back
    vector.insert(6, 100);

    vector
}

fn merge_vectors_remove_between_ifs(x: Field, y: Field) -> UnboundedVec<Field> {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([0, 0]);
    if x != y {
        vector.push_back(y);
        vector.push_back(x);
    } else {
        vector.push_back(x);
    }

    let elem = vector.remove(2);
    assert(elem == y);

    if x == 20 {
        vector.push_back(20);
    }

    vector.push_back(15);

    if x != 20 {
        vector.push_back(50);
    }

    vector
}

// Previously, we'd get a type error when trying to assign an array of a different size to
// an existing array variable. Now, we infer the variable must be a vector.
fn regression_2370() {
    let mut vector: UnboundedVec<Field> = UnboundedVec::new();
    vector = UnboundedVec::from_array([1, 2, 3]);
}

fn regression_4418(x: Field) {
    let mut crash: [u8; 32] = x.to_be_bytes();

    if x != 0 {
        crash[0] = 10;
    }
}

fn regression_vector_call_result(x: Field, y: Field) {
    let mut vector = merge_vectors_return(x, y);
    if x != 0 {
        vector.push_back(5);
        vector.push_back(10);
    } else {
        vector.push_back(5);
    }
    assert(vector.len() == 5);
    assert(vector.get(0) == 0);
    assert(vector.get(1) == 0);
    assert(vector.get(2) == 10);
    assert(vector.get(3) == 5);
    assert(vector.get(4) == 10);
}

fn regression_4506() {
    let vector: UnboundedVec<Field> = UnboundedVec::from_array([1, 2, 3]);
    assert(vector == vector);
}

// TODO: Unconstrained functions with UnboundedVec
// struct foo {
//     a: u32,
//     b: u32,
//     c: [Field; 3],
// }
// fn regression_vector_to_brillig(x: u32) {
//     let mut vector: UnboundedVec<foo> =
//         UnboundedVec::from_array([foo { a: 0, b: 0, c: [0, 0, 0] }, foo { a: 1, b: 1, c: [0, 0, 0] }]);
//     let mut foo1 = foo { a: 10, b: 10, c: [0, 0, 0] };
//     foo1.c[x] = 7;
//     vector.push_back(foo1);
//     vector.push_back(foo { a: 0, b: 0, c: [0, 0, 0] });
//     let elem = vector.pop_back();
//     // Safety: for testing purposes
//     let ok = unsafe { bar(vector) };
//     assert(ok);
// }

// unconstrained fn bar(vector: UnboundedVec<foo>) -> bool {
//     let result = if vector.len() != 0 {
//         vector.get(0).a == 0
//     } else {
//         false
//     };

//     //Regression #11440
//     let len = vector.len();
//     let b = vector.pop_front();
//     println(vector.len());
//     println(len);

//     result
// }
