// docs:start:aes128
// Given a plaintext as an array of bytes, returns the corresponding aes128 ciphertext (CBC mode). Input padding is performed using PKCS#7, so that the output length is `input.len() + (16 - input.len() % 16)`.
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
