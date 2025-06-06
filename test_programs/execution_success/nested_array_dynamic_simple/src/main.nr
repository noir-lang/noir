fn main(x: u32) {
    // x = 3
    let array: [[(Field, [Field; 1], [Field; 1]); 1]; 1] = [[(1, [2], [3])]];

    let fetched_value = array[x - 3];
    assert(fetched_value[0].0 == 1);
    assert(fetched_value[0].1[0] == 2);
    assert(fetched_value[0].2[0] == 3);
}
