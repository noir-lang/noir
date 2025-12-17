fn main(dummy_val: u32, dummy_arr: [u32; 1], x: u32, y: u32) {
    let mut list = dummy_arr.as_list();
    if x == 3 {
        list = list.push_back(dummy_val);
    } else {
        // This else branch executes when x != 3
        // Should fail - popping from empty list with active predicate
        let (list, _) = list.pop_back();
        let (s2, x) = list.pop_back();
        assert_eq(x, y);
        assert_eq(s2.len(), 1);
    }
    assert_eq(list.len(), 2);
}
