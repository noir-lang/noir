fn main(mut array: [Field; 2], i: u32) {
    assert_eq(array[i - 1], 5);
    assert_eq(array[i], 10);

    array[i] = 2;

    let array2 = [array, array];

    assert_eq(array2[0][0], 5);
    assert_eq(array2[0][i], 2);
    assert_eq(array2[i][0], 5);
    assert_eq(array2[i][i], 2);
}
