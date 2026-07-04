// A `to_bits`/`to_radix` decomposition into a single limb constrains exactly
// `value < radix` and returns `[value]` (the one-limb reconstruction sum is the
// identity), so the SSA simplifies it to a range check plus a cast instead of
// a full decomposition (limb witness + Brillig limb hint + recomposition).
//
// Endianness is irrelevant for a single limb: both orders are exercised and
// must return the same limb.
fn main(x: Field, b: Field, y: Field, cond: bool, expected_byte: u8, expected_bit: bool) {
    // Single-limb byte decompositions: simplified to an 8-bit range check on `x`.
    let le_byte: [u8; 1] = x.to_le_bytes();
    assert(le_byte[0] == expected_byte);
    let be_byte: [u8; 1] = x.to_be_bytes();
    assert(be_byte[0] == expected_byte);

    // Single-limb bit decompositions: simplified to a 1-bit range check on `b`.
    let le_bit: [bool; 1] = b.to_le_bits();
    assert(le_bit[0] == expected_bit);
    let be_bit: [bool; 1] = b.to_be_bits();
    assert(be_bit[0] == expected_bit);

    // Two-limb control: must keep the full decomposition.
    let two_bytes: [u8; 2] = y.to_le_bytes();
    assert(two_bytes[0] as Field + two_bytes[1] as Field * 256 == y);

    // Predicated single-limb decomposition of a value that does NOT fit in one
    // byte: with `cond = false` the emitted range check is predicated off and
    // must not fail (`flatten_cfg` replaces the range-checked value with its
    // predicated form for the rest of the branch, issue #8617).
    if cond {
        let bad_byte: [u8; 1] = y.to_le_bytes();
        assert(bad_byte[0] == expected_byte);
    }
}
