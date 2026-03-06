unconstrained fn main(from: u32, to: u32) {
    let mut sum = 0_u32;
    for i in from..=to {
        sum += i as u32;
    }
    assert_eq(sum, 0);
}
