fn main(x: pub u128, y: pub u128) -> pub u128 {
    let sum = x + y;
    let sum_field = sum as Field;
    sum_field.assert_max_bit_size::<8>();

    if x < 255 as u128 { 1 as u128 } else { 2 as u128 }
}
