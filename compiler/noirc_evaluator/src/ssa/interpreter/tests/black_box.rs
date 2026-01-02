use crate::ssa::{
    interpreter::tests::{expect_values, expect_values_with_args, from_constant, from_u32_vector},
    ir::types::NumericType,
};

#[test]
fn test_msm() {
    let src = "
  acir(inline) fn main f0  {
    b0(v0: Field, v1: Field):
      v2 = make_array [Field 1, Field 17631683881184975370165255887551781615748388533673675138860, u1 0] : [(Field, Field, u1); 1]
      v3 = make_array [v0, v1] : [(Field, Field); 1]
      v4= call multi_scalar_mul(v2, v3, u1 1) -> [(Field, Field, u1); 1]
      return v4
  }
      ";
    let values = expect_values_with_args(
        src,
        vec![
            from_constant(1_u128.into(), NumericType::NativeField),
            from_constant(0_u128.into(), NumericType::NativeField),
        ],
    );

    assert!(values.len() == 1);
}

#[test]
fn test_ec_add() {
    let src = "
  acir(inline) fn main f0  {
    b0(v0: Field):
      v1 = call embedded_curve_add(v0, Field 17631683881184975370165255887551781615748388533673675138860, u1 0, v0, Field 17631683881184975370165255887551781615748388533673675138860, u1 0, u1 1) -> [(Field, Field, u1); 1]
      return v1
  }
      ";
    let values =
        expect_values_with_args(src, vec![from_constant(1_u128.into(), NumericType::NativeField)]);

    assert!(values.len() == 1);
}

#[test]
fn test_pedersen() {
    let src = r#"
  acir(inline) fn main f0  {
    b0():
      separator = make_array b"DEFAULT_DOMAIN_SEPARATOR"
      v1 = call derive_pedersen_generators(separator, u32 0) -> [(Field, Field, u1); 1]
      return v1
  }
      "#;
    let values = expect_values(src);
    let result = values[0].as_array_or_vector().unwrap();
    assert_eq!(result.elements.borrow().len(), 3);
}
#[test]
fn test_aes() {
    let src = "
  acir(inline) fn main f0  {
    b0(v0: [u8; 12], v1: [u8; 16], v2: [u8; 16]):
      v4 = call aes128_encrypt(v0, v1, v2) -> [u8; 16]
      return v4
  }
      ";
    let a = from_u32_vector(
        &[107, 101, 118, 108, 111, 118, 101, 115, 114, 117, 115, 116],
        NumericType::unsigned(8),
    );
    let iv = from_u32_vector(
        &[48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48],
        NumericType::unsigned(8),
    );
    let key = from_u32_vector(
        &[48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48],
        NumericType::unsigned(8),
    );
    let values = expect_values_with_args(src, vec![a, iv, key]);

    let result = values[0].as_array_or_vector().unwrap();
    let result = result.elements.borrow();
    let result = result.iter().map(|v| v.as_u8().unwrap());
    assert_eq!(
        result.collect::<Vec<u8>>(),
        vec![244, 14, 126, 172, 171, 40, 208, 186, 173, 184, 226, 105, 238, 122, 205, 191]
    );
}

#[test]
fn test_blake3() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: [u8; 5]):
        v3 = call blake3(v0) -> [u8; 32]
        return v3
    }
      ";
    let input = from_u32_vector(&[104, 101, 108, 108, 111], NumericType::unsigned(8));

    let values = expect_values_with_args(src, vec![input]);
    assert!(values.len() == 1);
    let result = values[0].as_array_or_vector().unwrap();
    let result = result.elements.borrow();
    let result = result.iter().map(|v| v.as_u8().unwrap());
    assert_eq!(
        result.collect::<Vec<u8>>(),
        vec![
            234, 143, 22, 61, 179, 134, 130, 146, 94, 68, 145, 197, 229, 141, 75, 179, 80, 110,
            248, 193, 78, 183, 138, 134, 233, 8, 197, 98, 74, 103, 32, 15
        ]
    );
}

#[test]
fn test_blake2s() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: [u8; 5]):
        v3 = call blake2s(v0) -> [u8; 32]
        return v3
    }
      ";
    let input = from_u32_vector(&[104, 101, 108, 108, 111], NumericType::unsigned(8));

    let values = expect_values_with_args(src, vec![input]);
    assert!(values.len() == 1);
    let result = values[0].as_array_or_vector().unwrap();
    let result = result.elements.borrow();
    let result = result.iter().map(|v| v.as_u8().unwrap());
    assert_eq!(
        result.collect::<Vec<u8>>(),
        vec![
            25, 33, 59, 172, 197, 141, 238, 109, 189, 227, 206, 185, 164, 124, 187, 51, 11, 61,
            134, 248, 204, 168, 153, 126, 176, 11, 228, 86, 241, 64, 202, 37
        ]
    );
}

#[test]
fn test_keccak() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: [u64; 25]):
        v1 = call keccakf1600(v0) -> [u64; 25]
        return v1
    }
      ";
    let input = from_u32_vector(
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
        NumericType::unsigned(64),
    );

    let values = expect_values_with_args(src, vec![input]);
    assert!(values.len() == 1);
}

#[test]
fn test_poseidon() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: [Field; 4]):
        v1 = call poseidon2_permutation(v0) -> [Field; 4]
        return v1
    }
      ";
    let input = from_u32_vector(&[1, 2, 3, 4], NumericType::NativeField);

    let values = expect_values_with_args(src, vec![input]);
    assert_eq!(values.len(), 1);
}

#[test]
fn test_sha256() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: [u32; 16], v1: [u32; 8]):
        v2 = call sha256_compression(v0, v1) -> [u32; 8]
        return v2
    }
      ";
    let input = from_u32_vector(
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6],
        NumericType::unsigned(32),
    );
    let state = from_u32_vector(&[1, 2, 3, 4, 5, 6, 7, 8], NumericType::unsigned(32));

    let values = expect_values_with_args(src, vec![input, state]);
    assert!(values.len() == 1);
}

#[test]
fn test_ecdsa_k1() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
        v4 = call ecdsa_secp256k1(v0, v1, v2, v3, u1 1) -> u1
        return v4
    }
      ";
    let pub_key_x = from_u32_vector(
        &[
            160, 67, 77, 158, 71, 243, 200, 98, 53, 71, 124, 123, 26, 230, 174, 93, 52, 66, 212,
            155, 25, 67, 194, 183, 82, 166, 142, 42, 71, 226, 71, 199,
        ],
        NumericType::unsigned(8),
    );
    let pub_key_y = from_u32_vector(
        &[
            137, 58, 186, 66, 84, 25, 188, 39, 163, 182, 199, 230, 147, 162, 76, 105, 111, 121, 76,
            46, 216, 119, 161, 89, 60, 190, 229, 59, 3, 115, 104, 215,
        ],
        NumericType::unsigned(8),
    );

    let signature = from_u32_vector(
        &[
            229, 8, 28, 128, 171, 66, 125, 195, 112, 52, 111, 74, 14, 49, 170, 43, 173, 141, 151,
            152, 195, 128, 97, 219, 154, 229, 90, 78, 141, 244, 84, 253, 40, 17, 152, 148, 52, 78,
            113, 183, 135, 112, 204, 147, 29, 97, 244, 128, 236, 187, 11, 137, 214, 235, 105, 105,
            1, 97, 228, 154, 113, 95, 205, 85,
        ],
        NumericType::unsigned(8),
    );
    let message = from_u32_vector(
        &[
            58, 115, 244, 18, 58, 92, 210, 18, 31, 33, 205, 126, 141, 53, 136, 53, 71, 105, 73,
            208, 53, 217, 194, 218, 104, 6, 180, 99, 58, 200, 193, 226,
        ],
        NumericType::unsigned(8),
    );

    let values = expect_values_with_args(src, vec![pub_key_x, pub_key_y, signature, message]);
    assert!(values.len() == 1);
    let result = values[0].as_bool().unwrap();
    assert!(result);
}

#[test]
fn test_ecdsa_r1() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
        v4 = call ecdsa_secp256r1(v0, v1, v2, v3, u1 1) -> u1
        return v4
    }
      ";
    let pub_key_x = from_u32_vector(
        &[
            85, 15, 71, 16, 3, 243, 223, 151, 195, 223, 80, 106, 199, 151, 246, 114, 31, 177, 161,
            251, 123, 143, 111, 131, 210, 36, 73, 138, 101, 200, 142, 36,
        ],
        NumericType::unsigned(8),
    );
    let pub_key_y = from_u32_vector(
        &[
            19, 96, 147, 215, 1, 46, 80, 154, 115, 113, 92, 189, 11, 0, 163, 204, 15, 244, 181,
            192, 27, 63, 250, 25, 106, 177, 251, 50, 112, 54, 184, 230,
        ],
        NumericType::unsigned(8),
    );

    let signature = from_u32_vector(
        &[
            44, 112, 168, 208, 132, 182, 43, 252, 92, 224, 54, 65, 202, 249, 247, 42, 212, 218,
            140, 129, 191, 230, 236, 148, 135, 187, 94, 27, 239, 98, 161, 50, 24, 173, 158, 226,
            158, 175, 53, 31, 220, 80, 241, 82, 12, 66, 94, 155, 144, 138, 7, 39, 139, 67, 176,
            236, 123, 135, 39, 120, 193, 78, 7, 132,
        ],
        NumericType::unsigned(8),
    );
    let message = from_u32_vector(
        &[
            84, 112, 91, 163, 186, 175, 219, 223, 186, 140, 95, 154, 112, 247, 168, 155, 238, 152,
            217, 6, 181, 62, 49, 7, 77, 167, 186, 236, 220, 13, 169, 173,
        ],
        NumericType::unsigned(8),
    );

    let values = expect_values_with_args(src, vec![pub_key_x, pub_key_y, signature, message]);
    assert!(values.len() == 1);
    let result = values[0].as_bool().unwrap();
    assert!(result);
}
