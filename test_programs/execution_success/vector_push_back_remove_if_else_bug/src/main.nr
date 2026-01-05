fn main(x: u32) {
    let mut vector = #[2, 3];
    vector[x] = 4;
    if x == 0 {
        vector = vector.push_back(5);
    }
    assert(vector[x] == 4);
}
