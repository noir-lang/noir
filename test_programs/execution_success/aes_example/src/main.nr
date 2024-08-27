// docs:start:aes_example
fn main() -> pub [u8; 16] {
    let input: [u8; 4] = [0, 12, 3, 15]; // Random bytes, will be padded to 16 bytes.
    let iv: [u8; 16] = [0; 16]; // Initialisation vector
    let key: [u8; 16] = [0; 16]; // AES key
    let ciphertext = std::aes128::aes128_encrypt(input, iv, key); // In this case, the output length will be 16 bytes.
    ciphertext.as_array()
}
// docs:start:aes_example
