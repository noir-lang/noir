// Prover.toml:
// u8 = 100
// positive_i8 = 100
// negative_i8 = -100
// seven = 7

fn main(u8: u8, positive_i8: i8, negative_i8: i8, seven: u8) {
    check_bitshifts(u8, positive_i8, negative_i8, seven);

    // Safety: test program
    unsafe { check_bitshifts_unconstrained_wrapper(u8, positive_i8, negative_i8, seven) };

    comptime { check_bitshifts(100, 100, -100, 7) };
}

unconstrained fn check_bitshifts_unconstrained_wrapper(
    u8: u8,
    positive_i8: i8,
    negative_i8: i8,
    seven: u8,
) {
    check_bitshifts(u8, positive_i8, negative_i8, seven)
}

fn check_bitshifts(u8: u8, positive_i8: i8, negative_i8: i8, seven: u8) {
    assert_eq(u8 >> seven, 0);
    assert_eq(u8 << seven, 0);
    assert_eq(positive_i8 >> seven, 0);
    assert_eq(positive_i8 << seven, 0);
    assert_eq(negative_i8 >> seven, -1);
    assert_eq(negative_i8 << seven, 0);
}
