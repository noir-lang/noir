use crate::field::field_less_than;
use crate::runtime::is_unconstrained;

// The low and high decomposition of the field modulus
global PLO: Field = 53438638232309528389504892708671455233;
global PHI: Field = 64323764613183177041862057485226039389;

pub(crate) global TWO_POW_128: Field = 0x100000000000000000000000000000000;
global TWO_POW_64: Field = 0x10000000000000000;

// Decomposes a single field into two 16 byte fields.
fn compute_decomposition(mut x: Field) -> (Field, Field) {
    // Here's we're taking advantage of truncating 64 bit limbs from the input field
    // and then subtracting them from the input such the field division is equivalent to integer division.
    let low_lower_64 = (x as u64) as Field;
    x = (x - low_lower_64) / TWO_POW_64;
    let low_upper_64 = (x as u64) as Field;

    let high = (x - low_upper_64) / TWO_POW_64;
    let low = low_upper_64 * TWO_POW_64 + low_lower_64;

    (low, high)
}

pub(crate) unconstrained fn decompose_hint(x: Field) -> (Field, Field) {
    compute_decomposition(x)
}

unconstrained fn lte_hint(x: Field, y: Field) -> bool {
    if x == y {
        true
    } else {
        field_less_than(x, y)
    }
}

// Assert that (alo > blo && ahi >= bhi) || (alo <= blo && ahi > bhi)
fn assert_gt_limbs(a: (Field, Field), b: (Field, Field)) {
    let (alo, ahi) = a;
    let (blo, bhi) = b;
    // Safety: borrow is enforced to be boolean due to its type.
    // if borrow is 0, it asserts that (alo > blo && ahi >= bhi)
    // if borrow is 1, it asserts that (alo <= blo && ahi > bhi)
    unsafe {
        let borrow = lte_hint(alo, blo);

        let rlo = alo - blo - 1 + (borrow as Field) * TWO_POW_128;
        let rhi = ahi - bhi - (borrow as Field);

        rlo.assert_max_bit_size::<128>();
        rhi.assert_max_bit_size::<128>();
    }
}

/// Decompose a single field into two 16 byte fields.
pub fn decompose(x: Field) -> (Field, Field) {
    if is_unconstrained() {
        compute_decomposition(x)
    } else {
        // Safety: decomposition is properly checked below
        unsafe {
            // Take hints of the decomposition
            let (xlo, xhi) = decompose_hint(x);

            // Range check the limbs
            xlo.assert_max_bit_size::<128>();
            xhi.assert_max_bit_size::<128>();

            // Check that the decomposition is correct
            assert_eq(x, xlo + TWO_POW_128 * xhi);

            // Assert that the decomposition of P is greater than the decomposition of x
            assert_gt_limbs((PLO, PHI), (xlo, xhi));
            (xlo, xhi)
        }
    }
}

pub fn assert_gt(a: Field, b: Field) {
    if is_unconstrained() {
        assert(
            // Safety: already unconstrained
            unsafe { field_less_than(b, a) },
        );
    } else {
        // Decompose a and b
        let a_limbs = decompose(a);
        let b_limbs = decompose(b);

        // Assert that a_limbs is greater than b_limbs
        assert_gt_limbs(a_limbs, b_limbs)
    }
}

pub fn assert_lt(a: Field, b: Field) {
    assert_gt(b, a);
}

pub fn gt(a: Field, b: Field) -> bool {
    if is_unconstrained() {
        // Safety: unsafe in unconstrained
        unsafe {
            field_less_than(b, a)
        }
    } else if a == b {
        false
    } else {
        // Safety: Take a hint of the comparison and verify it
        unsafe {
            if field_less_than(a, b) {
                assert_gt(b, a);
                false
            } else {
                assert_gt(a, b);
                true
            }
        }
    }
}

pub fn lt(a: Field, b: Field) -> bool {
    gt(b, a)
}

pub fn eq(a: Field, b: Field) -> bool {
    a == b
}

pub fn lte(a: Field, b: Field) -> bool {
    if is_unconstrained() {
        unsafe {
            lte_hint(a, b)
        }
    } else {
        if a == b {
            true
        } else {
            unsafe {
                if field_less_than(a, b) {
                    assert_gt(b, a);
                    true
                } else {
                    false
                }
            }
        }
    }
}

pub fn gte(a: Field, b: Field) -> bool {
    if is_unconstrained() {
        unsafe {
            !field_less_than(a, b)
        }
    } else if a == b {
        true
    } else {
        unsafe {
            if field_less_than(a, b) {
                false
            } else {
                assert_gt(a, b);
                true
            }
        }
    }
}

pub fn assert_lte(a: Field, b: Field) {
    if is_unconstrained() {
        unsafe {
            assert(lte_hint(a, b));
        }
    } else {
        if a != b {
            unsafe {
                if !field_less_than(a, b) {
                    assert(false);
                }
                assert_gt(b, a);
            }
        }
    }
}

pub fn assert_gte(a: Field, b: Field) {
    if is_unconstrained() {
        unsafe {
            assert(!field_less_than(a, b));
        }
    } else {
        if a != b {
            unsafe {
                if field_less_than(a, b) {
                    assert(false);
                }
                assert_gt(a, b);
            }
        }
    }
}

mod tests {
    // TODO: Allow imports from "super"
    use super::{
        assert_gt, assert_gte, assert_lte, decompose, gt, gte, lte, lte_hint, PHI, PLO, TWO_POW_128,
    };

    #[test]
    fn check_assert_lte() {
        assert_lte(0, 0);
        assert_lte(0, 1);
        assert_lte(100, 100);
    }

    #[test]
    fn check_assert_gte() {
        assert_gte(0, 0);
        assert_gte(1, 0);
        assert_gte(100, 100);
    }

    #[test]
    fn check_lte() {
        assert(lte(0, 0));
        assert(lte(1, 1));
        assert(lte(0, 1));
        assert(!lte(1, 0));
        assert(lte(0x99, 0x100));
        assert(!lte(0x101, 0x100));
    }

    #[test]
    fn check_gte() {
        assert(gte(0, 0));
        assert(gte(1, 1));
        assert(gte(1, 0));
        assert(!gte(0, 1));
        assert(gte(0x100, 0x99));
        assert(!gte(0x100, 0x101));
    }

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
    unconstrained fn check_lte_hint() {
        assert(lte_hint(0, 1));
        assert(lte_hint(0, 0x100));
        assert(lte_hint(0x100, TWO_POW_128 - 1));
        assert(!lte_hint(0 - 1, 0));
        assert(lte_hint(0, 0));
        assert(lte_hint(0x100, 0x100));
        assert(lte_hint(0 - 1, 0 - 1));
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
        let p_bytes = crate::field::modulus_le_bytes();
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
}
