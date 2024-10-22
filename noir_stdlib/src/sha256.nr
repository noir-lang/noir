// This file is kept for backwards compatibility.
#[deprecated]
pub fn digest<let N: u32>(msg: [u8; N]) -> [u8; 32] {
    crate::hash::sha256::digest(msg)
}

#[deprecated]
pub fn sha256_var<let N: u32>(msg: [u8; N], message_size: u64) -> [u8; 32] {
    crate::hash::sha256::sha256_var(msg, message_size)
}
