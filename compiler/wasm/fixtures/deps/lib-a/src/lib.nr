use dep::lib_b::assert_non_zero;

pub fn divide(a: u64, b: u64) -> u64 {
    assert_non_zero(b);
    a / b
}
