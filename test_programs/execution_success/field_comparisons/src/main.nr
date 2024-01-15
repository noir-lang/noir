use dep::std::field::bn254::{PLO, PHI, TWO_POW_128, decompose, decompose_unsafe, lt_unsafe, lte_unsafe, assert_gt, gt};

fn check_plo_phi() {
    assert_eq(PLO + PHI * TWO_POW_128, 0);
    let p_bytes = dep::std::field::modulus_le_bytes();
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

fn check_decompose_unsafe() {
    assert_eq(decompose_unsafe(TWO_POW_128), (0, 1));
    assert_eq(decompose_unsafe(TWO_POW_128 + 0x1234567890), (0x1234567890, 1));
    assert_eq(decompose_unsafe(0x1234567890), (0x1234567890, 0));
}

fn check_decompose() {
    assert_eq(decompose(TWO_POW_128), (0, 1));
    assert_eq(decompose(TWO_POW_128 + 0x1234567890), (0x1234567890, 1));
    assert_eq(decompose(0x1234567890), (0x1234567890, 0));
}

fn check_lt_unsafe() {
    assert(lt_unsafe(0, 1, 16));
    assert(lt_unsafe(0, 0x100, 16));
    assert(lt_unsafe(0x100, TWO_POW_128 - 1, 16));
    assert(!lt_unsafe(0, TWO_POW_128, 16));
}

fn check_lte_unsafe() {
    assert(lte_unsafe(0, 1, 16));
    assert(lte_unsafe(0, 0x100, 16));
    assert(lte_unsafe(0x100, TWO_POW_128 - 1, 16));
    assert(!lte_unsafe(0, TWO_POW_128, 16));

    assert(lte_unsafe(0, 0, 16));
    assert(lte_unsafe(0x100, 0x100, 16));
    assert(lte_unsafe(TWO_POW_128 - 1, TWO_POW_128 - 1, 16));
    assert(lte_unsafe(TWO_POW_128, TWO_POW_128, 16));
}

fn check_assert_gt() {
    assert_gt(1, 0);
    assert_gt(0x100, 0);
    assert_gt((0 - 1), (0 - 2));
    assert_gt(TWO_POW_128, 0);
    assert_gt(0 - 1, 0);
}

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

fn checks() {
    check_plo_phi();
    check_decompose_unsafe();
    check_decompose();
    check_lt_unsafe();
    check_lte_unsafe();
    check_assert_gt();
    check_gt();
}

unconstrained fn checks_in_brillig() {
    checks();
}

fn main() {
    checks();
    checks_in_brillig();
}
