
use std::collections::unbounded_vec::UnboundedVec;
use std::collections::unbounded_vec::vector_enumerate;

fn main(x: u32) {
    // The parameters to this function must come directly from witness values (inputs to main).
    regression_dynamic_vector_index(x - 1, x - 4);
}

fn regression_dynamic_vector_index(x: u32, y: u32) {
    let mut vector: UnboundedVec<Field> = UnboundedVec::new();
    for i in 0..5 {
        vector.push_back(i as Field);
    }
    assert(vector.len() == 5);


     dynamic_vector_index_set_if(vector, x, y);
     dynamic_vector_index_set_else(vector, x, y);
     dynamic_vector_index_set_nested_if_else_else(vector, x, y);
     dynamic_vector_index_set_nested_if_else_if(vector, x, y + 1);
     dynamic_vector_index_if(vector, x);
     dynamic_array_index_if([0, 1, 2, 3, 4], x);
     dynamic_vector_index_else(vector, x);

   dynamic_vector_merge_if(vector, x);
   dynamic_vector_merge_else(vector, x);
    dynamic_vector_merge_two_ifs(vector, x);
    dynamic_vector_merge_mutate_between_ifs(vector, x, y);
    dynamic_vector_merge_push_then_pop(vector, x, y);
}
fn dynamic_vector_index_set_if(mut vector: UnboundedVec<Field>, x: u32, y: u32) {
    assert(vector.get(x) == 4);
    assert(vector.get(y) == 1);
    vector.set(y, 0);
    assert(vector.get(x) == 4);
    assert(vector.get(1) == 0);
    if x as u32 < 10 {
        assert(vector.get(x) == 4);
        vector.set(x, vector.get(x) - 2);
        vector.set(x - 1, vector.get(x));
    } else {
        vector.set(x, 0);
    }
    assert(vector.get(3) == 2);
    assert(vector.get(4) == 2);
}

fn dynamic_vector_index_set_else(mut vector: UnboundedVec<Field>, x: u32, y: u32) {
    assert(vector.get(x) == 4);
    assert(vector.get(y) == 1);
    vector.set(y, 0);
    assert(vector.get(x) == 4);
    assert(vector.get(1) == 0);
    if x as u32 > 10 {
        assert(vector.get(x) == 4);
        vector.set(x, vector.get(x) - 2);
        vector.set(x - 1, vector.get(x));
    } else {
        vector.set(x, 0);
    }
    assert(vector.get(4) == 0);
}
// This tests the case of missing a store instruction in the else branch
// of merging vectors
fn dynamic_vector_index_if(mut vector:UnboundedVec<Field>, x: u32) {
    if x as u32 < 10 {
        assert(vector.get(x) == 4);
        vector.set(x, vector.get(x) - 2);
    } else {
        assert(vector.get(x) == 0);
    }
    assert(vector.get(4) == 2);
}

fn dynamic_array_index_if(mut array: [Field; 5], x: u32) {
    if x as u32 < 10 {
        assert(array[x] == 4);
        array[x] = array[x] - 2;
    } else {
        assert(array[x] == 0);
    }
    assert(array[4] == 2);
}
// This tests the case of missing a store instruction in the then branch
// of merging vectors
fn dynamic_vector_index_else(mut vector: UnboundedVec<Field>, x: u32) {
    if x as u32 > 10 {
        assert(vector.get(x) == 0);
    } else {
        assert(vector.get(x) == 4);
        vector.set(x, vector.get(x) - 2);
    }
    assert(vector.get(4) == 2);
}

fn dynamic_vector_merge_if(mut vector: UnboundedVec<Field>, x: u32) {
    if x as u32 < 10 {
        assert(vector.get(x) == 4);
        vector.set(x, vector.get(x) - 2);

        vector.push_back(10);
        // Having an array set here checks whether we appropriately
        // handle a vector length that is not yet resolving to a constant
        // during flattening
        vector.set(x, 10);
        assert(vector.get(vector.len() - 1) == 10);
        assert(vector.len() == 6);

        vector.set(x, 20);
        vector.set(x, vector.get(x) + 10);
        vector.push_front(11);
        assert(vector.get(0) == 11);
        assert(vector.len() == 7);
        assert(vector.get(5) == 30);
        vector.push_front(12);
        assert(vector.get(0) == 12);
        assert(vector.len() == 8);
        assert(vector.get(6) == 30);

        let last_elem = vector.pop_back();
        assert(last_elem == 10);
        assert(vector.len() == 7);

        let first_elem = vector.pop_front();
        assert(first_elem == 12);
        assert(vector.len() == 6);

        vector.insert(x as u32 - 2, 20);
        assert(vector.get(2) == 20);
        assert(vector.get(6) == 30);
        assert(vector.len() == 7);

        let removed_elem = vector.remove(x as u32 - 1);
        // The deconstructed tuple assigns to the vector but is not seen outside of the if statement
        // without a direct assignment


        assert(removed_elem == 1);
        assert(vector.len() == 6);
    } else {
        assert(vector.get(x) == 0);
         vector.push_back(20);
    }

    assert(vector.len() == 6);
    assert(vector.get(vector.len() - 1) == 30);
}

fn dynamic_vector_merge_else(mut vector: UnboundedVec<Field>, x: u32) {
    if x as u32 > 10 {
        assert(vector.get(x) == 0);
        vector.set(x, 2);
    } else {
        assert(vector.get(x) == 4);
        vector.set(x, vector.get(x) - 2);
         vector.push_back(10);
    }
    assert(vector.len() == 6);
    assert(vector.get(vector.len() - 1) == 10);

    vector.push_back(20);
    assert(vector.len() == 7);
    assert(vector.get(vector.len() - 1) == 20);
}

fn dynamic_vector_index_set_nested_if_else_else(mut vector: UnboundedVec<Field>, x: u32, y: u32) {
    assert(vector.get(x) == 4);
    assert(vector.get(y) == 1);
    vector.set(y, 0);
    assert(vector.get(x) == 4);
    assert(vector.get(1) == 0);
    if x as u32 < 10 {
        vector.set(x, vector.get(x) - 2);
        if y != 1 {
            vector.set(x, vector.get(x) + 20);
        } else {
            if x == 5 {
                // We should not hit this case
                assert(vector.get(x) == 22);
            } else {
                vector.set(x, 10);
               vector.push_back(15);
                assert(vector.len() == 6);
            }
            assert(vector.get(4) == 10);
        }
    } else {
        vector.set(x, 0);
    }
    assert(vector.get(4) == 10);
    assert(vector.len() == 6);
    assert(vector.get(vector.len() - 1) == 15);

   vector.push_back(20);
    assert(vector.len() == 7);
    assert(vector.get(vector.len() - 1) == 20);
}

fn dynamic_vector_index_set_nested_if_else_if(mut vector: UnboundedVec<Field>, x: u32, y: u32) {
    assert(vector.get(x) == 4);
    assert(vector.get(y) == 2);
    vector.set(y, 0);
    assert(vector.get(x) == 4);
    assert(vector.get(2) == 0);
    if x as u32 < 10 {
        vector.set(x, vector.get(x) - 2);
        // TODO: this panics as we have a load for the vector in flattening
        if y == 1 {
            vector.set(x, vector.get(x) + 20);
        } else {
            if x == 4 {
                vector.set(x, 5);
            }
            assert(vector.get(4) == 5);
        }
    } else {
        vector.set(x, 0);
    }
    assert(vector.get(4) == 5);
}

fn dynamic_vector_merge_two_ifs(mut vector: UnboundedVec<Field>, x: u32) {
    if x as u32 > 10 {
        assert(vector.get(x) == 0);
        vector.set(x, 2);
    } else {
        assert(vector.get(x) == 4);
        vector.set(x, vector.get(x) - 2);
       vector.push_back(10);
    }

    assert(vector.len() == 6);
    assert(vector.get(vector.len() - 1) == 10);

    if x == 20 {
       vector.push_back(20);
    }

    vector.push_back(15);

    assert(vector.len() == 7);
    assert(vector.get(vector.len() - 1) == 15);

    // vector.push_back(20);
    // assert(vector.len() == 8);
    // assert(vector.get(vector.len() - 1) == 20);
}

fn dynamic_vector_merge_mutate_between_ifs(mut vector: UnboundedVec<Field>, x: u32, y: u32) {
    if x != y {
        vector.set(x, 50);
        vector.push_back(y as Field);
        vector.push_back(x as Field);
    } else {
        vector.set(x, vector.get(x) - 2);
        vector.push_back(x as Field);
    }

    vector.push_back(30);
    assert(vector.len() == 8);

    if x == 20 {
        vector.push_back(20);
    }

    vector.push_back(15);

    if x != 20 {
        vector.push_back(50);
    }

    vector.push_back(60);
    assert(vector.len() == 11);
    assert(vector.get(x) == 50);
    assert(vector.get(vector.len() - 4) == 30);
    assert(vector.get(vector.len() - 3) == 15);
    assert(vector.get(vector.len() - 2) == 50);
    assert(vector.get(vector.len() - 1) == 60);
}

fn dynamic_vector_merge_push_then_pop(mut vector: UnboundedVec<Field>, x: u32, y: u32) {
    if x != y {
        vector.set(x, 5);
        vector.push_back(y as Field);
        vector.push_back(x as Field);
        assert(vector.len() == 7);

        let elem = vector.pop_back();
        assert(vector.len() == 6);
        assert(elem == x as Field);

    } else {
        vector.push_back(x as Field);
    }

 vector.push_back(30);
    assert(vector.len() == 7);

    if x == 20 {
         vector.push_back(20);
    }

    let elem = vector.pop_back();
    assert(elem == 30);

    let  elem = vector.pop_back();
    assert(elem == y as Field);
}
