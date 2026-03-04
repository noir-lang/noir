fn main() {
    // 0xFF is invalid UTF-8
    let bytes: [u8; 3] = [0x41, 0xFF, 0x42];
    let s: str<3> = bytes.as_str_unchecked();
    let roundtrip: [u8; 3] = s.as_bytes();

    assert(roundtrip[0] == 0x41);
    assert(roundtrip[1] == 0xFF);
    assert(roundtrip[2] == 0x42);
}
