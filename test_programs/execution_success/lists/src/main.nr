use std::list;

fn main(x: Field, y: pub Field) {
    let mut list = &[0; 2];
    assert(list[0] == 0);
    assert(list[0] != 1);
    list[0] = x;
    assert(list[0] == x);

    let list_plus_10 = list.push_back(y);
    assert(list_plus_10[2] == 10);
    assert(list_plus_10[2] != 8);
    assert(list_plus_10.len() == 3);

    let mut new_list = &[];
    for i in 0..5 {
        new_list = new_list.push_back(i);
    }
    assert(new_list.len() == 5);

    new_list = new_list.push_front(20);
    assert(new_list[0] == 20);
    assert(new_list.len() == 6);

    let (popped_list, last_elem) = new_list.pop_back();
    assert(last_elem == 4);
    assert(popped_list.len() == 5);

    let (first_elem, rest_of_list) = popped_list.pop_front();
    assert(first_elem == 20);
    assert(rest_of_list.len() == 4);

    new_list = rest_of_list.insert(2, 100);
    assert(new_list[2] == 100);
    assert(new_list[4] == 3);
    assert(new_list.len() == 5);

    let (remove_list, removed_elem) = new_list.remove(3);
    assert(removed_elem == 2);
    assert(remove_list[3] == 3);
    assert(remove_list.len() == 4);

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
    regression_merge_lists(x, y);
    regression_2370();

    regression_4418(x);
    regression_list_call_result(x, y);
    regression_4506();
    regression_list_to_brillig(x as u32 % 3);
}

// Ensure that lists of struct/tuple values work.
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
fn regression_merge_lists(x: Field, y: Field) {
    merge_lists_if(x, y);
    merge_lists_else(x);
}

fn merge_lists_if(x: Field, y: Field) {
    let list = merge_lists_return(x, y);
    assert(list.len() == 3);
    assert(list[2] == 10);

    let list = merge_lists_mutate(x, y);
    assert(list.len() == 4);
    assert(list[3] == 5);

    let list = merge_lists_mutate_in_loop(x, y);
    assert(list.len() == 7);
    assert(list[6] == 4);

    let list = merge_lists_mutate_two_ifs(x, y);
    assert(list.len() == 6);
    assert(list[3] == 5);
    assert(list[4] == 15);
    assert(list[5] == 30);

    let list = merge_lists_mutate_between_ifs(x, y);
    assert(list.len() == 8);
    assert(list[3] == 5);
    assert(list[4] == 30);
    assert(list[5] == 15);
    assert(list[6] == 50);
    assert(list[7] == 60);

    merge_lists_push_then_pop(x, y);

    let list = merge_lists_push_then_insert(x, y);
    assert(list.len() == 7);
    assert(list[1] == 50);
    assert(list[2] == 0);
    assert(list[5] == 30);
    assert(list[6] == 100);

    let list = merge_lists_remove_between_ifs(x, y);
    assert(list.len() == 5);
}

fn merge_lists_else(x: Field) {
    let list = merge_lists_return(x, 5);
    assert(list[0] == 0);
    assert(list[1] == 0);
    assert(list.len() == 2);

    let list = merge_lists_mutate(x, 5);
    assert(list[2] == 5);
    assert(list.len() == 3);

    let list = merge_lists_mutate_in_loop(x, 5);
    assert(list[2] == 5);
    assert(list.len() == 3);
}

// Test returning a merged list without a mutation
fn merge_lists_return(x: Field, y: Field) -> [Field] {
    let list = &[0; 2];
    if x != y {
        if x != 20 {
            list.push_back(y)
        } else {
            list
        }
    } else {
        list
    }
}

// Test mutating a list inside of an if statement
fn merge_lists_mutate(x: Field, y: Field) -> [Field] {
    let mut list = &[0; 2];
    if x != y {
        list = list.push_back(y);
        list = list.push_back(x);
    } else {
        list = list.push_back(x);
    }
    list
}

// Test mutating a list inside of a loop in an if statement
fn merge_lists_mutate_in_loop(x: Field, y: Field) -> [Field] {
    let mut list = &[0; 2];
    if x != y {
        for i in 0..5 {
            list = list.push_back(i as Field);
        }
    } else {
        list = list.push_back(x);
    }
    list
}

fn merge_lists_mutate_two_ifs(x: Field, y: Field) -> [Field] {
    let mut list = &[0; 2];
    if x != y {
        list = list.push_back(y);
        list = list.push_back(x);
    } else {
        list = list.push_back(x);
    }

    if x == 20 {
        list = list.push_back(20);
    }

    list = list.push_back(15);
    list = list.push_back(30);

    list
}

fn merge_lists_mutate_between_ifs(x: Field, y: Field) -> [Field] {
    let mut list = &[0; 2];
    if x != y {
        list = list.push_back(y);
        list = list.push_back(x);
    } else {
        list = list.push_back(x);
    }

    list = list.push_back(30);

    if x == 20 {
        list = list.push_back(20);
    }

    list = list.push_back(15);

    if x != 20 {
        list = list.push_back(50);
    }

    list = list.push_back(60);

    list
}

fn merge_lists_push_then_pop(x: Field, y: Field) {
    let mut list = &[0; 2];
    if x != y {
        list = list.push_back(y);
        list = list.push_back(x);
    } else {
        list = list.push_back(x);
    }

    list = list.push_back(30);

    if x == 20 {
        list = list.push_back(20);
    }

    let (list, elem) = list.pop_back();
    assert(list.len() == 4);
    assert(elem == 30);

    let (list, elem) = list.pop_back();
    assert(list.len() == 3);
    assert(elem == x);
}

fn merge_lists_push_then_insert(x: Field, y: Field) -> [Field] {
    let mut list = &[0; 2];
    if x != y {
        list = list.push_back(y);
        list = list.push_back(x);
    } else {
        list = list.push_back(x);
    }

    list = list.push_back(30);

    if x == 20 {
        list = list.push_back(20);
        list = list.push_back(15);
    }

    list = list.insert(1, 50);
    // Test that we can use list insert the same as list push back
    list = list.insert(6, 100);

    list
}

fn merge_lists_remove_between_ifs(x: Field, y: Field) -> [Field] {
    let mut list = &[0; 2];
    if x != y {
        list = list.push_back(y);
        list = list.push_back(x);
    } else {
        list = list.push_back(x);
    }

    let (mut list, elem) = list.remove(2);
    assert(elem == y);

    if x == 20 {
        list = list.push_back(20);
    }

    list = list.push_back(15);

    if x != 20 {
        list = list.push_back(50);
    }

    list
}
// Previously, we'd get a type error when trying to assign an array of a different size to
// an existing array variable. Now, we infer the variable must be a list.
fn regression_2370() {
    let mut list = &[];
    list = &[1, 2, 3];
}

fn regression_4418(x: Field) {
    let mut crash: [u8; 32] = x.to_be_bytes();

    if x != 0 {
        crash[0] = 10;
    }
}

fn regression_list_call_result(x: Field, y: Field) {
    let mut list = merge_lists_return(x, y);
    if x != 0 {
        list = list.push_back(5);
        list = list.push_back(10);
    } else {
        list = list.push_back(5);
    }
    assert(list.len() == 5);
    assert(list[0] == 0);
    assert(list[1] == 0);
    assert(list[2] == 10);
    assert(list[3] == 5);
    assert(list[4] == 10);
}

fn regression_4506() {
    let list: [Field] = &[1, 2, 3];
    assert(list == list);
}

struct foo {
    a: u32,
    b: u32,
    c: [Field; 3],
}
fn regression_list_to_brillig(x: u32) {
    let mut list = &[foo { a: 0, b: 0, c: [0, 0, 0] }, foo { a: 1, b: 1, c: [0, 0, 0] }];
    let mut foo1 = foo { a: 10, b: 10, c: [0, 0, 0] };
    foo1.c[x] = 7;
    list = list.push_back(foo1);
    list = list.push_back(foo { a: 0, b: 0, c: [0, 0, 0] });
    let (new_list, _) = list.pop_back();
    list = new_list;
    // Safety: for testing purposes
    let ok = unsafe { bar(list) };
    assert(ok);
}

unconstrained fn bar(list: [foo]) -> bool {
    if list.len() != 0 {
        list[0].a == 0
    } else {
        false
    }
}
