struct foo {
    a: u32,
    b: u32,
    c: [Field; 3],
}
fn main(cond: bool) {
    // Issue 10170
    let mut list = &[0, 1];
    if cond {
        list = list.push_back(10);
    }
    for i in 0..2 {
        list = list.push_back(i);
        assert_eq(list[list.len() - 1], i);
        let (new_list, _) = list.pop_back();
        list = new_list;
    }

    // Same, but with nested lists
    let mut list = &[foo { a: 0, b: 0, c: [0, 0, 0] }, foo { a: 1, b: 1, c: [0, 0, 0] }];
    if cond {
        list = list.push_back(foo { a: 10, b: 10, c: [0, 0, 0] });
    }
    for i in 0..2 {
        list = list.push_back(foo { a: i, b: i, c: [0, 0, 0] });
        assert_eq(list[list.len() - 1].a, i);
        let (new_list, _) = list.pop_back();
        list = new_list;
    }

    // push after pop
    let mut list = &[0, 1];
    if !cond {
        list = list.push_back(10);
    }
    let (new_list, _) = list.pop_back();
    list = new_list.push_back(11);
    assert_eq(list[list.len() - 1], 11);
}
