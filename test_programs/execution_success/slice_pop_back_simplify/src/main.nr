pub fn main(x: u32, y: u32, i: u32) -> pub ((u32, u32), u32) {
    let mut slice = &[(x, 2u32)];
    if (i < 10) {
        let (new_slice, _) = slice.pop_back();
        slice = new_slice.push_back((y, 4u32));
    }
    (slice[i], slice.len())
}
