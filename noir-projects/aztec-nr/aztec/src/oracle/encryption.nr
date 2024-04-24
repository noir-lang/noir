
#[oracle(aes128Encrypt)]
pub fn aes128_encrypt_oracle<N>(input: [u8; N], iv: [u8; 16], key: [u8; 16]) -> [u8; N] {}

unconstrained pub fn aes128_encrypt<N>(input: [u8; N], iv: [u8; 16], key: [u8; 16]) -> [u8; N] {
    aes128_encrypt_oracle(input, iv, key)
}
