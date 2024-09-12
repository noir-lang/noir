fn main(
    a: Field,
    b: Field
) {
    let mut res: u8 = 15;
    if b - a != -1 {
        res = 3;
    }
    else {
        res = 8;
    }
    assert(res == 3);
}
