// Nested arrays are not yet implemented.
#[oracle(byte_array_vec)]
unconstrained fn byte_array_vec() -> [[u8; 2]] {}

// Strings are arrays of u8, so they should also be rejected.
#[oracle(str_vec)]
unconstrained fn str_vec() -> [str<2>] {}

unconstrained fn main() {
    let _ = byte_array_vec();
    let _ = str_vec();
}
