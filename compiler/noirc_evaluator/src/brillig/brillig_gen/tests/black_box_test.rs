use acvm::assert_circuit_snapshot;

use crate::acir::tests::ssa_to_acir_program;

// Tests Blake2s hash function with message input and 32-byte output
#[test]
fn brillig_blake2s() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [u8; 10]):
        v2 = call f1(v0) -> [u8; 32]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: [u8; 10]):
        v1 = call blake2s(v0) -> [u8; 32]
        return v1
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w2, bits: 8
    BLACKBOX::RANGE input: w3, bits: 8
    BLACKBOX::RANGE input: w4, bits: 8
    BLACKBOX::RANGE input: w5, bits: 8
    BLACKBOX::RANGE input: w6, bits: 8
    BLACKBOX::RANGE input: w7, bits: 8
    BLACKBOX::RANGE input: w8, bits: 8
    BLACKBOX::RANGE input: w9, bits: 8
    BRILLIG CALL func: 0, inputs: [[w0, w1, w2, w3, w4, w5, w6, w7, w8, w9]], outputs: [[w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41]]
    BLACKBOX::RANGE input: w10, bits: 8
    BLACKBOX::RANGE input: w11, bits: 8
    BLACKBOX::RANGE input: w12, bits: 8
    BLACKBOX::RANGE input: w13, bits: 8
    BLACKBOX::RANGE input: w14, bits: 8
    BLACKBOX::RANGE input: w15, bits: 8
    BLACKBOX::RANGE input: w16, bits: 8
    BLACKBOX::RANGE input: w17, bits: 8
    BLACKBOX::RANGE input: w18, bits: 8
    BLACKBOX::RANGE input: w19, bits: 8
    BLACKBOX::RANGE input: w20, bits: 8
    BLACKBOX::RANGE input: w21, bits: 8
    BLACKBOX::RANGE input: w22, bits: 8
    BLACKBOX::RANGE input: w23, bits: 8
    BLACKBOX::RANGE input: w24, bits: 8
    BLACKBOX::RANGE input: w25, bits: 8
    BLACKBOX::RANGE input: w26, bits: 8
    BLACKBOX::RANGE input: w27, bits: 8
    BLACKBOX::RANGE input: w28, bits: 8
    BLACKBOX::RANGE input: w29, bits: 8
    BLACKBOX::RANGE input: w30, bits: 8
    BLACKBOX::RANGE input: w31, bits: 8
    BLACKBOX::RANGE input: w32, bits: 8
    BLACKBOX::RANGE input: w33, bits: 8
    BLACKBOX::RANGE input: w34, bits: 8
    BLACKBOX::RANGE input: w35, bits: 8
    BLACKBOX::RANGE input: w36, bits: 8
    BLACKBOX::RANGE input: w37, bits: 8
    BLACKBOX::RANGE input: w38, bits: 8
    BLACKBOX::RANGE input: w39, bits: 8
    BLACKBOX::RANGE input: w40, bits: 8
    BLACKBOX::RANGE input: w41, bits: 8

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32878
     2: @0 = const u32 110
     3: sp[2] = const u32 10
     4: sp[3] = const u32 0
     5: @68 = calldata copy [sp[3]; sp[2]]
     6: @68 = cast @68 to u8
     7: @69 = cast @69 to u8
     8: @70 = cast @70 to u8
     9: @71 = cast @71 to u8
    10: @72 = cast @72 to u8
    11: @73 = cast @73 to u8
    12: @74 = cast @74 to u8
    13: @75 = cast @75 to u8
    14: @76 = cast @76 to u8
    15: @77 = cast @77 to u8
    16: sp[1] = const u32 68
    17: sp[3] = const u32 10
    18: sp[2] = @1
    19: sp[4] = const u32 11
    20: @1 = u32 add @1, sp[4]
    21: sp[2] = indirect const u32 1
    22: sp[4] = u32 add sp[2], @2
    23: @3 = sp[1]
    24: @4 = sp[4]
    25: @5 = sp[3]
    26: call 40
    27: sp[1] = sp[2]
    28: call 51
    29: call 52
    30: sp[2] = u32 add sp[1], @2
    31: sp[3] = const u32 78
    32: sp[4] = const u32 32
    33: @3 = sp[2]
    34: @4 = sp[3]
    35: @5 = sp[4]
    36: call 40
    37: sp[2] = const u32 78
    38: sp[3] = const u32 32
    39: stop &[sp[2]; sp[3]]
    40: @7 = u32 add @3, @5
    41: @8 = @3
    42: @9 = @4
    43: @10 = u32 eq @8, @7
    44: jump if @10 to 50
    45: @6 = load @8
    46: store @6 at @9
    47: @8 = u32 add @8, @2
    48: @9 = u32 add @9, @2
    49: jump to 43
    50: return
    51: return
    52: call 63
    53: sp[2] = @1
    54: sp[3] = const u32 33
    55: @1 = u32 add @1, sp[3]
    56: sp[2] = indirect const u32 1
    57: sp[3] = u32 add sp[1], @2
    58: sp[4] = const u32 10
    59: sp[5] = u32 add sp[2], @2
    60: blake2s(message: &[sp[3]; sp[4]], output: [sp[5]; 32])
    61: sp[1] = sp[2]
    62: return
    63: @4 = const u32 30830
    64: @3 = u32 lt @0, @4
    65: jump if @3 to 68
    66: @1 = indirect const u64 15764276373176857197
    67: trap &[@1; @2]
    68: return
    ");
}

// Tests Blake3 hash function with message input and 32-byte output
#[test]
fn brillig_blake3() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [u8; 10]):
        v2 = call f1(v0) -> [u8; 32]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: [u8; 10]):
        v1 = call blake3(v0) -> [u8; 32]
        return v1
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w2, bits: 8
    BLACKBOX::RANGE input: w3, bits: 8
    BLACKBOX::RANGE input: w4, bits: 8
    BLACKBOX::RANGE input: w5, bits: 8
    BLACKBOX::RANGE input: w6, bits: 8
    BLACKBOX::RANGE input: w7, bits: 8
    BLACKBOX::RANGE input: w8, bits: 8
    BLACKBOX::RANGE input: w9, bits: 8
    BRILLIG CALL func: 0, inputs: [[w0, w1, w2, w3, w4, w5, w6, w7, w8, w9]], outputs: [[w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41]]
    BLACKBOX::RANGE input: w10, bits: 8
    BLACKBOX::RANGE input: w11, bits: 8
    BLACKBOX::RANGE input: w12, bits: 8
    BLACKBOX::RANGE input: w13, bits: 8
    BLACKBOX::RANGE input: w14, bits: 8
    BLACKBOX::RANGE input: w15, bits: 8
    BLACKBOX::RANGE input: w16, bits: 8
    BLACKBOX::RANGE input: w17, bits: 8
    BLACKBOX::RANGE input: w18, bits: 8
    BLACKBOX::RANGE input: w19, bits: 8
    BLACKBOX::RANGE input: w20, bits: 8
    BLACKBOX::RANGE input: w21, bits: 8
    BLACKBOX::RANGE input: w22, bits: 8
    BLACKBOX::RANGE input: w23, bits: 8
    BLACKBOX::RANGE input: w24, bits: 8
    BLACKBOX::RANGE input: w25, bits: 8
    BLACKBOX::RANGE input: w26, bits: 8
    BLACKBOX::RANGE input: w27, bits: 8
    BLACKBOX::RANGE input: w28, bits: 8
    BLACKBOX::RANGE input: w29, bits: 8
    BLACKBOX::RANGE input: w30, bits: 8
    BLACKBOX::RANGE input: w31, bits: 8
    BLACKBOX::RANGE input: w32, bits: 8
    BLACKBOX::RANGE input: w33, bits: 8
    BLACKBOX::RANGE input: w34, bits: 8
    BLACKBOX::RANGE input: w35, bits: 8
    BLACKBOX::RANGE input: w36, bits: 8
    BLACKBOX::RANGE input: w37, bits: 8
    BLACKBOX::RANGE input: w38, bits: 8
    BLACKBOX::RANGE input: w39, bits: 8
    BLACKBOX::RANGE input: w40, bits: 8
    BLACKBOX::RANGE input: w41, bits: 8

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32878
     2: @0 = const u32 110
     3: sp[2] = const u32 10
     4: sp[3] = const u32 0
     5: @68 = calldata copy [sp[3]; sp[2]]
     6: @68 = cast @68 to u8
     7: @69 = cast @69 to u8
     8: @70 = cast @70 to u8
     9: @71 = cast @71 to u8
    10: @72 = cast @72 to u8
    11: @73 = cast @73 to u8
    12: @74 = cast @74 to u8
    13: @75 = cast @75 to u8
    14: @76 = cast @76 to u8
    15: @77 = cast @77 to u8
    16: sp[1] = const u32 68
    17: sp[3] = const u32 10
    18: sp[2] = @1
    19: sp[4] = const u32 11
    20: @1 = u32 add @1, sp[4]
    21: sp[2] = indirect const u32 1
    22: sp[4] = u32 add sp[2], @2
    23: @3 = sp[1]
    24: @4 = sp[4]
    25: @5 = sp[3]
    26: call 40
    27: sp[1] = sp[2]
    28: call 51
    29: call 52
    30: sp[2] = u32 add sp[1], @2
    31: sp[3] = const u32 78
    32: sp[4] = const u32 32
    33: @3 = sp[2]
    34: @4 = sp[3]
    35: @5 = sp[4]
    36: call 40
    37: sp[2] = const u32 78
    38: sp[3] = const u32 32
    39: stop &[sp[2]; sp[3]]
    40: @7 = u32 add @3, @5
    41: @8 = @3
    42: @9 = @4
    43: @10 = u32 eq @8, @7
    44: jump if @10 to 50
    45: @6 = load @8
    46: store @6 at @9
    47: @8 = u32 add @8, @2
    48: @9 = u32 add @9, @2
    49: jump to 43
    50: return
    51: return
    52: call 63
    53: sp[2] = @1
    54: sp[3] = const u32 33
    55: @1 = u32 add @1, sp[3]
    56: sp[2] = indirect const u32 1
    57: sp[3] = u32 add sp[1], @2
    58: sp[4] = const u32 10
    59: sp[5] = u32 add sp[2], @2
    60: blake3(message: &[sp[3]; sp[4]], output: [sp[5]; 32])
    61: sp[1] = sp[2]
    62: return
    63: @4 = const u32 30830
    64: @3 = u32 lt @0, @4
    65: jump if @3 to 68
    66: @1 = indirect const u64 15764276373176857197
    67: trap &[@1; @2]
    68: return
    ");
}

// Tests Keccakf1600 permutation with 25-element input and output arrays
#[test]
fn brillig_keccakf1600() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [u64; 25]):
        v2 = call f1(v0) -> [u64; 25]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: [u64; 25]):
        v1 = call keccakf1600(v0) -> [u64; 25]
        return v1
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 64
    BLACKBOX::RANGE input: w1, bits: 64
    BLACKBOX::RANGE input: w2, bits: 64
    BLACKBOX::RANGE input: w3, bits: 64
    BLACKBOX::RANGE input: w4, bits: 64
    BLACKBOX::RANGE input: w5, bits: 64
    BLACKBOX::RANGE input: w6, bits: 64
    BLACKBOX::RANGE input: w7, bits: 64
    BLACKBOX::RANGE input: w8, bits: 64
    BLACKBOX::RANGE input: w9, bits: 64
    BLACKBOX::RANGE input: w10, bits: 64
    BLACKBOX::RANGE input: w11, bits: 64
    BLACKBOX::RANGE input: w12, bits: 64
    BLACKBOX::RANGE input: w13, bits: 64
    BLACKBOX::RANGE input: w14, bits: 64
    BLACKBOX::RANGE input: w15, bits: 64
    BLACKBOX::RANGE input: w16, bits: 64
    BLACKBOX::RANGE input: w17, bits: 64
    BLACKBOX::RANGE input: w18, bits: 64
    BLACKBOX::RANGE input: w19, bits: 64
    BLACKBOX::RANGE input: w20, bits: 64
    BLACKBOX::RANGE input: w21, bits: 64
    BLACKBOX::RANGE input: w22, bits: 64
    BLACKBOX::RANGE input: w23, bits: 64
    BLACKBOX::RANGE input: w24, bits: 64
    BRILLIG CALL func: 0, inputs: [[w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24]], outputs: [[w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47, w48, w49]]
    BLACKBOX::RANGE input: w25, bits: 64
    BLACKBOX::RANGE input: w26, bits: 64
    BLACKBOX::RANGE input: w27, bits: 64
    BLACKBOX::RANGE input: w28, bits: 64
    BLACKBOX::RANGE input: w29, bits: 64
    BLACKBOX::RANGE input: w30, bits: 64
    BLACKBOX::RANGE input: w31, bits: 64
    BLACKBOX::RANGE input: w32, bits: 64
    BLACKBOX::RANGE input: w33, bits: 64
    BLACKBOX::RANGE input: w34, bits: 64
    BLACKBOX::RANGE input: w35, bits: 64
    BLACKBOX::RANGE input: w36, bits: 64
    BLACKBOX::RANGE input: w37, bits: 64
    BLACKBOX::RANGE input: w38, bits: 64
    BLACKBOX::RANGE input: w39, bits: 64
    BLACKBOX::RANGE input: w40, bits: 64
    BLACKBOX::RANGE input: w41, bits: 64
    BLACKBOX::RANGE input: w42, bits: 64
    BLACKBOX::RANGE input: w43, bits: 64
    BLACKBOX::RANGE input: w44, bits: 64
    BLACKBOX::RANGE input: w45, bits: 64
    BLACKBOX::RANGE input: w46, bits: 64
    BLACKBOX::RANGE input: w47, bits: 64
    BLACKBOX::RANGE input: w48, bits: 64
    BLACKBOX::RANGE input: w49, bits: 64

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32886
     2: @0 = const u32 118
     3: sp[2] = const u32 25
     4: sp[3] = const u32 0
     5: @68 = calldata copy [sp[3]; sp[2]]
     6: @68 = cast @68 to u64
     7: @69 = cast @69 to u64
     8: @70 = cast @70 to u64
     9: @71 = cast @71 to u64
    10: @72 = cast @72 to u64
    11: @73 = cast @73 to u64
    12: @74 = cast @74 to u64
    13: @75 = cast @75 to u64
    14: @76 = cast @76 to u64
    15: @77 = cast @77 to u64
    16: @78 = cast @78 to u64
    17: @79 = cast @79 to u64
    18: @80 = cast @80 to u64
    19: @81 = cast @81 to u64
    20: @82 = cast @82 to u64
    21: @83 = cast @83 to u64
    22: @84 = cast @84 to u64
    23: @85 = cast @85 to u64
    24: @86 = cast @86 to u64
    25: @87 = cast @87 to u64
    26: @88 = cast @88 to u64
    27: @89 = cast @89 to u64
    28: @90 = cast @90 to u64
    29: @91 = cast @91 to u64
    30: @92 = cast @92 to u64
    31: sp[1] = const u32 68
    32: sp[3] = const u32 25
    33: sp[2] = @1
    34: sp[4] = const u32 26
    35: @1 = u32 add @1, sp[4]
    36: sp[2] = indirect const u32 1
    37: sp[4] = u32 add sp[2], @2
    38: @3 = sp[1]
    39: @4 = sp[4]
    40: @5 = sp[3]
    41: call 55
    42: sp[1] = sp[2]
    43: call 66
    44: call 67
    45: sp[2] = u32 add sp[1], @2
    46: sp[3] = const u32 93
    47: sp[4] = const u32 25
    48: @3 = sp[2]
    49: @4 = sp[3]
    50: @5 = sp[4]
    51: call 55
    52: sp[2] = const u32 93
    53: sp[3] = const u32 25
    54: stop &[sp[2]; sp[3]]
    55: @7 = u32 add @3, @5
    56: @8 = @3
    57: @9 = @4
    58: @10 = u32 eq @8, @7
    59: jump if @10 to 65
    60: @6 = load @8
    61: store @6 at @9
    62: @8 = u32 add @8, @2
    63: @9 = u32 add @9, @2
    64: jump to 58
    65: return
    66: return
    67: call 77
    68: sp[2] = @1
    69: sp[3] = const u32 26
    70: @1 = u32 add @1, sp[3]
    71: sp[2] = indirect const u32 1
    72: sp[3] = u32 add sp[1], @2
    73: sp[4] = u32 add sp[2], @2
    74: keccakf1600(input: [sp[3]; 25], output: [sp[4]; 25])
    75: sp[1] = sp[2]
    76: return
    77: @4 = const u32 30838
    78: @3 = u32 lt @0, @4
    79: jump if @3 to 82
    80: @1 = indirect const u64 15764276373176857197
    81: trap &[@1; @2]
    82: return
    ");
}

// Tests ECDSA signature verification on secp256k1 curve
#[test]
fn brillig_ecdsa_secp256k1() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
        v5 = call f1(v0, v1, v2, v3) -> u1
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
        v4 = call ecdsa_secp256k1(v0, v1, v2, v3, u1 1) -> u1
        return v4
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59, w60, w61, w62, w63, w64, w65, w66, w67, w68, w69, w70, w71, w72, w73, w74, w75, w76, w77, w78, w79, w80, w81, w82, w83, w84, w85, w86, w87, w88, w89, w90, w91, w92, w93, w94, w95, w96, w97, w98, w99, w100, w101, w102, w103, w104, w105, w106, w107, w108, w109, w110, w111, w112, w113, w114, w115, w116, w117, w118, w119, w120, w121, w122, w123, w124, w125, w126, w127, w128, w129, w130, w131, w132, w133, w134, w135, w136, w137, w138, w139, w140, w141, w142, w143, w144, w145, w146, w147, w148, w149, w150, w151, w152, w153, w154, w155, w156, w157, w158, w159]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w2, bits: 8
    BLACKBOX::RANGE input: w3, bits: 8
    BLACKBOX::RANGE input: w4, bits: 8
    BLACKBOX::RANGE input: w5, bits: 8
    BLACKBOX::RANGE input: w6, bits: 8
    BLACKBOX::RANGE input: w7, bits: 8
    BLACKBOX::RANGE input: w8, bits: 8
    BLACKBOX::RANGE input: w9, bits: 8
    BLACKBOX::RANGE input: w10, bits: 8
    BLACKBOX::RANGE input: w11, bits: 8
    BLACKBOX::RANGE input: w12, bits: 8
    BLACKBOX::RANGE input: w13, bits: 8
    BLACKBOX::RANGE input: w14, bits: 8
    BLACKBOX::RANGE input: w15, bits: 8
    BLACKBOX::RANGE input: w16, bits: 8
    BLACKBOX::RANGE input: w17, bits: 8
    BLACKBOX::RANGE input: w18, bits: 8
    BLACKBOX::RANGE input: w19, bits: 8
    BLACKBOX::RANGE input: w20, bits: 8
    BLACKBOX::RANGE input: w21, bits: 8
    BLACKBOX::RANGE input: w22, bits: 8
    BLACKBOX::RANGE input: w23, bits: 8
    BLACKBOX::RANGE input: w24, bits: 8
    BLACKBOX::RANGE input: w25, bits: 8
    BLACKBOX::RANGE input: w26, bits: 8
    BLACKBOX::RANGE input: w27, bits: 8
    BLACKBOX::RANGE input: w28, bits: 8
    BLACKBOX::RANGE input: w29, bits: 8
    BLACKBOX::RANGE input: w30, bits: 8
    BLACKBOX::RANGE input: w31, bits: 8
    BLACKBOX::RANGE input: w32, bits: 8
    BLACKBOX::RANGE input: w33, bits: 8
    BLACKBOX::RANGE input: w34, bits: 8
    BLACKBOX::RANGE input: w35, bits: 8
    BLACKBOX::RANGE input: w36, bits: 8
    BLACKBOX::RANGE input: w37, bits: 8
    BLACKBOX::RANGE input: w38, bits: 8
    BLACKBOX::RANGE input: w39, bits: 8
    BLACKBOX::RANGE input: w40, bits: 8
    BLACKBOX::RANGE input: w41, bits: 8
    BLACKBOX::RANGE input: w42, bits: 8
    BLACKBOX::RANGE input: w43, bits: 8
    BLACKBOX::RANGE input: w44, bits: 8
    BLACKBOX::RANGE input: w45, bits: 8
    BLACKBOX::RANGE input: w46, bits: 8
    BLACKBOX::RANGE input: w47, bits: 8
    BLACKBOX::RANGE input: w48, bits: 8
    BLACKBOX::RANGE input: w49, bits: 8
    BLACKBOX::RANGE input: w50, bits: 8
    BLACKBOX::RANGE input: w51, bits: 8
    BLACKBOX::RANGE input: w52, bits: 8
    BLACKBOX::RANGE input: w53, bits: 8
    BLACKBOX::RANGE input: w54, bits: 8
    BLACKBOX::RANGE input: w55, bits: 8
    BLACKBOX::RANGE input: w56, bits: 8
    BLACKBOX::RANGE input: w57, bits: 8
    BLACKBOX::RANGE input: w58, bits: 8
    BLACKBOX::RANGE input: w59, bits: 8
    BLACKBOX::RANGE input: w60, bits: 8
    BLACKBOX::RANGE input: w61, bits: 8
    BLACKBOX::RANGE input: w62, bits: 8
    BLACKBOX::RANGE input: w63, bits: 8
    BLACKBOX::RANGE input: w64, bits: 8
    BLACKBOX::RANGE input: w65, bits: 8
    BLACKBOX::RANGE input: w66, bits: 8
    BLACKBOX::RANGE input: w67, bits: 8
    BLACKBOX::RANGE input: w68, bits: 8
    BLACKBOX::RANGE input: w69, bits: 8
    BLACKBOX::RANGE input: w70, bits: 8
    BLACKBOX::RANGE input: w71, bits: 8
    BLACKBOX::RANGE input: w72, bits: 8
    BLACKBOX::RANGE input: w73, bits: 8
    BLACKBOX::RANGE input: w74, bits: 8
    BLACKBOX::RANGE input: w75, bits: 8
    BLACKBOX::RANGE input: w76, bits: 8
    BLACKBOX::RANGE input: w77, bits: 8
    BLACKBOX::RANGE input: w78, bits: 8
    BLACKBOX::RANGE input: w79, bits: 8
    BLACKBOX::RANGE input: w80, bits: 8
    BLACKBOX::RANGE input: w81, bits: 8
    BLACKBOX::RANGE input: w82, bits: 8
    BLACKBOX::RANGE input: w83, bits: 8
    BLACKBOX::RANGE input: w84, bits: 8
    BLACKBOX::RANGE input: w85, bits: 8
    BLACKBOX::RANGE input: w86, bits: 8
    BLACKBOX::RANGE input: w87, bits: 8
    BLACKBOX::RANGE input: w88, bits: 8
    BLACKBOX::RANGE input: w89, bits: 8
    BLACKBOX::RANGE input: w90, bits: 8
    BLACKBOX::RANGE input: w91, bits: 8
    BLACKBOX::RANGE input: w92, bits: 8
    BLACKBOX::RANGE input: w93, bits: 8
    BLACKBOX::RANGE input: w94, bits: 8
    BLACKBOX::RANGE input: w95, bits: 8
    BLACKBOX::RANGE input: w96, bits: 8
    BLACKBOX::RANGE input: w97, bits: 8
    BLACKBOX::RANGE input: w98, bits: 8
    BLACKBOX::RANGE input: w99, bits: 8
    BLACKBOX::RANGE input: w100, bits: 8
    BLACKBOX::RANGE input: w101, bits: 8
    BLACKBOX::RANGE input: w102, bits: 8
    BLACKBOX::RANGE input: w103, bits: 8
    BLACKBOX::RANGE input: w104, bits: 8
    BLACKBOX::RANGE input: w105, bits: 8
    BLACKBOX::RANGE input: w106, bits: 8
    BLACKBOX::RANGE input: w107, bits: 8
    BLACKBOX::RANGE input: w108, bits: 8
    BLACKBOX::RANGE input: w109, bits: 8
    BLACKBOX::RANGE input: w110, bits: 8
    BLACKBOX::RANGE input: w111, bits: 8
    BLACKBOX::RANGE input: w112, bits: 8
    BLACKBOX::RANGE input: w113, bits: 8
    BLACKBOX::RANGE input: w114, bits: 8
    BLACKBOX::RANGE input: w115, bits: 8
    BLACKBOX::RANGE input: w116, bits: 8
    BLACKBOX::RANGE input: w117, bits: 8
    BLACKBOX::RANGE input: w118, bits: 8
    BLACKBOX::RANGE input: w119, bits: 8
    BLACKBOX::RANGE input: w120, bits: 8
    BLACKBOX::RANGE input: w121, bits: 8
    BLACKBOX::RANGE input: w122, bits: 8
    BLACKBOX::RANGE input: w123, bits: 8
    BLACKBOX::RANGE input: w124, bits: 8
    BLACKBOX::RANGE input: w125, bits: 8
    BLACKBOX::RANGE input: w126, bits: 8
    BLACKBOX::RANGE input: w127, bits: 8
    BLACKBOX::RANGE input: w128, bits: 8
    BLACKBOX::RANGE input: w129, bits: 8
    BLACKBOX::RANGE input: w130, bits: 8
    BLACKBOX::RANGE input: w131, bits: 8
    BLACKBOX::RANGE input: w132, bits: 8
    BLACKBOX::RANGE input: w133, bits: 8
    BLACKBOX::RANGE input: w134, bits: 8
    BLACKBOX::RANGE input: w135, bits: 8
    BLACKBOX::RANGE input: w136, bits: 8
    BLACKBOX::RANGE input: w137, bits: 8
    BLACKBOX::RANGE input: w138, bits: 8
    BLACKBOX::RANGE input: w139, bits: 8
    BLACKBOX::RANGE input: w140, bits: 8
    BLACKBOX::RANGE input: w141, bits: 8
    BLACKBOX::RANGE input: w142, bits: 8
    BLACKBOX::RANGE input: w143, bits: 8
    BLACKBOX::RANGE input: w144, bits: 8
    BLACKBOX::RANGE input: w145, bits: 8
    BLACKBOX::RANGE input: w146, bits: 8
    BLACKBOX::RANGE input: w147, bits: 8
    BLACKBOX::RANGE input: w148, bits: 8
    BLACKBOX::RANGE input: w149, bits: 8
    BLACKBOX::RANGE input: w150, bits: 8
    BLACKBOX::RANGE input: w151, bits: 8
    BLACKBOX::RANGE input: w152, bits: 8
    BLACKBOX::RANGE input: w153, bits: 8
    BLACKBOX::RANGE input: w154, bits: 8
    BLACKBOX::RANGE input: w155, bits: 8
    BLACKBOX::RANGE input: w156, bits: 8
    BLACKBOX::RANGE input: w157, bits: 8
    BLACKBOX::RANGE input: w158, bits: 8
    BLACKBOX::RANGE input: w159, bits: 8
    BRILLIG CALL func: 0, inputs: [[w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31], [w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59, w60, w61, w62, w63], [w64, w65, w66, w67, w68, w69, w70, w71, w72, w73, w74, w75, w76, w77, w78, w79, w80, w81, w82, w83, w84, w85, w86, w87, w88, w89, w90, w91, w92, w93, w94, w95, w96, w97, w98, w99, w100, w101, w102, w103, w104, w105, w106, w107, w108, w109, w110, w111, w112, w113, w114, w115, w116, w117, w118, w119, w120, w121, w122, w123, w124, w125, w126, w127], [w128, w129, w130, w131, w132, w133, w134, w135, w136, w137, w138, w139, w140, w141, w142, w143, w144, w145, w146, w147, w148, w149, w150, w151, w152, w153, w154, w155, w156, w157, w158, w159]], outputs: [w160]
    BLACKBOX::RANGE input: w160, bits: 1

    unconstrained func 0: foo
      0: @2 = const u32 1
      1: @1 = const u32 32997
      2: @0 = const u32 229
      3: sp[5] = const u32 160
      4: sp[6] = const u32 0
      5: @68 = calldata copy [sp[6]; sp[5]]
      6: @68 = cast @68 to u8
      7: @69 = cast @69 to u8
      8: @70 = cast @70 to u8
      9: @71 = cast @71 to u8
     10: @72 = cast @72 to u8
     11: @73 = cast @73 to u8
     12: @74 = cast @74 to u8
     13: @75 = cast @75 to u8
     14: @76 = cast @76 to u8
     15: @77 = cast @77 to u8
     16: @78 = cast @78 to u8
     17: @79 = cast @79 to u8
     18: @80 = cast @80 to u8
     19: @81 = cast @81 to u8
     20: @82 = cast @82 to u8
     21: @83 = cast @83 to u8
     22: @84 = cast @84 to u8
     23: @85 = cast @85 to u8
     24: @86 = cast @86 to u8
     25: @87 = cast @87 to u8
     26: @88 = cast @88 to u8
     27: @89 = cast @89 to u8
     28: @90 = cast @90 to u8
     29: @91 = cast @91 to u8
     30: @92 = cast @92 to u8
     31: @93 = cast @93 to u8
     32: @94 = cast @94 to u8
     33: @95 = cast @95 to u8
     34: @96 = cast @96 to u8
     35: @97 = cast @97 to u8
     36: @98 = cast @98 to u8
     37: @99 = cast @99 to u8
     38: @100 = cast @100 to u8
     39: @101 = cast @101 to u8
     40: @102 = cast @102 to u8
     41: @103 = cast @103 to u8
     42: @104 = cast @104 to u8
     43: @105 = cast @105 to u8
     44: @106 = cast @106 to u8
     45: @107 = cast @107 to u8
     46: @108 = cast @108 to u8
     47: @109 = cast @109 to u8
     48: @110 = cast @110 to u8
     49: @111 = cast @111 to u8
     50: @112 = cast @112 to u8
     51: @113 = cast @113 to u8
     52: @114 = cast @114 to u8
     53: @115 = cast @115 to u8
     54: @116 = cast @116 to u8
     55: @117 = cast @117 to u8
     56: @118 = cast @118 to u8
     57: @119 = cast @119 to u8
     58: @120 = cast @120 to u8
     59: @121 = cast @121 to u8
     60: @122 = cast @122 to u8
     61: @123 = cast @123 to u8
     62: @124 = cast @124 to u8
     63: @125 = cast @125 to u8
     64: @126 = cast @126 to u8
     65: @127 = cast @127 to u8
     66: @128 = cast @128 to u8
     67: @129 = cast @129 to u8
     68: @130 = cast @130 to u8
     69: @131 = cast @131 to u8
     70: @132 = cast @132 to u8
     71: @133 = cast @133 to u8
     72: @134 = cast @134 to u8
     73: @135 = cast @135 to u8
     74: @136 = cast @136 to u8
     75: @137 = cast @137 to u8
     76: @138 = cast @138 to u8
     77: @139 = cast @139 to u8
     78: @140 = cast @140 to u8
     79: @141 = cast @141 to u8
     80: @142 = cast @142 to u8
     81: @143 = cast @143 to u8
     82: @144 = cast @144 to u8
     83: @145 = cast @145 to u8
     84: @146 = cast @146 to u8
     85: @147 = cast @147 to u8
     86: @148 = cast @148 to u8
     87: @149 = cast @149 to u8
     88: @150 = cast @150 to u8
     89: @151 = cast @151 to u8
     90: @152 = cast @152 to u8
     91: @153 = cast @153 to u8
     92: @154 = cast @154 to u8
     93: @155 = cast @155 to u8
     94: @156 = cast @156 to u8
     95: @157 = cast @157 to u8
     96: @158 = cast @158 to u8
     97: @159 = cast @159 to u8
     98: @160 = cast @160 to u8
     99: @161 = cast @161 to u8
    100: @162 = cast @162 to u8
    101: @163 = cast @163 to u8
    102: @164 = cast @164 to u8
    103: @165 = cast @165 to u8
    104: @166 = cast @166 to u8
    105: @167 = cast @167 to u8
    106: @168 = cast @168 to u8
    107: @169 = cast @169 to u8
    108: @170 = cast @170 to u8
    109: @171 = cast @171 to u8
    110: @172 = cast @172 to u8
    111: @173 = cast @173 to u8
    112: @174 = cast @174 to u8
    113: @175 = cast @175 to u8
    114: @176 = cast @176 to u8
    115: @177 = cast @177 to u8
    116: @178 = cast @178 to u8
    117: @179 = cast @179 to u8
    118: @180 = cast @180 to u8
    119: @181 = cast @181 to u8
    120: @182 = cast @182 to u8
    121: @183 = cast @183 to u8
    122: @184 = cast @184 to u8
    123: @185 = cast @185 to u8
    124: @186 = cast @186 to u8
    125: @187 = cast @187 to u8
    126: @188 = cast @188 to u8
    127: @189 = cast @189 to u8
    128: @190 = cast @190 to u8
    129: @191 = cast @191 to u8
    130: @192 = cast @192 to u8
    131: @193 = cast @193 to u8
    132: @194 = cast @194 to u8
    133: @195 = cast @195 to u8
    134: @196 = cast @196 to u8
    135: @197 = cast @197 to u8
    136: @198 = cast @198 to u8
    137: @199 = cast @199 to u8
    138: @200 = cast @200 to u8
    139: @201 = cast @201 to u8
    140: @202 = cast @202 to u8
    141: @203 = cast @203 to u8
    142: @204 = cast @204 to u8
    143: @205 = cast @205 to u8
    144: @206 = cast @206 to u8
    145: @207 = cast @207 to u8
    146: @208 = cast @208 to u8
    147: @209 = cast @209 to u8
    148: @210 = cast @210 to u8
    149: @211 = cast @211 to u8
    150: @212 = cast @212 to u8
    151: @213 = cast @213 to u8
    152: @214 = cast @214 to u8
    153: @215 = cast @215 to u8
    154: @216 = cast @216 to u8
    155: @217 = cast @217 to u8
    156: @218 = cast @218 to u8
    157: @219 = cast @219 to u8
    158: @220 = cast @220 to u8
    159: @221 = cast @221 to u8
    160: @222 = cast @222 to u8
    161: @223 = cast @223 to u8
    162: @224 = cast @224 to u8
    163: @225 = cast @225 to u8
    164: @226 = cast @226 to u8
    165: @227 = cast @227 to u8
    166: sp[1] = const u32 68
    167: sp[6] = const u32 32
    168: sp[5] = @1
    169: sp[7] = const u32 33
    170: @1 = u32 add @1, sp[7]
    171: sp[5] = indirect const u32 1
    172: sp[7] = u32 add sp[5], @2
    173: @3 = sp[1]
    174: @4 = sp[7]
    175: @5 = sp[6]
    176: call 220
    177: sp[1] = sp[5]
    178: sp[2] = const u32 100
    179: sp[6] = const u32 32
    180: sp[5] = @1
    181: sp[7] = const u32 33
    182: @1 = u32 add @1, sp[7]
    183: sp[5] = indirect const u32 1
    184: sp[7] = u32 add sp[5], @2
    185: @3 = sp[2]
    186: @4 = sp[7]
    187: @5 = sp[6]
    188: call 220
    189: sp[2] = sp[5]
    190: sp[3] = const u32 132
    191: sp[6] = const u32 64
    192: sp[5] = @1
    193: sp[7] = const u32 65
    194: @1 = u32 add @1, sp[7]
    195: sp[5] = indirect const u32 1
    196: sp[7] = u32 add sp[5], @2
    197: @3 = sp[3]
    198: @4 = sp[7]
    199: @5 = sp[6]
    200: call 220
    201: sp[3] = sp[5]
    202: sp[4] = const u32 196
    203: sp[6] = const u32 32
    204: sp[5] = @1
    205: sp[7] = const u32 33
    206: @1 = u32 add @1, sp[7]
    207: sp[5] = indirect const u32 1
    208: sp[7] = u32 add sp[5], @2
    209: @3 = sp[4]
    210: @4 = sp[7]
    211: @5 = sp[6]
    212: call 220
    213: sp[4] = sp[5]
    214: call 231
    215: call 232
    216: @228 = sp[1]
    217: sp[2] = const u32 228
    218: sp[3] = const u32 1
    219: stop &[sp[2]; sp[3]]
    220: @7 = u32 add @3, @5
    221: @8 = @3
    222: @9 = @4
    223: @10 = u32 eq @8, @7
    224: jump if @10 to 230
    225: @6 = load @8
    226: store @6 at @9
    227: @8 = u32 add @8, @2
    228: @9 = u32 add @9, @2
    229: jump to 223
    230: return
    231: return
    232: call 242
    233: sp[5] = const bool 1
    234: sp[7] = u32 add sp[4], @2
    235: sp[8] = const u32 32
    236: sp[9] = u32 add sp[1], @2
    237: sp[10] = u32 add sp[2], @2
    238: sp[11] = u32 add sp[3], @2
    239: ecdsa_secp256k1(hashed_msg: &[sp[7]; sp[8]], public_key_x: [sp[9]; 32], public_key_y: [sp[10]; 32], signature: [sp[11]; 64], result: sp[6])
    240: sp[1] = sp[6]
    241: return
    242: @4 = const u32 30949
    243: @3 = u32 lt @0, @4
    244: jump if @3 to 247
    245: @1 = indirect const u64 15764276373176857197
    246: trap &[@1; @2]
    247: return
    ");
}

// Tests ECDSA signature verification on secp256r1 curve
#[test]
fn brillig_ecdsa_secp256r1() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
        v5 = call f1(v0, v1, v2, v3) -> u1
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
        v4 = call ecdsa_secp256r1(v0, v1, v2, v3, u1 1) -> u1
        return v4
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59, w60, w61, w62, w63, w64, w65, w66, w67, w68, w69, w70, w71, w72, w73, w74, w75, w76, w77, w78, w79, w80, w81, w82, w83, w84, w85, w86, w87, w88, w89, w90, w91, w92, w93, w94, w95, w96, w97, w98, w99, w100, w101, w102, w103, w104, w105, w106, w107, w108, w109, w110, w111, w112, w113, w114, w115, w116, w117, w118, w119, w120, w121, w122, w123, w124, w125, w126, w127, w128, w129, w130, w131, w132, w133, w134, w135, w136, w137, w138, w139, w140, w141, w142, w143, w144, w145, w146, w147, w148, w149, w150, w151, w152, w153, w154, w155, w156, w157, w158, w159]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w2, bits: 8
    BLACKBOX::RANGE input: w3, bits: 8
    BLACKBOX::RANGE input: w4, bits: 8
    BLACKBOX::RANGE input: w5, bits: 8
    BLACKBOX::RANGE input: w6, bits: 8
    BLACKBOX::RANGE input: w7, bits: 8
    BLACKBOX::RANGE input: w8, bits: 8
    BLACKBOX::RANGE input: w9, bits: 8
    BLACKBOX::RANGE input: w10, bits: 8
    BLACKBOX::RANGE input: w11, bits: 8
    BLACKBOX::RANGE input: w12, bits: 8
    BLACKBOX::RANGE input: w13, bits: 8
    BLACKBOX::RANGE input: w14, bits: 8
    BLACKBOX::RANGE input: w15, bits: 8
    BLACKBOX::RANGE input: w16, bits: 8
    BLACKBOX::RANGE input: w17, bits: 8
    BLACKBOX::RANGE input: w18, bits: 8
    BLACKBOX::RANGE input: w19, bits: 8
    BLACKBOX::RANGE input: w20, bits: 8
    BLACKBOX::RANGE input: w21, bits: 8
    BLACKBOX::RANGE input: w22, bits: 8
    BLACKBOX::RANGE input: w23, bits: 8
    BLACKBOX::RANGE input: w24, bits: 8
    BLACKBOX::RANGE input: w25, bits: 8
    BLACKBOX::RANGE input: w26, bits: 8
    BLACKBOX::RANGE input: w27, bits: 8
    BLACKBOX::RANGE input: w28, bits: 8
    BLACKBOX::RANGE input: w29, bits: 8
    BLACKBOX::RANGE input: w30, bits: 8
    BLACKBOX::RANGE input: w31, bits: 8
    BLACKBOX::RANGE input: w32, bits: 8
    BLACKBOX::RANGE input: w33, bits: 8
    BLACKBOX::RANGE input: w34, bits: 8
    BLACKBOX::RANGE input: w35, bits: 8
    BLACKBOX::RANGE input: w36, bits: 8
    BLACKBOX::RANGE input: w37, bits: 8
    BLACKBOX::RANGE input: w38, bits: 8
    BLACKBOX::RANGE input: w39, bits: 8
    BLACKBOX::RANGE input: w40, bits: 8
    BLACKBOX::RANGE input: w41, bits: 8
    BLACKBOX::RANGE input: w42, bits: 8
    BLACKBOX::RANGE input: w43, bits: 8
    BLACKBOX::RANGE input: w44, bits: 8
    BLACKBOX::RANGE input: w45, bits: 8
    BLACKBOX::RANGE input: w46, bits: 8
    BLACKBOX::RANGE input: w47, bits: 8
    BLACKBOX::RANGE input: w48, bits: 8
    BLACKBOX::RANGE input: w49, bits: 8
    BLACKBOX::RANGE input: w50, bits: 8
    BLACKBOX::RANGE input: w51, bits: 8
    BLACKBOX::RANGE input: w52, bits: 8
    BLACKBOX::RANGE input: w53, bits: 8
    BLACKBOX::RANGE input: w54, bits: 8
    BLACKBOX::RANGE input: w55, bits: 8
    BLACKBOX::RANGE input: w56, bits: 8
    BLACKBOX::RANGE input: w57, bits: 8
    BLACKBOX::RANGE input: w58, bits: 8
    BLACKBOX::RANGE input: w59, bits: 8
    BLACKBOX::RANGE input: w60, bits: 8
    BLACKBOX::RANGE input: w61, bits: 8
    BLACKBOX::RANGE input: w62, bits: 8
    BLACKBOX::RANGE input: w63, bits: 8
    BLACKBOX::RANGE input: w64, bits: 8
    BLACKBOX::RANGE input: w65, bits: 8
    BLACKBOX::RANGE input: w66, bits: 8
    BLACKBOX::RANGE input: w67, bits: 8
    BLACKBOX::RANGE input: w68, bits: 8
    BLACKBOX::RANGE input: w69, bits: 8
    BLACKBOX::RANGE input: w70, bits: 8
    BLACKBOX::RANGE input: w71, bits: 8
    BLACKBOX::RANGE input: w72, bits: 8
    BLACKBOX::RANGE input: w73, bits: 8
    BLACKBOX::RANGE input: w74, bits: 8
    BLACKBOX::RANGE input: w75, bits: 8
    BLACKBOX::RANGE input: w76, bits: 8
    BLACKBOX::RANGE input: w77, bits: 8
    BLACKBOX::RANGE input: w78, bits: 8
    BLACKBOX::RANGE input: w79, bits: 8
    BLACKBOX::RANGE input: w80, bits: 8
    BLACKBOX::RANGE input: w81, bits: 8
    BLACKBOX::RANGE input: w82, bits: 8
    BLACKBOX::RANGE input: w83, bits: 8
    BLACKBOX::RANGE input: w84, bits: 8
    BLACKBOX::RANGE input: w85, bits: 8
    BLACKBOX::RANGE input: w86, bits: 8
    BLACKBOX::RANGE input: w87, bits: 8
    BLACKBOX::RANGE input: w88, bits: 8
    BLACKBOX::RANGE input: w89, bits: 8
    BLACKBOX::RANGE input: w90, bits: 8
    BLACKBOX::RANGE input: w91, bits: 8
    BLACKBOX::RANGE input: w92, bits: 8
    BLACKBOX::RANGE input: w93, bits: 8
    BLACKBOX::RANGE input: w94, bits: 8
    BLACKBOX::RANGE input: w95, bits: 8
    BLACKBOX::RANGE input: w96, bits: 8
    BLACKBOX::RANGE input: w97, bits: 8
    BLACKBOX::RANGE input: w98, bits: 8
    BLACKBOX::RANGE input: w99, bits: 8
    BLACKBOX::RANGE input: w100, bits: 8
    BLACKBOX::RANGE input: w101, bits: 8
    BLACKBOX::RANGE input: w102, bits: 8
    BLACKBOX::RANGE input: w103, bits: 8
    BLACKBOX::RANGE input: w104, bits: 8
    BLACKBOX::RANGE input: w105, bits: 8
    BLACKBOX::RANGE input: w106, bits: 8
    BLACKBOX::RANGE input: w107, bits: 8
    BLACKBOX::RANGE input: w108, bits: 8
    BLACKBOX::RANGE input: w109, bits: 8
    BLACKBOX::RANGE input: w110, bits: 8
    BLACKBOX::RANGE input: w111, bits: 8
    BLACKBOX::RANGE input: w112, bits: 8
    BLACKBOX::RANGE input: w113, bits: 8
    BLACKBOX::RANGE input: w114, bits: 8
    BLACKBOX::RANGE input: w115, bits: 8
    BLACKBOX::RANGE input: w116, bits: 8
    BLACKBOX::RANGE input: w117, bits: 8
    BLACKBOX::RANGE input: w118, bits: 8
    BLACKBOX::RANGE input: w119, bits: 8
    BLACKBOX::RANGE input: w120, bits: 8
    BLACKBOX::RANGE input: w121, bits: 8
    BLACKBOX::RANGE input: w122, bits: 8
    BLACKBOX::RANGE input: w123, bits: 8
    BLACKBOX::RANGE input: w124, bits: 8
    BLACKBOX::RANGE input: w125, bits: 8
    BLACKBOX::RANGE input: w126, bits: 8
    BLACKBOX::RANGE input: w127, bits: 8
    BLACKBOX::RANGE input: w128, bits: 8
    BLACKBOX::RANGE input: w129, bits: 8
    BLACKBOX::RANGE input: w130, bits: 8
    BLACKBOX::RANGE input: w131, bits: 8
    BLACKBOX::RANGE input: w132, bits: 8
    BLACKBOX::RANGE input: w133, bits: 8
    BLACKBOX::RANGE input: w134, bits: 8
    BLACKBOX::RANGE input: w135, bits: 8
    BLACKBOX::RANGE input: w136, bits: 8
    BLACKBOX::RANGE input: w137, bits: 8
    BLACKBOX::RANGE input: w138, bits: 8
    BLACKBOX::RANGE input: w139, bits: 8
    BLACKBOX::RANGE input: w140, bits: 8
    BLACKBOX::RANGE input: w141, bits: 8
    BLACKBOX::RANGE input: w142, bits: 8
    BLACKBOX::RANGE input: w143, bits: 8
    BLACKBOX::RANGE input: w144, bits: 8
    BLACKBOX::RANGE input: w145, bits: 8
    BLACKBOX::RANGE input: w146, bits: 8
    BLACKBOX::RANGE input: w147, bits: 8
    BLACKBOX::RANGE input: w148, bits: 8
    BLACKBOX::RANGE input: w149, bits: 8
    BLACKBOX::RANGE input: w150, bits: 8
    BLACKBOX::RANGE input: w151, bits: 8
    BLACKBOX::RANGE input: w152, bits: 8
    BLACKBOX::RANGE input: w153, bits: 8
    BLACKBOX::RANGE input: w154, bits: 8
    BLACKBOX::RANGE input: w155, bits: 8
    BLACKBOX::RANGE input: w156, bits: 8
    BLACKBOX::RANGE input: w157, bits: 8
    BLACKBOX::RANGE input: w158, bits: 8
    BLACKBOX::RANGE input: w159, bits: 8
    BRILLIG CALL func: 0, inputs: [[w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31], [w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59, w60, w61, w62, w63], [w64, w65, w66, w67, w68, w69, w70, w71, w72, w73, w74, w75, w76, w77, w78, w79, w80, w81, w82, w83, w84, w85, w86, w87, w88, w89, w90, w91, w92, w93, w94, w95, w96, w97, w98, w99, w100, w101, w102, w103, w104, w105, w106, w107, w108, w109, w110, w111, w112, w113, w114, w115, w116, w117, w118, w119, w120, w121, w122, w123, w124, w125, w126, w127], [w128, w129, w130, w131, w132, w133, w134, w135, w136, w137, w138, w139, w140, w141, w142, w143, w144, w145, w146, w147, w148, w149, w150, w151, w152, w153, w154, w155, w156, w157, w158, w159]], outputs: [w160]
    BLACKBOX::RANGE input: w160, bits: 1

    unconstrained func 0: foo
      0: @2 = const u32 1
      1: @1 = const u32 32997
      2: @0 = const u32 229
      3: sp[5] = const u32 160
      4: sp[6] = const u32 0
      5: @68 = calldata copy [sp[6]; sp[5]]
      6: @68 = cast @68 to u8
      7: @69 = cast @69 to u8
      8: @70 = cast @70 to u8
      9: @71 = cast @71 to u8
     10: @72 = cast @72 to u8
     11: @73 = cast @73 to u8
     12: @74 = cast @74 to u8
     13: @75 = cast @75 to u8
     14: @76 = cast @76 to u8
     15: @77 = cast @77 to u8
     16: @78 = cast @78 to u8
     17: @79 = cast @79 to u8
     18: @80 = cast @80 to u8
     19: @81 = cast @81 to u8
     20: @82 = cast @82 to u8
     21: @83 = cast @83 to u8
     22: @84 = cast @84 to u8
     23: @85 = cast @85 to u8
     24: @86 = cast @86 to u8
     25: @87 = cast @87 to u8
     26: @88 = cast @88 to u8
     27: @89 = cast @89 to u8
     28: @90 = cast @90 to u8
     29: @91 = cast @91 to u8
     30: @92 = cast @92 to u8
     31: @93 = cast @93 to u8
     32: @94 = cast @94 to u8
     33: @95 = cast @95 to u8
     34: @96 = cast @96 to u8
     35: @97 = cast @97 to u8
     36: @98 = cast @98 to u8
     37: @99 = cast @99 to u8
     38: @100 = cast @100 to u8
     39: @101 = cast @101 to u8
     40: @102 = cast @102 to u8
     41: @103 = cast @103 to u8
     42: @104 = cast @104 to u8
     43: @105 = cast @105 to u8
     44: @106 = cast @106 to u8
     45: @107 = cast @107 to u8
     46: @108 = cast @108 to u8
     47: @109 = cast @109 to u8
     48: @110 = cast @110 to u8
     49: @111 = cast @111 to u8
     50: @112 = cast @112 to u8
     51: @113 = cast @113 to u8
     52: @114 = cast @114 to u8
     53: @115 = cast @115 to u8
     54: @116 = cast @116 to u8
     55: @117 = cast @117 to u8
     56: @118 = cast @118 to u8
     57: @119 = cast @119 to u8
     58: @120 = cast @120 to u8
     59: @121 = cast @121 to u8
     60: @122 = cast @122 to u8
     61: @123 = cast @123 to u8
     62: @124 = cast @124 to u8
     63: @125 = cast @125 to u8
     64: @126 = cast @126 to u8
     65: @127 = cast @127 to u8
     66: @128 = cast @128 to u8
     67: @129 = cast @129 to u8
     68: @130 = cast @130 to u8
     69: @131 = cast @131 to u8
     70: @132 = cast @132 to u8
     71: @133 = cast @133 to u8
     72: @134 = cast @134 to u8
     73: @135 = cast @135 to u8
     74: @136 = cast @136 to u8
     75: @137 = cast @137 to u8
     76: @138 = cast @138 to u8
     77: @139 = cast @139 to u8
     78: @140 = cast @140 to u8
     79: @141 = cast @141 to u8
     80: @142 = cast @142 to u8
     81: @143 = cast @143 to u8
     82: @144 = cast @144 to u8
     83: @145 = cast @145 to u8
     84: @146 = cast @146 to u8
     85: @147 = cast @147 to u8
     86: @148 = cast @148 to u8
     87: @149 = cast @149 to u8
     88: @150 = cast @150 to u8
     89: @151 = cast @151 to u8
     90: @152 = cast @152 to u8
     91: @153 = cast @153 to u8
     92: @154 = cast @154 to u8
     93: @155 = cast @155 to u8
     94: @156 = cast @156 to u8
     95: @157 = cast @157 to u8
     96: @158 = cast @158 to u8
     97: @159 = cast @159 to u8
     98: @160 = cast @160 to u8
     99: @161 = cast @161 to u8
    100: @162 = cast @162 to u8
    101: @163 = cast @163 to u8
    102: @164 = cast @164 to u8
    103: @165 = cast @165 to u8
    104: @166 = cast @166 to u8
    105: @167 = cast @167 to u8
    106: @168 = cast @168 to u8
    107: @169 = cast @169 to u8
    108: @170 = cast @170 to u8
    109: @171 = cast @171 to u8
    110: @172 = cast @172 to u8
    111: @173 = cast @173 to u8
    112: @174 = cast @174 to u8
    113: @175 = cast @175 to u8
    114: @176 = cast @176 to u8
    115: @177 = cast @177 to u8
    116: @178 = cast @178 to u8
    117: @179 = cast @179 to u8
    118: @180 = cast @180 to u8
    119: @181 = cast @181 to u8
    120: @182 = cast @182 to u8
    121: @183 = cast @183 to u8
    122: @184 = cast @184 to u8
    123: @185 = cast @185 to u8
    124: @186 = cast @186 to u8
    125: @187 = cast @187 to u8
    126: @188 = cast @188 to u8
    127: @189 = cast @189 to u8
    128: @190 = cast @190 to u8
    129: @191 = cast @191 to u8
    130: @192 = cast @192 to u8
    131: @193 = cast @193 to u8
    132: @194 = cast @194 to u8
    133: @195 = cast @195 to u8
    134: @196 = cast @196 to u8
    135: @197 = cast @197 to u8
    136: @198 = cast @198 to u8
    137: @199 = cast @199 to u8
    138: @200 = cast @200 to u8
    139: @201 = cast @201 to u8
    140: @202 = cast @202 to u8
    141: @203 = cast @203 to u8
    142: @204 = cast @204 to u8
    143: @205 = cast @205 to u8
    144: @206 = cast @206 to u8
    145: @207 = cast @207 to u8
    146: @208 = cast @208 to u8
    147: @209 = cast @209 to u8
    148: @210 = cast @210 to u8
    149: @211 = cast @211 to u8
    150: @212 = cast @212 to u8
    151: @213 = cast @213 to u8
    152: @214 = cast @214 to u8
    153: @215 = cast @215 to u8
    154: @216 = cast @216 to u8
    155: @217 = cast @217 to u8
    156: @218 = cast @218 to u8
    157: @219 = cast @219 to u8
    158: @220 = cast @220 to u8
    159: @221 = cast @221 to u8
    160: @222 = cast @222 to u8
    161: @223 = cast @223 to u8
    162: @224 = cast @224 to u8
    163: @225 = cast @225 to u8
    164: @226 = cast @226 to u8
    165: @227 = cast @227 to u8
    166: sp[1] = const u32 68
    167: sp[6] = const u32 32
    168: sp[5] = @1
    169: sp[7] = const u32 33
    170: @1 = u32 add @1, sp[7]
    171: sp[5] = indirect const u32 1
    172: sp[7] = u32 add sp[5], @2
    173: @3 = sp[1]
    174: @4 = sp[7]
    175: @5 = sp[6]
    176: call 220
    177: sp[1] = sp[5]
    178: sp[2] = const u32 100
    179: sp[6] = const u32 32
    180: sp[5] = @1
    181: sp[7] = const u32 33
    182: @1 = u32 add @1, sp[7]
    183: sp[5] = indirect const u32 1
    184: sp[7] = u32 add sp[5], @2
    185: @3 = sp[2]
    186: @4 = sp[7]
    187: @5 = sp[6]
    188: call 220
    189: sp[2] = sp[5]
    190: sp[3] = const u32 132
    191: sp[6] = const u32 64
    192: sp[5] = @1
    193: sp[7] = const u32 65
    194: @1 = u32 add @1, sp[7]
    195: sp[5] = indirect const u32 1
    196: sp[7] = u32 add sp[5], @2
    197: @3 = sp[3]
    198: @4 = sp[7]
    199: @5 = sp[6]
    200: call 220
    201: sp[3] = sp[5]
    202: sp[4] = const u32 196
    203: sp[6] = const u32 32
    204: sp[5] = @1
    205: sp[7] = const u32 33
    206: @1 = u32 add @1, sp[7]
    207: sp[5] = indirect const u32 1
    208: sp[7] = u32 add sp[5], @2
    209: @3 = sp[4]
    210: @4 = sp[7]
    211: @5 = sp[6]
    212: call 220
    213: sp[4] = sp[5]
    214: call 231
    215: call 232
    216: @228 = sp[1]
    217: sp[2] = const u32 228
    218: sp[3] = const u32 1
    219: stop &[sp[2]; sp[3]]
    220: @7 = u32 add @3, @5
    221: @8 = @3
    222: @9 = @4
    223: @10 = u32 eq @8, @7
    224: jump if @10 to 230
    225: @6 = load @8
    226: store @6 at @9
    227: @8 = u32 add @8, @2
    228: @9 = u32 add @9, @2
    229: jump to 223
    230: return
    231: return
    232: call 242
    233: sp[5] = const bool 1
    234: sp[7] = u32 add sp[4], @2
    235: sp[8] = const u32 32
    236: sp[9] = u32 add sp[1], @2
    237: sp[10] = u32 add sp[2], @2
    238: sp[11] = u32 add sp[3], @2
    239: ecdsa_secp256r1(hashed_msg: &[sp[7]; sp[8]], public_key_x: [sp[9]; 32], public_key_y: [sp[10]; 32], signature: [sp[11]; 64], result: sp[6])
    240: sp[1] = sp[6]
    241: return
    242: @4 = const u32 30949
    243: @3 = u32 lt @0, @4
    244: jump if @3 to 247
    245: @1 = indirect const u64 15764276373176857197
    246: trap &[@1; @2]
    247: return
    ");
}

// Tests multi-scalar multiplication on embedded curve
#[test]
fn brillig_multi_scalar_mul() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [(Field, Field, u1); 2], v1: [(Field, Field); 2]):
        v3 = call f1(v0, v1) -> [(Field, Field, u1); 1]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: [(Field, Field, u1); 2], v1: [(Field, Field); 2]):
        v2 = call multi_scalar_mul(v0, v1, u1 1) -> [(Field, Field, u1); 1]
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w2, bits: 1
    BLACKBOX::RANGE input: w5, bits: 1
    BRILLIG CALL func: 0, inputs: [[w0, w1, w2, w3, w4, w5], [w6, w7, w8, w9]], outputs: [[w10, w11, w12]]
    BLACKBOX::RANGE input: w12, bits: 1

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32849
     2: @0 = const u32 81
     3: sp[3] = const u32 10
     4: sp[4] = const u32 0
     5: @68 = calldata copy [sp[4]; sp[3]]
     6: @70 = cast @70 to bool
     7: @73 = cast @73 to bool
     8: sp[1] = const u32 68
     9: sp[4] = const u32 6
    10: sp[3] = @1
    11: sp[5] = const u32 7
    12: @1 = u32 add @1, sp[5]
    13: sp[3] = indirect const u32 1
    14: sp[5] = u32 add sp[3], @2
    15: @3 = sp[1]
    16: @4 = sp[5]
    17: @5 = sp[4]
    18: call 44
    19: sp[1] = sp[3]
    20: sp[2] = const u32 74
    21: sp[4] = const u32 4
    22: sp[3] = @1
    23: sp[5] = const u32 5
    24: @1 = u32 add @1, sp[5]
    25: sp[3] = indirect const u32 1
    26: sp[5] = u32 add sp[3], @2
    27: @3 = sp[2]
    28: @4 = sp[5]
    29: @5 = sp[4]
    30: call 44
    31: sp[2] = sp[3]
    32: call 55
    33: call 56
    34: sp[2] = u32 add sp[1], @2
    35: sp[3] = const u32 78
    36: sp[4] = const u32 3
    37: @3 = sp[2]
    38: @4 = sp[3]
    39: @5 = sp[4]
    40: call 44
    41: sp[2] = const u32 78
    42: sp[3] = const u32 3
    43: stop &[sp[2]; sp[3]]
    44: @7 = u32 add @3, @5
    45: @8 = @3
    46: @9 = @4
    47: @10 = u32 eq @8, @7
    48: jump if @10 to 54
    49: @6 = load @8
    50: store @6 at @9
    51: @8 = u32 add @8, @2
    52: @9 = u32 add @9, @2
    53: jump to 47
    54: return
    55: return
    56: call 70
    57: sp[3] = const bool 1
    58: sp[4] = @1
    59: sp[5] = const u32 4
    60: @1 = u32 add @1, sp[5]
    61: sp[4] = indirect const u32 1
    62: sp[5] = u32 add sp[1], @2
    63: sp[6] = const u32 6
    64: sp[7] = u32 add sp[2], @2
    65: sp[8] = const u32 4
    66: sp[9] = u32 add sp[4], @2
    67: multi_scalar_mul(points: &[sp[5]; sp[6]], scalars: &[sp[7]; sp[8]], outputs: [sp[9]; 3])
    68: sp[1] = sp[4]
    69: return
    70: @4 = const u32 30801
    71: @3 = u32 lt @0, @4
    72: jump if @3 to 75
    73: @1 = indirect const u64 15764276373176857197
    74: trap &[@1; @2]
    75: return
    ");
}

// Tests embedded curve point addition
#[test]
fn brillig_embedded_curve_add() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field, v2: u1, v3: Field, v4: Field, v5: u1):
        v7 = call f1(v0, v1, v2, v3, v4, v5) -> [(Field, Field, u1); 1]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field, v2: u1, v3: Field, v4: Field, v5: u1):
        v6 = call embedded_curve_add(v0, v1, v2, v3, v4, v5, u1 1) -> [(Field, Field, u1); 1]
        return v6
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w2, bits: 1
    BLACKBOX::RANGE input: w5, bits: 1
    BRILLIG CALL func: 0, inputs: [w0, w1, w2, w3, w4, w5], outputs: [[w6, w7, w8]]
    BLACKBOX::RANGE input: w8, bits: 1

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32845
     2: @0 = const u32 77
     3: sp[7] = const u32 6
     4: sp[8] = const u32 0
     5: @68 = calldata copy [sp[8]; sp[7]]
     6: @70 = cast @70 to bool
     7: @73 = cast @73 to bool
     8: sp[1] = @68
     9: sp[2] = @69
    10: sp[3] = @70
    11: sp[4] = @71
    12: sp[5] = @72
    13: sp[6] = @73
    14: call 26
    15: call 27
    16: sp[2] = u32 add sp[1], @2
    17: sp[3] = const u32 74
    18: sp[4] = const u32 3
    19: @3 = sp[2]
    20: @4 = sp[3]
    21: @5 = sp[4]
    22: call 37
    23: sp[2] = const u32 74
    24: sp[3] = const u32 3
    25: stop &[sp[2]; sp[3]]
    26: return
    27: call 48
    28: sp[7] = const bool 1
    29: sp[8] = @1
    30: sp[9] = const u32 4
    31: @1 = u32 add @1, sp[9]
    32: sp[8] = indirect const u32 1
    33: sp[9] = u32 add sp[8], @2
    34: embedded_curve_add(input1_x: sp[1], input1_y: sp[2], input1_infinite: sp[3], input2_x: sp[4], input2_y: sp[5], input2_infinite: sp[6], result: [sp[9]; 3])
    35: sp[1] = sp[8]
    36: return
    37: @7 = u32 add @3, @5
    38: @8 = @3
    39: @9 = @4
    40: @10 = u32 eq @8, @7
    41: jump if @10 to 47
    42: @6 = load @8
    43: store @6 at @9
    44: @8 = u32 add @8, @2
    45: @9 = u32 add @9, @2
    46: jump to 40
    47: return
    48: @4 = const u32 30797
    49: @3 = u32 lt @0, @4
    50: jump if @3 to 53
    51: @1 = indirect const u64 15764276373176857197
    52: trap &[@1; @2]
    53: return
    ");
}

// Tests Poseidon2 permutation hash function
#[test]
fn brillig_poseidon2_permutation() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 4]):
        v2 = call f1(v0) -> [Field; 4]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: [Field; 4]):
        v1 = call poseidon2_permutation(v0) -> [Field; 4]
        return v1
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [[w0, w1, w2, w3]], outputs: [[w4, w5, w6, w7]]

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32844
     2: @0 = const u32 76
     3: sp[2] = const u32 4
     4: sp[3] = const u32 0
     5: @68 = calldata copy [sp[3]; sp[2]]
     6: sp[1] = const u32 68
     7: sp[3] = const u32 4
     8: sp[2] = @1
     9: sp[4] = const u32 5
    10: @1 = u32 add @1, sp[4]
    11: sp[2] = indirect const u32 1
    12: sp[4] = u32 add sp[2], @2
    13: @3 = sp[1]
    14: @4 = sp[4]
    15: @5 = sp[3]
    16: call 30
    17: sp[1] = sp[2]
    18: call 41
    19: call 42
    20: sp[2] = u32 add sp[1], @2
    21: sp[3] = const u32 72
    22: sp[4] = const u32 4
    23: @3 = sp[2]
    24: @4 = sp[3]
    25: @5 = sp[4]
    26: call 30
    27: sp[2] = const u32 72
    28: sp[3] = const u32 4
    29: stop &[sp[2]; sp[3]]
    30: @7 = u32 add @3, @5
    31: @8 = @3
    32: @9 = @4
    33: @10 = u32 eq @8, @7
    34: jump if @10 to 40
    35: @6 = load @8
    36: store @6 at @9
    37: @8 = u32 add @8, @2
    38: @9 = u32 add @9, @2
    39: jump to 33
    40: return
    41: return
    42: call 53
    43: sp[2] = @1
    44: sp[3] = const u32 5
    45: @1 = u32 add @1, sp[3]
    46: sp[2] = indirect const u32 1
    47: sp[3] = u32 add sp[1], @2
    48: sp[4] = const u32 4
    49: sp[5] = u32 add sp[2], @2
    50: poseidon2_permutation(message: &[sp[3]; sp[4]], output: [sp[5]; 4])
    51: sp[1] = sp[2]
    52: return
    53: @4 = const u32 30796
    54: @3 = u32 lt @0, @4
    55: jump if @3 to 58
    56: @1 = indirect const u64 15764276373176857197
    57: trap &[@1; @2]
    58: return
    ");
}

// Tests SHA256 compression function with input and hash values
#[test]
fn brillig_sha256_compression() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [u32; 16], v1: [u32; 8]):
        v3 = call f1(v0, v1) -> [u32; 8]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: [u32; 16], v1: [u32; 8]):
        v2 = call sha256_compression(v0, v1) -> [u32; 8]
        return v2
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 32
    BLACKBOX::RANGE input: w2, bits: 32
    BLACKBOX::RANGE input: w3, bits: 32
    BLACKBOX::RANGE input: w4, bits: 32
    BLACKBOX::RANGE input: w5, bits: 32
    BLACKBOX::RANGE input: w6, bits: 32
    BLACKBOX::RANGE input: w7, bits: 32
    BLACKBOX::RANGE input: w8, bits: 32
    BLACKBOX::RANGE input: w9, bits: 32
    BLACKBOX::RANGE input: w10, bits: 32
    BLACKBOX::RANGE input: w11, bits: 32
    BLACKBOX::RANGE input: w12, bits: 32
    BLACKBOX::RANGE input: w13, bits: 32
    BLACKBOX::RANGE input: w14, bits: 32
    BLACKBOX::RANGE input: w15, bits: 32
    BLACKBOX::RANGE input: w16, bits: 32
    BLACKBOX::RANGE input: w17, bits: 32
    BLACKBOX::RANGE input: w18, bits: 32
    BLACKBOX::RANGE input: w19, bits: 32
    BLACKBOX::RANGE input: w20, bits: 32
    BLACKBOX::RANGE input: w21, bits: 32
    BLACKBOX::RANGE input: w22, bits: 32
    BLACKBOX::RANGE input: w23, bits: 32
    BRILLIG CALL func: 0, inputs: [[w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15], [w16, w17, w18, w19, w20, w21, w22, w23]], outputs: [[w24, w25, w26, w27, w28, w29, w30, w31]]
    BLACKBOX::RANGE input: w24, bits: 32
    BLACKBOX::RANGE input: w25, bits: 32
    BLACKBOX::RANGE input: w26, bits: 32
    BLACKBOX::RANGE input: w27, bits: 32
    BLACKBOX::RANGE input: w28, bits: 32
    BLACKBOX::RANGE input: w29, bits: 32
    BLACKBOX::RANGE input: w30, bits: 32
    BLACKBOX::RANGE input: w31, bits: 32

    unconstrained func 0: foo
     0: @2 = const u32 1
     1: @1 = const u32 32868
     2: @0 = const u32 100
     3: sp[3] = const u32 24
     4: sp[4] = const u32 0
     5: @68 = calldata copy [sp[4]; sp[3]]
     6: @68 = cast @68 to u32
     7: @69 = cast @69 to u32
     8: @70 = cast @70 to u32
     9: @71 = cast @71 to u32
    10: @72 = cast @72 to u32
    11: @73 = cast @73 to u32
    12: @74 = cast @74 to u32
    13: @75 = cast @75 to u32
    14: @76 = cast @76 to u32
    15: @77 = cast @77 to u32
    16: @78 = cast @78 to u32
    17: @79 = cast @79 to u32
    18: @80 = cast @80 to u32
    19: @81 = cast @81 to u32
    20: @82 = cast @82 to u32
    21: @83 = cast @83 to u32
    22: @84 = cast @84 to u32
    23: @85 = cast @85 to u32
    24: @86 = cast @86 to u32
    25: @87 = cast @87 to u32
    26: @88 = cast @88 to u32
    27: @89 = cast @89 to u32
    28: @90 = cast @90 to u32
    29: @91 = cast @91 to u32
    30: sp[1] = const u32 68
    31: sp[4] = const u32 16
    32: sp[3] = @1
    33: sp[5] = const u32 17
    34: @1 = u32 add @1, sp[5]
    35: sp[3] = indirect const u32 1
    36: sp[5] = u32 add sp[3], @2
    37: @3 = sp[1]
    38: @4 = sp[5]
    39: @5 = sp[4]
    40: call 66
    41: sp[1] = sp[3]
    42: sp[2] = const u32 84
    43: sp[4] = const u32 8
    44: sp[3] = @1
    45: sp[5] = const u32 9
    46: @1 = u32 add @1, sp[5]
    47: sp[3] = indirect const u32 1
    48: sp[5] = u32 add sp[3], @2
    49: @3 = sp[2]
    50: @4 = sp[5]
    51: @5 = sp[4]
    52: call 66
    53: sp[2] = sp[3]
    54: call 77
    55: call 78
    56: sp[2] = u32 add sp[1], @2
    57: sp[3] = const u32 92
    58: sp[4] = const u32 8
    59: @3 = sp[2]
    60: @4 = sp[3]
    61: @5 = sp[4]
    62: call 66
    63: sp[2] = const u32 92
    64: sp[3] = const u32 8
    65: stop &[sp[2]; sp[3]]
    66: @7 = u32 add @3, @5
    67: @8 = @3
    68: @9 = @4
    69: @10 = u32 eq @8, @7
    70: jump if @10 to 76
    71: @6 = load @8
    72: store @6 at @9
    73: @8 = u32 add @8, @2
    74: @9 = u32 add @9, @2
    75: jump to 69
    76: return
    77: return
    78: call 89
    79: sp[3] = @1
    80: sp[4] = const u32 9
    81: @1 = u32 add @1, sp[4]
    82: sp[3] = indirect const u32 1
    83: sp[4] = u32 add sp[1], @2
    84: sp[5] = u32 add sp[2], @2
    85: sp[6] = u32 add sp[3], @2
    86: sha256_compression(input: [sp[4]; 16], hash_values: [sp[5]; 8], output: [sp[6]; 8])
    87: sp[1] = sp[3]
    88: return
    89: @4 = const u32 30820
    90: @3 = u32 lt @0, @4
    91: jump if @3 to 94
    92: @1 = indirect const u64 15764276373176857197
    93: trap &[@1; @2]
    94: return
    ");
}

// Tests AES128 encryption with plaintext, IV, and key inputs
#[test]
fn brillig_aes128_encrypt() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [u8; 16], v1: [u8; 16], v2: [u8; 16]):
        v4 = call f1(v0, v1, v2) -> [u8; 32]
        return
    }

    brillig(inline) fn foo f1 {
      b0(v0: [u8; 16], v1: [u8; 16], v2: [u8; 16]):
        v3 = call aes128_encrypt(v0, v1, v2) -> [u8; 32]
        return v3
    }
    ";

    let program = ssa_to_acir_program(src);
    assert_eq!(program.unconstrained_functions.len(), 1);
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47]
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 8
    BLACKBOX::RANGE input: w1, bits: 8
    BLACKBOX::RANGE input: w2, bits: 8
    BLACKBOX::RANGE input: w3, bits: 8
    BLACKBOX::RANGE input: w4, bits: 8
    BLACKBOX::RANGE input: w5, bits: 8
    BLACKBOX::RANGE input: w6, bits: 8
    BLACKBOX::RANGE input: w7, bits: 8
    BLACKBOX::RANGE input: w8, bits: 8
    BLACKBOX::RANGE input: w9, bits: 8
    BLACKBOX::RANGE input: w10, bits: 8
    BLACKBOX::RANGE input: w11, bits: 8
    BLACKBOX::RANGE input: w12, bits: 8
    BLACKBOX::RANGE input: w13, bits: 8
    BLACKBOX::RANGE input: w14, bits: 8
    BLACKBOX::RANGE input: w15, bits: 8
    BLACKBOX::RANGE input: w16, bits: 8
    BLACKBOX::RANGE input: w17, bits: 8
    BLACKBOX::RANGE input: w18, bits: 8
    BLACKBOX::RANGE input: w19, bits: 8
    BLACKBOX::RANGE input: w20, bits: 8
    BLACKBOX::RANGE input: w21, bits: 8
    BLACKBOX::RANGE input: w22, bits: 8
    BLACKBOX::RANGE input: w23, bits: 8
    BLACKBOX::RANGE input: w24, bits: 8
    BLACKBOX::RANGE input: w25, bits: 8
    BLACKBOX::RANGE input: w26, bits: 8
    BLACKBOX::RANGE input: w27, bits: 8
    BLACKBOX::RANGE input: w28, bits: 8
    BLACKBOX::RANGE input: w29, bits: 8
    BLACKBOX::RANGE input: w30, bits: 8
    BLACKBOX::RANGE input: w31, bits: 8
    BLACKBOX::RANGE input: w32, bits: 8
    BLACKBOX::RANGE input: w33, bits: 8
    BLACKBOX::RANGE input: w34, bits: 8
    BLACKBOX::RANGE input: w35, bits: 8
    BLACKBOX::RANGE input: w36, bits: 8
    BLACKBOX::RANGE input: w37, bits: 8
    BLACKBOX::RANGE input: w38, bits: 8
    BLACKBOX::RANGE input: w39, bits: 8
    BLACKBOX::RANGE input: w40, bits: 8
    BLACKBOX::RANGE input: w41, bits: 8
    BLACKBOX::RANGE input: w42, bits: 8
    BLACKBOX::RANGE input: w43, bits: 8
    BLACKBOX::RANGE input: w44, bits: 8
    BLACKBOX::RANGE input: w45, bits: 8
    BLACKBOX::RANGE input: w46, bits: 8
    BLACKBOX::RANGE input: w47, bits: 8
    BRILLIG CALL func: 0, inputs: [[w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15], [w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31], [w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47]], outputs: [[w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59, w60, w61, w62, w63, w64, w65, w66, w67, w68, w69, w70, w71, w72, w73, w74, w75, w76, w77, w78, w79]]
    BLACKBOX::RANGE input: w48, bits: 8
    BLACKBOX::RANGE input: w49, bits: 8
    BLACKBOX::RANGE input: w50, bits: 8
    BLACKBOX::RANGE input: w51, bits: 8
    BLACKBOX::RANGE input: w52, bits: 8
    BLACKBOX::RANGE input: w53, bits: 8
    BLACKBOX::RANGE input: w54, bits: 8
    BLACKBOX::RANGE input: w55, bits: 8
    BLACKBOX::RANGE input: w56, bits: 8
    BLACKBOX::RANGE input: w57, bits: 8
    BLACKBOX::RANGE input: w58, bits: 8
    BLACKBOX::RANGE input: w59, bits: 8
    BLACKBOX::RANGE input: w60, bits: 8
    BLACKBOX::RANGE input: w61, bits: 8
    BLACKBOX::RANGE input: w62, bits: 8
    BLACKBOX::RANGE input: w63, bits: 8
    BLACKBOX::RANGE input: w64, bits: 8
    BLACKBOX::RANGE input: w65, bits: 8
    BLACKBOX::RANGE input: w66, bits: 8
    BLACKBOX::RANGE input: w67, bits: 8
    BLACKBOX::RANGE input: w68, bits: 8
    BLACKBOX::RANGE input: w69, bits: 8
    BLACKBOX::RANGE input: w70, bits: 8
    BLACKBOX::RANGE input: w71, bits: 8
    BLACKBOX::RANGE input: w72, bits: 8
    BLACKBOX::RANGE input: w73, bits: 8
    BLACKBOX::RANGE input: w74, bits: 8
    BLACKBOX::RANGE input: w75, bits: 8
    BLACKBOX::RANGE input: w76, bits: 8
    BLACKBOX::RANGE input: w77, bits: 8
    BLACKBOX::RANGE input: w78, bits: 8
    BLACKBOX::RANGE input: w79, bits: 8

    unconstrained func 0: foo
      0: @2 = const u32 1
      1: @1 = const u32 32916
      2: @0 = const u32 148
      3: sp[4] = const u32 48
      4: sp[5] = const u32 0
      5: @68 = calldata copy [sp[5]; sp[4]]
      6: @68 = cast @68 to u8
      7: @69 = cast @69 to u8
      8: @70 = cast @70 to u8
      9: @71 = cast @71 to u8
     10: @72 = cast @72 to u8
     11: @73 = cast @73 to u8
     12: @74 = cast @74 to u8
     13: @75 = cast @75 to u8
     14: @76 = cast @76 to u8
     15: @77 = cast @77 to u8
     16: @78 = cast @78 to u8
     17: @79 = cast @79 to u8
     18: @80 = cast @80 to u8
     19: @81 = cast @81 to u8
     20: @82 = cast @82 to u8
     21: @83 = cast @83 to u8
     22: @84 = cast @84 to u8
     23: @85 = cast @85 to u8
     24: @86 = cast @86 to u8
     25: @87 = cast @87 to u8
     26: @88 = cast @88 to u8
     27: @89 = cast @89 to u8
     28: @90 = cast @90 to u8
     29: @91 = cast @91 to u8
     30: @92 = cast @92 to u8
     31: @93 = cast @93 to u8
     32: @94 = cast @94 to u8
     33: @95 = cast @95 to u8
     34: @96 = cast @96 to u8
     35: @97 = cast @97 to u8
     36: @98 = cast @98 to u8
     37: @99 = cast @99 to u8
     38: @100 = cast @100 to u8
     39: @101 = cast @101 to u8
     40: @102 = cast @102 to u8
     41: @103 = cast @103 to u8
     42: @104 = cast @104 to u8
     43: @105 = cast @105 to u8
     44: @106 = cast @106 to u8
     45: @107 = cast @107 to u8
     46: @108 = cast @108 to u8
     47: @109 = cast @109 to u8
     48: @110 = cast @110 to u8
     49: @111 = cast @111 to u8
     50: @112 = cast @112 to u8
     51: @113 = cast @113 to u8
     52: @114 = cast @114 to u8
     53: @115 = cast @115 to u8
     54: sp[1] = const u32 68
     55: sp[5] = const u32 16
     56: sp[4] = @1
     57: sp[6] = const u32 17
     58: @1 = u32 add @1, sp[6]
     59: sp[4] = indirect const u32 1
     60: sp[6] = u32 add sp[4], @2
     61: @3 = sp[1]
     62: @4 = sp[6]
     63: @5 = sp[5]
     64: call 102
     65: sp[1] = sp[4]
     66: sp[2] = const u32 84
     67: sp[5] = const u32 16
     68: sp[4] = @1
     69: sp[6] = const u32 17
     70: @1 = u32 add @1, sp[6]
     71: sp[4] = indirect const u32 1
     72: sp[6] = u32 add sp[4], @2
     73: @3 = sp[2]
     74: @4 = sp[6]
     75: @5 = sp[5]
     76: call 102
     77: sp[2] = sp[4]
     78: sp[3] = const u32 100
     79: sp[5] = const u32 16
     80: sp[4] = @1
     81: sp[6] = const u32 17
     82: @1 = u32 add @1, sp[6]
     83: sp[4] = indirect const u32 1
     84: sp[6] = u32 add sp[4], @2
     85: @3 = sp[3]
     86: @4 = sp[6]
     87: @5 = sp[5]
     88: call 102
     89: sp[3] = sp[4]
     90: call 113
     91: call 114
     92: sp[2] = u32 add sp[1], @2
     93: sp[3] = const u32 116
     94: sp[4] = const u32 32
     95: @3 = sp[2]
     96: @4 = sp[3]
     97: @5 = sp[4]
     98: call 102
     99: sp[2] = const u32 116
    100: sp[3] = const u32 32
    101: stop &[sp[2]; sp[3]]
    102: @7 = u32 add @3, @5
    103: @8 = @3
    104: @9 = @4
    105: @10 = u32 eq @8, @7
    106: jump if @10 to 112
    107: @6 = load @8
    108: store @6 at @9
    109: @8 = u32 add @8, @2
    110: @9 = u32 add @9, @2
    111: jump to 105
    112: return
    113: return
    114: call 128
    115: sp[4] = @1
    116: sp[5] = const u32 33
    117: @1 = u32 add @1, sp[5]
    118: sp[4] = indirect const u32 1
    119: sp[5] = u32 add sp[1], @2
    120: sp[6] = const u32 16
    121: sp[7] = u32 add sp[2], @2
    122: sp[8] = u32 add sp[3], @2
    123: sp[9] = u32 add sp[4], @2
    124: sp[10] = const u32 32
    125: aes_128_encrypt(inputs: &[sp[5]; sp[6]], iv: [sp[7]; 16], key: [sp[8]; 16], outputs: &[sp[9]; sp[10]])
    126: sp[1] = sp[4]
    127: return
    128: @4 = const u32 30868
    129: @3 = u32 lt @0, @4
    130: jump if @3 to 133
    131: @1 = indirect const u64 15764276373176857197
    132: trap &[@1; @2]
    133: return
    ");
}
