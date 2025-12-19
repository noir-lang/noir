fn main(dummy_val: u32, dummy_arr: [u32; 3], x: u32, y: u32) {
    let arr = [];
    let s = arr.as_slice();
    let s = s.push_back((dummy_val, dummy_arr));
    let s = s.push_back((dummy_val, dummy_arr));
    let s = s.push_back((x, dummy_arr));

    let (s2, x) = s.pop_back();

    assert_eq(x.0, y);
    assert_eq(s2[0].0, dummy_val);
    assert_eq(s2[0].1, dummy_arr);
    assert_eq(s2[1].0, dummy_val);
    assert_eq(s2[1].1, dummy_arr);
    assert_eq(s2.len(), 2);
}
