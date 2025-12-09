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

    let bytes =
        Program::serialize_program_with_format(&program, acir::SerializationFormat::BincodeLegacy);
    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 143, 49, 10, 128, 48, 12, 69, 127, 170, 7, 113, 212, 77, 241, 8, 34, 56, 137, 163, 139, 155, 7, 16, 55, 199, 30, 65, 188, 128, 167, 16, 61, 78, 55, 71, 23, 119, 29, 90, 140, 116, 105, 31, 132, 36, 240, 73, 254, 39, 252, 9, 223, 34, 216, 4, 186, 71, 112, 130, 200, 67, 43, 152, 54, 237, 235, 81, 101, 107, 178, 55, 229, 38, 101, 219, 197, 249, 89, 77, 199, 48, 23, 234, 94, 46, 237, 195, 241, 46, 132, 121, 192, 102, 179, 3, 95, 38, 206, 3, 2, 103, 244, 195, 16, 1, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);

    let bytes = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 144, 189, 74, 3, 65, 16, 199, 239, 46, 62, 136, 165, 118, 138, 79, 32, 34, 88, 137, 165, 8, 50, 108, 246, 70, 89, 184, 253, 112, 118, 55, 106, 121, 90, 216, 222, 37, 47, 16, 176, 144, 8, 65, 84, 252, 122, 135, 121, 4, 155, 116, 150, 54, 246, 174, 1, 197, 42, 201, 84, 195, 240, 159, 63, 252, 126, 197, 197, 232, 40, 26, 25, 148, 53, 190, 189, 26, 255, 238, 96, 132, 198, 225, 147, 140, 68, 104, 2, 156, 170, 96, 208, 123, 80, 166, 196, 179, 165, 27, 235, 164, 45, 209, 183, 245, 237, 166, 247, 72, 225, 0, 201, 94, 142, 116, 172, 32, 32, 105, 223, 60, 86, 202, 160, 32, 144, 86, 119, 149, 17, 211, 242, 65, 255, 61, 91, 206, 102, 79, 158, 47, 18, 42, 126, 66, 107, 229, 46, 241, 100, 157, 135, 171, 252, 186, 183, 205, 207, 92, 115, 189, 127, 184, 178, 193, 31, 59, 231, 252, 230, 184, 221, 226, 9, 127, 241, 128, 63, 211, 75, 231, 250, 4, 228, 252, 230, 236, 193, 145, 234, 137, 128, 224, 4, 37, 254, 4, 227, 251, 121, 113, 239, 98, 183, 82, 242, 223, 177, 25, 19, 134, 72, 6, 122, 162, 138, 201, 67, 231, 78, 76, 61, 128, 78, 146, 196, 49, 250, 230, 37, 121, 76, 208, 129, 68, 50, 81, 194, 159, 226, 230, 27, 239, 33, 11, 251, 112, 1, 0, 0]");

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

    let bytes =
        Program::serialize_program_with_format(&program, acir::SerializationFormat::BincodeLegacy);
    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 141, 11, 10, 0, 32, 12, 66, 87, 235, 127, 255, 3, 183, 224, 5, 214, 64, 84, 68, 151, 236, 189, 21, 72, 232, 195, 35, 224, 226, 47, 50, 236, 232, 155, 23, 184, 194, 45, 208, 217, 153, 120, 147, 13, 167, 83, 37, 51, 249, 169, 221, 255, 54, 129, 45, 40, 232, 188, 0, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);

    let default_bytes = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(default_bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 144, 75, 75, 3, 65, 16, 132, 201, 230, 225, 227, 103, 69, 240, 230, 201, 131, 199, 161, 157, 109, 165, 113, 210, 51, 116, 247, 196, 92, 87, 4, 175, 27, 5, 207, 158, 92, 114, 136, 47, 16, 255, 158, 67, 196, 33, 183, 143, 42, 170, 232, 174, 230, 110, 115, 149, 217, 27, 69, 214, 245, 195, 246, 159, 29, 195, 2, 95, 190, 125, 22, 65, 54, 119, 75, 198, 168, 234, 136, 91, 92, 29, 15, 49, 249, 216, 162, 174, 187, 143, 121, 0, 127, 51, 143, 171, 211, 146, 59, 129, 16, 186, 183, 179, 28, 140, 206, 61, 4, 144, 130, 247, 175, 41, 18, 155, 62, 117, 195, 197, 95, 199, 168, 82, 83, 105, 60, 232, 46, 160, 143, 85, 154, 84, 154, 110, 146, 96, 75, 30, 12, 171, 54, 27, 98, 182, 148, 75, 239, 193, 225, 209, 87, 18, 90, 22, 215, 37, 144, 114, 181, 161, 232, 243, 168, 25, 79, 166, 179, 207, 148, 47, 3, 249, 61, 163, 223, 10, 90, 22, 118, 75, 8, 25, 119, 241, 119, 80, 69, 49, 183, 40, 189, 112, 141, 218, 255, 148, 95, 202, 26, 38, 64, 140, 173, 171, 243, 244, 191, 238, 86, 173, 160, 44, 1, 0, 0]");

    let program_de = Program::deserialize_program(&default_bytes).unwrap();
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

    let bytes =
        Program::serialize_program_with_format(&program, acir::SerializationFormat::BincodeLegacy);
    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 81, 237, 10, 128, 32, 12, 116, 246, 193, 160, 127, 61, 65, 111, 22, 17, 253, 8, 164, 31, 17, 61, 127, 69, 91, 204, 156, 48, 7, 58, 61, 239, 240, 142, 129, 139, 11, 239, 5, 116, 174, 169, 131, 75, 139, 177, 193, 153, 10, 192, 206, 141, 254, 243, 223, 70, 15, 222, 32, 236, 168, 175, 219, 185, 236, 199, 56, 79, 33, 52, 4, 225, 143, 250, 244, 170, 192, 27, 74, 95, 229, 122, 104, 21, 80, 70, 146, 17, 152, 251, 198, 208, 166, 32, 21, 185, 123, 14, 239, 21, 156, 157, 92, 163, 94, 232, 115, 22, 2, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);

    let default_bytes = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(default_bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 146, 207, 78, 27, 49, 16, 198, 119, 247, 212, 199, 232, 51, 180, 79, 208, 166, 66, 234, 161, 39, 212, 179, 229, 216, 147, 213, 72, 94, 219, 29, 207, 166, 77, 111, 155, 180, 234, 53, 32, 245, 94, 21, 242, 79, 155, 128, 0, 33, 174, 60, 8, 15, 131, 55, 144, 16, 34, 17, 144, 240, 201, 26, 123, 230, 155, 239, 55, 147, 245, 103, 157, 210, 42, 70, 103, 195, 193, 159, 197, 234, 46, 172, 44, 224, 223, 165, 42, 137, 192, 178, 248, 142, 108, 33, 4, 129, 86, 195, 143, 55, 19, 231, 149, 211, 16, 14, 170, 250, 35, 161, 49, 152, 183, 164, 49, 191, 254, 163, 78, 198, 104, 125, 201, 241, 101, 188, 143, 54, 55, 48, 152, 21, 165, 17, 12, 84, 132, 225, 133, 65, 11, 146, 132, 114, 69, 27, 173, 188, 147, 60, 188, 73, 222, 38, 187, 79, 154, 30, 125, 19, 234, 249, 127, 201, 196, 149, 188, 86, 47, 188, 129, 108, 230, 9, 52, 42, 201, 112, 125, 238, 9, 187, 241, 34, 188, 164, 232, 45, 182, 20, 14, 211, 236, 204, 151, 109, 131, 106, 35, 56, 92, 16, 112, 73, 86, 116, 165, 41, 33, 12, 79, 101, 8, 64, 44, 138, 232, 95, 230, 49, 112, 21, 17, 197, 206, 153, 100, 180, 163, 197, 3, 189, 254, 99, 122, 53, 218, 110, 147, 168, 34, 155, 105, 187, 199, 208, 48, 251, 91, 141, 90, 77, 242, 160, 142, 252, 248, 158, 66, 53, 254, 132, 4, 138, 211, 105, 27, 89, 4, 252, 9, 213, 228, 179, 101, 200, 129, 142, 190, 190, 127, 55, 90, 118, 242, 2, 255, 187, 106, 103, 175, 171, 157, 86, 243, 102, 198, 90, 178, 108, 57, 223, 27, 92, 108, 72, 8, 169, 53, 69, 58, 43, 169, 100, 222, 200, 108, 71, 179, 19, 215, 233, 4, 224, 237, 120, 90, 213, 123, 142, 0, 115, 219, 8, 252, 158, 174, 24, 142, 239, 240, 205, 55, 132, 226, 96, 23, 95, 160, 112, 212, 251, 176, 165, 120, 181, 217, 206, 210, 146, 224, 158, 135, 135, 77, 24, 237, 33, 24, 189, 222, 206, 39, 202, 156, 45, 223, 119, 20, 168, 142, 247, 217, 249, 170, 190, 95, 145, 6, 71, 127, 226, 29, 70, 160, 180, 54, 116, 188, 164, 188, 178, 125, 11, 131, 35, 244, 228, 97, 3, 0, 0]");

    let program_de = Program::deserialize_program(&default_bytes).unwrap();
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

    let bytes =
        Program::serialize_program_with_format(&program, acir::SerializationFormat::BincodeLegacy);
    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 84, 219, 10, 194, 48, 12, 109, 154, 109, 22, 244, 201, 47, 24, 232, 127, 137, 12, 223, 42, 250, 232, 231, 187, 66, 50, 178, 88, 181, 233, 182, 64, 73, 27, 206, 201, 101, 39, 12, 220, 220, 194, 120, 128, 238, 13, 121, 79, 62, 197, 81, 225, 25, 219, 187, 34, 3, 40, 199, 86, 215, 240, 110, 251, 26, 232, 236, 53, 146, 161, 177, 142, 225, 123, 89, 230, 54, 245, 207, 61, 75, 253, 211, 110, 180, 227, 233, 232, 189, 35, 31, 52, 193, 187, 207, 165, 153, 117, 66, 254, 64, 126, 120, 220, 159, 241, 246, 186, 12, 215, 24, 247, 50, 169, 226, 24, 6, 192, 160, 106, 25, 249, 211, 144, 223, 240, 156, 119, 97, 159, 61, 243, 177, 142, 15, 204, 111, 234, 248, 216, 9, 222, 20, 20, 119, 206, 155, 116, 97, 193, 73, 47, 204, 80, 53, 61, 217, 73, 189, 207, 10, 7, 5, 57, 216, 228, 127, 233, 23, 30, 50, 248, 127, 156, 181, 164, 172, 92, 185, 246, 152, 9, 114, 174, 55, 111, 172, 240, 81, 180, 5, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);

    let default_bytes = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(default_bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 85, 203, 110, 211, 64, 20, 181, 227, 20, 250, 25, 145, 224, 11, 224, 11, 74, 80, 5, 11, 86, 21, 235, 209, 196, 190, 137, 70, 26, 207, 12, 119, 198, 165, 97, 55, 9, 136, 109, 30, 75, 54, 136, 54, 47, 57, 1, 149, 10, 117, 203, 135, 240, 49, 140, 67, 28, 156, 138, 60, 212, 20, 240, 234, 250, 200, 190, 231, 220, 51, 231, 218, 165, 214, 164, 158, 136, 208, 48, 41, 116, 247, 253, 44, 175, 137, 160, 49, 124, 252, 22, 38, 136, 32, 12, 121, 205, 140, 0, 173, 9, 19, 17, 156, 29, 142, 164, 10, 101, 4, 186, 107, 211, 39, 200, 56, 103, 141, 42, 229, 252, 237, 39, 22, 121, 67, 38, 84, 98, 116, 207, 14, 142, 16, 105, 179, 223, 158, 196, 9, 39, 6, 48, 214, 157, 43, 206, 4, 80, 36, 161, 140, 107, 76, 208, 95, 148, 189, 31, 94, 197, 219, 124, 249, 254, 249, 43, 18, 110, 127, 206, 187, 19, 178, 210, 191, 36, 11, 118, 36, 179, 195, 19, 38, 26, 28, 182, 145, 246, 119, 178, 115, 39, 27, 238, 82, 254, 72, 38, 38, 139, 69, 63, 143, 69, 249, 224, 94, 54, 82, 172, 56, 220, 207, 139, 195, 137, 66, 136, 88, 72, 13, 124, 255, 170, 144, 157, 186, 130, 40, 138, 46, 136, 110, 96, 221, 247, 75, 193, 165, 74, 106, 156, 133, 5, 180, 51, 67, 48, 9, 10, 114, 74, 121, 2, 186, 243, 133, 106, 13, 104, 72, 236, 210, 74, 27, 14, 184, 118, 129, 118, 198, 24, 164, 206, 173, 136, 252, 206, 122, 107, 53, 235, 83, 103, 163, 19, 113, 70, 66, 23, 229, 113, 173, 105, 32, 139, 248, 7, 59, 168, 102, 111, 183, 83, 23, 119, 179, 112, 217, 14, 159, 50, 132, 208, 120, 227, 26, 51, 68, 179, 55, 96, 71, 207, 133, 129, 6, 224, 249, 203, 199, 143, 6, 115, 41, 219, 93, 9, 54, 245, 246, 247, 235, 237, 217, 105, 182, 146, 17, 53, 180, 42, 85, 179, 125, 85, 160, 32, 52, 138, 208, 217, 147, 83, 85, 166, 25, 205, 77, 212, 251, 44, 235, 117, 13, 230, 38, 238, 255, 69, 71, 42, 155, 122, 7, 251, 245, 222, 168, 187, 188, 239, 73, 238, 238, 182, 255, 71, 183, 131, 53, 110, 151, 109, 122, 44, 17, 88, 67, 100, 4, 239, 198, 121, 100, 71, 139, 180, 78, 11, 76, 110, 187, 38, 207, 128, 170, 249, 134, 181, 70, 74, 50, 55, 9, 46, 143, 230, 34, 227, 13, 236, 236, 5, 196, 18, 155, 71, 171, 60, 15, 214, 224, 15, 175, 139, 163, 204, 237, 32, 166, 169, 96, 185, 201, 173, 180, 0, 118, 243, 85, 30, 28, 51, 224, 209, 130, 113, 5, 91, 189, 91, 254, 45, 110, 45, 220, 191, 156, 183, 40, 74, 235, 221, 86, 218, 127, 10, 246, 158, 171, 126, 96, 47, 78, 140, 84, 54, 93, 124, 8, 179, 20, 174, 49, 113, 201, 248, 19, 97, 10, 106, 42, 245, 7, 0, 0]");

    let program_de = Program::deserialize_program(&default_bytes).unwrap();
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

    let bytes =
        Program::serialize_program_with_format(&program, acir::SerializationFormat::BincodeLegacy);
    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 81, 65, 10, 0, 32, 8, 115, 106, 255, 232, 255, 175, 172, 131, 70, 129, 7, 211, 129, 108, 135, 13, 28, 3, 189, 24, 251, 196, 180, 51, 27, 227, 210, 76, 49, 38, 165, 128, 110, 14, 159, 57, 201, 123, 187, 221, 170, 185, 114, 55, 205, 123, 207, 166, 190, 165, 4, 15, 104, 144, 91, 71, 10, 197, 194, 40, 2, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);

    let default_bytes = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(default_bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 205, 83, 75, 78, 195, 48, 16, 77, 210, 114, 15, 174, 195, 2, 113, 132, 145, 227, 12, 200, 34, 254, 48, 99, 23, 186, 108, 179, 96, 155, 180, 23, 64, 124, 155, 74, 21, 2, 132, 184, 12, 135, 193, 36, 226, 179, 130, 44, 16, 170, 55, 158, 159, 222, 123, 51, 154, 201, 230, 237, 97, 48, 210, 43, 107, 184, 57, 223, 124, 216, 96, 132, 198, 139, 103, 25, 136, 208, 120, 56, 85, 222, 32, 51, 40, 83, 224, 217, 206, 157, 117, 210, 22, 200, 203, 217, 122, 31, 181, 165, 233, 158, 81, 190, 90, 229, 165, 149, 199, 160, 138, 228, 90, 69, 127, 145, 102, 235, 62, 226, 167, 14, 111, 251, 194, 217, 170, 255, 15, 220, 252, 171, 252, 210, 186, 170, 181, 14, 73, 188, 51, 87, 173, 14, 37, 120, 36, 205, 245, 83, 169, 12, 10, 2, 105, 117, 174, 76, 151, 230, 250, 234, 4, 228, 107, 178, 155, 252, 252, 210, 155, 78, 235, 159, 161, 77, 68, 25, 240, 55, 180, 102, 49, 0, 106, 52, 140, 50, 249, 191, 89, 37, 91, 59, 171, 241, 192, 6, 30, 29, 169, 137, 240, 8, 78, 80, 92, 220, 200, 202, 203, 52, 27, 61, 184, 144, 151, 74, 126, 139, 214, 27, 66, 31, 200, 64, 167, 145, 155, 241, 189, 96, 70, 242, 160, 227, 122, 139, 35, 228, 250, 37, 94, 64, 148, 231, 73, 68, 205, 5, 124, 30, 71, 253, 6, 10, 180, 163, 42, 42, 3, 0, 0]");

    let program_de = Program::deserialize_program(&default_bytes).unwrap();
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

    let bytes =
        Program::serialize_program_with_format(&program, acir::SerializationFormat::BincodeLegacy);
    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 81, 59, 10, 131, 64, 16, 157, 15, 222, 35, 101, 210, 37, 228, 8, 33, 144, 42, 88, 218, 216, 121, 0, 177, 179, 244, 8, 226, 5, 60, 133, 232, 113, 236, 44, 109, 236, 85, 88, 101, 92, 23, 119, 45, 124, 240, 96, 216, 125, 204, 188, 55, 195, 176, 5, 43, 206, 240, 38, 226, 68, 18, 255, 168, 8, 203, 187, 77, 196, 218, 128, 85, 120, 3, 39, 32, 9, 237, 51, 250, 39, 237, 171, 124, 212, 254, 183, 202, 178, 32, 188, 191, 187, 95, 218, 196, 249, 167, 29, 138, 94, 13, 115, 236, 187, 26, 148, 53, 30, 232, 25, 182, 33, 23, 156, 205, 35, 181, 182, 60, 228, 222, 151, 60, 165, 39, 225, 107, 119, 8, 253, 74, 122, 205, 96, 118, 108, 90, 204, 149, 193, 209, 189, 175, 53, 147, 9, 35, 191, 119, 205, 214, 247, 2, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);

    let default_bytes = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(default_bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 205, 212, 205, 74, 3, 49, 16, 0, 224, 108, 246, 69, 60, 234, 77, 241, 9, 164, 8, 158, 196, 163, 8, 18, 210, 108, 148, 192, 110, 54, 230, 167, 234, 177, 42, 120, 221, 221, 190, 64, 81, 177, 84, 40, 162, 226, 223, 205, 7, 152, 71, 240, 210, 155, 71, 47, 222, 77, 11, 171, 149, 150, 218, 22, 15, 230, 20, 194, 36, 204, 124, 51, 4, 31, 181, 119, 156, 100, 86, 164, 210, 52, 78, 59, 229, 158, 72, 154, 240, 230, 61, 115, 90, 115, 105, 201, 190, 176, 146, 27, 67, 132, 140, 248, 65, 216, 74, 21, 75, 35, 110, 26, 245, 139, 10, 141, 227, 147, 51, 17, 5, 151, 66, 42, 103, 77, 129, 130, 86, 234, 108, 111, 155, 227, 182, 210, 60, 18, 140, 90, 254, 50, 62, 50, 28, 140, 188, 90, 49, 134, 107, 187, 197, 117, 122, 220, 78, 92, 76, 44, 215, 137, 201, 238, 98, 33, 57, 213, 132, 165, 73, 85, 72, 218, 79, 184, 40, 94, 209, 28, 26, 191, 2, 220, 11, 90, 140, 214, 53, 116, 151, 160, 185, 0, 79, 27, 171, 240, 0, 117, 168, 111, 110, 207, 47, 195, 219, 218, 33, 60, 43, 200, 43, 208, 133, 15, 104, 192, 187, 191, 18, 158, 239, 17, 246, 251, 203, 232, 86, 105, 81, 243, 57, 19, 69, 181, 231, 242, 121, 154, 28, 221, 40, 87, 141, 5, 27, 60, 11, 58, 154, 91, 167, 37, 169, 209, 216, 113, 147, 93, 211, 126, 133, 36, 241, 164, 116, 215, 31, 76, 233, 94, 252, 185, 17, 154, 218, 8, 79, 102, 132, 191, 27, 143, 203, 198, 227, 209, 141, 31, 161, 233, 103, 100, 152, 51, 251, 169, 153, 135, 51, 114, 6, 37, 103, 254, 15, 56, 131, 217, 71, 110, 34, 36, 52, 132, 244, 232, 137, 124, 61, 86, 83, 95, 100, 68, 190, 62, 129, 236, 19, 155, 126, 197, 103, 18, 4, 0, 0]");

    let program_de = Program::deserialize_program(&default_bytes).unwrap();
    assert_eq!(program_de, program);
}
