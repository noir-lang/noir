fn main() {
    let empty_slice: [u8] = &[];

    if empty_slice != &[] {
        let _ = empty_slice.pop_front();
    }

    if empty_slice.len() != 0 {
        let _ = empty_slice.pop_front();
    }
}
