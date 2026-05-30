fn pick(flag: bool, value: u128) -> u128 {
    if flag { value } else { 0 as u128 }
}

fn main(x: pub u8, y: pub u8, z: pub u8) -> pub u128 {
    let xx = x as u128;
    let yy = y as u128;
    let zz = z as u128;

    let lhs = x % 9;
    let rhs = y % 8;
    let exact_sum = lhs + rhs;
    let exact_product = lhs * rhs;

    let high_rhs = zz + 240 as u128;
    let bounded_difference = 500 as u128 - high_rhs;
    let bounded_quotient = xx / (yy + 16 as u128);
    let bounded_remainder = (xx + 100 as u128) % 512 as u128;
    let bounded_shift = xx >> 4 as u128;

    pick((exact_sum as u128) < 16 as u128, 1)
        + pick((exact_product as u128) < 57 as u128, 2)
        + pick(bounded_difference < 261 as u128, 4)
        + pick(bounded_quotient < 16 as u128, 8)
        + pick(bounded_remainder < 356 as u128, 16)
        + pick(bounded_shift < 16 as u128, 32)
}
