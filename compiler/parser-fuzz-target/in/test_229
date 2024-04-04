// You can only do bit operations with integers.
// (Kobi/Daira/Circom/#37) https://github.com/iden3/circom/issues/37
fn main(x: Field, y: Field, a: Field, b: Field) {
    let x_as_u8 = x as u8;
    let y_as_u8 = y as u8;

    assert((x_as_u8 & y_as_u8) == x_as_u8);
    //bitwise and with 1 bit:
    let flag = (x == 0) & (y == 16);
    assert(flag);
    //bitwise and with odd bits:
    let a_as_u8 = a as u8;
    let b_as_u8 = b as u8;
    assert((a_as_u8 & b_as_u8) == a_as_u8);
}

