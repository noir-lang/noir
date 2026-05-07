global BROKEN_BYTES: [u8; 4] = [102, 111, 128, 111];
global S: str<4> = BROKEN_BYTES.as_str_unchecked();

fn main() {
    // This breaks SSA before #12269, causing a length mismatch in expected types
    println(S);
}
