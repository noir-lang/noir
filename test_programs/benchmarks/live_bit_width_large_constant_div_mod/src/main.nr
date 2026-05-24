fn main(x: pub u8, y: pub u8) -> pub u128 {
    let xx = x as u128;
    let yy = y as u128;

    let quotient = xx / 512 as u128;
    let remainder = yy % 512 as u128;

    quotient + remainder
}
