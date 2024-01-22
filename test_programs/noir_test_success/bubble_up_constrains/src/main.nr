use dep::std::field::bn254::{TWO_POW_128, assert_gt};

#[test(should_fail_with="Balance too low")]
fn test_assert_gt_should_fail_eq() {
    let a: u64 = 4;
    let b: u64 = 5;
    assert(a >= b, "Balance too low");
    assert(a >= b, "Redundant constrain");
}
