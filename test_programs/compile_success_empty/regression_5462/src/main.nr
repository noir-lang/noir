fn main() {
    let empty_vector: [u8] = [].as_vector();

    if empty_vector != [].as_vector() {
        let _ = empty_vector.pop_front();
    }

    if empty_vector.len() != 0 {
        let _ = empty_vector.pop_front();
    }
}
