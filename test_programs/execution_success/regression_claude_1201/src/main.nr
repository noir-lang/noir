fn main(s: str<2>, x: u8) -> pub [u8; 2] {
    // Safety: testing context
    unsafe {
        mutate_bytes_then_observe_original(s, x)
    }
}

// Asserts that mutating the byte array returned by `as_bytes` does not
// corrupt the source string `s`: a converted view must not alias storage
// that the caller still reads later.
unconstrained fn mutate_bytes_then_observe_original(s: str<2>, x: u8) -> [u8; 2] {
    let mut bytes = s.as_bytes();
    bytes[0] = x;
    assert_eq(s.as_bytes()[0], 97);
    bytes
}
