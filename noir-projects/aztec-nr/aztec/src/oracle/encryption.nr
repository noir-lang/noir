#[oracle(aes128Encrypt)]
pub fn aes128_encrypt_oracle<N, M>(input: [u8; N], iv: [u8; 16], key: [u8; 16]) -> [u8; M] {}

// AES 128 CBC with PKCS7 is padding to multiples of 16 bytes so M has to be a multiple of 16!
// (e.g. from 65 bytes long input you get 80 bytes long output and M has to be set to `80`)
unconstrained pub fn aes128_encrypt<N, M>(input: [u8; N], iv: [u8; 16], key: [u8; 16]) -> [u8; M] {
    aes128_encrypt_oracle(input, iv, key)
}
