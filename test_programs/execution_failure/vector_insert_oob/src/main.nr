fn main(dummy_val: u32, dummy_arr: [u32; 1], x: u32, y: u32) {
    let mut vector = dummy_arr.as_vector();
    if x == 3 {
        vector = vector.push_back(dummy_val);
    } else {
        // This else branch executes when x != 3
        // Should fail - inserting into empty vector with active predicate
        let (vector, _) = vector.remove(0);
        let s2 = vector.insert(0, 5);
        assert_eq(x, y);
        assert_eq(s2.len(), 1);
    }
    assert_eq(vector.len(), 2);
}
