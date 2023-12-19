fn main() {
    let slice = [1, 2, 3];
    let array: [Field; 3] = slice.as_array();

    assert(array[0] == 1);
    assert(array[1] == 2);
    assert(array[2] == 3);
}
