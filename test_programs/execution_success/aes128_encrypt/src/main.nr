use dep::std;

unconstrained fn decode_ascii(ascii: u8) -> u8 {
    if ascii < 58 {
        ascii - 48
    } else if ascii < 71 {
        ascii - 55
    } else {
        ascii - 87
    }
}

unconstrained fn decode_hex<N, M>(s: str<N>) -> [u8; M] {
    let mut result: [u8; M] = [0; M];
    let as_bytes = s.as_bytes();
    for i in 0..N {
        if i % 2 != 0 {
            continue;
        }
        result[i/2] = decode_ascii(as_bytes[i]) * 16 + decode_ascii(as_bytes[i + 1]);
    }
    result
}

unconstrained fn cipher(plaintext: [u8; 12], iv: [u8; 16], key: [u8; 16]) -> [u8; 16] {
    let slice_res = std::aes128::aes128_encrypt(plaintext, iv, key);
    let mut result = [0; 16];
    for i in 0..16 {
        result[i] = slice_res[i];
    }
    result
}

fn main(inputs: str<12>, iv: str<16>, key: str<16>, output: str<32>) {
    let result = std::aes128::aes128_encrypt(inputs.as_bytes(), iv.as_bytes(), key.as_bytes());
    let output_bytes: [u8; 16] = decode_hex(output);
    for i in 0..16 {
        assert(result[i] == output_bytes[i]);
    }
    let unconstrained_result = cipher(inputs.as_bytes(), iv.as_bytes(), key.as_bytes());
    for i in 0..16 {
        assert(unconstrained_result[i] == output_bytes[i]);
    }
}
