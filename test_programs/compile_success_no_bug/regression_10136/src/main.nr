fn main(array: [Field; 1], cond: bool) -> pub u32 {
    let slice = array.as_slice();
    let mut slice = std::hint::black_box(slice);
    if cond {
        slice = slice.push_back(1);
    }
    slice.len()
}
