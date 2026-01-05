pub fn main(x: u32, y: u32, i: u32) -> pub ((u32, u32), u32) {
    let mut vector = #[(x, 2u32)];
    if (i < 10) {
        let (new_vector, _) = vector.pop_back();
        vector = new_vector.push_back((y, 4u32));
    }
    (vector[i], vector.len())
}
