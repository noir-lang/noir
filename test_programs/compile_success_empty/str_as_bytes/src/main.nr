fn main() {
    let a = "hello";
    let b = a.as_bytes();
    assert(b[0] == 104);
    assert(b[1] == 101);
    assert(b[2] == 108);
    assert(b[3] == 108);
    assert(b[4] == 111);
    assert(b.len() == 5);
    let mut c = a.as_bytes_vec();
    assert(c[0] == 104);
    assert(c[1] == 101);
    assert(c[2] == 108);
    assert(c[3] == 108);
    assert(c[4] == 111);
    assert(c.len() == 5);
}
