// This file is kept for backwards compatibility.
#[deprecated("sha256 is being deprecated from the stdlib, use https://github.com/noir-lang/sha256 instead")]
pub fn digest<let N: u32>(msg: [u8; N]) -> [u8; 32] {
    crate::hash::sha256::digest(msg)
}

#[deprecated("sha256 is being deprecated from the stdlib, use https://github.com/noir-lang/sha256 instead")]
pub fn sha256_var<let N: u32>(msg: [u8; N], message_size: u64) -> [u8; 32] {
    crate::hash::sha256::sha256_var(msg, message_size)
}
