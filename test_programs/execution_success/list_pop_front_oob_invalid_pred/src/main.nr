fn main(dummy_val: u32, dummy_arr: [u32; 1], x: u32, y: u32) {
    let mut list = dummy_arr.as_list();
    if x == 3 {
        list = list.push_back(dummy_val);
    } else {
        let (_, list) = list.pop_front();
        let (x, s2) = list.pop_front();
        assert_eq(x, y);
        assert_eq(s2.len(), 1);
    }
    assert_eq(list.len(), 2);
}
