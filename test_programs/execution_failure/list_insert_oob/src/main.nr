fn main(dummy_val: u32, dummy_arr: [u32; 1], x: u32, y: u32) {
    let mut list = dummy_arr.as_list();
    if x == 3 {
        list = list.push_back(dummy_val);
    } else {
        // This else branch executes when x != 3
        // Should fail - inserting into empty list with active predicate
        let (list, _) = list.remove(0);
        let s2 = list.insert(0, 5);
        assert_eq(x, y);
        assert_eq(s2.len(), 1);
    }
    assert_eq(list.len(), 2);
}
