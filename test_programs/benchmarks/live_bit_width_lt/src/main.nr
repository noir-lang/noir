fn main(x: pub u8, y: pub u8) -> pub u128 {
    let xx = x as u128;
    let yy = y as u128;
    let sum = xx + yy;

    if sum < 256 as u128 { 1 as u128 } else { 2 as u128 }
}
