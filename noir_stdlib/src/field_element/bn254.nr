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

/// Decompose a single field into two 16 byte fields.
pub fn decompose(x: Field) -> (Field, Field) {
    // Take hints of the decomposition
    let (xlo, xhi) = decompose_unsafe(x);
    let borrow = lt_unsafe(PLO, xlo, 16);

    // Range check the limbs
    xlo.assert_max_bit_size(128);
    xhi.assert_max_bit_size(128);

    // Check that the decomposition is correct
    assert_eq(x, xlo + TWO_POW_128 * xhi);

    // Check that (xlo < plo && xhi <= phi) || (xlo >= plo && xhi < phi)
    let rlo = PLO - xlo + (borrow as Field) * TWO_POW_128;
    let rhi = PHI - xhi - (borrow as Field);

    rlo.assert_max_bit_size(128);
    rhi.assert_max_bit_size(128);

    (xlo, xhi)
}

unconstrained fn lt_unsafe(x: Field, y: Field, num_bytes: u32) -> bool {
    let x_bytes = x.__to_le_radix(256, num_bytes);
    let y_bytes = y.__to_le_radix(256, num_bytes);
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

unconstrained fn lte_unsafe(x: Field, y: Field, num_bytes: u32) -> bool {
    lt_unsafe(x, y, num_bytes) | (x == y)
}

pub fn assert_gt(a: Field, b: Field) {
    // Decompose a and b
    let (alo, ahi) = decompose(a);
    let (blo, bhi) = decompose(b);

    let borrow = lte_unsafe(alo, blo, 16);

    // Assert that (alo > blo && ahi >= bhi) || (alo <= blo && ahi > bhi)
    let rlo = alo - blo - 1 + (borrow as Field) * TWO_POW_128;
    let rhi = ahi - bhi - (borrow as Field);

    rlo.assert_max_bit_size(128);
    rhi.assert_max_bit_size(128);
}

pub fn assert_lt(a: Field, b: Field) {
    assert_gt(b, a);
}

pub fn gt(a: Field, b: Field) -> bool {
    if a == b {
        false
    } else if lt_unsafe(a, b, 32) {
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
