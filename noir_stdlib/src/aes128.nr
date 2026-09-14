// docs:start:aes128
/// Given a plaintext as an array of bytes, returns the corresponding aes128 ciphertext (CBC mode). Input padding is performed using PKCS#7, so that the output length is `input.len() + (16 - input.len() % 16)`.
pub fn aes128_encrypt<let N: u32>(
    input: [u8; N],
    iv: [u8; 16],
    key: [u8; 16],
) -> [u8; N + 16 - N % 16] {
    let padding_length = (16 - N % 16) as u8;
    let mut padded_input: [u8; N + 16 - N % 16] = [0; N + 16 - N % 16];
    for i in 0..N {
        padded_input[i] = input[i];
    }
    for i in N..N + 16 - N % 16 {
        padded_input[i] = padding_length;
    }
    let output = aes128_encrypt_padded_input(padded_input, iv, key);
    output
}

#[foreign(aes128_encrypt)]
fn aes128_encrypt_padded_input<let N: u32>(input: [u8; N], iv: [u8; 16], key: [u8; 16]) -> [u8; N] {}

// docs:end:aes128

mod tests {
    use super::aes128_encrypt;

    #[test]
    fn encrypt() {
        let input = "kevlovesrust".as_bytes();
        let iv = "0000000000000000".as_bytes();
        let key = "0000000000000000".as_bytes();
        let output = [244, 14, 126, 172, 171, 40, 208, 186, 173, 184, 226, 105, 238, 122, 205, 191];
        assert_eq(aes128_encrypt(input, iv, key), output);
    }
}
