fn main() {
    let empty_vector: [u8] = &[];

    if empty_vector != &[] {
        let _ = empty_vector.pop_front();
    }

    if empty_vector.len() != 0 {
        let _ = empty_vector.pop_front();
    }
}
