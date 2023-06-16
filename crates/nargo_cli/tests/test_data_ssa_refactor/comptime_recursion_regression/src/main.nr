fn main(x: Field, y: Field) {
    let flag = (x == 1) | (y == 2);
    assert(flag | false == flag);
}
