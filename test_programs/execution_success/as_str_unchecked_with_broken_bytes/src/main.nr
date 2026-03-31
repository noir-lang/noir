fn main() {
    check_runtime();
    check_comptime();
}

fn check_runtime() {
    // 0xFF is invalid UTF-8
    let bytes: [u8; 3] = [0x41, 0xFF, 0x42];
    let s: str<3> = bytes.as_str_unchecked();
    let roundtrip: [u8; 3] = s.as_bytes();

    assert(roundtrip[0] == 0x41);
    assert(roundtrip[1] == 0xFF);
    assert(roundtrip[2] == 0x42);
}

fn check_comptime() {
    let (raw, roundtripped_bytes) = lossy_roundtrip!();
    assert_eq(raw, [102, 111, 128, 111]);
    assert_eq(roundtripped_bytes, [102, 111, 128, 111]);
}

comptime fn lossy_roundtrip() -> Quoted {
    let raw: [u8; 4] = [102, 111, 128, 111]; // "fo\x80o" (0x80 is invalid UTF-8)
    let s: str<4> = raw.as_str_unchecked();
    quote { ($raw, ($s).as_bytes()) }
}
