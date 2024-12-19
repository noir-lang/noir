unconstrained fn decode_ascii(ascii: u8) -> u8 {
    if ascii < 58 {
        ascii - 48
    } else if ascii < 71 {
        ascii - 55
    } else {
        ascii - 87
    }
}

unconstrained fn decode_hex<let N: u32, let M: u32>(s: str<N>) -> [u8; M] {
    let mut result: [u8; M] = [0; M];
    let as_bytes = s.as_bytes();
    for i in 0..N {
        if i % 2 != 0 {
            continue;
        }
        result[i / 2] = decode_ascii(as_bytes[i]) * 16 + decode_ascii(as_bytes[i + 1]);
    }
    result
}

unconstrained fn cipher(plaintext: [u8; 12], iv: [u8; 16], key: [u8; 16]) -> [u8; 16] {
    let result = std::aes128::aes128_encrypt(plaintext, iv, key);
    result.as_array()
}

fn main(inputs: str<12>, iv: str<16>, key: str<16>, output: str<32>) {
    let result: [u8; 16] =
        std::aes128::aes128_encrypt(inputs.as_bytes(), iv.as_bytes(), key.as_bytes()).as_array();

    let output_bytes: [u8; 16] = unsafe {
        //@safety: testing context
        decode_hex(output)
    };
    assert(result == output_bytes);

    let unconstrained_result = unsafe {
        //@safety: testing context
        cipher(inputs.as_bytes(), iv.as_bytes(), key.as_bytes())
    };
    assert(unconstrained_result == output_bytes);
}
