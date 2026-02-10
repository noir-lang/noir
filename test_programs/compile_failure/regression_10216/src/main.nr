fn main(mut x: u32) {
    let xs = [&mut x].as_vector();
    println(xs);
}
