struct foo {
    a: u32,
    b: u32,
    c: [Field; 3],
}
fn main(cond: bool) {
    // Issue 10170
    let mut slice = &[0, 1];
    if cond {
        slice = slice.push_back(10);
    }
    for i in 0..2 {
        slice = slice.push_back(i);
        assert_eq(slice[slice.len() - 1], i);
        let (new_slice, _) = slice.pop_back();
        slice = new_slice;
    }

    // Same, but with nested slices
    let mut slice = &[foo { a: 0, b: 0, c: [0, 0, 0] }, foo { a: 1, b: 1, c: [0, 0, 0] }];
    if cond {
        slice = slice.push_back(foo { a: 10, b: 10, c: [0, 0, 0] });
    }
    for i in 0..2 {
        slice = slice.push_back(foo { a: i, b: i, c: [0, 0, 0] });
        assert_eq(slice[slice.len() - 1].a, i);
        let (new_slice, _) = slice.pop_back();
        slice = new_slice;
    }

    // push after pop
    let mut slice = &[0, 1];
    if !cond {
        slice = slice.push_back(10);
    }
    let (new_slice, _) = slice.pop_back();
    slice = new_slice.push_back(11);
    assert_eq(slice[slice.len() - 1], 11);
}
