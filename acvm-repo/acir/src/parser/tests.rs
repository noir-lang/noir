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
