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
fn current_witness() {
    let src = "
    current witness: w1
    private parameters: []
    public parameters: []
    return values: []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn private_parameters() {
    let src = "
    current witness: w4
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn public_parameters() {
    let src = "
    current witness: w9
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: [w5, w6, w7, w8, w9]
    return values: []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn return_values() {
    let src = "
    current witness: w12
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: [w5, w6, w7, w8, w9]
    return values: [w10, w11, w12]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn assert_zero_opcodes() {
    let src = "
    current witness: w9
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: [w5, w6, w7, w8, w9]
    return values: []
    EXPR 1*w0 + -1*w5 + 0 = 0
    EXPR 1*w1 + -1*w6 + 0 = 0
    EXPR 1*w2 + -1*w7 + 0 = 0
    EXPR 1*w3 + -1*w8 + 0 = 0
    EXPR 1*w4 + -1*w9 + 0 = 0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn assert_zero_with_mul_terms() {
    let src = "
    current witness: w6
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    EXPR 1*w0*w1 + -1*w3 + 0 = 0
    EXPR 1*w3*w3 + -1*w4 + 0 = 0
    EXPR 1*w4*w4 + -1*w5 + 0 = 0
    EXPR 1*w5*w5 + -1*w6 + 0 = 0
    EXPR -1*w2 + 1*w6 + 0 = 0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn range_check() {
    let src = "
    current witness: w5
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::RANGE [w0]:32 bits []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn xor() {
    let src = "
    current witness: w2
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    BLACKBOX::RANGE [w0]:32 bits []
    BLACKBOX::RANGE [w1]:32 bits []
    BLACKBOX::XOR [w0, w1]:32 bits [w2]
    EXPR 1*w2 + -15 = 0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn aes128_encrypt() {
    // This ACIR represents an accurately constrained aes128 encryption in ACIR
    let src = "
    current witness: w75
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43]
    public parameters: [w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59]
    return values: []
    BLACKBOX::RANGE [w0]:8 bits []
    BLACKBOX::RANGE [w1]:8 bits []
    BLACKBOX::RANGE [w2]:8 bits []
    BLACKBOX::RANGE [w3]:8 bits []
    BLACKBOX::RANGE [w4]:8 bits []
    BLACKBOX::RANGE [w5]:8 bits []
    BLACKBOX::RANGE [w6]:8 bits []
    BLACKBOX::RANGE [w7]:8 bits []
    BLACKBOX::RANGE [w8]:8 bits []
    BLACKBOX::RANGE [w9]:8 bits []
    BLACKBOX::RANGE [w10]:8 bits []
    BLACKBOX::RANGE [w11]:8 bits []
    BLACKBOX::RANGE [w12]:8 bits []
    BLACKBOX::RANGE [w13]:8 bits []
    BLACKBOX::RANGE [w14]:8 bits []
    BLACKBOX::RANGE [w15]:8 bits []
    BLACKBOX::RANGE [w16]:8 bits []
    BLACKBOX::RANGE [w17]:8 bits []
    BLACKBOX::RANGE [w18]:8 bits []
    BLACKBOX::RANGE [w19]:8 bits []
    BLACKBOX::RANGE [w20]:8 bits []
    BLACKBOX::RANGE [w21]:8 bits []
    BLACKBOX::RANGE [w22]:8 bits []
    BLACKBOX::RANGE [w23]:8 bits []
    BLACKBOX::RANGE [w24]:8 bits []
    BLACKBOX::RANGE [w25]:8 bits []
    BLACKBOX::RANGE [w26]:8 bits []
    BLACKBOX::RANGE [w27]:8 bits []
    BLACKBOX::RANGE [w28]:8 bits []
    BLACKBOX::RANGE [w29]:8 bits []
    BLACKBOX::RANGE [w30]:8 bits []
    BLACKBOX::RANGE [w31]:8 bits []
    BLACKBOX::RANGE [w32]:8 bits []
    BLACKBOX::RANGE [w33]:8 bits []
    BLACKBOX::RANGE [w34]:8 bits []
    BLACKBOX::RANGE [w35]:8 bits []
    BLACKBOX::RANGE [w36]:8 bits []
    BLACKBOX::RANGE [w37]:8 bits []
    BLACKBOX::RANGE [w38]:8 bits []
    BLACKBOX::RANGE [w39]:8 bits []
    BLACKBOX::RANGE [w40]:8 bits []
    BLACKBOX::RANGE [w41]:8 bits []
    BLACKBOX::RANGE [w42]:8 bits []
    BLACKBOX::RANGE [w43]:8 bits []
    BLACKBOX::RANGE [w44]:8 bits []
    BLACKBOX::RANGE [w45]:8 bits []
    BLACKBOX::RANGE [w46]:8 bits []
    BLACKBOX::RANGE [w47]:8 bits []
    BLACKBOX::RANGE [w48]:8 bits []
    BLACKBOX::RANGE [w49]:8 bits []
    BLACKBOX::RANGE [w50]:8 bits []
    BLACKBOX::RANGE [w51]:8 bits []
    BLACKBOX::RANGE [w52]:8 bits []
    BLACKBOX::RANGE [w53]:8 bits []
    BLACKBOX::RANGE [w54]:8 bits []
    BLACKBOX::RANGE [w55]:8 bits []
    BLACKBOX::RANGE [w56]:8 bits []
    BLACKBOX::RANGE [w57]:8 bits []
    BLACKBOX::RANGE [w58]:8 bits []
    BLACKBOX::RANGE [w59]:8 bits []
    BLACKBOX::AES128_ENCRYPT [w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43] [w60, w61, w62, w63, w64, w65, w66, w67, w68, w69, w70, w71, w72, w73, w74, w75]
    EXPR -1*w44 + 1*w60 + 0 = 0
    EXPR -1*w45 + 1*w61 + 0 = 0
    EXPR -1*w46 + 1*w62 + 0 = 0
    EXPR -1*w47 + 1*w63 + 0 = 0
    EXPR -1*w48 + 1*w64 + 0 = 0
    EXPR -1*w49 + 1*w65 + 0 = 0
    EXPR -1*w50 + 1*w66 + 0 = 0
    EXPR -1*w51 + 1*w67 + 0 = 0
    EXPR -1*w52 + 1*w68 + 0 = 0
    EXPR -1*w53 + 1*w69 + 0 = 0
    EXPR -1*w54 + 1*w70 + 0 = 0
    EXPR -1*w55 + 1*w71 + 0 = 0
    EXPR -1*w56 + 1*w72 + 0 = 0
    EXPR -1*w57 + 1*w73 + 0 = 0
    EXPR -1*w58 + 1*w74 + 0 = 0
    EXPR -1*w59 + 1*w75 + 0 = 0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn blake2s() {
    let src = "
    current witness: w68
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: [w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36]
    return values: []
    BLACKBOX::BLAKE2S [w0, w1, w2, w3, w4] [w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59, w60, w61, w62, w63, w64, w65, w66, w67, w68]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn blake3() {
    let src = "
    current witness: w37
    private parameters: [w0, w1, w2, w3, w4]
    public parameters: []
    return values: []
    BLACKBOX::BLAKE3 [w0, w1, w2, w3, w4] [w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31, w32, w33, w34, w35, w36]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn ecdsa_secp256k1() {
    let input_witnesses: Vec<String> = (0..161).map(|i| format!("w{i}")).collect();
    let inputs_str = input_witnesses.join(", ");

    let src = format!(
        "
    current witness: w161
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256K1 [{inputs_str}] [w161]
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
    current witness: w100
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256K1 [{inputs_str}] [w100]
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
    current witness: w161
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256R1 [{inputs_str}] [w161]
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
    current witness: w100
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::ECDSA_SECP256R1 [{inputs_str}] [w100]
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
    current witness: w50
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::KECCAKF1600 [{inputs_str}] [{outputs_str}]
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
        current witness: w49
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::KECCAKF1600 [{inputs_str}] [{outputs_str}]
        "
    );
    let _ = Circuit::from_str(&src).unwrap();
}

#[test]
fn embedded_curve_add() {
    let src = "
    current witness: w9
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::EMBEDDED_CURVE_ADD [w0, w1, w2, w3, w4, w5, w6] [w7, w8, w9]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
#[should_panic]
fn embedded_curve_add_wrong_output_count() {
    let src = "
        current witness: w9
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::EMBEDDED_CURVE_ADD [w0, w1, w2, w3, w4, w5, w6] [w7, w8]
    ";
    let _ = Circuit::from_str(src).unwrap();
}

#[test]
fn poseidon2_permutation() {
    let src = "
    current witness: w5
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::POSEIDON2_PERMUTATION [w0, w1, w2] [w3, w4, w5]
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
    current witness: w31
    private parameters: []
    public parameters: []
    return values: []
    BLACKBOX::SHA256_COMPRESSION [{inputs_str}] [{outputs_str}]
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
        current witness: w31
        private parameters: []
        public parameters: []
        return values: []
        BLACKBOX::SHA256_COMPRESSION [{inputs_str}] [{outputs_str}]
        "
    );
    let _ = Circuit::from_str(&src).unwrap();
}

#[test]
fn memory_read() {
    let src = "
    current witness: w1
    private parameters: []
    public parameters: []
    return values: []
    MEM (id: 0, read at: EXPR [ (1, w0) 0 ], value: EXPR [ (1, w1) 0 ])
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_write() {
    let src = "
    current witness: w1
    private parameters: []
    public parameters: []
    return values: []
    MEM (id: 3, write EXPR [ (1, w0) 0 ] at: EXPR [ (1, w1) 0 ])
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_init() {
    let src = "
    current witness: w4
    private parameters: []
    public parameters: []
    return values: []
    INIT (id: 4, len: 5, witnesses: [w0, w1, w2, w3, w4])
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_init_duplicate_witness() {
    let src = "
    current witness: w4
    private parameters: []
    public parameters: []
    return values: []
    INIT (id: 4, len: 2, witnesses: [w0, w0])
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn memory_databus() {
    let src = "
    current witness: w5
    private parameters: [w0, w1, w2, w3, w4, w5]
    public parameters: []
    return values: []
    INIT CALLDATA 0 (id: 1, len: 5, witnesses: [w1, w2, w3, w4, w5])
    INIT RETURNDATA (id: 2, len: 1, witnesses: [w6])
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call() {
    let src = "
    current witness: w2
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) (-1, w1) 0 ]], outputs: [w3]
    EXPR 1*w0*w3 + -1*w1*w3 + -1 = 0
    EXPR -1*w0 + 1*w2 + 0 = 0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call_with_predicate() {
    let src = "
    current witness: w2
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BRILLIG CALL func 0: PREDICATE: EXPR [ 1 ]
    inputs: [EXPR [ (1, w0) (-1, w1) 0 ]], outputs: [w3]
    EXPR 1*w0*w3 + -1*w1*w3 + -1 = 0
    EXPR -1*w0 + 1*w2 + 0 = 0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn brillig_call_with_memory_array_input() {
    let src = "
    current witness: w2
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BRILLIG CALL func 0: inputs: [EXPR [ 2 ], MemoryArray(0)], outputs: []
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn call() {
    let src = "
    current witness: w2
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    CALL func 1: inputs: [w0, w1], outputs: [w2]
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn call_with_predicate() {
    let src = "
    current witness: w2
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    CALL func 1: PREDICATE: EXPR [ 1 ]
    inputs: [w0, w1], outputs: [w2]
    ";
    assert_circuit_roundtrip(src);
}

/// ACIR taken from `test_programs/execution_success/array_dynamic`
#[test]
fn array_dynamic() {
    let src = "
    current witness: w78
    private parameters: [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18]
    public parameters: []
    return values: []
    BLACKBOX::RANGE [w0]:32 bits []
    BLACKBOX::RANGE [w1]:32 bits []
    BLACKBOX::RANGE [w2]:32 bits []
    BLACKBOX::RANGE [w3]:32 bits []
    BLACKBOX::RANGE [w4]:32 bits []
    INIT (id: 0, len: 5, witnesses: [w0, w1, w2, w3, w4])
    BLACKBOX::RANGE [w5]:32 bits []
    BLACKBOX::RANGE [w6]:32 bits []
    INIT (id: 1, len: 5, witnesses: [w7, w8, w9, w10, w11])
    BLACKBOX::RANGE [w12]:32 bits []
    BLACKBOX::RANGE [w13]:32 bits []
    BLACKBOX::RANGE [w14]:32 bits []
    BLACKBOX::RANGE [w15]:32 bits []
    BLACKBOX::RANGE [w16]:32 bits []
    BLACKBOX::RANGE [w17]:32 bits []
    EXPR 5*w6 + -1*w19 + 0 = 0
    BLACKBOX::RANGE [w19]:32 bits []
    EXPR 1*w5 + -1*w19 + -1*w20 + 0 = 0
    BLACKBOX::RANGE [w20]:32 bits []
    EXPR 1*w20 + -1*w21 + -5 = 0
    EXPR 1*w21 + -1*w22 + -3 = 0
    MEM (id: 0, read at: EXPR [ (1, w21) 0 ], value: EXPR [ (1, w23) 0 ])
    EXPR 1*w23 + -111 = 0
    MEM (id: 0, read at: EXPR [ (1, w22) 0 ], value: EXPR [ (1, w24) 0 ])
    EXPR 1*w24 + -101 = 0
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w22) 4294967291 ], EXPR [ 4294967296 ]], outputs: [w25, w26]
    BLACKBOX::RANGE [w26]:32 bits []
    EXPR 1*w22 + -4294967296*w25 + -1*w26 + 4294967291 = 0
    EXPR -1*w25 + 0 = 0
    EXPR -1*w27 + 0 = 0
    MEM (id: 0, write EXPR [ (1, w27) 0 ] at: EXPR [ (1, w22) 0 ])
    MEM (id: 0, read at: EXPR [ (1, w21) 0 ], value: EXPR [ (1, w28) 0 ])
    EXPR 1*w28 + -111 = 0
    EXPR -1*w29 + 1 = 0
    MEM (id: 0, read at: EXPR [ (1, w29) 0 ], value: EXPR [ (1, w30) 0 ])
    EXPR 1*w30 + 0 = 0
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w21) 4294967286 ], EXPR [ 4294967296 ]], outputs: [w31, w32]
    BLACKBOX::RANGE [w31]:1 bits []
    BLACKBOX::RANGE [w32]:32 bits []
    EXPR 1*w21 + -4294967296*w31 + -1*w32 + 4294967286 = 0
    EXPR -1*w21*w31 + 1*w21 + -1*w33 + 0 = 0
    MEM (id: 0, read at: EXPR [ (1, w33) 0 ], value: EXPR [ (1, w34) 0 ])
    EXPR -1*w31*w34 + 2*w31 + 1*w34 + -1*w35 + -2 = 0
    BLACKBOX::RANGE [w35]:32 bits []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w21) 4294967291 ], EXPR [ 4294967296 ]], outputs: [w36, w37]
    BLACKBOX::RANGE [w36]:1 bits []
    BLACKBOX::RANGE [w37]:32 bits []
    EXPR 1*w21 + -4294967296*w36 + -1*w37 + 4294967291 = 0
    EXPR 1*w31*w36 + -1*w36 + 0 = 0
    EXPR -1*w21*w31 + 1*w21 + -1*w38 + 0 = 0
    MEM (id: 0, read at: EXPR [ (1, w38) 0 ], value: EXPR [ (1, w39) 0 ])
    MEM (id: 0, read at: EXPR [ (1, w27) 0 ], value: EXPR [ (1, w40) 0 ])
    MEM (id: 0, read at: EXPR [ (1, w29) 0 ], value: EXPR [ (1, w41) 0 ])
    EXPR -1*w42 + 2 = 0
    MEM (id: 0, read at: EXPR [ (1, w42) 0 ], value: EXPR [ (1, w43) 0 ])
    EXPR -1*w44 + 3 = 0
    MEM (id: 0, read at: EXPR [ (1, w44) 0 ], value: EXPR [ (1, w45) 0 ])
    EXPR -1*w46 + 4 = 0
    MEM (id: 0, read at: EXPR [ (1, w46) 0 ], value: EXPR [ (1, w47) 0 ])
    INIT (id: 3, len: 5, witnesses: [w40, w41, w43, w45, w47])
    EXPR -1*w31*w35 + 1*w31*w39 + 1*w35 + -1*w48 + 0 = 0
    MEM (id: 3, write EXPR [ (1, w48) 0 ] at: EXPR [ (1, w38) 0 ])
    MEM (id: 3, read at: EXPR [ (1, w46) 0 ], value: EXPR [ (1, w49) 0 ])
    MEM (id: 0, read at: EXPR [ (1, w46) 0 ], value: EXPR [ (1, w50) 0 ])
    EXPR -1*w31*w36 + 0 = 0
    EXPR 1*w21*w31 + -1*w51 + 0 = 0
    MEM (id: 0, read at: EXPR [ (1, w51) 0 ], value: EXPR [ (1, w52) 0 ])
    EXPR -1*w31*w52 + 1*w52 + -1*w53 + 0 = 0
    MEM (id: 0, write EXPR [ (1, w53) 0 ] at: EXPR [ (1, w51) 0 ])
    MEM (id: 0, read at: EXPR [ (1, w46) 0 ], value: EXPR [ (1, w54) 0 ])
    EXPR -1*w31 + -1*w55 + 1 = 0
    EXPR -1*w31*w49 + 1*w31*w50 + 1*w49 + -1*w56 + 0 = 0
    EXPR 1*w31*w54 + 1*w55*w56 + -109 = 0
    EXPR -1*w57 + 246 = 0
    EXPR -1*w58 + 159 = 0
    EXPR -1*w59 + 32 = 0
    EXPR -1*w60 + 176 = 0
    EXPR -1*w61 + 8 = 0
    INIT (id: 4, len: 5, witnesses: [w57, w58, w59, w60, w61])
    MEM (id: 4, read at: EXPR [ (1, w7) 0 ], value: EXPR [ (1, w62) 0 ])
    MEM (id: 4, read at: EXPR [ (1, w8) 0 ], value: EXPR [ (1, w63) 0 ])
    MEM (id: 4, read at: EXPR [ (1, w9) 0 ], value: EXPR [ (1, w64) 0 ])
    MEM (id: 4, read at: EXPR [ (1, w10) 0 ], value: EXPR [ (1, w65) 0 ])
    MEM (id: 4, read at: EXPR [ (1, w11) 0 ], value: EXPR [ (1, w66) 0 ])
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w62) (1, w63) (1, w64) (1, w65) (1, w66) 0 ]], outputs: [w67]
    EXPR 1*w62*w67 + 1*w63*w67 + 1*w64*w67 + 1*w65*w67 + 1*w66*w67 + -1 = 0
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w18) 0 ], EXPR [ 4294967296 ]], outputs: [w68, w69]
    BLACKBOX::RANGE [w68]:222 bits []
    BLACKBOX::RANGE [w69]:32 bits []
    EXPR 1*w18 + -4294967296*w68 + -1*w69 + 0 = 0
    EXPR -1*w68 + -1*w70 + 5096253676302562286669017222071363378443840053029366383258766538131 = 0
    BLACKBOX::RANGE [w70]:222 bits []
    BRILLIG CALL func 1: inputs: [EXPR [ (-1, w68) 5096253676302562286669017222071363378443840053029366383258766538131 ]], outputs: [w71]
    EXPR -1*w68*w71 + 5096253676302562286669017222071363378443840053029366383258766538131*w71 + 1*w72 + -1 = 0
    EXPR -1*w68*w72 + 5096253676302562286669017222071363378443840053029366383258766538131*w72 + 0 = 0
    EXPR 1*w69*w72 + 268435455*w72 + -1*w73 + 0 = 0
    BLACKBOX::RANGE [w73]:32 bits []
    BRILLIG CALL func 0: inputs: [EXPR [ (-1, w69) 4294967299 ], EXPR [ 4294967296 ]], outputs: [w74, w75]
    BLACKBOX::RANGE [w74]:1 bits []
    BLACKBOX::RANGE [w75]:32 bits []
    EXPR -1*w69 + -4294967296*w74 + -1*w75 + 4294967299 = 0
    EXPR -1*w17*w74 + 1*w17 + -3*w74 + -1*w76 + 3 = 0
    BLACKBOX::RANGE [w76]:32 bits []
    EXPR -1*w74*w76 + 1*w76 + -1*w77 + 0 = 0
    MEM (id: 1, read at: EXPR [ (1, w77) 0 ], value: EXPR [ (1, w78) 0 ])
    EXPR 1*w15*w74 + -1*w74*w78 + -1*w15 + 1*w78 + 0 = 0
    ";
    assert_circuit_roundtrip(src);
}

#[test]
fn fold_basic() {
    let src = "
    func 0
    current witness: w2
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    CALL func 1: PREDICATE: EXPR [ 1 ]
    inputs: [w0, w1], outputs: [w2]

    func 1
    current witness: w3
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) (-1, w1) 0 ]], outputs: [w3]
    EXPR 1*w0*w3 + -1*w1*w3 + -1 = 0
    EXPR -1*w0 + 1*w2 + 0 = 0
    ";
    assert_program_roundtrip(src);
}

#[test]
fn fold_basic_mismatched_ids() {
    let src = "
    func 0
    current witness: w2
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    CALL func 1: PREDICATE: EXPR [ 1 ]
    inputs: [w0, w1], outputs: [w2]

    func 2
    current witness: w3
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) (-1, w1) 0 ]], outputs: [w3]
    EXPR [ (1, w0, w3) (-1, w1, w3) -1 ]
    EXPR [ (-1, w0) (1, w2) 0 ]
    ";
    let result = Program::from_str(src).err().unwrap();
    let ParserError::UnexpectedFunctionId { expected, found, .. } = result.get_error() else {
        panic!("Expected `UnexpectedFunctionId` error");
    };
    assert_eq!(expected, 1);
    assert_eq!(found, 2);
}
