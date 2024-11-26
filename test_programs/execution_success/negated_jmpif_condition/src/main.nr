fn main(mut x: Field) {
    let mut q = 0;

    if x != 10 {
        q = 2;
    }

    assert(q == 2);
}
