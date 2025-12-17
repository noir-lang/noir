fn main(index: u32) {
    let mut list = &[(0, [1, 2]), (3, [4, 5])];
    list[index] = (6, [7, 8]);
    println(list);
}
