fn main(x: Field, y: pub Field) {
    let mut a = x + 2;
    let mut b = x + 5;
    b = y + 2; assert(a != b);
}
