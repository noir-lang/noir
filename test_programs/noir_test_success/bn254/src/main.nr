// TODO: Allow imports from "super"
use std::field::bn254::{decompose_hint, decompose, compute_lt, assert_gt, gt, lt, TWO_POW_128, compute_lte, PLO, PHI};

fn main() { }

#[test]
fn check_decompose() {
    assert_eq(decompose(TWO_POW_128), (0, 1));
    assert_eq(decompose(TWO_POW_128 + 0x1234567890), (0x1234567890, 1));
    assert_eq(decompose(0x1234567890), (0x1234567890, 0));
}

#[test]
unconstrained fn check_decompose_unconstrained() {
    assert_eq(decompose(TWO_POW_128), (0, 1));
    assert_eq(decompose(TWO_POW_128 + 0x1234567890), (0x1234567890, 1));
    assert_eq(decompose(0x1234567890), (0x1234567890, 0));
}

#[test]
fn check_compute_lt() {
    assert(compute_lt(0, 1, 16));
    assert(compute_lt(0, 0x100, 16));
    assert(compute_lt(0x100, TWO_POW_128 - 1, 16));
    assert(!compute_lt(0, TWO_POW_128, 16));
}

#[test]
fn check_compute_lte() {
    assert(compute_lte(0, 1, 16));
    assert(compute_lte(0, 0x100, 16));
    assert(compute_lte(0x100, TWO_POW_128 - 1, 16));
    assert(!compute_lte(0, TWO_POW_128, 16));

    assert(compute_lte(0, 0, 16));
    assert(compute_lte(0x100, 0x100, 16));
    assert(compute_lte(TWO_POW_128 - 1, TWO_POW_128 - 1, 16));
    assert(compute_lte(TWO_POW_128, TWO_POW_128, 16));
}

#[test]
fn check_assert_gt() {
    assert_gt(1, 0);
    assert_gt(0x100, 0);
    assert_gt((0 - 1), (0 - 2));
    assert_gt(TWO_POW_128, 0);
    assert_gt(0 - 1, 0);
}

#[test]
unconstrained fn check_assert_gt_unconstrained() {
    assert_gt(1, 0);
    assert_gt(0x100, 0);
    assert_gt((0 - 1), (0 - 2));
    assert_gt(TWO_POW_128, 0);
    assert_gt(0 - 1, 0);
}

#[test]
fn check_gt() {
    assert(gt(1, 0));
    assert(gt(0x100, 0));
    assert(gt((0 - 1), (0 - 2)));
    assert(gt(TWO_POW_128, 0));
    assert(!gt(0, 0));
    assert(!gt(0, 0x100));
    assert(gt(0 - 1, 0 - 2));
    assert(!gt(0 - 2, 0 - 1));
}

#[test]
unconstrained fn check_gt_unconstrained() {
    assert(gt(1, 0));
    assert(gt(0x100, 0));
    assert(gt((0 - 1), (0 - 2)));
    assert(gt(TWO_POW_128, 0));
    assert(!gt(0, 0));
    assert(!gt(0, 0x100));
    assert(gt(0 - 1, 0 - 2));
    assert(!gt(0 - 2, 0 - 1));
}

#[test]
fn check_plo_phi() {
    assert_eq(PLO + PHI * TWO_POW_128, 0);
    let p_bytes = std::field::modulus_le_bytes();
    let mut p_low: Field = 0;
    let mut p_high: Field = 0;

    let mut offset = 1;
    for i in 0..16 {
        p_low += (p_bytes[i] as Field) * offset;
        p_high += (p_bytes[i + 16] as Field) * offset;
        offset *= 256;
    }
    assert_eq(p_low, PLO);
    assert_eq(p_high, PHI);
}
