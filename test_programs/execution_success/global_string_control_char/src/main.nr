// A string global containing a control character is valid UTF-8 but can't be printed as a
// Noir string literal (Rust's `{:?}` escapes it as `\u{..}`, which Noir doesn't lex), so
// `nargo expand` must print the initializer expression instead of the value.
global BYTES: [u8; 4] = [102, 7, 111, 111];
global S: str<4> = BYTES.as_str_unchecked();

fn main(x: Field) {
    let bytes = S.as_bytes();
    assert(bytes[1] as Field + x == 12);
    assert(x == 5);
}
