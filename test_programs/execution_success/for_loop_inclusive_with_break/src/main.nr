unconstrained fn main() {
    let mut sum = 0_u32;
    for i in 0..=255_u8 {
        if i == 5 {
            break;
        }
        sum += i as u32;
    }
    assert_eq(sum, 10);
}
