// This program would not be simplified to an empty circuit without the compiler canonicalizing the multiplication
// operator using the commutative property of multiplication.
pub fn main(a: Field) {
    let mut x: Field = 0;
    if (a == 0) {
        x = 1;
    }
    assert((a * x) == (x * a));
}
