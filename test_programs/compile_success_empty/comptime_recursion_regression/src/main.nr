fn main(x: field, y: field) {
    let flag = (x == 1) | (y == 2);
    assert(flag | false == flag);
}
