fn main(x: Field, y: Field) {
    let mut z = x + y;
    assert(z == 3);
    z = x * y;
    assert(z == 2);
}
