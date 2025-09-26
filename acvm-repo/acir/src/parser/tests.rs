use insta::assert_snapshot;

use crate::{
    circuit::{Circuit, Program},
    parser::ParserError,
};

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

fn assert_program_roundtrip(src: &str) {
    let program = Program::from_str(src).unwrap();
    let program = program.to_string();
    let program = trim_leading_whitespace_from_lines(&program);
    let src = trim_leading_whitespace_from_lines(src);
    if program != src {
        println!("Expected:\n~~~\n{src}\n~~~\nGot:\n~~~\n{program}\n~~~");
        similar_asserts::assert_eq!(program, src);
    }
}

#[test]
fn private_parameters() {
    let src = "
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn public_parameters() {
    let src = "
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: [w5, w6, w7, w8, w9]
    return values: []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn return_values() {
    let src = "
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: [w5, w6, w7, w8, w9]
    return values: [w10, w11, w12]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn computed_current_witness() {
    let src = "
    private parameters: [w0, w1]
    public parameters: [w3]
    return values: [w2]
    ";
    let circuit = Circuit::from_str(src).unwrap();
    assert_eq!(circuit.current_witness_index, 3);
}

#[test]
fn assert_zero_opcodes() {
    let src = "
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: [w5, w6, w7, w8, w9]
    return values: []
    EXPR w5 = w0
    EXPR w6 = w1
    EXPR w7 = w2
    EXPR w8 = w3
    EXPR w9 = w4
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn assert_zero_with_mul_terms() {
    let src = "
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    EXPR w3 = w0*w1
    EXPR w4 = w3*w3
    EXPR w5 = w4*w4
    EXPR w6 = w5*w5
    EXPR w6 = w2
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn range_check() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::RANGE inputs: [w0], bits: 32, outputs: []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn xor() {
    let src = "
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    BLACKBOX::RANGE inputs: [w0], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w1], bits: 32, outputs: []
    BLACKBOX::XOR inputs: [w0, w1], bits: 32, outputs: [w2]
    EXPR w2 = 15
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn aes128_encrypt() {
    // This ACIR represents an accurately constrained aes128 encryption in ACIR
    let src = "
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43]
    public parameters: [w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59]
    return values: []
    BLACKBOX::RANGE inputs: [w0], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w1], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w2], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w3], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w4], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w5], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w6], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w7], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w8], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w9], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w10], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w11], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w12], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w13], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w14], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w15], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w16], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w17], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w18], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w19], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w20], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w21], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w22], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w23], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w24], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w25], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w26], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w27], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w28], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w29], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w30], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w31], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w32], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w33], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w34], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w35], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w36], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w37], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w38], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w39], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w40], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w41], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w42], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w43], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w44], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w45], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w46], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w47], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w48], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w49], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w50], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w51], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w52], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w53], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w54], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w55], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w56], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w57], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w58], bits: 8, outputs: []
    BLACKBOX::RANGE inputs: [w59], bits: 8, outputs: []
    BLACKBOX::AES128_ENCRYPT inputs: [w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43], outputs: [w60, w61, w62, w63, w64, w65, w66, w67, w68, w69, w70, w71, w72, w73, w74, w75]
    EXPR w60 = w44
    EXPR w61 = w45
    EXPR w62 = w46
    EXPR w63 = w47
    EXPR w64 = w48
    EXPR w65 = w49
    EXPR w66 = w50
    EXPR w67 = w51
    EXPR w68 = w52
    EXPR w69 = w53
    EXPR w70 = w54
    EXPR w71 = w55
    EXPR w72 = w56
    EXPR w73 = w57
    EXPR w74 = w58
    EXPR w75 = w59
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn blake2s() {
    let src = "
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: [w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36]
    return values: []
    BLACKBOX::BLAKE2S inputs: [w0, w1, w2, w3, w4], outputs: [w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59, w60, w61, w62, w63, w64, w65, w66, w67, w68]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn blake3() {
    let src = "
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: []
    BLACKBOX::BLAKE3 inputs: [w0, w1, w2, w3, w4], outputs: [w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn ecdsa_secp256k1() {
    let input_witnesses: Vec<String> = (0..161).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let src = format!(
        "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256K1 inputs: [{inputs_str}], outputs: [w161]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic = "Expected 32 inputs for public_key_y, found 3"]
fn ecdsa_secp256k1_missing_inputs() {
    let input_witnesses: Vec<String> = (0..100).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let src = format!(
        "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256K1 inputs: [{inputs_str}], outputs: [w100]
    "
    );
    let _ = Circuit::from_str(&src).unwrap();
}

#[test]
fn ecdsa_secp256r1() {
    let input_witnesses: Vec<String> = (0..161).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let src = format!(
        "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256R1 inputs: [{inputs_str}], outputs: [w161]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic = "Expected 32 inputs for public_key_y, found 3"]
fn ecdsa_secp256r1_missing_inputs() {
    let input_witnesses: Vec<String> = (0..100).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let src = format!(
        "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256R1 inputs: [{inputs_str}], outputs: [w100]
    "
    );
    let _ = Circuit::from_str(&src).unwrap();
}

#[test]
fn keccakf1600() {
    let input_witnesses: Vec<String> = (0..25).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let output_witnesses: Vec<String> = (25..50).map(|i| format!("w{i}")).collect();
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::KECCAKF1600 inputs: [{inputs_str}], outputs: [{outputs_str}]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic = "Expected 25 inputs for Keccakf1600 inputs, found 24"]
fn keccakf1600_missing_inputs() {
    let input_witnesses: Vec<String> = (0..24).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let output_witnesses: Vec<String> = (24..49).map(|i| format!("w{i}")).collect();
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::KECCAKF1600 inputs: [{inputs_str}], outputs: [{outputs_str}]
        "
    );
    let _ = Circuit::from_str(&src).unwrap();
}

#[test]
fn embedded_curve_add() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::EMBEDDED_CURVE_ADD inputs: [w0, w1, w2, w3, w4, w5, w6], outputs: [w7, w8, w9]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
#[should_panic]
fn embedded_curve_add_wrong_output_count() {
    let src = "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::EMBEDDED_CURVE_ADD inputs: [w0, w1, w2, w3, w4, w5, w6], outputs: [w7, w8]
    ";
    let _ = Circuit::from_str(src).unwrap();
}

#[test]
fn poseidon2_permutation() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::POSEIDON2_PERMUTATION inputs: [w0, w1, w2], outputs: [w3, w4, w5]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn sha256_compression() {
    let input_witnesses: Vec<String> = (0..24).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let output_witnesses: Vec<String> = (24..32).map(|i| format!("w{i}")).collect();
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::SHA256_COMPRESSION inputs: [{inputs_str}], outputs: [{outputs_str}]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic]
fn sha256_compression_missing_outputs() {
    let input_witnesses: Vec<String> = (0..24).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let output_witnesses: Vec<String> = (24..31).map(|i| format!("w{i}")).collect(); // should be 8 total
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::SHA256_COMPRESSION inputs: [{inputs_str}], outputs: [{outputs_str}]
        "
    );
    let _ = Circuit::from_str(&src).unwrap();
}

#[test]
fn memory_read() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    MEM id: 0, read at: w0, value: w1
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_write() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    MEM id: 3, write: w0, at: w1
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_init() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    INIT id: 4, len: 5, witnesses: [w0, w1, w2, w3, w4]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_init_duplicate_witness() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    INIT id: 4, len: 2, witnesses: [w0, w0]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_databus() {
    let src = "
    private parameters: [w0, w1, w2, w3, w4, w5]
    public parameters: []
    return values: []
    INIT CALLDATA 0 id: 1, len: 5, witnesses: [w1, w2, w3, w4, w5]
    INIT RETURNDATA id: 2, len: 1, witnesses: [w6]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call() {
    let src = "
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [w0 - w1], outputs: [w3]
    EXPR 0 = w0*w3 - w1*w3 - 1
    EXPR w2 = w0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call_with_predicate() {
    let src = "
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, predicate: 1, inputs: [w0 - w1], outputs: [w3]
    EXPR 0 = w0*w3 - w1*w3 - 1
    EXPR w2 = w0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call_with_memory_array_input() {
    let src = "
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [2, MemoryArray(0)], outputs: []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn call() {
    let src = "
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    CALL func: 1, inputs: [w0, w1], outputs: [w2]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn call_with_predicate() {
    let src = "
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w2]
    ";
    assert_circuit_roundtrip(src);
}

/// ACIR taken from `test_programs/execution_success/array_dynamic`
#[test]
fn array_dynamic() {
    let src = "
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18]
    public parameters: []
    return values: []
    BLACKBOX::RANGE inputs: [w0], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w1], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w2], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w3], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w4], bits: 32, outputs: []
    INIT id: 0, len: 5, witnesses: [w0, w1, w2, w3, w4]
    BLACKBOX::RANGE inputs: [w5], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w6], bits: 32, outputs: []
    INIT id: 1, len: 5, witnesses: [w7, w8, w9, w10, w11]
    BLACKBOX::RANGE inputs: [w12], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w13], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w14], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w15], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w16], bits: 32, outputs: []
    BLACKBOX::RANGE inputs: [w17], bits: 32, outputs: []
    EXPR w19 = 5*w6
    BLACKBOX::RANGE inputs: [w19], bits: 32, outputs: []
    EXPR w20 = w5 - w19
    BLACKBOX::RANGE inputs: [w20], bits: 32, outputs: []
    EXPR w21 = w20 - 5
    EXPR w22 = w21 - 3
    MEM id: 0, read at: w21, value: w23
    EXPR w23 = 111
    MEM id: 0, read at: w22, value: w24
    EXPR w24 = 101
    BRILLIG CALL func: 0, inputs: [w22 + 4294967291, 4294967296], outputs: [w25, w26]
    BLACKBOX::RANGE inputs: [w26], bits: 32, outputs: []
    EXPR w26 = w22 - 4294967296*w25 + 4294967291
    EXPR w25 = 0
    EXPR w27 = 0
    MEM id: 0, write: w27, at: w22
    MEM id: 0, read at: w21, value: w28
    EXPR w28 = 111
    EXPR w29 = 1
    MEM id: 0, read at: w29, value: w30
    EXPR w30 = 0
    BRILLIG CALL func: 0, inputs: [w21 + 4294967286, 4294967296], outputs: [w31, w32]
    BLACKBOX::RANGE inputs: [w31], bits: 1, outputs: []
    BLACKBOX::RANGE inputs: [w32], bits: 32, outputs: []
    EXPR w32 = w21 - 4294967296*w31 + 4294967286
    EXPR w33 = -w21*w31 + w21
    MEM id: 0, read at: w33, value: w34
    EXPR w35 = -w31*w34 + 2*w31 + w34 - 2
    BLACKBOX::RANGE inputs: [w35], bits: 32, outputs: []
    BRILLIG CALL func: 0, inputs: [w21 + 4294967291, 4294967296], outputs: [w36, w37]
    BLACKBOX::RANGE inputs: [w36], bits: 1, outputs: []
    BLACKBOX::RANGE inputs: [w37], bits: 32, outputs: []
    EXPR w37 = w21 - 4294967296*w36 + 4294967291
    EXPR w36 = w31*w36
    EXPR w38 = -w21*w31 + w21
    MEM id: 0, read at: w38, value: w39
    MEM id: 0, read at: w27, value: w40
    MEM id: 0, read at: w29, value: w41
    EXPR w42 = 2
    MEM id: 0, read at: w42, value: w43
    EXPR w44 = 3
    MEM id: 0, read at: w44, value: w45
    EXPR w46 = 0
    MEM id: 0, read at: w46, value: w47
    INIT id: 3, len: 5, witnesses: [w40, w41, w43, w45, w47]
    EXPR w48 = -w31*w35 + w31*w39 + w35
    MEM id: 3, write: w48, at: w38
    MEM id: 3, read at: w46, value: w49
    MEM id: 0, read at: w46, value: w50
    EXPR 0 = -w31*w36
    EXPR w51 = w21*w31
    MEM id: 0, read at: w51, value: w52
    EXPR w53 = -w31*w52 + w52
    MEM id: 0, write: w53, at: w51
    MEM id: 0, read at: w46, value: w54
    EXPR w55 = -w31 + 1
    EXPR w56 = -w31*w49 + w31*w50 + w49
    EXPR 0 = w31*w54 + w55*w56 - 109
    EXPR w57 = 246
    EXPR w58 = 159
    EXPR w59 = 32
    EXPR w60 = 176
    EXPR w61 = 8
    INIT id: 4, len: 5, witnesses: [w57, w58, w59, w60, w61]
    MEM id: 4, read at: w7, value: w62
    MEM id: 4, read at: w8, value: w63
    MEM id: 4, read at: w9, value: w64
    MEM id: 4, read at: w10, value: w65
    MEM id: 4, read at: w11, value: w66
    BRILLIG CALL func: 1, inputs: [w62 + w63 + w64 + w65 + w66], outputs: [w67]
    EXPR 0 = w62*w67 + w63*w67 + w64*w67 + w65*w67 + w66*w67 - 1
    BRILLIG CALL func: 0, inputs: [w18, 4294967296], outputs: [w68, w69]
    BLACKBOX::RANGE inputs: [w68], bits: 222, outputs: []
    BLACKBOX::RANGE inputs: [w69], bits: 32, outputs: []
    EXPR w69 = w18 - 4294967296*w68
    EXPR w70 = -w68 + 5096253676302562286669017222071363378443840053029366383258766538131
    BLACKBOX::RANGE inputs: [w70], bits: 222, outputs: []
    BRILLIG CALL func: 1, inputs: [-w68 + 5096253676302562286669017222071363378443840053029366383258766538131], outputs: [w71]
    EXPR w72 = w68*w71 - 5096253676302562286669017222071363378443840053029366383258766538131*w71 + 1
    EXPR 0 = -w68*w72 + 5096253676302562286669017222071363378443840053029366383258766538131*w72
    EXPR w73 = w69*w72 + 268435455*w72
    BLACKBOX::RANGE inputs: [w73], bits: 32, outputs: []
    BRILLIG CALL func: 0, inputs: [-w69 + 4294967299, 4294967296], outputs: [w74, w75]
    BLACKBOX::RANGE inputs: [w74], bits: 1, outputs: []
    BLACKBOX::RANGE inputs: [w75], bits: 32, outputs: []
    EXPR w75 = -w69 - 4294967296*w74 + 4294967299
    EXPR w76 = -w17*w74 + w17 - 3*w74 + 3
    BLACKBOX::RANGE inputs: [w76], bits: 32, outputs: []
    EXPR w77 = -w74*w76 + w76
    MEM id: 1, read at: w77, value: w78
    EXPR w78 = -w15*w74 + w74*w78 + w15
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn fold_basic() {
    let src = "
    func 0
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w2]

    func 1
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BRILLIG CALL func: 0, inputs: [w0 - w1], outputs: [w3]
    EXPR 0 = w0*w3 - w1*w3 - 1
    EXPR w2 = w0
    ";
    assert_program_roundtrip(src);
}

#[test]
fn fold_basic_mismatched_ids() {
    let src = "
    func 0
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w2]

    func 2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BRILLIG CALL func: 0, inputs: [w0 - w1], outputs: [w3]
    EXPR w0*w3 - w1*w3 - 1 = 0
    EXPR w0 = w2
    ";
    let result = Program::from_str(src).err().unwrap();
    let ParserError::UnexpectedFunctionId { expected, found, .. } = result.get_error() else {
        panic!("Expected `UnexpectedFunctionId` error");
    };
    assert_eq!(expected, 1);
    assert_eq!(found, 2);
}

#[test]
fn assert_zero_equation() {
    let src = "
    private parameters: [w0, w1, w2, w2]
    public parameters: []
    return values: []
    EXPR -w0 + w1 - 10 + 20 + w0*w2 = w2 - w3 + w0*w1 - w1*w2 - 30
    ";
    let circuit = Circuit::from_str(src).unwrap();
    assert_snapshot!(circuit.to_string(), @r"
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    EXPR w3 = -w0*w2 + w0*w1 - w1*w2 + w0 - w1 + w2 - 40
    ");
}

#[test]
fn does_not_negate_when_equal_to_zero() {
    let src = "
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    EXPR w0*w1 + w0*w2 = 0
    ";
    let circuit = Circuit::from_str(src).unwrap();
    assert_snapshot!(circuit.to_string(), @r"
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    EXPR 0 = w0*w1 + w0*w2
    ");
}
