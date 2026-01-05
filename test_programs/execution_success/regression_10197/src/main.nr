fn main(index: u32) {
    let mut vector = #[(0, [1, 2]), (3, [4, 5])];
    vector[index] = (6, [7, 8]);
    println(vector);
}
