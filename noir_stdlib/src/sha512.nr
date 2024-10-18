// This file is kept for backwards compatibility.
#[deprecated]
pub fn digest<let N: u32>(msg: [u8; N]) -> [u8; 64] {
    crate::hash::sha512::digest(msg)
}
