fn main(x: Field) {
    let mut slice = &[1, 2, 3];
    slice[x] = 1; // Index out of bounds
}
