// Nested arrays are not yet implemented.
#[oracle(byte_array_vec)]
unconstrained fn byte_array_vec() -> [[u8; 2]] {}

// The vector doesn't have to be top level.
#[oracle(byte_array_vec_in_tuple)]
unconstrained fn byte_array_vec_in_tuple() -> (u32, [[u8; 2]], bool) {}

// Strings are arrays of u8, so they should also be rejected.
#[oracle(str_vec)]
unconstrained fn str_vec() -> [str<2>] {}

// Nested vectors are not implemented.
#[oracle(byte_vec_vec)]
unconstrained fn byte_vec_vec() -> [[u8]] {}

unconstrained fn main() {
    let _ = byte_array_vec();
    let _ = byte_array_vec_in_tuple();
    let _ = byte_vec_vec();
    let _ = str_vec();
}
