fn main(index: u32) {
    let vector: [()] = &[(), (), ()];
    println(vector);

    let mut vector: [()] = &[(), (), ()];
    vector[index] = ();
    println(vector);
}
