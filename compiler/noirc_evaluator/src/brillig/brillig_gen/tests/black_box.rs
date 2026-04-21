use crate::{
    assert_artifact_snapshot, brillig::brillig_gen::tests::ssa_to_brillig_artifacts,
    ssa::ir::map::Id,
};

// Tests Blake2s hash function with message input and 32-byte output
#[test]
fn brillig_blake2s() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [u8; 10]):
        v1 = call blake2s(v0) -> [u8; 32]
        return v1
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[3] = @1
    1: sp[4] = const u32 33
    2: @1 = u32 add @1, sp[4]
    3: sp[3] = indirect const u32 1
    4: sp[4] = u32 add sp[2], @2
    5: sp[5] = u32 add sp[3], @2
    6: blake2s(message: [sp[4]; 10], output: [sp[5]; 32])
    7: sp[2] = sp[3]
    8: return
    ");
}

// Tests Blake3 hash function with message input and 32-byte output
#[test]
fn brillig_blake3() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [u8; 10]):
        v1 = call blake3(v0) -> [u8; 32]
        return v1
    }
    ";
    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[3] = @1
    1: sp[4] = const u32 33
    2: @1 = u32 add @1, sp[4]
    3: sp[3] = indirect const u32 1
    4: sp[4] = u32 add sp[2], @2
    5: sp[5] = u32 add sp[3], @2
    6: blake3(message: [sp[4]; 10], output: [sp[5]; 32])
    7: sp[2] = sp[3]
    8: return
    ");
}

// Tests Keccakf1600 permutation with 25-element input and output arrays
#[test]
fn brillig_keccakf1600() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [u64; 25]):
        v1 = call keccakf1600(v0) -> [u64; 25]
        return v1
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[3] = @1
    1: sp[4] = const u32 26
    2: @1 = u32 add @1, sp[4]
    3: sp[3] = indirect const u32 1
    4: sp[4] = u32 add sp[2], @2
    5: sp[5] = u32 add sp[3], @2
    6: keccakf1600(input: [sp[4]; 25], output: [sp[5]; 25])
    7: sp[2] = sp[3]
    8: return
    ");
}

// Tests ECDSA signature verification on secp256k1 curve
#[test]
fn brillig_ecdsa_secp256k1() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
        v4 = call ecdsa_secp256k1(v0, v1, v2, v3, u1 1) -> u1
        return v4
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[6] = const bool 1
    1: sp[8] = u32 add sp[5], @2
    2: sp[9] = u32 add sp[2], @2
    3: sp[10] = u32 add sp[3], @2
    4: sp[11] = u32 add sp[4], @2
    5: ecdsa_secp256k1(hashed_msg: [sp[8]; 32], public_key_x: [sp[9]; 32], public_key_y: [sp[10]; 32], signature: [sp[11]; 64], result: sp[7])
    6: sp[2] = sp[7]
    7: return
    ");
}

// Tests ECDSA signature verification on secp256r1 curve
#[test]
fn brillig_ecdsa_secp256r1() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
        v4 = call ecdsa_secp256r1(v0, v1, v2, v3, u1 1) -> u1
        return v4
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[6] = const bool 1
    1: sp[8] = u32 add sp[5], @2
    2: sp[9] = u32 add sp[2], @2
    3: sp[10] = u32 add sp[3], @2
    4: sp[11] = u32 add sp[4], @2
    5: ecdsa_secp256r1(hashed_msg: [sp[8]; 32], public_key_x: [sp[9]; 32], public_key_y: [sp[10]; 32], signature: [sp[11]; 64], result: sp[7])
    6: sp[2] = sp[7]
    7: return
    ");
}

// Tests multi-scalar multiplication on embedded curve
#[test]
fn brillig_multi_scalar_mul() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [(Field, Field, u1); 2], v1: [(Field, Field); 2]):
        v2 = call multi_scalar_mul(v0, v1, u1 1) -> [(Field, Field, u1); 1]
        return v2
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: sp[4] = const bool 1
     1: sp[5] = @1
     2: sp[6] = const u32 4
     3: @1 = u32 add @1, sp[6]
     4: sp[5] = indirect const u32 1
     5: sp[6] = u32 add sp[2], @2
     6: sp[7] = u32 add sp[3], @2
     7: sp[8] = u32 add sp[5], @2
     8: multi_scalar_mul(points: [sp[6]; 6], scalars: [sp[7]; 4], outputs: [sp[8]; 3])
     9: sp[2] = sp[5]
    10: return
    ");
}

// Tests embedded curve point addition
#[test]
fn brillig_embedded_curve_add() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: Field, v1: Field, v2: u1, v3: Field, v4: Field, v5: u1):
        v6 = call embedded_curve_add(v0, v1, v2, v3, v4, v5, u1 1) -> [(Field, Field, u1); 1]
        return v6
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[8] = const bool 1
    1: sp[9] = @1
    2: sp[10] = const u32 4
    3: @1 = u32 add @1, sp[10]
    4: sp[9] = indirect const u32 1
    5: sp[10] = u32 add sp[9], @2
    6: embedded_curve_add(input1_x: sp[2], input1_y: sp[3], input1_infinite: sp[4], input2_x: sp[5], input2_y: sp[6], input2_infinite: sp[7], result: [sp[10]; 3])
    7: sp[2] = sp[9]
    8: return
    ");
}

// Tests Poseidon2 permutation hash function
#[test]
fn brillig_poseidon2_permutation() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [Field; 4]):
        v1 = call poseidon2_permutation(v0) -> [Field; 4]
        return v1
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
    0: sp[3] = @1
    1: sp[4] = const u32 5
    2: @1 = u32 add @1, sp[4]
    3: sp[3] = indirect const u32 1
    4: sp[4] = u32 add sp[2], @2
    5: sp[5] = u32 add sp[3], @2
    6: poseidon2_permutation(message: [sp[4]; 4], output: [sp[5]; 4])
    7: sp[2] = sp[3]
    8: return
    ");
}

// Tests SHA256 compression function with input and hash values
#[test]
fn brillig_sha256_compression() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [u32; 16], v1: [u32; 8]):
        v2 = call sha256_compression(v0, v1) -> [u32; 8]
        return v2
    }
    ";
    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: sp[4] = @1
     1: sp[5] = const u32 9
     2: @1 = u32 add @1, sp[5]
     3: sp[4] = indirect const u32 1
     4: sp[5] = u32 add sp[2], @2
     5: sp[6] = u32 add sp[3], @2
     6: sp[7] = u32 add sp[4], @2
     7: sha256_compression(input: [sp[5]; 16], hash_values: [sp[6]; 8], output: [sp[7]; 8])
     8: sp[2] = sp[4]
     9: return
    ");
}

// Tests AES128 encryption with plaintext, IV, and key inputs
#[test]
fn brillig_aes128_encrypt() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [u8; 16], v1: [u8; 16], v2: [u8; 16]):
        v3 = call aes128_encrypt(v0, v1, v2) -> [u8; 16]
        return v3
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: sp[5] = @1
     1: sp[6] = const u32 17
     2: @1 = u32 add @1, sp[6]
     3: sp[5] = indirect const u32 1
     4: sp[6] = u32 add sp[2], @2
     5: sp[7] = u32 add sp[3], @2
     6: sp[8] = u32 add sp[4], @2
     7: sp[9] = u32 add sp[5], @2
     8: aes_128_encrypt(inputs: [sp[6]; 16], iv: [sp[7]; 16], key: [sp[8]; 16], outputs: [sp[9]; 16])
     9: sp[2] = sp[5]
    10: return
    ");
}
