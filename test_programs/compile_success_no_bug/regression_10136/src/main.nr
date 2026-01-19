fn main(array: [Field; 1], cond: bool) -> pub u32 {
    let vector = array.as_vector();
    let mut vector = std::hint::black_box(vector);
    if cond {
        vector = vector.push_back(1);
    }
    vector.len()
}
