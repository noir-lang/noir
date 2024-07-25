use crate::runtime::is_unconstrained;

// The low and high decomposition of the field modulus
global PLO: Field = 53438638232309528389504892708671455233;
global PHI: Field = 64323764613183177041862057485226039389;

global TWO_POW_128: Field = 0x100000000000000000000000000000000;

// Decomposes a single field into two 16 byte fields.
fn compute_decomposition(x: Field) -> (Field, Field) {
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

unconstrained pub(crate) fn decompose_hint(x: Field) -> (Field, Field) {
    compute_decomposition(x)
}

fn compute_lt(x: Field, y: Field, num_bytes: u32) -> bool {
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

fn compute_lte(x: Field, y: Field, num_bytes: u32) -> bool {
    if x == y {
        true
    } else {
        compute_lt(x, y, num_bytes)
    }
}

unconstrained fn lt_32_hint(x: Field, y: Field) -> bool {
    compute_lt(x, y, 32)
}

unconstrained fn lte_16_hint(x: Field, y: Field) -> bool {
    compute_lte(x, y, 16)
}

// Assert that (alo > blo && ahi >= bhi) || (alo <= blo && ahi > bhi)
fn assert_gt_limbs(a: (Field, Field), b: (Field, Field)) {
    let (alo, ahi) = a;
    let (blo, bhi) = b;
    let borrow = lte_16_hint(alo, blo);

    let rlo = alo - blo - 1 + (borrow as Field) * TWO_POW_128;
    let rhi = ahi - bhi - (borrow as Field);

    rlo.assert_max_bit_size(128);
    rhi.assert_max_bit_size(128);
}

/// Decompose a single field into two 16 byte fields.
pub fn decompose(x: Field) -> (Field, Field) {
    if is_unconstrained() {
        compute_decomposition(x)
    } else {
        // Take hints of the decomposition
        let (xlo, xhi) = decompose_hint(x);

        // Range check the limbs
        xlo.assert_max_bit_size(128);
        xhi.assert_max_bit_size(128);

        // Check that the decomposition is correct
        assert_eq(x, xlo + TWO_POW_128 * xhi);

        // Assert that the decomposition of P is greater than the decomposition of x
        assert_gt_limbs((PLO, PHI), (xlo, xhi));
        (xlo, xhi)
    }
}

pub fn assert_gt(a: Field, b: Field) {
    if is_unconstrained() {
        assert(compute_lt(b, a, 32));
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
        compute_lt(b, a, 32)
    } else if a == b {
        false
    } else  {
        // Take a hint of the comparison and verify it
        if lt_32_hint(a, b) {
            assert_gt(b, a);
            false
        } else {
            assert_gt(a, b);
            true
        }
    }
}

pub fn lt(a: Field, b: Field) -> bool {
    gt(b, a)
}
