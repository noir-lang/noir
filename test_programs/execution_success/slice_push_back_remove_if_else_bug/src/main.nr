fn main(x: u32) {
    let mut slice = &[2, 3];
    slice[x] = 4;
    if x == 0 {
        slice = slice.push_back(5);
    }
    assert(slice[x] == 4);
}
