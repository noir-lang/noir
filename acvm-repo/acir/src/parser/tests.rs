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
fn computes_current_witness() {
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
    ASSERT w5 = w0
    ASSERT w6 = w1
    ASSERT w7 = w2
    ASSERT w8 = w3
    ASSERT w9 = w4
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn assert_zero_with_mul_terms() {
    let src = "
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    ASSERT w3 = w0*w1
    ASSERT w4 = w3*w3
    ASSERT w5 = w4*w4
    ASSERT w6 = w5*w5
    ASSERT w6 = w2
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn range_check() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::RANGE input: w0, bits: 32
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn and() {
    let src = "
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 32
    BLACKBOX::AND lhs: w0, rhs: w1, output: w2, bits: 32
    ASSERT w2 = 15
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn xor() {
    let src = "
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 32
    BLACKBOX::XOR lhs: w0, rhs: w1, output: w2, bits: 32
    ASSERT w2 = 15
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
    BLACKBOX::AES128_ENCRYPT inputs: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27], iv: [w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43], key: [w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59], outputs: [w60, w61, w62, w63, w64, w65, w66, w67, w68, w69, w70, w71, w72, w73, w74, w75]
    ASSERT w60 = w44
    ASSERT w61 = w45
    ASSERT w62 = w46
    ASSERT w63 = w47
    ASSERT w64 = w48
    ASSERT w65 = w49
    ASSERT w66 = w50
    ASSERT w67 = w51
    ASSERT w68 = w52
    ASSERT w69 = w53
    ASSERT w70 = w54
    ASSERT w71 = w55
    ASSERT w72 = w56
    ASSERT w73 = w57
    ASSERT w74 = w58
    ASSERT w75 = w59
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
    let public_key_x = (0..32).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let public_key_y = (32..64).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let signature = (64..128).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let hashed_message = (128..160).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");

    let src = format!(
    "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256K1 public_key_x: [{public_key_x}], public_key_y: [{public_key_y}], signature: [{signature}], hashed_message: [{hashed_message}], predicate: 1, output: w161
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic = "Expected 32 values for public_key_x, found 31"]
fn ecdsa_secp256k1_missing_inputs() {
    let public_key_x = (0..31).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let public_key_y = (32..64).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let signature = (64..128).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let hashed_message = (128..160).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");

    let src = format!(
    "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256K1 public_key_x: [{public_key_x}], public_key_y: [{public_key_y}], signature: [{signature}], hashed_message: [{hashed_message}], predicate: 1, output: w161
    "
    );
    let _ = Circuit::from_str(&src).unwrap();
}

#[test]
fn ecdsa_secp256r1() {
    let public_key_x = (0..32).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let public_key_y = (32..64).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let signature = (64..128).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let hashed_message = (128..160).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");

    let src = format!(
    "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256R1 public_key_x: [{public_key_x}], public_key_y: [{public_key_y}], signature: [{signature}], hashed_message: [{hashed_message}], predicate: 1, output: w161
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic = "Expected 64 values for signature, found 30"]
fn ecdsa_secp256r1_missing_inputs() {
    let public_key_x = (0..32).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let public_key_y = (32..64).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let signature = (64..94).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");
    let hashed_message = (128..160).map(|i| format!("w{i}")).collect::<Vec<_>>().join(", ");

    let src = format!(
    "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256R1 public_key_x: [{public_key_x}], public_key_y: [{public_key_y}], signature: [{signature}], hashed_message: [{hashed_message}], predicate: 1, output: w161
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
#[should_panic = "Expected 25 values for inputs, found 24"]
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
    BLACKBOX::EMBEDDED_CURVE_ADD input1: [w0, w1, w2], input2: [w3, w4, w5], predicate: w6, outputs: [w7, w8, w9]
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
        BLACKBOX::EMBEDDED_CURVE_ADD input1: [w0, w1, w2], input2: [w3, w4, w5], predicate: w6, outputs: [w7, w8]
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
    let input_witnesses: Vec<String> = (0..16).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let hash_value_witnesses: Vec<String> = (16..24).map(|i| format!("w{i}")).collect();
    let hash_values_str = hash_value_witnesses.join(", ");

    let output_witnesses: Vec<String> = (24..32).map(|i| format!("w{i}")).collect();
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::SHA256_COMPRESSION inputs: [{inputs_str}], hash_values: [{hash_values_str}], outputs: [{outputs_str}]
    "
    );
    assert_circuit_roundtrip(&src);
}

#[test]
#[should_panic]
fn sha256_compression_missing_outputs() {
    let input_witnesses: Vec<String> = (0..16).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let hash_value_witnesses: Vec<String> = (16..24).map(|i| format!("w{i}")).collect();
    let hash_values_str = hash_value_witnesses.join(", ");

    let output_witnesses: Vec<String> = (24..31).map(|i| format!("w{i}")).collect(); // should be 8 total
    let outputs_str = output_witnesses.join(", ");

    let src = format!(
        "
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::SHA256_COMPRESSION inputs: [{inputs_str}], hash_values: [{hash_values_str}], outputs: [{outputs_str}]
        "
    );
    let _ = Circuit::from_str(&src).unwrap();
}

#[test]
fn multi_scalar_mul() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::MULTI_SCALAR_MUL points: [w0, w1], scalars: [w2, w3], predicate: 1, outputs: [w4, w5, w6]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn recursive_aggregation() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::RECURSIVE_AGGREGATION verification_key: [w0], proof: [w1], public_inputs: [w2], key_hash: w3, proof_type: 4, predicate: 1
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_read() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    READ w1 = b0[w0]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_write() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    WRITE b3[w1] = w0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_init() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    INIT b4 = [w0, w1, w2, w3, w4]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_init_duplicate_witness() {
    let src = "
    private parameters: []
    public parameters: []
    return values: []
    INIT b4 = [w0, w0]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_databus() {
    let src = "
    private parameters: [w0, w1, w2, w3, w4, w5]
    public parameters: []
    return values: []
    INIT CALLDATA 0 b1 = [w1, w2, w3, w4, w5]
    INIT RETURNDATA b2 = [w6]
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
    ASSERT 0 = w0*w3 - w1*w3 - 1
    ASSERT w2 = w0
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
    ASSERT 0 = w0*w3 - w1*w3 - 1
    ASSERT w2 = w0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call_with_memory_array_input() {
    let src = "
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [2, b0], outputs: []
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
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 32
    BLACKBOX::RANGE input: w2, bits: 32
    BLACKBOX::RANGE input: w3, bits: 32
    BLACKBOX::RANGE input: w4, bits: 32
    INIT b0 = [w0, w1, w2, w3, w4]
    BLACKBOX::RANGE input: w5, bits: 32
    BLACKBOX::RANGE input: w6, bits: 32
    INIT b1 = [w7, w8, w9, w10, w11]
    BLACKBOX::RANGE input: w12, bits: 32
    BLACKBOX::RANGE input: w13, bits: 32
    BLACKBOX::RANGE input: w14, bits: 32
    BLACKBOX::RANGE input: w15, bits: 32
    BLACKBOX::RANGE input: w16, bits: 32
    BLACKBOX::RANGE input: w17, bits: 32
    ASSERT w19 = 5*w6
    BLACKBOX::RANGE input: w19, bits: 32
    ASSERT w20 = w5 - w19
    BLACKBOX::RANGE input: w20, bits: 32
    ASSERT w21 = w20 - 5
    ASSERT w22 = w21 - 3
    READ w23 = b0[w21]
    ASSERT w23 = 111
    READ w24 = b0[w22]
    ASSERT w24 = 101
    BRILLIG CALL func: 0, inputs: [w22 + 4294967291, 4294967296], outputs: [w25, w26]
    BLACKBOX::RANGE input: w26, bits: 32
    ASSERT w26 = w22 - 4294967296*w25 + 4294967291
    ASSERT w25 = 0
    ASSERT w27 = 0
    WRITE b0[w22] = w27
    READ w28 = b0[w21]
    ASSERT w28 = 111
    ASSERT w29 = 1
    READ w30 = b0[w29]
    ASSERT w30 = 0
    BRILLIG CALL func: 0, inputs: [w21 + 4294967286, 4294967296], outputs: [w31, w32]
    BLACKBOX::RANGE input: w31, bits: 1
    BLACKBOX::RANGE input: w32, bits: 32
    ASSERT w32 = w21 - 4294967296*w31 + 4294967286
    ASSERT w33 = -w21*w31 + w21
    READ w34 = b0[w33]
    ASSERT w35 = -w31*w34 + 2*w31 + w34 - 2
    BLACKBOX::RANGE input: w35, bits: 32
    BRILLIG CALL func: 0, inputs: [w21 + 4294967291, 4294967296], outputs: [w36, w37]
    BLACKBOX::RANGE input: w36, bits: 1
    BLACKBOX::RANGE input: w37, bits: 32
    ASSERT w37 = w21 - 4294967296*w36 + 4294967291
    ASSERT w36 = w31*w36
    ASSERT w38 = -w21*w31 + w21
    READ w39 = b0[w38]
    READ w40 = b0[w27]
    READ w41 = b0[w29]
    ASSERT w42 = 2
    READ w43 = b0[w42]
    ASSERT w44 = 3
    READ w45 = b0[w44]
    ASSERT w46 = 0
    READ w47 = b0[w46]
    INIT b3 = [w40, w41, w43, w45, w47]
    ASSERT w48 = -w31*w35 + w31*w39 + w35
    WRITE b3[w38] = w48
    READ w49 = b3[w46]
    READ w50 = b0[w46]
    ASSERT 0 = -w31*w36
    ASSERT w51 = w21*w31
    READ w52 = b0[w51]
    ASSERT w53 = -w31*w52 + w52
    WRITE b0[w51] = w53
    READ w54 = b0[w46]
    ASSERT w55 = -w31 + 1
    ASSERT w56 = -w31*w49 + w31*w50 + w49
    ASSERT 0 = w31*w54 + w55*w56 - 109
    ASSERT w57 = 246
    ASSERT w58 = 159
    ASSERT w59 = 32
    ASSERT w60 = 176
    ASSERT w61 = 8
    INIT b4 = [w57, w58, w59, w60, w61]
    READ w62 = b4[w7]
    READ w63 = b4[w8]
    READ w64 = b4[w9]
    READ w65 = b4[w10]
    READ w999 = b4[w11]
    BRILLIG CALL func: 1, inputs: [w62 + w63 + w64 + w65 + w999], outputs: [w67]
    ASSERT 0 = w62*w67 + w63*w67 + w64*w67 + w65*w67 + w999*w67 - 1
    BRILLIG CALL func: 0, inputs: [w18, 4294967296], outputs: [w68, w69]
    BLACKBOX::RANGE input: w68, bits: 222
    BLACKBOX::RANGE input: w69, bits: 32
    ASSERT w69 = w18 - 4294967296*w68
    ASSERT w70 = -w68 + 5096253676302562286669017222071363378443840053029366383258766538131
    BLACKBOX::RANGE input: w70, bits: 222
    BRILLIG CALL func: 1, inputs: [-w68 + 5096253676302562286669017222071363378443840053029366383258766538131], outputs: [w71]
    ASSERT w72 = w68*w71 - 5096253676302562286669017222071363378443840053029366383258766538131*w71 + 1
    ASSERT 0 = -w68*w72 + 5096253676302562286669017222071363378443840053029366383258766538131*w72
    ASSERT w73 = w69*w72 + 268435455*w72
    BLACKBOX::RANGE input: w73, bits: 32
    BRILLIG CALL func: 0, inputs: [-w69 + 4294967299, 4294967296], outputs: [w74, w75]
    BLACKBOX::RANGE input: w74, bits: 1
    BLACKBOX::RANGE input: w75, bits: 32
    ASSERT w75 = -w69 - 4294967296*w74 + 4294967299
    ASSERT w76 = -w17*w74 + w17 - 3*w74 + 3
    BLACKBOX::RANGE input: w76, bits: 32
    ASSERT w77 = -w74*w76 + w76
    READ w78 = b1[w77]
    ASSERT w78 = -w15*w74 + w74*w78 + w15
    ";

    let circuit = Circuit::from_str(src).unwrap();
    assert_eq!(circuit.current_witness_index, 999);

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
    ASSERT 0 = w0*w3 - w1*w3 - 1
    ASSERT w2 = w0
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
    ASSERT w0*w3 - w1*w3 - 1 = 0
    ASSERT w0 = w2
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
    ASSERT -w0 + w1 - 10 + 20 + w0*w2 = w2 - w3 + w0*w1 - w1*w2 - 30
    ";
    let circuit = Circuit::from_str(src).unwrap();
    assert_snapshot!(circuit.to_string(), @r"
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    ASSERT w3 = -w0*w2 + w0*w1 - w1*w2 + w0 - w1 + w2 - 40
    ");
}

#[test]
fn does_not_negate_when_equal_to_zero() {
    let src = "
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    ASSERT w0*w1 + w0*w2 = 0
    ";
    let circuit = Circuit::from_str(src).unwrap();
    assert_snapshot!(circuit.to_string(), @r"
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    ASSERT 0 = w0*w1 + w0*w2
    ");
}
