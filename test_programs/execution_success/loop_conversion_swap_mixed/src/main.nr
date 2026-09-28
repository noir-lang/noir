// Swapping a byte array and a string through identity conversions, inside a loop. SSA simplifies
// `as_str_unchecked` and `as_bytes` to their argument, so the loop permutes the two buffers between
// loop-header parameters, and `rc_invariant` must accept the in-place write after it.
unconstrained fn main(bytes: [u8; 2], text: str<2>, n: u32) {
    let mut bytes = bytes;
    let mut text = text;
    for _i in 0..n {
        let previous = bytes.as_str_unchecked();
        bytes = text.as_bytes();
        text = previous;
    }
    bytes[0] = bytes[0] + 1;
    assert_eq(text.as_bytes(), [65, 66]);
    assert_eq(bytes, [68, 68]);
}
