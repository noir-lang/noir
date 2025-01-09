fn main(array_a: [Field; 100], array_b: [Field; 100], is_b: bool, whatever_index: u32) {
    let mut array = array_a;
    if is_b {
        array = array_b;
    }

    assert(array[whatever_index] == 0);
}
