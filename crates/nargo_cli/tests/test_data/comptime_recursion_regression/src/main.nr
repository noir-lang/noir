fn main(x: Field, y: Field) {
    let flag = (x == 1) | (y == 2);
    constrain flag | false == flag;
}
