fn main(index: u32) {
    let mut slice = &[(0, [1, 2]), (3, [4, 5])];
    slice[index] = (6, [7, 8]);
    println(slice);
}
