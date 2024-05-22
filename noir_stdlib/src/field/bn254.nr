// The low and high decomposition of the field modulus
global PLO: Field = 53438638232309528389504892708671455233;
global PHI: Field = 64323764613183177041862057485226039389;

global TWO_POW_128: Field = 0x100000000000000000000000000000000;

/// A hint for decomposing a single field into two 16 byte fields.
unconstrained fn decompose_unsafe(x: Field) -> (Field, Field) {
    let x_bytes = x.to_le_bytes(32);

    let mut low: Field = 0;
    let mut high: Field = 0;

    let mut offset = 1;
    for i in 0..16 {
        low += (x_bytes[i] as Field) * offset;
        high += (x_bytes[i + 16] as Field) * offset;
        offset *= 256;
    }

    (low, high)
}

// Assert that (alo > blo && ahi >= bhi) || (alo <= blo && ahi > bhi)
fn assert_gt_limbs(a: (Field, Field), b: (Field, Field)) {
    let (alo, ahi) = a;
    let (blo, bhi) = b;
    let borrow = lte_unsafe_16(alo, blo);

    let rlo = alo - blo - 1 + (borrow as Field) * TWO_POW_128;
    let rhi = ahi - bhi - (borrow as Field);

    rlo.assert_max_bit_size(128);
    rhi.assert_max_bit_size(128);
}

/// Decompose a single field into two 16 byte fields.
pub fn decompose(x: Field) -> (Field, Field) {
    // Take hints of the decomposition
    let (xlo, xhi) = decompose_unsafe(x);

    // Range check the limbs
    xlo.assert_max_bit_size(128);
    xhi.assert_max_bit_size(128);

    // Check that the decomposition is correct
    assert_eq(x, xlo + TWO_POW_128 * xhi);

    // Assert that the decomposition of P is greater than the decomposition of x
    assert_gt_limbs((PLO, PHI), (xlo, xhi));
    (xlo, xhi)
}

fn lt_unsafe_internal(x: Field, y: Field, num_bytes: u32) -> bool {
    let x_bytes = x.to_le_radix(256, num_bytes);
    let y_bytes = y.to_le_radix(256, num_bytes);
    let mut x_is_lt = false;
    let mut done = false;
    for i in 0..num_bytes {
        if (!done) {
            let x_byte = x_bytes[num_bytes - 1 - i];
            let y_byte = y_bytes[num_bytes - 1 - i];
            let bytes_match = x_byte == y_byte;
            if !bytes_match {
                x_is_lt = x_byte < y_byte;
                done = true;
            }
        }
    }
    x_is_lt
}

fn lte_unsafe_internal(x: Field, y: Field, num_bytes: u32) -> bool {
    if x == y {
        true
    } else {
        lt_unsafe_internal(x, y, num_bytes)
    }
}

unconstrained fn lt_unsafe_32(x: Field, y: Field) -> bool {
    lt_unsafe_internal(x, y, 32)
}

unconstrained fn lte_unsafe_16(x: Field, y: Field) -> bool {
    lte_unsafe_internal(x, y, 16)
}

pub fn assert_gt(a: Field, b: Field) {
    // Decompose a and b
    let a_limbs = decompose(a);
    let b_limbs = decompose(b);

    // Assert that a_limbs is greater than b_limbs
    assert_gt_limbs(a_limbs, b_limbs)
}

pub fn assert_lt(a: Field, b: Field) {
    assert_gt(b, a);
}

pub fn gt(a: Field, b: Field) -> bool {
    if a == b {
        false
    } else if lt_unsafe_32(a, b) {
        assert_gt(b, a);
        false
    } else {
        assert_gt(a, b);
        true
    }
}

pub fn lt(a: Field, b: Field) -> bool {
    gt(b, a)
}

mod tests {
    // TODO: Allow imports from "super"
    use crate::field::bn254::{
        decompose_unsafe, decompose, lt_unsafe_internal, assert_gt, gt, lt, TWO_POW_128,
        lte_unsafe_internal, PLO, PHI
    };

    #[test]
    fn check_decompose_unsafe() {
        assert_eq(decompose_unsafe(TWO_POW_128), (0, 1));
        assert_eq(decompose_unsafe(TWO_POW_128 + 0x1234567890), (0x1234567890, 1));
        assert_eq(decompose_unsafe(0x1234567890), (0x1234567890, 0));
    }

    #[test]
    fn check_decompose() {
        assert_eq(decompose(TWO_POW_128), (0, 1));
        assert_eq(decompose(TWO_POW_128 + 0x1234567890), (0x1234567890, 1));
        assert_eq(decompose(0x1234567890), (0x1234567890, 0));
    }

    #[test]
    fn check_lt_unsafe() {
        assert(lt_unsafe_internal(0, 1, 16));
        assert(lt_unsafe_internal(0, 0x100, 16));
        assert(lt_unsafe_internal(0x100, TWO_POW_128 - 1, 16));
        assert(!lt_unsafe_internal(0, TWO_POW_128, 16));
    }

    #[test]
    fn check_lte_unsafe() {
        assert(lte_unsafe_internal(0, 1, 16));
        assert(lte_unsafe_internal(0, 0x100, 16));
        assert(lte_unsafe_internal(0x100, TWO_POW_128 - 1, 16));
        assert(!lte_unsafe_internal(0, TWO_POW_128, 16));

        assert(lte_unsafe_internal(0, 0, 16));
        assert(lte_unsafe_internal(0x100, 0x100, 16));
        assert(lte_unsafe_internal(TWO_POW_128 - 1, TWO_POW_128 - 1, 16));
        assert(lte_unsafe_internal(TWO_POW_128, TWO_POW_128, 16));
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
