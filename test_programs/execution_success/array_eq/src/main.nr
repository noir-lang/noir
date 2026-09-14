// Checks both array equality (`a == b`) and inequality (`a != c`).
fn main(a: [Field; 32], b: [Field; 32], c: [Field; 32]) {
    assert(a == b);
    assert(a != c);
}
