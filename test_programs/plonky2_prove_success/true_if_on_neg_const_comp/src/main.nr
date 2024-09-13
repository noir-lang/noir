fn main(
    a: Field,
    b: Field
) {
    let mut res: bool = true;
    if b - a == -1 {
        res = false;
    }
    else {
        res = true;
    }
    assert(res);
}
