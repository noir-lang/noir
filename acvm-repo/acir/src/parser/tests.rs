use crate::circuit::Circuit;

/// Trims leading whitespace from each line of the input string
#[cfg(test)]
pub(crate) fn trim_leading_whitespace_from_lines(src: &str) -> String {
    let mut lines = src.trim_end().lines();
    let mut first_line = lines.next().unwrap();
    while first_line.is_empty() {
        first_line = lines.next().unwrap();
    }
    let first_line_original_length = first_line.len();
    let mut result = first_line.trim_start().to_string();
    let first_line_trimmed_length = result.len();

    // Try to see how many spaces we chopped off the first line
    let difference = first_line_original_length - first_line_trimmed_length;
    for line in lines {
        result.push('\n');
        // Try to remove just `difference` spaces to preserve indents
        if line.len() - line.trim_start().len() >= difference {
            result.push_str(&line.chars().skip(difference).collect::<String>());
        } else {
            result.push_str(line.trim_start());
        }
    }
    result
}

fn assert_circuit_roundtrip(src: &str) {
    let circuit = Circuit::from_str(src).unwrap();
    let circuit = circuit.to_string();
    let circuit = trim_leading_whitespace_from_lines(&circuit);
    let src = trim_leading_whitespace_from_lines(src);
    if circuit != src {
        println!("Expected:\n~~~\n{src}\n~~~\nGot:\n~~~\n{circuit}\n~~~");
        similar_asserts::assert_eq!(circuit, src);
    }
}

#[test]
fn current_witness() {
    let src = "
    current witness index : _1
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn private_parameters() {
    let src = "
    current witness index : _4
    private parameters indices : [_0, _1, _2, _3, _4]
    public parameters indices : []
    return value indices : []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn public_parameters() {
    let src = "
    current witness index : _9
    private parameters indices : [_0, _1, _2, _3, _4]
    public parameters indices : [_5, _6, _7, _8, _9]
    return value indices : []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn return_values() {
    let src = "
    current witness index : _12
    private parameters indices : [_0, _1, _2, _3, _4]
    public parameters indices : [_5, _6, _7, _8, _9]
    return value indices : [_10, _11, _12]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn assert_zero_opcodes() {
    let src = "
    current witness index : _9
    private parameters indices : [_0, _1, _2, _3, _4]
    public parameters indices : [_5, _6, _7, _8, _9]
    return value indices : []
    EXPR [ (1, _0) (-1, _5) 0 ]
    EXPR [ (1, _1) (-1, _6) 0 ]
    EXPR [ (1, _2) (-1, _7) 0 ]
    EXPR [ (1, _3) (-1, _8) 0 ]
    EXPR [ (1, _4) (-1, _9) 0 ]    
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn assert_zero_with_mul_terms() {
    let src = "
    current witness index : _6
    private parameters indices : [_0, _1, _2]
    public parameters indices : []
    return value indices : []
    EXPR [ (1, _0, _1) (-1, _3) 0 ]
    EXPR [ (1, _3, _3) (-1, _4) 0 ]
    EXPR [ (1, _4, _4) (-1, _5) 0 ]
    EXPR [ (1, _5, _5) (-1, _6) 0 ]
    EXPR [ (-1, _2) (1, _6) 0 ]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn range_and_xor() {
    let src = "
    current witness index : _2
    private parameters indices : [_0]
    public parameters indices : [_1]
    return value indices : []
    BLACKBOX::RANGE [(_0, 32)] []
    BLACKBOX::RANGE [(_1, 32)] []
    BLACKBOX::XOR [(_0, 32), (_1, 32)] [_2]
    EXPR [ (1, _2) -15 ]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn aes128_encrypt() {
    // This ACIR represents an accurately constrained aes128 encryption in ACIR
    let src = "
    current witness index : _75
    private parameters indices : [_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38, _39, _40, _41, _42, _43]
    public parameters indices : [_44, _45, _46, _47, _48, _49, _50, _51, _52, _53, _54, _55, _56, _57, _58, _59]
    return value indices : []
    BLACKBOX::RANGE [(_0, 8)] []
    BLACKBOX::RANGE [(_1, 8)] []
    BLACKBOX::RANGE [(_2, 8)] []
    BLACKBOX::RANGE [(_3, 8)] []
    BLACKBOX::RANGE [(_4, 8)] []
    BLACKBOX::RANGE [(_5, 8)] []
    BLACKBOX::RANGE [(_6, 8)] []
    BLACKBOX::RANGE [(_7, 8)] []
    BLACKBOX::RANGE [(_8, 8)] []
    BLACKBOX::RANGE [(_9, 8)] []
    BLACKBOX::RANGE [(_10, 8)] []
    BLACKBOX::RANGE [(_11, 8)] []
    BLACKBOX::RANGE [(_12, 8)] []
    BLACKBOX::RANGE [(_13, 8)] []
    BLACKBOX::RANGE [(_14, 8)] []
    BLACKBOX::RANGE [(_15, 8)] []
    BLACKBOX::RANGE [(_16, 8)] []
    BLACKBOX::RANGE [(_17, 8)] []
    BLACKBOX::RANGE [(_18, 8)] []
    BLACKBOX::RANGE [(_19, 8)] []
    BLACKBOX::RANGE [(_20, 8)] []
    BLACKBOX::RANGE [(_21, 8)] []
    BLACKBOX::RANGE [(_22, 8)] []
    BLACKBOX::RANGE [(_23, 8)] []
    BLACKBOX::RANGE [(_24, 8)] []
    BLACKBOX::RANGE [(_25, 8)] []
    BLACKBOX::RANGE [(_26, 8)] []
    BLACKBOX::RANGE [(_27, 8)] []
    BLACKBOX::RANGE [(_28, 8)] []
    BLACKBOX::RANGE [(_29, 8)] []
    BLACKBOX::RANGE [(_30, 8)] []
    BLACKBOX::RANGE [(_31, 8)] []
    BLACKBOX::RANGE [(_32, 8)] []
    BLACKBOX::RANGE [(_33, 8)] []
    BLACKBOX::RANGE [(_34, 8)] []
    BLACKBOX::RANGE [(_35, 8)] []
    BLACKBOX::RANGE [(_36, 8)] []
    BLACKBOX::RANGE [(_37, 8)] []
    BLACKBOX::RANGE [(_38, 8)] []
    BLACKBOX::RANGE [(_39, 8)] []
    BLACKBOX::RANGE [(_40, 8)] []
    BLACKBOX::RANGE [(_41, 8)] []
    BLACKBOX::RANGE [(_42, 8)] []
    BLACKBOX::RANGE [(_43, 8)] []
    BLACKBOX::RANGE [(_44, 8)] []
    BLACKBOX::RANGE [(_45, 8)] []
    BLACKBOX::RANGE [(_46, 8)] []
    BLACKBOX::RANGE [(_47, 8)] []
    BLACKBOX::RANGE [(_48, 8)] []
    BLACKBOX::RANGE [(_49, 8)] []
    BLACKBOX::RANGE [(_50, 8)] []
    BLACKBOX::RANGE [(_51, 8)] []
    BLACKBOX::RANGE [(_52, 8)] []
    BLACKBOX::RANGE [(_53, 8)] []
    BLACKBOX::RANGE [(_54, 8)] []
    BLACKBOX::RANGE [(_55, 8)] []
    BLACKBOX::RANGE [(_56, 8)] []
    BLACKBOX::RANGE [(_57, 8)] []
    BLACKBOX::RANGE [(_58, 8)] []
    BLACKBOX::RANGE [(_59, 8)] []
    BLACKBOX::AES128_ENCRYPT [(_12, 8), (_13, 8), (_14, 8), (_15, 8), (_16, 8), (_17, 8), (_18, 8), (_19, 8), (_20, 8), (_21, 8), (_22, 8), (_23, 8), (_24, 8), (_25, 8), (_26, 8), (_27, 8), (_28, 8), (_29, 8), (_30, 8), (_31, 8), (_32, 8), (_33, 8), (_34, 8), (_35, 8), (_36, 8), (_37, 8), (_38, 8), (_39, 8), (_40, 8), (_41, 8), (_42, 8), (_43, 8)] [_60, _61, _62, _63, _64, _65, _66, _67, _68, _69, _70, _71, _72, _73, _74, _75]
    EXPR [ (-1, _44) (1, _60) 0 ]
    EXPR [ (-1, _45) (1, _61) 0 ]
    EXPR [ (-1, _46) (1, _62) 0 ]
    EXPR [ (-1, _47) (1, _63) 0 ]
    EXPR [ (-1, _48) (1, _64) 0 ]
    EXPR [ (-1, _49) (1, _65) 0 ]
    EXPR [ (-1, _50) (1, _66) 0 ]
    EXPR [ (-1, _51) (1, _67) 0 ]
    EXPR [ (-1, _52) (1, _68) 0 ]
    EXPR [ (-1, _53) (1, _69) 0 ]
    EXPR [ (-1, _54) (1, _70) 0 ]
    EXPR [ (-1, _55) (1, _71) 0 ]
    EXPR [ (-1, _56) (1, _72) 0 ]
    EXPR [ (-1, _57) (1, _73) 0 ]
    EXPR [ (-1, _58) (1, _74) 0 ]
    EXPR [ (-1, _59) (1, _75) 0 ]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn blake2s() {
    let src = "
    current witness index : _68
    private parameters indices : [_0, _1, _2, _3, _4]
    public parameters indices : [_5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36]
    return value indices : []
    BLACKBOX::BLAKE2S [(_0, 8), (_1, 8), (_2, 8), (_3, 8), (_4, 8)] [_37, _38, _39, _40, _41, _42, _43, _44, _45, _46, _47, _48, _49, _50, _51, _52, _53, _54, _55, _56, _57, _58, _59, _60, _61, _62, _63, _64, _65, _66, _67, _68]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn blake3() {
    let src = "
    current witness index : _37
    private parameters indices : [_0, _1, _2, _3, _4]
    public parameters indices : []
    return value indices : []
    BLACKBOX::BLAKE3 [(_0, 8), (_1, 8), (_2, 8), (_3, 8), (_4, 8)] [_5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn ecdsa_secp256k1() {
    let input_witnesses: Vec<String> = (0..160).map(|i| format!("(_{i}, 8)")).collect();
    let inputs_str = input_witnesses.join(", ");

    let src = format!(
        "
    current witness index : _160
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    BLACKBOX::ECDSA_SECP256K1 [{inputs_str}] [_160]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic]
fn ecdsa_secp256k1_missing_inputs() {
    let input_witnesses: Vec<String> = (0..100).map(|i| format!("(_{i}, 8)")).collect();
    let inputs_str = input_witnesses.join(", ");

    let src = format!(
        "
    current witness index : _100
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    BLACKBOX::ECDSA_SECP256K1 [{inputs_str}] [_100]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
fn ecdsa_secp256r1() {
    let input_witnesses: Vec<String> = (0..160).map(|i| format!("(_{i}, 8)")).collect();
    let inputs_str = input_witnesses.join(", ");

    let src = format!(
        "
    current witness index : _160
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    BLACKBOX::ECDSA_SECP256R1 [{inputs_str}] [_160]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic]
fn ecdsa_secp256r1_missing_inputs() {
    let input_witnesses: Vec<String> = (0..100).map(|i| format!("(_{i}, 8)")).collect();
    let inputs_str = input_witnesses.join(", ");

    let src = format!(
        "
    current witness index : _100
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    BLACKBOX::ECDSA_SECP256R1 [{inputs_str}] [_100]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
fn keccakf1600() {
    let input_witnesses: Vec<String> = (0..25).map(|i| format!("(_{i}, 64)")).collect();
    let inputs_str = input_witnesses.join(", ");

    let output_witnesses: Vec<String> = (25..50).map(|i| format!("_{i}")).collect();
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
    current witness index : _50
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    BLACKBOX::KECCAKF1600 [{inputs_str}] [{outputs_str}]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic]
fn keccakf1600_missing_inputs() {
    let input_witnesses: Vec<String> = (0..24).map(|i| format!("(_{i}, 64)")).collect();
    let inputs_str = input_witnesses.join(", ");

    let output_witnesses: Vec<String> = (24..49).map(|i| format!("_{i}")).collect();
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
        current witness index : _49
        private parameters indices : []
        public parameters indices : []
        return value indices : []
        BLACKBOX::KECCAKF1600 [{inputs_str}] [{outputs_str}]
        "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
fn embedded_curve_add() {
    let src = "
    current witness index : _9
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    BLACKBOX::EMBEDDED_CURVE_ADD [(_0, 255), (_1, 255), (_2, 1), (_3, 255), (_4, 255), (_5, 1)] [_6, _7, _8]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
#[should_panic]
fn embedded_curve_add_wrong_output_count() {
    let src = "
        current witness index : _8
        private parameters indices : []
        public parameters indices : []
        return value indices : []
        BLACKBOX::EMBEDDED_CURVE_ADD [(_0, 255), (_1, 255), (_2, 1), (_3, 255), (_4, 255), (_5, 1)] [_6, _7]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn poseidon2_permutation() {
    let src = "
    current witness index : _5
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    BLACKBOX::POSEIDON2_PERMUTATION [(_0, 255), (_1, 255), (_2, 255)] [_3, _4, _5]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn sha256_compression() {
    let input_witnesses: Vec<String> = (0..24).map(|i| format!("(_{i}, 8)")).collect();
    let inputs_str = input_witnesses.join(", ");

    let output_witnesses: Vec<String> = (24..32).map(|i| format!("_{i}")).collect();
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
    current witness index : _31
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    BLACKBOX::SHA256_COMPRESSION [{inputs_str}] [{outputs_str}]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic]
fn sha256_compression_missing_outputs() {
    let input_witnesses: Vec<String> = (0..24).map(|i| format!("(_{i}, 8)")).collect();
    let inputs_str = input_witnesses.join(", ");

    let output_witnesses: Vec<String> = (24..31).map(|i| format!("_{i}")).collect(); // should be 8 total
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
        current witness index : _31
        private parameters indices : []
        public parameters indices : []
        return value indices : []
        BLACKBOX::SHA256_COMPRESSION [{inputs_str}] [{outputs_str}]
        "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
fn memory_read() {
    let src = "
    current witness index : _1
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    MEM (id: 0, read at: EXPR [ (1, _0) 0 ], value: EXPR [ (1, _1) 0 ]) 
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_read_with_predicate() {
    let src = "
    current witness index : _2
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    MEM PREDICATE: EXPR [ (1, _0) 0 ]
    (id: 0, read at: EXPR [ (1, _1) 0 ], value: EXPR [ (1, _2) 0 ]) 
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_write() {
    let src = "
    current witness index : _1
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    MEM (id: 3, write EXPR [ (1, _0) 0 ] at: EXPR [ (1, _1) 0 ])
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_init() {
    let src = "
    current witness index : _4
    private parameters indices : []
    public parameters indices : []
    return value indices : []
    INIT (id: 4, len: 5, witnesses: [_0, _1, _2, _3, _4])
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_databus() {
    let src = "
    current witness index : _5
    private parameters indices : [_0, _1, _2, _3, _4, _5]
    public parameters indices : []
    return value indices : []
    INIT CALLDATA 0 (id: 1, len: 5, witnesses: [_1, _2, _3, _4, _5])
    INIT RETURNDATA (id: 2, len: 1, witnesses: [_6])
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call() {
    let src = "
    current witness index : _2
    private parameters indices : [_0, _1, _2]
    public parameters indices : []
    return value indices : []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, _0) (-1, _1) 0 ]], outputs: [_3]
    EXPR [ (1, _0, _3) (-1, _1, _3) -1 ]
    EXPR [ (-1, _0) (1, _2) 0 ]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call_with_predicate() {
    let src = "
    current witness index : _2
    private parameters indices : [_0, _1, _2]
    public parameters indices : []
    return value indices : []
    BRILLIG CALL func 0: PREDICATE: EXPR [ 1 ]
    inputs: [EXPR [ (1, _0) (-1, _1) 0 ]], outputs: [_3]
    EXPR [ (1, _0, _3) (-1, _1, _3) -1 ]
    EXPR [ (-1, _0) (1, _2) 0 ]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call_with_memory_array_input() {
    let src = "
    current witness index : _2
    private parameters indices : [_0, _1, _2]
    public parameters indices : []
    return value indices : []
    BRILLIG CALL func 0: inputs: [EXPR [ 2 ], MemoryArray(0)], outputs: []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn call() {
    let src = "
    current witness index : _2
    private parameters indices : [_0]
    public parameters indices : [_1]
    return value indices : []
    CALL func 1: inputs: [_0, _1], outputs: [_2]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn call_with_predicate() {
    let src = "
    current witness index : _2
    private parameters indices : [_0]
    public parameters indices : [_1]
    return value indices : []
    CALL func 1: PREDICATE: EXPR [ 1 ]
    inputs: [_0, _1], outputs: [_2]
    ";
    assert_circuit_roundtrip(src);
}

/// ACIR taken from `test_programs/execution_success/array_dynamic`
#[test]
fn array_dynamic() {
    let src = "
    current witness index : _78
    private parameters indices : [_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18]
    public parameters indices : []
    return value indices : []
    BLACKBOX::RANGE [(_0, 32)] []
    BLACKBOX::RANGE [(_1, 32)] []
    BLACKBOX::RANGE [(_2, 32)] []
    BLACKBOX::RANGE [(_3, 32)] []
    BLACKBOX::RANGE [(_4, 32)] []
    INIT (id: 0, len: 5, witnesses: [_0, _1, _2, _3, _4])
    BLACKBOX::RANGE [(_5, 32)] []
    BLACKBOX::RANGE [(_6, 32)] []
    INIT (id: 1, len: 5, witnesses: [_7, _8, _9, _10, _11])
    BLACKBOX::RANGE [(_12, 32)] []
    BLACKBOX::RANGE [(_13, 32)] []
    BLACKBOX::RANGE [(_14, 32)] []
    BLACKBOX::RANGE [(_15, 32)] []
    BLACKBOX::RANGE [(_16, 32)] []
    BLACKBOX::RANGE [(_17, 32)] []
    EXPR [ (5, _6) (-1, _19) 0 ]
    BLACKBOX::RANGE [(_19, 32)] []
    EXPR [ (1, _5) (-1, _19) (-1, _20) 0 ]
    BLACKBOX::RANGE [(_20, 32)] []
    EXPR [ (1, _20) (-1, _21) -5 ]
    EXPR [ (1, _21) (-1, _22) -3 ]
    MEM (id: 0, read at: EXPR [ (1, _21) 0 ], value: EXPR [ (1, _23) 0 ]) 
    EXPR [ (1, _23) -111 ]
    MEM (id: 0, read at: EXPR [ (1, _22) 0 ], value: EXPR [ (1, _24) 0 ]) 
    EXPR [ (1, _24) -101 ]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, _22) 4294967291 ], EXPR [ 4294967296 ]], outputs: [_25, _26]
    BLACKBOX::RANGE [(_26, 32)] []
    EXPR [ (1, _22) (-4294967296, _25) (-1, _26) 4294967291 ]
    EXPR [ (-1, _25) 0 ]
    EXPR [ (-1, _27) 0 ]
    MEM (id: 0, write EXPR [ (1, _27) 0 ] at: EXPR [ (1, _22) 0 ]) 
    MEM (id: 0, read at: EXPR [ (1, _21) 0 ], value: EXPR [ (1, _28) 0 ]) 
    EXPR [ (1, _28) -111 ]
    EXPR [ (-1, _29) 1 ]
    MEM (id: 0, read at: EXPR [ (1, _29) 0 ], value: EXPR [ (1, _30) 0 ]) 
    EXPR [ (1, _30) 0 ]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, _21) 4294967286 ], EXPR [ 4294967296 ]], outputs: [_31, _32]
    BLACKBOX::RANGE [(_31, 1)] []
    BLACKBOX::RANGE [(_32, 32)] []
    EXPR [ (1, _21) (-4294967296, _31) (-1, _32) 4294967286 ]
    EXPR [ (-1, _21, _31) (1, _21) (-1, _33) 0 ]
    MEM (id: 0, read at: EXPR [ (1, _33) 0 ], value: EXPR [ (1, _34) 0 ]) 
    EXPR [ (-1, _31, _34) (2, _31) (1, _34) (-1, _35) -2 ]
    BLACKBOX::RANGE [(_35, 32)] []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, _21) 4294967291 ], EXPR [ 4294967296 ]], outputs: [_36, _37]
    BLACKBOX::RANGE [(_36, 1)] []
    BLACKBOX::RANGE [(_37, 32)] []
    EXPR [ (1, _21) (-4294967296, _36) (-1, _37) 4294967291 ]
    EXPR [ (1, _31, _36) (-1, _36) 0 ]
    EXPR [ (-1, _21, _31) (1, _21) (-1, _38) 0 ]
    MEM (id: 0, read at: EXPR [ (1, _38) 0 ], value: EXPR [ (1, _39) 0 ]) 
    MEM (id: 0, read at: EXPR [ (1, _27) 0 ], value: EXPR [ (1, _40) 0 ]) 
    MEM (id: 0, read at: EXPR [ (1, _29) 0 ], value: EXPR [ (1, _41) 0 ]) 
    EXPR [ (-1, _42) 2 ]
    MEM (id: 0, read at: EXPR [ (1, _42) 0 ], value: EXPR [ (1, _43) 0 ]) 
    EXPR [ (-1, _44) 3 ]
    MEM (id: 0, read at: EXPR [ (1, _44) 0 ], value: EXPR [ (1, _45) 0 ]) 
    EXPR [ (-1, _46) 4 ]
    MEM (id: 0, read at: EXPR [ (1, _46) 0 ], value: EXPR [ (1, _47) 0 ]) 
    INIT (id: 3, len: 5, witnesses: [_40, _41, _43, _45, _47])
    EXPR [ (-1, _31, _35) (1, _31, _39) (1, _35) (-1, _48) 0 ]
    MEM (id: 3, write EXPR [ (1, _48) 0 ] at: EXPR [ (1, _38) 0 ]) 
    MEM (id: 3, read at: EXPR [ (1, _46) 0 ], value: EXPR [ (1, _49) 0 ]) 
    MEM (id: 0, read at: EXPR [ (1, _46) 0 ], value: EXPR [ (1, _50) 0 ]) 
    EXPR [ (-1, _31, _36) 0 ]
    EXPR [ (1, _21, _31) (-1, _51) 0 ]
    MEM (id: 0, read at: EXPR [ (1, _51) 0 ], value: EXPR [ (1, _52) 0 ]) 
    EXPR [ (-1, _31, _52) (1, _52) (-1, _53) 0 ]
    MEM (id: 0, write EXPR [ (1, _53) 0 ] at: EXPR [ (1, _51) 0 ]) 
    MEM (id: 0, read at: EXPR [ (1, _46) 0 ], value: EXPR [ (1, _54) 0 ]) 
    EXPR [ (-1, _31) (-1, _55) 1 ]
    EXPR [ (-1, _31, _49) (1, _31, _50) (1, _49) (-1, _56) 0 ]
    EXPR [ (1, _31, _54) (1, _55, _56) -109 ]
    EXPR [ (-1, _57) 246 ]
    EXPR [ (-1, _58) 159 ]
    EXPR [ (-1, _59) 32 ]
    EXPR [ (-1, _60) 176 ]
    EXPR [ (-1, _61) 8 ]
    INIT (id: 4, len: 5, witnesses: [_57, _58, _59, _60, _61])
    MEM (id: 4, read at: EXPR [ (1, _7) 0 ], value: EXPR [ (1, _62) 0 ]) 
    MEM (id: 4, read at: EXPR [ (1, _8) 0 ], value: EXPR [ (1, _63) 0 ]) 
    MEM (id: 4, read at: EXPR [ (1, _9) 0 ], value: EXPR [ (1, _64) 0 ]) 
    MEM (id: 4, read at: EXPR [ (1, _10) 0 ], value: EXPR [ (1, _65) 0 ]) 
    MEM (id: 4, read at: EXPR [ (1, _11) 0 ], value: EXPR [ (1, _66) 0 ]) 
    BRILLIG CALL func 1: inputs: [EXPR [ (1, _62) (1, _63) (1, _64) (1, _65) (1, _66) 0 ]], outputs: [_67]
    EXPR [ (1, _62, _67) (1, _63, _67) (1, _64, _67) (1, _65, _67) (1, _66, _67) -1 ]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, _18) 0 ], EXPR [ 4294967296 ]], outputs: [_68, _69]
    BLACKBOX::RANGE [(_68, 222)] []
    BLACKBOX::RANGE [(_69, 32)] []
    EXPR [ (1, _18) (-4294967296, _68) (-1, _69) 0 ]
    EXPR [ (-1, _68) (-1, _70) 5096253676302562286669017222071363378443840053029366383258766538131 ]
    BLACKBOX::RANGE [(_70, 222)] []
    BRILLIG CALL func 1: inputs: [EXPR [ (-1, _68) 5096253676302562286669017222071363378443840053029366383258766538131 ]], outputs: [_71]
    EXPR [ (-1, _68, _71) (5096253676302562286669017222071363378443840053029366383258766538131, _71) (1, _72) -1 ]
    EXPR [ (-1, _68, _72) (5096253676302562286669017222071363378443840053029366383258766538131, _72) 0 ]
    EXPR [ (1, _69, _72) (268435455, _72) (-1, _73) 0 ]
    BLACKBOX::RANGE [(_73, 32)] []
    BRILLIG CALL func 0: inputs: [EXPR [ (-1, _69) 4294967299 ], EXPR [ 4294967296 ]], outputs: [_74, _75]
    BLACKBOX::RANGE [(_74, 1)] []
    BLACKBOX::RANGE [(_75, 32)] []
    EXPR [ (-1, _69) (-4294967296, _74) (-1, _75) 4294967299 ]
    EXPR [ (-1, _17, _74) (1, _17) (-3, _74) (-1, _76) 3 ]
    BLACKBOX::RANGE [(_76, 32)] []
    EXPR [ (-1, _74, _76) (1, _76) (-1, _77) 0 ]
    MEM (id: 1, read at: EXPR [ (1, _77) 0 ], value: EXPR [ (1, _78) 0 ]) 
    EXPR [ (1, _15, _74) (-1, _74, _78) (-1, _15) (1, _78) 0 ]
    ";
    assert_circuit_roundtrip(src);
}
