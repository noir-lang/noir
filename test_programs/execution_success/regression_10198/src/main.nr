fn main(index: u32) {
    let list: [()] = &[(), (), ()];
    println(list);

    let mut list: [()] = &[(), (), ()];
    list[index] = ();
    println(list);
}
