// A bitwise AND with a constant mask bounds the result to the mask's bit
// length, so a subsequent truncation to at least that many bits is redundant
// and can be removed (issue #8628): `x & 123` fits in 7 bits, so the truncate
// emitted for the `as u8` cast cannot change the value.
//
// Masks of the form `2^n - 1` (like `0xff`) are canonicalized to `truncate`
// earlier in SSA and never reach the pass as `and`, so the masks here are
// deliberately not of that form. The `& 0x1ab` case is bounded by 9 bits, so
// its 8-bit truncation is semantically active and must be kept (for the input
// in Prover.toml, `x & 0x1ab` is 417 and the cast must reduce it to 161).
fn main(x: u64, y: u32, expected_a: u8, expected_b: u8, expected_c: u16, expected_kept: u8) {
    // 123 = 0b111_1011 fits in 7 bits: the 8-bit truncate is removable.
    let a = (x & 123) as u8;
    assert(a == expected_a);

    // Non-contiguous 8-bit mask 0xab = 0b1010_1011: the 8-bit truncate is removable.
    let b = (x & 0xab) as u8;
    assert(b == expected_b);

    // Non-contiguous 13-bit mask on a u32: the 16-bit truncate is removable.
    let c = (y & 0x1234) as u16;
    assert(c == expected_c);

    // 9-bit mask 0x1ab: NOT removable for an 8-bit cast; must keep the truncation.
    let kept = (x & 0x1ab) as u8;
    assert(kept == expected_kept);
}
