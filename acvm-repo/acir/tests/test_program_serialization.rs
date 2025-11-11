//! This integration test defines a set of circuits which are used in order to test the acvm_js package.
//!
//! The acvm_js test suite contains serialized program [circuits][`Program`] which must be kept in sync with the format
//! outputted from the [ACIR crate][acir].
//! Breaking changes to the serialization format then require refreshing acvm_js's test suite.
//! This file contains Rust definitions of these circuits and outputs the updated serialized format.
//!
//! These tests also check this circuit serialization against an expected value, erroring if the serialization changes.
//! Generally in this situation we just need to refresh the `expected_serialization` variables to match the
//! actual output, **HOWEVER** note that this results in a breaking change to the backend ACIR format.

use acir::{
    circuit::{Circuit, Program, brillig::BrilligBytecode},
    native_types::Witness,
};
use acir_field::FieldElement;
use brillig::{
    BitSize, HeapArray, HeapValueType, HeapVector, IntegerBitSize, MemoryAddress, ValueOrArray,
};

#[test]
fn addition_circuit() {
    let src = "
    private parameters: [w1, w2]
    public parameters: []
    return values: [w3]
    ASSERT 0 = w1 + w2 - w3
    ";
    let mut circuit = Circuit::from_str(src).unwrap();
    circuit.current_witness_index = 4;

    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 143, 49, 10, 128, 48, 12, 69, 127, 170, 7, 113, 212, 77, 241, 8, 34, 56, 137, 163, 139, 155, 7, 16, 55, 199, 30, 65, 188, 128, 167, 16, 61, 78, 55, 71, 23, 119, 29, 90, 140, 116, 105, 31, 132, 36, 240, 73, 254, 39, 252, 9, 223, 34, 216, 4, 186, 71, 112, 130, 200, 67, 43, 152, 54, 237, 235, 81, 101, 107, 178, 55, 229, 38, 101, 219, 197, 249, 89, 77, 199, 48, 23, 234, 94, 46, 237, 195, 241, 46, 132, 121, 192, 102, 179, 3, 95, 38, 206, 3, 2, 103, 244, 195, 16, 1, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}

#[test]
fn multi_scalar_mul_circuit() {
    let src = "
    private parameters: [w1, w2, w3, w4, w5, w6]
    public parameters: []
    return values: [w7, w8, w9]
    BLACKBOX::MULTI_SCALAR_MUL points: [w1, w2, w3], scalars: [w4, w5], predicate: w6, outputs: [w7, w8, w9]
    ";
    let mut circuit = Circuit::from_str(src).unwrap();
    circuit.current_witness_index = 10;

    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 141, 11, 10, 0, 32, 12, 66, 87, 235, 127, 255, 3, 183, 224, 5, 214, 64, 84, 68, 151, 236, 189, 21, 72, 232, 195, 35, 224, 226, 47, 50, 236, 232, 155, 23, 184, 194, 45, 208, 217, 153, 120, 147, 13, 167, 83, 37, 51, 249, 169, 221, 255, 54, 129, 45, 40, 232, 188, 0, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}

#[test]
fn simple_brillig_foreign_call() {
    let w_input = Witness(1);
    let w_inverted = Witness(2);

    let value_address = MemoryAddress::direct(0);
    let zero_usize = MemoryAddress::direct(1);
    let one_usize = MemoryAddress::direct(2);

    let brillig_bytecode = BrilligBytecode {
        function_name: "invert_call".into(),
        bytecode: vec![
            brillig::Opcode::Const {
                destination: zero_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0_usize),
            },
            brillig::Opcode::Const {
                destination: one_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(1_usize),
            },
            brillig::Opcode::CalldataCopy {
                destination_address: value_address,
                size_address: one_usize,
                offset_address: zero_usize,
            },
            brillig::Opcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(value_address)],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(value_address)],
                input_value_types: vec![HeapValueType::field()],
            },
            brillig::Opcode::Stop {
                return_data: HeapVector { pointer: zero_usize, size: one_usize },
            },
        ],
    };

    let src = format!(
        "
    private parameters: [{w_input}, {w_inverted}]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [{w_input}], outputs: [{w_inverted}]
    "
    );
    let mut circuit = Circuit::from_str(&src).unwrap();
    circuit.current_witness_index = 8;

    let program =
        Program { functions: vec![circuit], unconstrained_functions: vec![brillig_bytecode] };

    let bytes = Program::serialize_program(&program);

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 81, 237, 10, 128, 32, 12, 116, 246, 193, 160, 127, 61, 65, 111, 22, 17, 253, 8, 164, 31, 17, 61, 127, 69, 91, 204, 156, 48, 7, 58, 61, 239, 240, 142, 129, 139, 11, 239, 5, 116, 174, 169, 131, 75, 139, 177, 193, 153, 10, 192, 206, 141, 254, 243, 223, 70, 15, 222, 32, 236, 168, 175, 219, 185, 236, 199, 56, 79, 33, 52, 4, 225, 143, 250, 244, 170, 192, 27, 74, 95, 229, 122, 104, 21, 80, 70, 146, 17, 152, 251, 198, 208, 166, 32, 21, 185, 123, 14, 239, 21, 156, 157, 92, 163, 94, 232, 115, 22, 2, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}

#[test]
fn complex_brillig_foreign_call() {
    let a = Witness(1);
    let b = Witness(2);
    let c = Witness(3);

    let a_times_2 = Witness(4);
    let b_times_3 = Witness(5);
    let c_times_4 = Witness(6);
    let a_plus_b_plus_c = Witness(7);
    let a_plus_b_plus_c_times_2 = Witness(8);

    let brillig_bytecode = BrilligBytecode {
        function_name: "complex_call".into(),
        bytecode: vec![
            brillig::Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(3_usize),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0_usize),
            },
            brillig::Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(32),
                size_address: MemoryAddress::direct(0),
                offset_address: MemoryAddress::direct(1),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress::direct(0),
                value: FieldElement::from(32_usize),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress::direct(3),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(1_usize),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress::direct(4),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(3_usize),
            },
            brillig::Opcode::CalldataCopy {
                destination_address: MemoryAddress::direct(1),
                size_address: MemoryAddress::direct(3),
                offset_address: MemoryAddress::direct(4),
            },
            // Oracles are named 'foreign calls' in brillig
            brillig::Opcode::ForeignCall {
                function: "complex".into(),
                inputs: vec![
                    ValueOrArray::HeapArray(HeapArray {
                        pointer: MemoryAddress::direct(0),
                        size: 3,
                    }),
                    ValueOrArray::MemoryAddress(MemoryAddress::direct(1)),
                ],
                input_value_types: vec![
                    HeapValueType::Array { size: 3, value_types: vec![HeapValueType::field()] },
                    HeapValueType::field(),
                ],
                destinations: vec![
                    ValueOrArray::HeapArray(HeapArray {
                        pointer: MemoryAddress::direct(0),
                        size: 3,
                    }),
                    ValueOrArray::MemoryAddress(MemoryAddress::direct(35)),
                    ValueOrArray::MemoryAddress(MemoryAddress::direct(36)),
                ],
                destination_value_types: vec![
                    HeapValueType::Array { size: 3, value_types: vec![HeapValueType::field()] },
                    HeapValueType::field(),
                    HeapValueType::field(),
                ],
            },
            brillig::Opcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(32_usize),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(5_usize),
            },
            brillig::Opcode::Stop {
                return_data: HeapVector {
                    pointer: MemoryAddress::direct(0),
                    size: MemoryAddress::direct(1),
                },
            },
        ],
    };

    let src = format!("
    private parameters: [{a}, {b}, {c}]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, inputs: [[{a}, {b}, {c}], {a} + {b} + {c}], outputs: [[{a_times_2}, {b_times_3}, {c_times_4}], {a_plus_b_plus_c}, {a_plus_b_plus_c_times_2}]
    ");
    let circuit = Circuit::from_str(&src).unwrap();
    let program =
        Program { functions: vec![circuit], unconstrained_functions: vec![brillig_bytecode] };

    let bytes = Program::serialize_program(&program);

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 84, 219, 10, 194, 48, 12, 109, 154, 109, 22, 244, 201, 47, 24, 232, 127, 137, 12, 223, 42, 250, 232, 231, 187, 66, 50, 178, 88, 181, 233, 182, 64, 73, 27, 206, 201, 101, 39, 12, 220, 220, 194, 120, 128, 238, 13, 121, 79, 62, 197, 81, 225, 25, 219, 187, 34, 3, 40, 199, 86, 215, 240, 110, 251, 26, 232, 236, 53, 146, 161, 177, 142, 225, 123, 89, 230, 54, 245, 207, 61, 75, 253, 211, 110, 180, 227, 233, 232, 189, 35, 31, 52, 193, 187, 207, 165, 153, 117, 66, 254, 64, 126, 120, 220, 159, 241, 246, 186, 12, 215, 24, 247, 50, 169, 226, 24, 6, 192, 160, 106, 25, 249, 211, 144, 223, 240, 156, 119, 97, 159, 61, 243, 177, 142, 15, 204, 111, 234, 248, 216, 9, 222, 20, 20, 119, 206, 155, 116, 97, 193, 73, 47, 204, 80, 53, 61, 217, 73, 189, 207, 10, 7, 5, 57, 216, 228, 127, 233, 23, 30, 50, 248, 127, 156, 181, 164, 172, 92, 185, 246, 152, 9, 114, 174, 55, 111, 172, 240, 81, 180, 5, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}

#[test]
fn memory_op_circuit() {
    let src = "
    private parameters: [w1, w2, w3]
    public parameters: []
    return values: [w4]
    INIT b0 = [w1, w2]
    WRITE b0[1] = w3
    READ w4 = b0[1]
    ";
    let mut circuit = Circuit::from_str(src).unwrap();
    circuit.current_witness_index = 5;

    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 81, 65, 10, 0, 32, 8, 115, 106, 255, 232, 255, 175, 172, 131, 70, 129, 7, 211, 129, 108, 135, 13, 28, 3, 189, 24, 251, 196, 180, 51, 27, 227, 210, 76, 49, 38, 165, 128, 110, 14, 159, 57, 201, 123, 187, 221, 170, 185, 114, 55, 205, 123, 207, 166, 190, 165, 4, 15, 104, 144, 91, 71, 10, 197, 194, 40, 2, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}

#[test]
fn nested_acir_call_circuit() {
    // Circuit for the following program:
    // fn main(x: Field, y: pub Field) {
    //     let z = nested_call(x, y);
    //     let z2 = nested_call(x, y);
    //     assert(z == z2);
    // }
    // #[fold]
    // fn nested_call(x: Field, y: Field) -> Field {
    //     inner_call(x + 2, y)
    // }
    // #[fold]
    // fn inner_call(x: Field, y: Field) -> Field {
    //     assert(x == y);
    //     x
    // }
    let src = "
    private parameters: [w0]
    public parameters: [w1]
    return values: []
    CALL func: 1, inputs: [w0, w1], outputs: [w2]
    CALL func: 1, inputs: [w0, w1], outputs: [w3]
    ASSERT 0 = w2 - w3
    ";
    let main = Circuit::from_str(src).unwrap();

    let src = "
    private parameters: [w0, w1]
    public parameters: []
    return values: [w3]
    ASSERT 0 = w0 - w2 + 2
    CALL func: 2, inputs: [w2, w1], outputs: [w3]
    ";
    let nested_call = Circuit::from_str(src).unwrap();

    let src = "
    private parameters: [w0, w1]
    public parameters: []
    return values: [w0]
    ASSERT 0 = w0 - w1
    ";
    let inner_call = Circuit::from_str(src).unwrap();

    let program =
        Program { functions: vec![main, nested_call, inner_call], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 81, 59, 10, 131, 64, 16, 157, 15, 222, 35, 101, 210, 37, 228, 8, 33, 144, 42, 88, 218, 216, 121, 0, 177, 179, 244, 8, 226, 5, 60, 133, 232, 113, 236, 44, 109, 236, 85, 88, 101, 92, 23, 119, 45, 124, 240, 96, 216, 125, 204, 188, 55, 195, 176, 5, 43, 206, 240, 38, 226, 68, 18, 255, 168, 8, 203, 187, 77, 196, 218, 128, 85, 120, 3, 39, 32, 9, 237, 51, 250, 39, 237, 171, 124, 212, 254, 183, 202, 178, 32, 188, 191, 187, 95, 218, 196, 249, 167, 29, 138, 94, 13, 115, 236, 187, 26, 148, 53, 30, 232, 25, 182, 33, 23, 156, 205, 35, 181, 182, 60, 228, 222, 151, 60, 165, 39, 225, 107, 119, 8, 253, 74, 122, 205, 96, 118, 108, 90, 204, 149, 193, 209, 189, 175, 53, 147, 9, 35, 191, 119, 205, 214, 247, 2, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}
