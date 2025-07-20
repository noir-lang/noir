use crate::field::field_less_than;
use crate::runtime::is_unconstrained;

global PLO: Field = 65535;
global PHI: Field = 32767;

pub(crate) global TWO_POW_16: Field = 0x10000; 

// Decomposes a single field into 2 bytes fields.
fn compute_decomposition(x: Field) -> (Field, Field) {
    let low = (x as u16) as Field;
    let high = (x - low) / TWO_POW_16;

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

        let rlo = alo - blo - 1 + (borrow as Field) * TWO_POW_16;
        let rhi = ahi - bhi - (borrow as Field);

        rlo.assert_max_bit_size::<16>();
        rhi.assert_max_bit_size::<16>();
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
            xlo.assert_max_bit_size::<16>();
            xhi.assert_max_bit_size::<16>();

            // Check that the decomposition is correct
            assert_eq(x, xlo + TWO_POW_16 * xhi);

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

mod tests {
    // TODO: Allow imports from "super"
    use crate::field::m31::{assert_gt, decompose, gt, lte_hint, PHI, PLO, TWO_POW_16};

    #[test]
    fn check_decompose() {
        // TODO : works till most ssa, check again after brillling_gen is fixed
        assert_eq(decompose(TWO_POW_16), (0, 1));
        assert_eq(decompose(4*TWO_POW_16 + 0x1234), (0x1234, 4));
        assert_eq(decompose(0x10001), (1, 1));
        assert_eq(decompose(0x1234), (0x1234, 0));        
    }

    #[test]
    unconstrained fn check_decompose_unconstrained() {
        assert_eq(decompose(TWO_POW_16), (0, 1));
        assert_eq(decompose(4*TWO_POW_16 + 0x1234), (0x1234, 4));
        assert_eq(decompose(0x10001), (1, 1));
        assert_eq(decompose(0x1234), (0x1234, 0));   
    }

    #[test]
    unconstrained fn check_lte_hint() {
        assert(lte_hint(0, 1));
        assert(lte_hint(0, 0x100));
        assert(lte_hint(0x100, TWO_POW_16 - 1));
        assert(!lte_hint(0 - 1, 0)); // wraps
        assert(lte_hint(2147483147, -500));

        assert(lte_hint(0, 0));
        assert(lte_hint(0x100, 0x100));
        assert(lte_hint(0 - 1, 0 - 1));
    }

    #[test]
    fn check_assert_gt() {
        // TODO : works till most ssa, check again after brillling_gen is fixed
        assert_gt(1, 0);
        assert_gt(0x100, 0);
        assert_gt((0 - 1), (0 - 2));
        assert_gt(TWO_POW_16, 0);
        assert_gt(0 - 1, 0);
    }

    #[test]
    unconstrained fn check_assert_gt_unconstrained() {
        assert_gt(1, 0);
        assert_gt(0x100, 0);
        assert_gt((0 - 1), (0 - 2));
        assert_gt(TWO_POW_16, 0);
        assert_gt(0 - 1, 0);
    }

    #[test]
    fn check_gt() {
        // TODO : works till most ssa, check again after brillling_gen is fixed
        assert(gt(1, 0));
        assert(gt(0x100, 0));
        assert(gt((0 - 1), (0 - 2)));
        assert(gt(TWO_POW_16, 0));
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
        assert(gt(TWO_POW_16, 0));
        assert(!gt(0, 0));
        assert(!gt(0, 0x100));
        assert(gt(0 - 1, 0 - 2));
        assert(!gt(0 - 2, 0 - 1));
    }

    #[test]
    fn check_plo_phi() {
        assert_eq(PLO + PHI * TWO_POW_16, 0);
        let p_bytes = crate::field::modulus_le_bytes();
        let mut p_low: Field = 0;
        let mut p_high: Field = 0;

        let mut offset = 1;
        // 2 as m31 comes in 4 limbs
        for i in 0..2 {
            p_low += (p_bytes[i] as Field) * offset;
            p_high += (p_bytes[i + 2] as Field) * offset;
            offset *= 256;
        }
        assert_eq(p_low, PLO);
        assert_eq(p_high, PHI);
    }


}