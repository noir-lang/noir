fn main(
    a: Field,
    b: Field
) {
    let mut res: u8 = 15;
    if a - b == -1 {
        res = 3;
    }
    else {
        res = 8;
    }
    assert(res < 5);
}
