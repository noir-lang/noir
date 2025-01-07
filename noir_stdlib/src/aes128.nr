#[foreign(aes128_encrypt)]
// docs:start:aes128
pub fn aes128_encrypt<let N: u32>(
    input: [u8; N],
    iv: [u8; 16],
    key: [u8; 16],
) -> [u8; N + 16 - N % 16] {}
// docs:end:aes128
