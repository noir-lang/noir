fn main(x: u8) -> pub u8 {
    // This would previously overflow in ACIR. Now it returns zero.
    1 >> x
}
