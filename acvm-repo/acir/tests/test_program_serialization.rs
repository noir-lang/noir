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
    SerializationFormat,
    circuit::{Circuit, Program, brillig::BrilligBytecode},
    native_types::Witness,
};
use acir_field::FieldElement;
use brillig::{
    BitSize, HeapArray, HeapValueType, HeapVector, IntegerBitSize, MemoryAddress, ValueOrArray,
    lengths::{SemanticLength, SemiFlattenedLength},
};

fn assert_deserialization(expected: &Program<FieldElement>, bytes: [Vec<u8>; 2]) {
    for (i, bytes) in bytes.iter().enumerate() {
        let program = Program::deserialize_program(bytes)
            .map_err(|e| format!("failed to deserialize format {i}: {e:?}"))
            .unwrap();
        assert_eq!(&program, expected, "incorrect deserialized program for format {i}");
    }
}

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

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 144, 187, 74, 3, 81, 16, 134, 179, 27, 31, 196, 82, 59, 197, 39, 16, 17, 172, 196, 82, 4, 25, 78, 206, 142, 114, 96, 207, 197, 153, 115, 162, 150, 171, 133, 237, 110, 242, 2, 1, 11, 137, 16, 68, 197, 91, 239, 139, 164, 179, 180, 177, 119, 8, 36, 164, 74, 50, 213, 48, 252, 252, 204, 247, 229, 55, 195, 179, 228, 116, 52, 222, 113, 115, 55, 154, 238, 224, 148, 197, 193, 155, 78, 68, 232, 34, 92, 154, 232, 144, 25, 140, 43, 240, 106, 237, 193, 7, 237, 11, 228, 166, 122, 220, 101, 70, 138, 39, 72, 254, 118, 104, 83, 9, 17, 201, 114, 253, 90, 26, 135, 138, 64, 123, 219, 49, 78, 77, 202, 251, 189, 239, 245, 214, 226, 201, 178, 21, 50, 185, 100, 182, 138, 67, 26, 111, 15, 54, 63, 143, 246, 223, 171, 234, 248, 116, 99, 231, 231, 224, 250, 43, 52, 123, 227, 191, 254, 175, 132, 218, 247, 23, 160, 151, 86, 181, 94, 2, 153, 174, 138, 8, 65, 145, 224, 202, 239, 220, 203, 242, 231, 144, 58, 165, 209, 115, 199, 122, 68, 24, 19, 57, 232, 170, 50, 9, 118, 251, 73, 77, 176, 193, 138, 19, 117, 142, 92, 127, 136, 54, 97, 140, 164, 4, 188, 128, 153, 209, 250, 31, 140, 53, 217, 21, 95, 1, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 204, 59, 14, 64, 48, 0, 128, 225, 62, 28, 196, 200, 70, 156, 64, 68, 98, 18, 163, 72, 108, 58, 147, 214, 98, 236, 13, 250, 24, 172, 157, 29, 64, 216, 93, 164, 155, 209, 98, 215, 19, 224, 159, 191, 252, 88, 201, 217, 120, 146, 47, 41, 99, 132, 142, 13, 161, 189, 22, 90, 29, 62, 120, 15, 194, 31, 6, 57, 19, 117, 37, 181, 177, 9, 183, 42, 95, 57, 175, 219, 32, 57, 139, 105, 31, 100, 102, 111, 125, 57, 132, 63, 55, 64, 65, 36, 36, 22, 226, 1, 27, 166, 206, 10, 172, 0, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
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

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 144, 75, 75, 3, 65, 16, 132, 201, 230, 225, 227, 103, 69, 240, 230, 201, 131, 199, 161, 157, 109, 165, 113, 210, 51, 116, 247, 196, 92, 87, 4, 175, 27, 5, 207, 158, 92, 114, 136, 47, 16, 255, 158, 67, 196, 33, 183, 143, 42, 170, 232, 174, 230, 110, 115, 149, 217, 27, 69, 214, 245, 195, 246, 159, 29, 195, 2, 95, 190, 125, 22, 65, 54, 119, 75, 198, 168, 234, 136, 91, 92, 29, 15, 49, 249, 216, 162, 174, 187, 143, 121, 0, 127, 51, 143, 171, 211, 146, 59, 129, 16, 186, 183, 179, 28, 140, 206, 61, 4, 144, 130, 247, 175, 41, 18, 155, 62, 117, 195, 197, 95, 199, 168, 82, 83, 105, 60, 232, 46, 160, 143, 85, 154, 84, 154, 110, 146, 96, 75, 30, 12, 171, 54, 27, 98, 182, 148, 75, 239, 193, 225, 209, 87, 18, 90, 22, 215, 37, 144, 114, 181, 161, 232, 243, 168, 25, 79, 166, 179, 207, 148, 47, 3, 249, 61, 163, 223, 10, 90, 22, 118, 75, 8, 25, 119, 241, 119, 80, 69, 49, 183, 40, 189, 112, 141, 218, 255, 148, 95, 202, 26, 38, 64, 140, 173, 171, 243, 244, 191, 238, 86, 173, 160, 44, 1, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 61, 198, 171, 10, 128, 48, 20, 0, 80, 116, 190, 63, 75, 193, 102, 50, 152, 199, 48, 12, 47, 19, 220, 6, 214, 253, 193, 30, 162, 213, 102, 19, 63, 81, 45, 183, 29, 226, 221, 113, 86, 206, 60, 53, 80, 54, 213, 243, 218, 106, 193, 26, 10, 96, 238, 78, 131, 226, 61, 163, 64, 151, 143, 91, 48, 215, 192, 149, 24, 165, 140, 80, 49, 138, 120, 100, 130, 74, 81, 89, 200, 139, 114, 143, 98, 146, 164, 153, 253, 109, 237, 11, 181, 107, 246, 16, 122, 0, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
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

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 146, 207, 78, 27, 49, 16, 198, 179, 123, 234, 99, 244, 25, 218, 39, 104, 83, 33, 245, 208, 19, 234, 217, 114, 236, 201, 106, 36, 175, 237, 142, 103, 211, 166, 183, 77, 64, 92, 3, 18, 119, 4, 228, 159, 54, 1, 1, 66, 92, 57, 240, 96, 120, 3, 9, 33, 18, 225, 128, 79, 214, 216, 51, 223, 124, 191, 153, 180, 55, 109, 23, 86, 49, 58, 27, 14, 15, 230, 203, 187, 176, 50, 135, 147, 91, 85, 16, 129, 101, 241, 23, 217, 66, 8, 2, 173, 134, 127, 159, 198, 206, 43, 167, 33, 28, 150, 213, 119, 66, 99, 48, 107, 74, 99, 246, 78, 81, 55, 70, 104, 125, 193, 241, 101, 180, 139, 54, 51, 208, 159, 230, 133, 17, 12, 148, 135, 193, 141, 65, 11, 146, 132, 114, 121, 11, 173, 124, 146, 60, 122, 248, 220, 216, 126, 146, 228, 236, 143, 80, 239, 126, 107, 140, 93, 193, 43, 237, 220, 27, 72, 167, 158, 64, 163, 146, 12, 247, 215, 158, 176, 19, 47, 194, 75, 138, 206, 98, 67, 225, 40, 73, 175, 124, 209, 50, 168, 214, 130, 131, 57, 1, 23, 100, 69, 71, 154, 2, 194, 224, 82, 134, 0, 196, 34, 143, 238, 101, 22, 3, 119, 17, 80, 236, 155, 73, 70, 51, 90, 188, 176, 235, 189, 102, 87, 161, 237, 212, 137, 42, 146, 153, 180, 186, 12, 53, 177, 227, 114, 216, 172, 147, 251, 85, 164, 199, 207, 12, 202, 209, 15, 36, 80, 156, 76, 90, 200, 34, 224, 127, 40, 199, 63, 45, 67, 6, 116, 246, 251, 235, 151, 225, 162, 147, 247, 237, 111, 43, 157, 126, 168, 116, 82, 206, 234, 249, 106, 201, 178, 233, 124, 183, 127, 179, 166, 32, 164, 214, 20, 217, 44, 149, 26, 179, 90, 101, 51, 154, 94, 184, 118, 59, 0, 111, 198, 147, 178, 218, 113, 4, 152, 217, 90, 96, 127, 178, 36, 56, 122, 130, 55, 91, 19, 138, 99, 157, 255, 130, 220, 81, 247, 219, 134, 226, 221, 122, 59, 11, 71, 130, 187, 30, 94, 246, 96, 184, 131, 96, 244, 106, 51, 223, 40, 115, 181, 120, 223, 82, 160, 60, 223, 101, 231, 203, 234, 121, 65, 106, 28, 189, 177, 119, 24, 121, 210, 202, 208, 249, 2, 242, 210, 246, 35, 236, 162, 101, 87, 93, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 143, 49, 14, 130, 48, 20, 134, 41, 46, 30, 195, 51, 232, 9, 20, 99, 226, 224, 68, 156, 13, 129, 23, 210, 164, 180, 164, 52, 38, 140, 189, 1, 45, 18, 87, 19, 5, 7, 244, 14, 14, 30, 76, 8, 177, 36, 14, 162, 111, 126, 223, 247, 255, 255, 72, 171, 227, 105, 172, 100, 189, 224, 152, 16, 28, 58, 30, 33, 7, 75, 201, 202, 197, 52, 36, 144, 103, 74, 63, 39, 214, 247, 67, 104, 240, 165, 51, 70, 49, 1, 251, 161, 145, 157, 101, 141, 183, 198, 116, 15, 92, 236, 252, 38, 178, 144, 165, 195, 104, 34, 114, 89, 45, 49, 7, 95, 32, 121, 93, 83, 1, 33, 240, 243, 118, 54, 29, 246, 127, 242, 246, 127, 60, 146, 183, 118, 121, 224, 9, 207, 97, 113, 106, 52, 86, 239, 51, 197, 234, 21, 227, 128, 67, 218, 2, 69, 213, 141, 80, 242, 190, 129, 136, 241, 116, 30, 4, 28, 146, 196, 240, 102, 119, 185, 194, 64, 130, 95, 255, 228, 197, 21, 44, 86, 186, 79, 125, 247, 120, 1, 82, 78, 27, 216, 177, 1, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
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
                        size: SemiFlattenedLength(3),
                    }),
                    ValueOrArray::MemoryAddress(MemoryAddress::direct(1)),
                ],
                input_value_types: vec![
                    HeapValueType::Array {
                        size: SemanticLength(3),
                        value_types: vec![HeapValueType::field()],
                    },
                    HeapValueType::field(),
                ],
                destinations: vec![
                    ValueOrArray::HeapArray(HeapArray {
                        pointer: MemoryAddress::direct(0),
                        size: SemiFlattenedLength(3),
                    }),
                    ValueOrArray::MemoryAddress(MemoryAddress::direct(35)),
                    ValueOrArray::MemoryAddress(MemoryAddress::direct(36)),
                ],
                destination_value_types: vec![
                    HeapValueType::Array {
                        size: SemanticLength(3),
                        value_types: vec![HeapValueType::field()],
                    },
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

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 85, 203, 110, 211, 64, 20, 181, 227, 20, 250, 25, 145, 224, 11, 224, 11, 74, 80, 5, 11, 86, 21, 235, 209, 196, 190, 137, 70, 26, 207, 12, 119, 198, 165, 97, 55, 9, 136, 109, 30, 75, 54, 136, 54, 47, 57, 1, 149, 10, 117, 203, 130, 15, 99, 28, 226, 224, 84, 228, 161, 166, 128, 87, 215, 71, 246, 61, 231, 158, 57, 215, 46, 181, 38, 245, 68, 132, 134, 73, 161, 187, 239, 103, 121, 77, 4, 141, 225, 227, 183, 48, 65, 4, 97, 200, 107, 102, 4, 104, 77, 152, 136, 224, 236, 112, 36, 85, 40, 35, 208, 93, 155, 62, 65, 198, 57, 107, 84, 41, 231, 111, 63, 177, 200, 27, 50, 161, 18, 163, 123, 118, 112, 132, 72, 155, 253, 246, 36, 78, 56, 49, 128, 177, 238, 92, 113, 38, 128, 34, 9, 101, 92, 99, 130, 254, 162, 236, 253, 168, 120, 155, 47, 223, 63, 127, 69, 194, 173, 143, 121, 119, 65, 85, 250, 119, 84, 193, 110, 84, 118, 120, 194, 68, 131, 195, 54, 202, 254, 46, 70, 238, 226, 192, 221, 73, 31, 201, 196, 100, 97, 232, 231, 97, 40, 31, 220, 203, 198, 137, 21, 135, 251, 121, 113, 56, 81, 8, 17, 11, 169, 129, 239, 95, 21, 178, 83, 87, 16, 69, 209, 197, 207, 13, 171, 251, 126, 41, 184, 84, 73, 141, 179, 176, 128, 118, 102, 8, 38, 65, 65, 78, 41, 79, 64, 119, 190, 80, 173, 1, 13, 137, 93, 70, 105, 195, 1, 215, 46, 198, 206, 20, 131, 212, 57, 21, 145, 223, 9, 111, 173, 38, 124, 234, 44, 116, 34, 206, 72, 232, 2, 60, 174, 53, 13, 100, 193, 254, 96, 7, 213, 236, 237, 118, 234, 66, 110, 22, 14, 219, 225, 83, 134, 16, 26, 111, 92, 99, 134, 104, 246, 6, 236, 232, 185, 48, 208, 0, 60, 127, 249, 248, 209, 96, 46, 101, 171, 41, 193, 166, 214, 254, 94, 173, 61, 59, 205, 214, 48, 162, 134, 86, 165, 106, 182, 175, 10, 12, 132, 70, 17, 58, 115, 114, 166, 202, 52, 99, 185, 137, 122, 159, 101, 189, 174, 193, 220, 196, 253, 191, 231, 71, 101, 83, 235, 96, 175, 214, 27, 85, 151, 247, 60, 197, 221, 173, 246, 255, 104, 117, 176, 198, 234, 178, 77, 143, 37, 2, 107, 136, 140, 224, 221, 56, 79, 235, 104, 17, 212, 105, 129, 201, 45, 214, 228, 25, 80, 53, 95, 174, 214, 72, 73, 230, 6, 193, 229, 185, 92, 100, 188, 129, 157, 189, 128, 88, 98, 243, 104, 149, 231, 193, 26, 252, 225, 117, 113, 148, 185, 27, 196, 52, 21, 44, 151, 184, 149, 22, 192, 110, 190, 197, 131, 99, 6, 60, 90, 48, 174, 96, 171, 119, 203, 223, 195, 173, 133, 251, 151, 243, 22, 69, 105, 189, 219, 74, 251, 63, 169, 222, 111, 203, 15, 236, 197, 137, 145, 202, 166, 139, 47, 96, 150, 193, 53, 22, 46, 9, 127, 2, 175, 169, 227, 172, 228, 7, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 82, 65, 78, 195, 48, 16, 140, 235, 22, 250, 140, 72, 240, 2, 120, 65, 9, 170, 224, 192, 169, 226, 140, 172, 196, 138, 44, 185, 113, 228, 248, 64, 142, 254, 65, 108, 131, 56, 112, 65, 130, 180, 66, 45, 127, 224, 192, 195, 104, 16, 78, 213, 22, 154, 88, 141, 79, 222, 181, 103, 118, 118, 119, 160, 86, 79, 47, 67, 37, 23, 23, 156, 80, 74, 226, 0, 81, 250, 224, 105, 89, 142, 56, 71, 185, 49, 133, 210, 95, 190, 183, 255, 0, 208, 248, 197, 107, 71, 212, 235, 138, 8, 54, 19, 201, 217, 132, 36, 49, 197, 166, 48, 109, 90, 108, 163, 190, 27, 97, 198, 14, 191, 63, 56, 170, 68, 78, 83, 138, 143, 237, 101, 248, 105, 64, 15, 22, 197, 106, 10, 203, 144, 85, 153, 251, 187, 112, 181, 179, 103, 89, 6, 44, 201, 132, 145, 179, 75, 194, 113, 40, 60, 57, 191, 78, 4, 142, 49, 127, 189, 61, 63, 107, 172, 10, 183, 241, 192, 13, 239, 201, 101, 101, 157, 8, 9, 20, 176, 52, 175, 105, 252, 181, 158, 154, 248, 48, 165, 254, 54, 30, 186, 225, 119, 234, 247, 93, 39, 245, 119, 167, 96, 173, 167, 38, 94, 140, 25, 199, 36, 78, 42, 192, 227, 252, 119, 95, 70, 190, 95, 97, 148, 254, 236, 88, 215, 67, 128, 242, 227, 6, 79, 25, 207, 71, 81, 196, 113, 150, 217, 135, 147, 127, 242, 167, 214, 38, 90, 89, 111, 148, 99, 130, 105, 4, 55, 195, 205, 72, 59, 213, 6, 186, 101, 141, 142, 55, 234, 232, 189, 129, 124, 155, 8, 150, 42, 189, 235, 181, 111, 81, 157, 181, 15, 223, 4, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
}

#[test]
fn memory_op_circuit() {
    let src = "
    private parameters: [w1, w2, w3]
    public parameters: []
    return values: [w4]
    INIT b0 = [w1, w2]
    ASSERT w5 = 1
    WRITE b0[w5] = w3
    READ w4 = b0[w5]
    ";
    let mut circuit = Circuit::from_str(src).unwrap();
    circuit.current_witness_index = 5;

    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 145, 59, 78, 195, 64, 16, 134, 19, 39, 185, 71, 74, 232, 64, 156, 0, 33, 36, 40, 120, 148, 8, 9, 173, 214, 235, 1, 173, 176, 119, 151, 217, 117, 192, 165, 227, 130, 214, 78, 56, 64, 4, 4, 98, 75, 17, 2, 196, 171, 133, 92, 36, 29, 37, 13, 61, 43, 91, 188, 26, 196, 84, 51, 163, 249, 53, 51, 223, 239, 116, 243, 221, 80, 48, 195, 165, 208, 217, 241, 248, 51, 39, 130, 6, 48, 184, 99, 33, 34, 8, 67, 14, 185, 17, 160, 53, 225, 194, 131, 163, 214, 165, 84, 76, 122, 160, 79, 226, 98, 13, 2, 137, 209, 170, 224, 38, 25, 185, 190, 100, 251, 132, 123, 181, 115, 110, 235, 94, 221, 41, 170, 142, 137, 20, 92, 84, 131, 113, 177, 168, 53, 160, 217, 6, 148, 73, 30, 132, 62, 49, 128, 129, 78, 111, 125, 46, 128, 34, 97, 50, 112, 185, 160, 213, 53, 189, 73, 187, 246, 119, 212, 91, 103, 7, 132, 77, 218, 115, 222, 58, 78, 231, 7, 179, 143, 155, 203, 247, 113, 188, 181, 51, 179, 240, 186, 18, 61, 169, 108, 105, 250, 222, 127, 179, 115, 241, 168, 90, 191, 161, 186, 223, 87, 158, 74, 149, 228, 82, 1, 150, 235, 94, 134, 213, 111, 195, 14, 245, 67, 104, 252, 67, 241, 252, 75, 209, 188, 81, 200, 59, 212, 0, 81, 20, 45, 59, 251, 151, 238, 215, 157, 198, 181, 10, 93, 159, 179, 31, 221, 116, 140, 96, 66, 20, 164, 212, 233, 172, 121, 69, 75, 38, 36, 176, 132, 233, 30, 232, 244, 193, 154, 96, 1, 24, 164, 150, 138, 71, 190, 252, 73, 63, 0, 160, 171, 184, 233, 173, 1, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 125, 140, 61, 14, 130, 48, 24, 64, 91, 74, 239, 193, 168, 155, 198, 19, 24, 99, 162, 131, 63, 163, 49, 113, 179, 131, 131, 66, 10, 11, 227, 119, 131, 182, 159, 137, 43, 147, 33, 28, 128, 192, 10, 92, 132, 141, 145, 133, 29, 18, 38, 22, 222, 246, 146, 151, 199, 140, 254, 69, 252, 11, 201, 73, 188, 93, 25, 30, 63, 175, 0, 137, 161, 214, 127, 116, 72, 182, 190, 47, 100, 112, 23, 210, 69, 165, 77, 229, 144, 121, 40, 175, 156, 213, 243, 44, 235, 117, 180, 204, 174, 251, 20, 224, 246, 88, 108, 154, 67, 152, 123, 122, 87, 119, 216, 14, 13, 196, 227, 252, 226, 25, 130, 37, 103, 19, 47, 184, 141, 212, 98, 74, 219, 74, 245, 79, 150, 21, 108, 157, 0, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
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

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack,  @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 211, 205, 74, 3, 49, 16, 0, 224, 221, 236, 139, 120, 212, 155, 226, 19, 72, 17, 60, 137, 71, 17, 36, 164, 217, 81, 2, 187, 217, 56, 73, 170, 30, 171, 130, 215, 253, 121, 129, 162, 98, 89, 161, 136, 138, 127, 55, 15, 190, 72, 111, 30, 189, 120, 55, 22, 86, 43, 45, 117, 45, 130, 57, 133, 97, 8, 51, 223, 76, 200, 65, 185, 101, 37, 55, 34, 145, 186, 56, 238, 85, 119, 42, 89, 12, 157, 91, 110, 17, 65, 26, 186, 43, 140, 4, 173, 169, 144, 33, 236, 5, 221, 68, 241, 36, 4, 93, 180, 207, 26, 44, 138, 142, 78, 68, 232, 159, 11, 169, 172, 209, 185, 231, 119, 19, 107, 62, 174, 25, 41, 21, 66, 40, 56, 51, 240, 52, 57, 51, 24, 206, 188, 88, 210, 26, 208, 108, 0, 38, 135, 101, 108, 35, 106, 0, 99, 157, 222, 68, 66, 2, 67, 202, 147, 184, 41, 36, 27, 20, 156, 231, 207, 51, 222, 228, 227, 19, 151, 51, 31, 174, 98, 127, 161, 51, 247, 176, 182, 124, 215, 110, 175, 111, 206, 46, 190, 172, 236, 63, 170, 172, 209, 127, 43, 94, 93, 82, 112, 186, 67, 249, 143, 79, 121, 215, 10, 69, 203, 213, 72, 21, 67, 199, 227, 234, 210, 153, 119, 165, 108, 51, 18, 124, 56, 230, 247, 16, 140, 69, 73, 91, 44, 178, 160, 211, 75, 54, 232, 136, 198, 142, 144, 109, 187, 192, 47, 157, 243, 191, 54, 241, 106, 152, 144, 90, 38, 228, 107, 176, 164, 26, 44, 25, 63, 216, 49, 122, 110, 7, 70, 249, 210, 239, 122, 89, 48, 37, 159, 95, 241, 101, 255, 193, 231, 79, 189, 82, 181, 80, 188, 17, 148, 123, 71, 226, 234, 55, 200, 92, 83, 33, 253, 252, 212, 233, 59, 226, 130, 57, 211, 226, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 144, 161, 10, 2, 65, 20, 69, 223, 123, 243, 35, 70, 109, 138, 95, 32, 139, 96, 18, 163, 8, 6, 193, 109, 11, 202, 172, 197, 56, 127, 240, 222, 27, 193, 186, 193, 228, 7, 136, 182, 13, 254, 200, 54, 163, 197, 238, 40, 6, 147, 227, 134, 189, 237, 194, 229, 114, 56, 70, 253, 190, 48, 222, 29, 146, 69, 150, 237, 80, 1, 133, 202, 239, 102, 74, 119, 28, 228, 121, 106, 55, 179, 212, 174, 60, 171, 94, 91, 240, 59, 72, 97, 211, 93, 142, 109, 213, 43, 58, 231, 201, 240, 228, 220, 116, 222, 238, 223, 70, 219, 203, 90, 146, 234, 225, 239, 97, 100, 162, 55, 32, 32, 200, 28, 240, 180, 62, 2, 252, 129, 64, 209, 27, 250, 152, 32, 165, 151, 137, 224, 131, 197, 4, 34, 148, 102, 136, 48, 46, 229, 205, 0, 204, 79, 117, 204, 11, 148, 185, 1, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
}
