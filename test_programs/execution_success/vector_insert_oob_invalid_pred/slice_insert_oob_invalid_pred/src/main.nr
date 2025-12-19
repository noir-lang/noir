fn main(dummy_val: u32, dummy_arr: [u32; 1], x: u32, y: u32) {
    let mut slice = dummy_arr.as_slice();
    if x == 3 {
        slice = slice.push_back(dummy_val);
    } else {
        let (slice, _) = slice.remove(0);
        let s2 = slice.insert(0, 5);
        assert_eq(x, y);
        assert_eq(s2.len(), 1);
    }
    assert_eq(slice.len(), 2);
}
