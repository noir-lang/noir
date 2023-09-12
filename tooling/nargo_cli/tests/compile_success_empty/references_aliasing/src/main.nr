fn increment(mut r: &mut Field) {
    *r = *r + 1;
}

fn main() {
    let mut x = 100;
    let mut xref = &mut x;
    increment(xref);
    assert(*xref == 101);
}
