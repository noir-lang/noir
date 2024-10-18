fn main(x: Field, y: pub Field) {
    basic_check(x, y);
}

#['inline(never)]
fn basic_check(x: Field, y: Field) {
    assert(x != y);
}
