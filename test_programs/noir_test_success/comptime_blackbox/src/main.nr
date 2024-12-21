//! Tests to show that the comptime interpreter implement blackbox functions.
use std::bigint;
use std::embedded_curve_ops::{EmbeddedCurvePoint, EmbeddedCurveScalar, multi_scalar_mul};

/// Test that all bigint operations work in comptime.
#[test]
fn test_bigint() {
    let result: [u8] = comptime {
        let a = bigint::Secpk1Fq::from_le_bytes(&[0, 1, 2, 3, 4]);
        let b = bigint::Secpk1Fq::from_le_bytes(&[5, 6, 7, 8, 9]);
        let c = (a + b) * b / a - a;
        c.to_le_bytes()
    };
    // Do the same calculation outside comptime.
    let a = bigint::Secpk1Fq::from_le_bytes(&[0, 1, 2, 3, 4]);
    let b = bigint::Secpk1Fq::from_le_bytes(&[5, 6, 7, 8, 9]);
    let c = bigint::Secpk1Fq::from_le_bytes(result);
    assert_eq(c, (a + b) * b / a - a);
}

/// Test that to_le_radix returns an array.
#[test]
fn test_to_le_radix() {
    comptime {
        let field = 2;
        let bytes: [u8; 8] = field.to_le_radix(256);
        let _num = bigint::BigInt::from_le_bytes(bytes, bigint::bn254_fq);
    };
}

#[test]
fn test_bitshift() {
    let c = comptime {
        let a: i32 = 10;
        let b: u32 = 4;
        a << b as u8
    };
    assert_eq(c, 160);
}

#[test]
fn test_aes128_encrypt() {
    let ciphertext = comptime {
        let plaintext: [u8; 5] = [1, 2, 3, 4, 5];
        let iv: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let key: [u8; 16] = [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25];
        std::aes128::aes128_encrypt(plaintext, iv, key)
    };
    let clear_len = 5;
    let cipher_len = clear_len + 16 - clear_len % 16;
    assert_eq(ciphertext.len(), cipher_len);
}

#[test]
fn test_blake2s() {
    let hash = comptime {
        let input = [104, 101, 108, 108, 111];
        std::hash::blake2s(input)
    };
    assert_eq(hash[0], 0x19);
    assert_eq(hash[31], 0x25);
}

#[test]
fn test_blake3() {
    let hash = comptime {
        let input = [104, 101, 108, 108, 111];
        std::hash::blake3(input)
    };
    assert_eq(hash[0], 0xea);
    assert_eq(hash[31], 0x0f);
}

/// Test that ecdsa_secp256k1 is implemented.
#[test]
fn test_ecdsa_secp256k1() {
    let (valid_array, valid_slice) = comptime {
        let pub_key_x: [u8; 32] = hex_to_bytes("a0434d9e47f3c86235477c7b1ae6ae5d3442d49b1943c2b752a68e2a47e247c7").as_array();
        let pub_key_y: [u8; 32] = hex_to_bytes("893aba425419bc27a3b6c7e693a24c696f794c2ed877a1593cbee53b037368d7").as_array();
        let signature: [u8; 64] = hex_to_bytes("e5081c80ab427dc370346f4a0e31aa2bad8d9798c38061db9ae55a4e8df454fd28119894344e71b78770cc931d61f480ecbb0b89d6eb69690161e49a715fcd55").as_array();
        let hashed_message: [u8; 32] = hex_to_bytes("3a73f4123a5cd2121f21cd7e8d358835476949d035d9c2da6806b4633ac8c1e2").as_array();

        let valid_array = std::ecdsa_secp256k1::verify_signature(pub_key_x, pub_key_y, signature, hashed_message);
        let valid_slice = std::ecdsa_secp256k1::verify_signature_slice(pub_key_x, pub_key_y, signature, hashed_message.as_slice());

        (valid_array, valid_slice)
    };
    assert(valid_array);
    assert(valid_slice);
}

/// Test that ecdsa_secp256r1 is implemented.
#[test]
fn test_ecdsa_secp256r1() {
    let (valid_array, valid_slice) = comptime {
        let pub_key_x: [u8; 32] = hex_to_bytes("550f471003f3df97c3df506ac797f6721fb1a1fb7b8f6f83d224498a65c88e24").as_array();
        let pub_key_y: [u8; 32] = hex_to_bytes("136093d7012e509a73715cbd0b00a3cc0ff4b5c01b3ffa196ab1fb327036b8e6").as_array();
        let signature: [u8; 64] = hex_to_bytes("2c70a8d084b62bfc5ce03641caf9f72ad4da8c81bfe6ec9487bb5e1bef62a13218ad9ee29eaf351fdc50f1520c425e9b908a07278b43b0ec7b872778c14e0784").as_array();
        let hashed_message: [u8; 32] = hex_to_bytes("54705ba3baafdbdfba8c5f9a70f7a89bee98d906b53e31074da7baecdc0da9ad").as_array();

        let valid_array = std::ecdsa_secp256r1::verify_signature(pub_key_x, pub_key_y, signature, hashed_message);
        let valid_slice = std::ecdsa_secp256r1::verify_signature_slice(pub_key_x, pub_key_y, signature, hashed_message.as_slice());
        (valid_array, valid_slice)
    };
    assert(valid_array);
    assert(valid_slice);
}

/// Test that sha256_compression is implemented.
#[test]
fn test_sha256() {
    let hash = comptime {
        let input: [u8; 1] = [0xbd];
        std::hash::sha256(input)
    };
    assert_eq(hash[0], 0x68);
    assert_eq(hash[31], 0x2b);
}

/// Test that `embedded_curve_add` and `multi_scalar_mul` are implemented.
#[test]
fn test_embedded_curve_ops() {
    let (sum, mul) = comptime {
        let s1 = EmbeddedCurveScalar { lo: 1, hi: 0 };
        let s2 = EmbeddedCurveScalar { lo: 2, hi: 0 };
        let g1 = EmbeddedCurvePoint { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };
        let g2 = multi_scalar_mul([g1], [s2]);
        let sum = g1 + g2;
        let mul = multi_scalar_mul([g1, g2], [s1, s1]);
        (sum, mul)
    };
    assert_eq(sum, mul);
}

/// Parse a lowercase hexadecimal string (without 0x prefix) as byte slice.
comptime fn hex_to_bytes<let N: u32>(s: str<N>) -> [u8] {
    assert(N % 2 == 0);
    let mut out = &[];
    let bz = s.as_bytes();
    let mut h: u32 = 0;
    for i in 0 .. bz.len() {
        let ascii = bz[i];
        let d = if ascii < 58 {
            ascii - 48
        } else {
            assert(ascii >= 97); // enforce >= 'a'
            assert(ascii <= 102); // enforce <= 'f'
            ascii - 87
        };
        h = h * 16 + d as u32;
        if i % 2 == 1 {
            out = out.push_back(h as u8);
            h = 0;
        }
    }
    out
}
