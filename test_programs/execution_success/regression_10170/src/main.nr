fn main(cond: bool) {
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
}
