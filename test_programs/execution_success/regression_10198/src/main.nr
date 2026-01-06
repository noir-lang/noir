fn main(index: u32) {
    let vector: [()] = [(), (), ()].as_vector();
    println(vector);

    let mut vector: [()] = [(), (), ()].as_vector();
    vector[index] = ();
    println(vector);
}
