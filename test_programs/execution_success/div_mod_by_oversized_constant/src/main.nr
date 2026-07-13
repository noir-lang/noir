// Unsigned division/modulo where the constant divisor is strictly greater than the
// maximum possible value of the dividend: `wide` and `masked` are provably at most
// 255, so `/ 1000`, `% 1000`, `/ 300` and `% 300` are compile-time simplifiable to
// `0` / the dividend. The `/ 100` and `% 100` cases are NOT simplifiable (100 is
// within the dividend's range) and must keep full division semantics.
fn main(
    x: u8,
    y: u32,
    expected_q: u32,
    expected_r: u32,
    expected_masked_q: u32,
    expected_masked_r: u32,
    expected_small_q: u32,
    expected_small_r: u32,
) {
    // Upcast-bounded dividend: at most 8 bits.
    let wide = x as u32;
    assert(wide / 1000 == expected_q);
    assert(wide % 1000 == expected_r);

    // Mask-bounded dividend: at most 8 bits.
    let masked = y & 0xff;
    assert(masked / 300 == expected_masked_q);
    assert(masked % 300 == expected_masked_r);

    // In-range divisor: must not be simplified away.
    assert(wide / 100 == expected_small_q);
    assert(wide % 100 == expected_small_r);
}
