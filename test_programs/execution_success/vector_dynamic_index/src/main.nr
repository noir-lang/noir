fn main(x: u32) {
    // The parameters to this function must come directly from witness values (inputs to main).
    regression_dynamic_vector_index(x - 1, x - 4);
}

fn regression_dynamic_vector_index(x: u32, y: u32) {
    let mut vector = &[];
    for i in 0..5 {
        vector = vector.push_back(i as Field);
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

fn dynamic_vector_index_set_if(mut vector: [Field], x: u32, y: u32) {
    assert(vector[x] == 4);
    assert(vector[y] == 1);
    vector[y] = 0;
    assert(vector[x] == 4);
    assert(vector[1] == 0);
    if x as u32 < 10 {
        assert(vector[x] == 4);
        vector[x] = vector[x] - 2;
        vector[x - 1] = vector[x];
    } else {
        vector[x] = 0;
    }
    assert(vector[3] == 2);
    assert(vector[4] == 2);
}

fn dynamic_vector_index_set_else(mut vector: [Field], x: u32, y: u32) {
    assert(vector[x] == 4);
    assert(vector[y] == 1);
    vector[y] = 0;
    assert(vector[x] == 4);
    assert(vector[1] == 0);
    if x as u32 > 10 {
        assert(vector[x] == 4);
        vector[x] = vector[x] - 2;
        vector[x - 1] = vector[x];
    } else {
        vector[x] = 0;
    }
    assert(vector[4] == 0);
}
// This tests the case of missing a store instruction in the else branch
// of merging vectors
fn dynamic_vector_index_if(mut vector: [Field], x: u32) {
    if x as u32 < 10 {
        assert(vector[x] == 4);
        vector[x] = vector[x] - 2;
    } else {
        assert(vector[x] == 0);
    }
    assert(vector[4] == 2);
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
fn dynamic_vector_index_else(mut vector: [Field], x: u32) {
    if x as u32 > 10 {
        assert(vector[x] == 0);
    } else {
        assert(vector[x] == 4);
        vector[x] = vector[x] - 2;
    }
    assert(vector[4] == 2);
}

fn dynamic_vector_merge_if(mut vector: [Field], x: u32) {
    if x as u32 < 10 {
        assert(vector[x] == 4);
        vector[x] = vector[x] - 2;

        vector = vector.push_back(10);
        // Having an array set here checks whether we appropriately
        // handle a vector length that is not yet resolving to a constant
        // during flattening
        vector[x] = 10;
        assert(vector[vector.len() - 1] == 10);
        assert(vector.len() == 6);

        vector[x] = 20;
        vector[x] = vector[x] + 10;

        vector = vector.push_front(11);
        assert(vector[0] == 11);
        assert(vector.len() == 7);
        assert(vector[5] == 30);

        vector = vector.push_front(12);
        assert(vector[0] == 12);
        assert(vector.len() == 8);
        assert(vector[6] == 30);

        let (popped_vector, last_elem) = vector.pop_back();
        assert(last_elem == 10);
        assert(popped_vector.len() == 7);

        let (first_elem, rest_of_vector) = popped_vector.pop_front();
        assert(first_elem == 12);
        assert(rest_of_vector.len() == 6);

        vector = rest_of_vector.insert(x as u32 - 2, 20);
        assert(vector[2] == 20);
        assert(vector[6] == 30);
        assert(vector.len() == 7);

        let (removed_vector, removed_elem) = vector.remove(x as u32 - 1);
        // The deconstructed tuple assigns to the vector but is not seen outside of the if statement
        // without a direct assignment
        vector = removed_vector;

        assert(removed_elem == 1);
        assert(vector.len() == 6);
    } else {
        assert(vector[x] == 0);
        vector = vector.push_back(20);
    }

    assert(vector.len() == 6);
    assert(vector[vector.len() - 1] == 30);
}

fn dynamic_vector_merge_else(mut vector: [Field], x: u32) {
    if x as u32 > 10 {
        assert(vector[x] == 0);
        vector[x] = 2;
    } else {
        assert(vector[x] == 4);
        vector[x] = vector[x] - 2;
        vector = vector.push_back(10);
    }
    assert(vector.len() == 6);
    assert(vector[vector.len() - 1] == 10);

    vector = vector.push_back(20);
    assert(vector.len() == 7);
    assert(vector[vector.len() - 1] == 20);
}

fn dynamic_vector_index_set_nested_if_else_else(mut vector: [Field], x: u32, y: u32) {
    assert(vector[x] == 4);
    assert(vector[y] == 1);
    vector[y] = 0;
    assert(vector[x] == 4);
    assert(vector[1] == 0);
    if x as u32 < 10 {
        vector[x] = vector[x] - 2;
        if y != 1 {
            vector[x] = vector[x] + 20;
        } else {
            if x == 5 {
                // We should not hit this case
                assert(vector[x] == 22);
            } else {
                vector[x] = 10;
                vector = vector.push_back(15);
                assert(vector.len() == 6);
            }
            assert(vector[4] == 10);
        }
    } else {
        vector[x] = 0;
    }
    assert(vector[4] == 10);
    assert(vector.len() == 6);
    assert(vector[vector.len() - 1] == 15);

    vector = vector.push_back(20);
    assert(vector.len() == 7);
    assert(vector[vector.len() - 1] == 20);
}

fn dynamic_vector_index_set_nested_if_else_if(mut vector: [Field], x: u32, y: u32) {
    assert(vector[x] == 4);
    assert(vector[y] == 2);
    vector[y] = 0;
    assert(vector[x] == 4);
    assert(vector[2] == 0);
    if x as u32 < 10 {
        vector[x] = vector[x] - 2;
        // TODO: this panics as we have a load for the vector in flattening
        if y == 1 {
            vector[x] = vector[x] + 20;
        } else {
            if x == 4 {
                vector[x] = 5;
            }
            assert(vector[4] == 5);
        }
    } else {
        vector[x] = 0;
    }
    assert(vector[4] == 5);
}

fn dynamic_vector_merge_two_ifs(mut vector: [Field], x: u32) {
    if x as u32 > 10 {
        assert(vector[x] == 0);
        vector[x] = 2;
    } else {
        assert(vector[x] == 4);
        vector[x] = vector[x] - 2;
        vector = vector.push_back(10);
    }

    assert(vector.len() == 6);
    assert(vector[vector.len() - 1] == 10);

    if x == 20 {
        vector = vector.push_back(20);
    }

    vector = vector.push_back(15);

    assert(vector.len() == 7);
    assert(vector[vector.len() - 1] == 15);

    vector = vector.push_back(20);
    assert(vector.len() == 8);
    assert(vector[vector.len() - 1] == 20);
}

fn dynamic_vector_merge_mutate_between_ifs(mut vector: [Field], x: u32, y: u32) {
    if x != y {
        vector[x] = 50;
        vector = vector.push_back(y as Field);
        vector = vector.push_back(x as Field);
    } else {
        vector[x] = vector[x] - 2;
        vector = vector.push_back(x as Field);
    }

    vector = vector.push_back(30);
    assert(vector.len() == 8);

    if x == 20 {
        vector = vector.push_back(20);
    }

    vector = vector.push_back(15);

    if x != 20 {
        vector = vector.push_back(50);
    }

    vector = vector.push_back(60);
    assert(vector.len() == 11);
    assert(vector[x] == 50);
    assert(vector[vector.len() - 4] == 30);
    assert(vector[vector.len() - 3] == 15);
    assert(vector[vector.len() - 2] == 50);
    assert(vector[vector.len() - 1] == 60);
}

fn dynamic_vector_merge_push_then_pop(mut vector: [Field], x: u32, y: u32) {
    if x != y {
        vector[x] = 5;
        vector = vector.push_back(y as Field);
        vector = vector.push_back(x as Field);
        assert(vector.len() == 7);

        let (popped_vector, elem) = vector.pop_back();
        assert(vector.len() == 7);
        assert(elem == x as Field);
        vector = popped_vector;
    } else {
        vector = vector.push_back(x as Field);
    }

    vector = vector.push_back(30);
    assert(vector.len() == 7);

    if x == 20 {
        vector = vector.push_back(20);
    }

    let (vector, elem) = vector.pop_back();
    assert(elem == 30);

    let (_, elem) = vector.pop_back();
    assert(elem == y as Field);
}
