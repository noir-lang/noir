fn main(
    a: Field,
    b: Field
) {
    let mut res: bool = false;
    if (a - b) == -1 {
        res = true;
    }
    else {
        res = false;
    }
    // This crashes
    assert(res);
}
