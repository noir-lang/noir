fn main(x: bool) {
    println(f"x's value: {x}");
    if x {
        let empty_vector: [u8] = [].as_vector();
        let _ = empty_vector.pop_front();
    }
}
