use std::cmp::{min, max};

fn main() { }

#[test]
fn sanity_check_min() {
    assert_eq(min(0 as u64, 1 as u64), 0);
    assert_eq(min(0 as u64, 0 as u64), 0);
    assert_eq(min(1 as u64, 1 as u64), 1);
    assert_eq(min(255 as u8, 0 as u8), 0);
}

#[test]
fn sanity_check_max() {
    assert_eq(max(0 as u64, 1 as u64), 1);
    assert_eq(max(0 as u64, 0 as u64), 0);
    assert_eq(max(1 as u64, 1 as u64), 1);
    assert_eq(max(255 as u8, 0 as u8), 255);
}
