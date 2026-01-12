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
     0: call 0
     1: sp[2] = @1
     2: sp[3] = const u32 33
     3: @1 = u32 add @1, sp[3]
     4: sp[2] = indirect const u32 1
     5: sp[3] = u32 add sp[1], @2
     6: sp[4] = u32 add sp[2], @2
     7: blake2s(message: [sp[3]; 10], output: [sp[4]; 32])
     8: sp[1] = sp[2]
     9: return
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
     0: call 0
     1: sp[2] = @1
     2: sp[3] = const u32 33
     3: @1 = u32 add @1, sp[3]
     4: sp[2] = indirect const u32 1
     5: sp[3] = u32 add sp[1], @2
     6: sp[4] = u32 add sp[2], @2
     7: blake3(message: [sp[3]; 10], output: [sp[4]; 32])
     8: sp[1] = sp[2]
     9: return
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
     0: call 0
     1: sp[2] = @1
     2: sp[3] = const u32 26
     3: @1 = u32 add @1, sp[3]
     4: sp[2] = indirect const u32 1
     5: sp[3] = u32 add sp[1], @2
     6: sp[4] = u32 add sp[2], @2
     7: keccakf1600(input: [sp[3]; 25], output: [sp[4]; 25])
     8: sp[1] = sp[2]
     9: return
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
    0: call 0
    1: sp[5] = const bool 1
    2: sp[7] = u32 add sp[4], @2
    3: sp[8] = u32 add sp[1], @2
    4: sp[9] = u32 add sp[2], @2
    5: sp[10] = u32 add sp[3], @2
    6: ecdsa_secp256k1(hashed_msg: [sp[7]; 32], public_key_x: [sp[8]; 32], public_key_y: [sp[9]; 32], signature: [sp[10]; 64], result: sp[6])
    7: sp[1] = sp[6]
    8: return
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
    0: call 0
    1: sp[5] = const bool 1
    2: sp[7] = u32 add sp[4], @2
    3: sp[8] = u32 add sp[1], @2
    4: sp[9] = u32 add sp[2], @2
    5: sp[10] = u32 add sp[3], @2
    6: ecdsa_secp256r1(hashed_msg: [sp[7]; 32], public_key_x: [sp[8]; 32], public_key_y: [sp[9]; 32], signature: [sp[10]; 64], result: sp[6])
    7: sp[1] = sp[6]
    8: return
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
     0: call 0
     1: sp[3] = const bool 1
     2: sp[4] = @1
     3: sp[5] = const u32 4
     4: @1 = u32 add @1, sp[5]
     5: sp[4] = indirect const u32 1
     6: sp[5] = u32 add sp[1], @2
     7: sp[6] = u32 add sp[2], @2
     8: sp[7] = u32 add sp[4], @2
     9: multi_scalar_mul(points: [sp[5]; 6], scalars: [sp[6]; 4], outputs: [sp[7]; 3])
    10: sp[1] = sp[4]
    11: return
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
     0: call 0
     1: sp[7] = const bool 1
     2: sp[8] = @1
     3: sp[9] = const u32 4
     4: @1 = u32 add @1, sp[9]
     5: sp[8] = indirect const u32 1
     6: sp[9] = u32 add sp[8], @2
     7: embedded_curve_add(input1_x: sp[1], input1_y: sp[2], input1_infinite: sp[3], input2_x: sp[4], input2_y: sp[5], input2_infinite: sp[6], result: [sp[9]; 3])
     8: sp[1] = sp[8]
     9: return
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
     0: call 0
     1: sp[2] = @1
     2: sp[3] = const u32 5
     3: @1 = u32 add @1, sp[3]
     4: sp[2] = indirect const u32 1
     5: sp[3] = u32 add sp[1], @2
     6: sp[4] = u32 add sp[2], @2
     7: poseidon2_permutation(message: [sp[3]; 4], output: [sp[4]; 4])
     8: sp[1] = sp[2]
     9: return
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
     0: call 0
     1: sp[3] = @1
     2: sp[4] = const u32 9
     3: @1 = u32 add @1, sp[4]
     4: sp[3] = indirect const u32 1
     5: sp[4] = u32 add sp[1], @2
     6: sp[5] = u32 add sp[2], @2
     7: sp[6] = u32 add sp[3], @2
     8: sha256_compression(input: [sp[4]; 16], hash_values: [sp[5]; 8], output: [sp[6]; 8])
     9: sp[1] = sp[3]
    10: return
    ");
}

// Tests AES128 encryption with plaintext, IV, and key inputs
#[test]
fn brillig_aes128_encrypt() {
    let src = "
    brillig(inline) fn foo f0 {
      b0(v0: [u8; 16], v1: [u8; 16], v2: [u8; 16]):
        v3 = call aes128_encrypt(v0, v1, v2) -> [u8; 32]
        return v3
    }
    ";

    let brillig = ssa_to_brillig_artifacts(src);
    let foo = &brillig.ssa_function_to_brillig[&Id::test_new(0)];
    assert_artifact_snapshot!(foo, @r"
    fn foo
     0: call 0
     1: sp[4] = @1
     2: sp[5] = const u32 33
     3: @1 = u32 add @1, sp[5]
     4: sp[4] = indirect const u32 1
     5: sp[5] = u32 add sp[1], @2
     6: sp[6] = u32 add sp[2], @2
     7: sp[7] = u32 add sp[3], @2
     8: sp[8] = u32 add sp[4], @2
     9: aes_128_encrypt(inputs: [sp[5]; 16], iv: [sp[6]; 16], key: [sp[7]; 16], outputs: [sp[8]; 32])
    10: sp[1] = sp[4]
    11: return
    ");
}
