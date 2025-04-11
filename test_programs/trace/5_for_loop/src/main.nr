fn main(mut x: Field) {
    for i in 0..10 {
        x -= 100 * (i as Field);
        println(x);
    }
}
