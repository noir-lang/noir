use crate::field::field_less_than;
use crate::runtime::is_unconstrained;

// The low and high decomposition of the field modulus
pub(crate) global PLO: Field = 53438638232309528389504892708671455233;
pub(crate) global PHI: Field = 64323764613183177041862057485226039389;

pub(crate) global TWO_POW_128: Field = 0x100000000000000000000000000000000;

// Decomposes a single field into two 16 byte fields.
fn compute_decomposition(x: Field) -> (Field, Field) {
    // Here's we're taking advantage of truncating 128 bit limbs from the input field
    // and then subtracting them from the input such the field division is equivalent to integer division.
    let low = (x as u128) as Field;
    let high = (x - low) / TWO_POW_128;

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
///
/// Returns `(lo, hi)` with `x == lo + hi * 2^128`, where `lo` and `hi` are each less than `2^128`.
/// The decomposition is fully constrained and canonical: it is checked to be less than the BN254
/// modulus, so there is exactly one valid `(lo, hi)` for each `x`.
///
/// The functions in this module use the BN254 modulus, so they are only meaningful when compiling
/// for the BN254 field.
///
/// Example:
///
/// ```noir
/// let (lo, hi) = std::field::bn254::decompose(0x100000000000000000000000000000003);
/// assert_eq(lo, 3);
/// assert_eq(hi, 1);
/// ```
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

/// Asserts that `a` is greater than `b`, comparing them as integers in `0..p`.
///
/// Fully constrained. Only meaningful when compiling for the BN254 field.
///
/// Example:
///
/// ```noir
/// std::field::bn254::assert_gt(2, 1);
/// // -1 is the largest field element.
/// std::field::bn254::assert_gt(-1, 1);
/// ```
///
/// # Failures
///
/// Fails if `a` is less than or equal to `b`.
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

/// Asserts that `a` is less than `b`, comparing them as integers in `0..p`.
///
/// Fully constrained. Only meaningful when compiling for the BN254 field.
///
/// Example:
///
/// ```noir
/// std::field::bn254::assert_lt(1, 2);
/// ```
///
/// # Failures
///
/// Fails if `a` is greater than or equal to `b`.
pub fn assert_lt(a: Field, b: Field) {
    assert_gt(b, a);
}

/// Returns `true` if `a` is greater than `b`, comparing them as integers in `0..p`.
///
/// The result is fully constrained. Only meaningful when compiling for the BN254 field; prefer
/// [`Field::lt`], which works on any field.
///
/// Example:
///
/// ```noir
/// assert(std::field::bn254::gt(2, 1));
/// assert(!std::field::bn254::gt(1, 1));
/// ```
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

/// Returns `true` if `a` is less than `b`, comparing them as integers in `0..p`.
///
/// The result is fully constrained. Only meaningful when compiling for the BN254 field; prefer
/// [`Field::lt`], which works on any field.
///
/// Example:
///
/// ```noir
/// assert(std::field::bn254::lt(1, 2));
/// assert(!std::field::bn254::lt(-1, 1));
/// ```
pub fn lt(a: Field, b: Field) -> bool {
    gt(b, a)
}

mod tests {
    // TODO: Allow imports from "super"
    use crate::field::bn254::{assert_gt, decompose, gt, lt, lte_hint, PHI, PLO, TWO_POW_128};
    use crate::internal::test_unconstrained;
    use super::assert_lt;

    #[test]
    fn check_decompose() {
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
    #[test_unconstrained]
    fn check_gt() {
        assert(gt(1, 0));
        assert(gt(0x100, 0));
        assert(gt((0 - 1), (0 - 2)));
        assert(gt(TWO_POW_128, 0));
        assert(!gt(0, 0));
        assert(!gt(0, 0x100));
        assert(gt(0 - 1, 0 - 2));
        assert(!gt(0 - 2, 0 - 1));
        assert_gt(0 - 1, 0);
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

    #[test]
    fn check_decompose_edge_cases() {
        assert_eq(decompose(0), (0, 0));
        assert_eq(decompose(TWO_POW_128 - 1), (TWO_POW_128 - 1, 0));
        assert_eq(decompose(TWO_POW_128 + 1), (1, 1));
        assert_eq(decompose(TWO_POW_128 * 2), (0, 2));
        assert_eq(decompose(TWO_POW_128 * 2 + 0x1234567890), (0x1234567890, 2));
    }

    #[test]
    fn check_decompose_large_values() {
        let large_field = 0xffffffffffffffff;
        let (lo, hi) = decompose(large_field);
        assert_eq(large_field, lo + TWO_POW_128 * hi);

        let large_value = large_field - TWO_POW_128;
        let (lo2, hi2) = decompose(large_value);
        assert_eq(large_value, lo2 + TWO_POW_128 * hi2);
    }

    #[test]
    fn check_lt_comprehensive() {
        assert(lt(0, 1));
        assert_lt(0, 1);
        assert(!lt(1, 0));
        assert(!lt(0, 0));
        assert(!lt(42, 42));

        assert(lt(TWO_POW_128 - 1, TWO_POW_128));
        assert(!lt(TWO_POW_128, TWO_POW_128 - 1));
    }
}
