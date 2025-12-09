fn main(index: u32) {
    let slice: [()] = &[(), (), ()];
    println(slice);

    let mut slice: [()] = &[(), (), ()];
    slice[index] = ();
    println(slice);
}
