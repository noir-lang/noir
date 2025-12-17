fn main(x: u32) {
    // The parameters to this function must come directly from witness values (inputs to main).
    regression_dynamic_list_index(x - 1, x - 4);
}

fn regression_dynamic_list_index(x: u32, y: u32) {
    let mut list = &[];
    for i in 0..5 {
        list = list.push_back(i as Field);
    }
    assert(list.len() == 5);

    dynamic_list_index_set_if(list, x, y);
    dynamic_list_index_set_else(list, x, y);
    dynamic_list_index_set_nested_if_else_else(list, x, y);
    dynamic_list_index_set_nested_if_else_if(list, x, y + 1);
    dynamic_list_index_if(list, x);
    dynamic_array_index_if([0, 1, 2, 3, 4], x);
    dynamic_list_index_else(list, x);

    dynamic_list_merge_if(list, x);
    dynamic_list_merge_else(list, x);
    dynamic_list_merge_two_ifs(list, x);
    dynamic_list_merge_mutate_between_ifs(list, x, y);
    dynamic_list_merge_push_then_pop(list, x, y);
}

fn dynamic_list_index_set_if(mut list: [Field], x: u32, y: u32) {
    assert(list[x] == 4);
    assert(list[y] == 1);
    list[y] = 0;
    assert(list[x] == 4);
    assert(list[1] == 0);
    if x as u32 < 10 {
        assert(list[x] == 4);
        list[x] = list[x] - 2;
        list[x - 1] = list[x];
    } else {
        list[x] = 0;
    }
    assert(list[3] == 2);
    assert(list[4] == 2);
}

fn dynamic_list_index_set_else(mut list: [Field], x: u32, y: u32) {
    assert(list[x] == 4);
    assert(list[y] == 1);
    list[y] = 0;
    assert(list[x] == 4);
    assert(list[1] == 0);
    if x as u32 > 10 {
        assert(list[x] == 4);
        list[x] = list[x] - 2;
        list[x - 1] = list[x];
    } else {
        list[x] = 0;
    }
    assert(list[4] == 0);
}
// This tests the case of missing a store instruction in the else branch
// of merging lists
fn dynamic_list_index_if(mut list: [Field], x: u32) {
    if x as u32 < 10 {
        assert(list[x] == 4);
        list[x] = list[x] - 2;
    } else {
        assert(list[x] == 0);
    }
    assert(list[4] == 2);
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
// of merging lists
fn dynamic_list_index_else(mut list: [Field], x: u32) {
    if x as u32 > 10 {
        assert(list[x] == 0);
    } else {
        assert(list[x] == 4);
        list[x] = list[x] - 2;
    }
    assert(list[4] == 2);
}

fn dynamic_list_merge_if(mut list: [Field], x: u32) {
    if x as u32 < 10 {
        assert(list[x] == 4);
        list[x] = list[x] - 2;

        list = list.push_back(10);
        // Having an array set here checks whether we appropriately
        // handle a list length that is not yet resolving to a constant
        // during flattening
        list[x] = 10;
        assert(list[list.len() - 1] == 10);
        assert(list.len() == 6);

        list[x] = 20;
        list[x] = list[x] + 10;

        list = list.push_front(11);
        assert(list[0] == 11);
        assert(list.len() == 7);
        assert(list[5] == 30);

        list = list.push_front(12);
        assert(list[0] == 12);
        assert(list.len() == 8);
        assert(list[6] == 30);

        let (popped_list, last_elem) = list.pop_back();
        assert(last_elem == 10);
        assert(popped_list.len() == 7);

        let (first_elem, rest_of_list) = popped_list.pop_front();
        assert(first_elem == 12);
        assert(rest_of_list.len() == 6);

        list = rest_of_list.insert(x as u32 - 2, 20);
        assert(list[2] == 20);
        assert(list[6] == 30);
        assert(list.len() == 7);

        let (removed_list, removed_elem) = list.remove(x as u32 - 1);
        // The deconstructed tuple assigns to the list but is not seen outside of the if statement
        // without a direct assignment
        list = removed_list;

        assert(removed_elem == 1);
        assert(list.len() == 6);
    } else {
        assert(list[x] == 0);
        list = list.push_back(20);
    }

    assert(list.len() == 6);
    assert(list[list.len() - 1] == 30);
}

fn dynamic_list_merge_else(mut list: [Field], x: u32) {
    if x as u32 > 10 {
        assert(list[x] == 0);
        list[x] = 2;
    } else {
        assert(list[x] == 4);
        list[x] = list[x] - 2;
        list = list.push_back(10);
    }
    assert(list.len() == 6);
    assert(list[list.len() - 1] == 10);

    list = list.push_back(20);
    assert(list.len() == 7);
    assert(list[list.len() - 1] == 20);
}

fn dynamic_list_index_set_nested_if_else_else(mut list: [Field], x: u32, y: u32) {
    assert(list[x] == 4);
    assert(list[y] == 1);
    list[y] = 0;
    assert(list[x] == 4);
    assert(list[1] == 0);
    if x as u32 < 10 {
        list[x] = list[x] - 2;
        if y != 1 {
            list[x] = list[x] + 20;
        } else {
            if x == 5 {
                // We should not hit this case
                assert(list[x] == 22);
            } else {
                list[x] = 10;
                list = list.push_back(15);
                assert(list.len() == 6);
            }
            assert(list[4] == 10);
        }
    } else {
        list[x] = 0;
    }
    assert(list[4] == 10);
    assert(list.len() == 6);
    assert(list[list.len() - 1] == 15);

    list = list.push_back(20);
    assert(list.len() == 7);
    assert(list[list.len() - 1] == 20);
}

fn dynamic_list_index_set_nested_if_else_if(mut list: [Field], x: u32, y: u32) {
    assert(list[x] == 4);
    assert(list[y] == 2);
    list[y] = 0;
    assert(list[x] == 4);
    assert(list[2] == 0);
    if x as u32 < 10 {
        list[x] = list[x] - 2;
        // TODO: this panics as we have a load for the list in flattening
        if y == 1 {
            list[x] = list[x] + 20;
        } else {
            if x == 4 {
                list[x] = 5;
            }
            assert(list[4] == 5);
        }
    } else {
        list[x] = 0;
    }
    assert(list[4] == 5);
}

fn dynamic_list_merge_two_ifs(mut list: [Field], x: u32) {
    if x as u32 > 10 {
        assert(list[x] == 0);
        list[x] = 2;
    } else {
        assert(list[x] == 4);
        list[x] = list[x] - 2;
        list = list.push_back(10);
    }

    assert(list.len() == 6);
    assert(list[list.len() - 1] == 10);

    if x == 20 {
        list = list.push_back(20);
    }

    list = list.push_back(15);

    assert(list.len() == 7);
    assert(list[list.len() - 1] == 15);

    list = list.push_back(20);
    assert(list.len() == 8);
    assert(list[list.len() - 1] == 20);
}

fn dynamic_list_merge_mutate_between_ifs(mut list: [Field], x: u32, y: u32) {
    if x != y {
        list[x] = 50;
        list = list.push_back(y as Field);
        list = list.push_back(x as Field);
    } else {
        list[x] = list[x] - 2;
        list = list.push_back(x as Field);
    }

    list = list.push_back(30);
    assert(list.len() == 8);

    if x == 20 {
        list = list.push_back(20);
    }

    list = list.push_back(15);

    if x != 20 {
        list = list.push_back(50);
    }

    list = list.push_back(60);
    assert(list.len() == 11);
    assert(list[x] == 50);
    assert(list[list.len() - 4] == 30);
    assert(list[list.len() - 3] == 15);
    assert(list[list.len() - 2] == 50);
    assert(list[list.len() - 1] == 60);
}

fn dynamic_list_merge_push_then_pop(mut list: [Field], x: u32, y: u32) {
    if x != y {
        list[x] = 5;
        list = list.push_back(y as Field);
        list = list.push_back(x as Field);
        assert(list.len() == 7);

        let (popped_list, elem) = list.pop_back();
        assert(list.len() == 7);
        assert(elem == x as Field);
        list = popped_list;
    } else {
        list = list.push_back(x as Field);
    }

    list = list.push_back(30);
    assert(list.len() == 7);

    if x == 20 {
        list = list.push_back(20);
    }

    let (list, elem) = list.pop_back();
    assert(elem == 30);

    let (_, elem) = list.pop_back();
    assert(elem == y as Field);
}
