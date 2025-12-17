fn main(x: u32, array: [Field; 5]) {
    insert(x, array);
    insert_dynamic_array(x, array);
}

fn insert(x: u32, array: [Field; 5]) {
    let mut list = array.as_list();

    assert_eq(x, 5);
    list = list.insert(x - 2, 10);
    assert_eq(list.len(), 6);
    let expected = &[1, 2, 3, 10, 4, 5];
    for i in 0..6 {
        assert_eq(list[i], expected[i]);
    }
}

fn insert_dynamic_array(x: u32, array: [Field; 5]) {
    let mut value_to_insert = array;
    value_to_insert[x - 1] = 10;
    let mut list = &[array, array, array];
    list = list.insert(x - 3, value_to_insert);
    assert_eq(list.len(), 4);
    let expected = &[[1, 2, 3, 4, 5], [1, 2, 3, 4, 5], [1, 2, 3, 4, 10], [1, 2, 3, 4, 5]];
    for i in 0..3 {
        for j in 0..5 {
            assert_eq(list[i][j], expected[i][j]);
        }
    }
}
