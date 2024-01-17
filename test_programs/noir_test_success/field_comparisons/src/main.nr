use dep::std::field::bn254::{TWO_POW_128, assert_gt};

#[test(should_fail)]
fn test_assert_gt_should_fail_eq() {
    assert_gt(0, 0);
}

#[test(should_fail)]
fn test_assert_gt_should_fail_low_lt() {
    assert_gt(0, 0x100);
}

#[test(should_fail)]
fn test_assert_gt_should_fail_high_lt() {
    assert_gt(TWO_POW_128, TWO_POW_128 + 0x100);
}
