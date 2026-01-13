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
    BRILLIG CALL func: 0, predicate: 1, inputs: [{w_input}], outputs: [{w_inverted}]
    "
    );
    let mut circuit = Circuit::from_str(&src).unwrap();
    circuit.current_witness_index = 8;

    let program =
        Program { functions: vec![circuit], unconstrained_functions: vec![brillig_bytecode] };

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 146, 207, 78, 27, 49, 16, 198, 179, 123, 234, 99, 244, 25, 218, 39, 104, 83, 33, 245, 208, 19, 234, 217, 114, 236, 201, 106, 36, 175, 237, 142, 103, 211, 166, 183, 77, 90, 113, 13, 72, 220, 17, 144, 127, 218, 4, 4, 8, 241, 2, 60, 24, 222, 64, 66, 136, 68, 130, 196, 158, 86, 99, 251, 251, 102, 126, 243, 165, 189, 105, 187, 176, 138, 209, 217, 112, 120, 48, 95, 254, 11, 43, 115, 56, 185, 85, 5, 17, 88, 22, 191, 145, 45, 132, 32, 208, 106, 248, 243, 97, 236, 188, 114, 26, 194, 97, 89, 125, 37, 52, 6, 179, 166, 52, 230, 223, 41, 234, 198, 8, 173, 47, 56, 158, 140, 246, 209, 102, 6, 250, 211, 188, 48, 130, 129, 242, 48, 184, 49, 104, 65, 146, 80, 46, 111, 161, 149, 143, 150, 71, 247, 31, 27, 219, 191, 36, 57, 251, 37, 212, 206, 107, 141, 177, 43, 120, 229, 157, 123, 3, 233, 212, 19, 104, 84, 146, 119, 182, 49, 120, 147, 69, 114, 237, 9, 59, 81, 78, 120, 73, 145, 79, 212, 11, 71, 73, 122, 229, 139, 150, 65, 181, 86, 28, 204, 9, 184, 32, 43, 58, 210, 20, 16, 6, 151, 50, 4, 32, 22, 121, 100, 40, 179, 88, 184, 139, 152, 163, 45, 147, 140, 189, 104, 241, 188, 129, 222, 203, 13, 84, 104, 59, 245, 67, 21, 249, 78, 90, 93, 134, 154, 251, 113, 57, 108, 214, 143, 251, 85, 220, 1, 63, 141, 80, 142, 190, 33, 129, 226, 100, 210, 66, 22, 1, 255, 66, 57, 254, 110, 25, 50, 160, 179, 159, 159, 63, 13, 23, 157, 236, 134, 184, 77, 58, 125, 151, 116, 82, 206, 234, 148, 104, 201, 178, 233, 124, 183, 127, 179, 230, 32, 164, 214, 20, 217, 44, 157, 26, 179, 218, 101, 179, 154, 94, 184, 118, 59, 0, 111, 214, 147, 178, 218, 115, 4, 152, 217, 218, 224, 255, 100, 73, 112, 244, 8, 111, 182, 102, 20, 195, 49, 255, 1, 185, 163, 238, 151, 13, 199, 187, 245, 118, 22, 19, 9, 238, 122, 120, 78, 211, 112, 15, 193, 232, 85, 190, 95, 145, 185, 90, 156, 111, 17, 40, 207, 247, 217, 249, 178, 122, 10, 72, 141, 163, 55, 246, 14, 35, 79, 90, 13, 116, 190, 128, 188, 28, 251, 1, 193, 125, 61, 190, 163, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 144, 79, 10, 130, 64, 20, 198, 29, 219, 116, 140, 206, 80, 39, 40, 67, 104, 209, 74, 90, 135, 232, 67, 6, 198, 25, 25, 135, 192, 229, 220, 96, 70, 147, 182, 65, 105, 11, 235, 22, 29, 44, 69, 26, 161, 69, 214, 91, 127, 127, 126, 239, 155, 228, 250, 116, 158, 106, 217, 172, 56, 38, 4, 71, 142, 79, 200, 209, 210, 178, 246, 48, 141, 8, 20, 74, 231, 207, 153, 245, 253, 16, 26, 149, 244, 137, 113, 66, 192, 46, 148, 26, 79, 204, 145, 173, 84, 219, 221, 96, 122, 0, 46, 246, 65, 139, 85, 202, 202, 97, 52, 21, 133, 172, 215, 152, 67, 32, 144, 188, 109, 168, 128, 8, 248, 101, 183, 152, 143, 51, 124, 250, 237, 255, 252, 72, 222, 187, 117, 66, 95, 248, 14, 75, 50, 19, 99, 13, 121, 6, 172, 113, 25, 7, 28, 209, 206, 80, 214, 253, 19, 90, 62, 182, 16, 51, 158, 45, 195, 144, 67, 154, 26, 191, 217, 166, 114, 49, 144, 240, 87, 157, 188, 122, 130, 37, 58, 31, 90, 223, 28, 47, 248, 217, 227, 190, 213, 1, 0, 0]");

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
    BRILLIG CALL func: 0, predicate: 1, inputs: [[{a}, {b}, {c}], {a} + {b} + {c}], outputs: [[{a_times_2}, {b_times_3}, {c_times_4}], {a_plus_b_plus_c}, {a_plus_b_plus_c_times_2}]
    ");
    let circuit = Circuit::from_str(&src).unwrap();
    let program =
        Program { functions: vec![circuit], unconstrained_functions: vec![brillig_bytecode] };

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 149, 205, 110, 211, 64, 20, 133, 237, 56, 133, 62, 70, 36, 120, 2, 120, 130, 18, 84, 193, 130, 85, 197, 122, 52, 177, 111, 172, 145, 198, 51, 195, 204, 184, 212, 236, 38, 1, 177, 205, 207, 146, 13, 162, 77, 156, 200, 9, 168, 84, 168, 47, 192, 131, 49, 14, 113, 112, 42, 146, 88, 77, 161, 89, 57, 87, 246, 253, 238, 61, 62, 103, 92, 235, 76, 219, 49, 243, 53, 225, 76, 245, 63, 206, 139, 107, 196, 112, 4, 159, 127, 248, 177, 148, 192, 52, 122, 75, 52, 3, 165, 16, 97, 1, 156, 29, 166, 92, 248, 60, 0, 213, 55, 217, 51, 73, 40, 37, 97, 19, 83, 250, 254, 11, 9, 156, 49, 97, 34, 214, 106, 96, 70, 71, 82, 226, 100, 216, 157, 70, 49, 69, 26, 100, 164, 122, 87, 148, 48, 192, 18, 249, 60, 106, 17, 134, 127, 35, 7, 63, 27, 206, 246, 159, 235, 158, 191, 65, 254, 206, 219, 156, 187, 64, 213, 254, 31, 202, 171, 134, 50, 227, 19, 194, 66, 10, 187, 144, 195, 42, 66, 86, 81, 224, 238, 70, 79, 121, 172, 115, 51, 12, 11, 51, 212, 15, 30, 228, 235, 68, 130, 194, 195, 226, 226, 112, 42, 36, 4, 196, 199, 122, 231, 138, 189, 74, 88, 247, 187, 144, 228, 212, 182, 67, 2, 75, 107, 98, 219, 79, 13, 221, 154, 119, 41, 226, 22, 37, 126, 169, 218, 155, 75, 208, 177, 100, 232, 20, 211, 24, 84, 239, 27, 86, 10, 164, 70, 145, 117, 58, 14, 109, 225, 218, 134, 193, 114, 181, 196, 118, 152, 0, 253, 201, 73, 103, 61, 39, 51, 59, 165, 93, 229, 12, 249, 54, 6, 147, 86, 162, 33, 143, 199, 39, 51, 106, 230, 79, 119, 51, 27, 21, 189, 92, 194, 140, 159, 19, 9, 190, 118, 38, 45, 162, 145, 34, 239, 192, 164, 47, 153, 134, 16, 228, 249, 235, 167, 79, 70, 139, 81, 118, 238, 232, 109, 107, 237, 238, 213, 218, 49, 179, 60, 204, 1, 214, 184, 201, 69, 210, 189, 42, 17, 16, 14, 2, 105, 197, 41, 72, 141, 89, 78, 185, 89, 117, 190, 242, 118, 91, 129, 190, 89, 119, 255, 157, 30, 141, 109, 173, 189, 189, 90, 111, 157, 186, 190, 231, 91, 172, 46, 181, 251, 87, 169, 189, 13, 82, 215, 77, 118, 204, 37, 144, 144, 229, 128, 15, 147, 194, 173, 233, 210, 168, 179, 18, 201, 198, 115, 250, 2, 176, 88, 68, 180, 147, 10, 78, 236, 34, 114, 245, 94, 46, 114, 174, 103, 230, 175, 32, 226, 50, 57, 90, 231, 60, 218, 80, 127, 124, 93, 94, 101, 161, 6, 210, 137, 128, 213, 81, 208, 201, 74, 197, 126, 113, 22, 140, 142, 9, 208, 96, 73, 92, 171, 173, 255, 91, 125, 100, 110, 61, 184, 123, 185, 104, 81, 30, 109, 112, 219, 209, 238, 199, 213, 251, 165, 252, 192, 92, 156, 104, 46, 76, 182, 60, 1, 115, 15, 110, 144, 112, 5, 252, 5, 155, 147, 188, 188, 42, 8, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 82, 75, 78, 195, 48, 20, 140, 235, 22, 122, 140, 72, 112, 2, 56, 65, 9, 170, 96, 193, 170, 98, 141, 162, 196, 138, 44, 185, 113, 228, 120, 65, 150, 190, 129, 63, 32, 22, 108, 144, 32, 173, 80, 203, 45, 56, 24, 13, 194, 169, 218, 210, 38, 86, 227, 149, 127, 51, 111, 222, 188, 129, 90, 189, 188, 13, 149, 88, 92, 49, 76, 8, 78, 130, 144, 144, 39, 79, 139, 114, 196, 88, 88, 24, 35, 149, 254, 246, 189, 195, 11, 128, 198, 47, 94, 59, 162, 94, 87, 68, 176, 153, 72, 204, 38, 56, 77, 8, 50, 210, 180, 105, 177, 141, 250, 110, 132, 25, 107, 126, 127, 112, 82, 137, 156, 102, 4, 157, 218, 205, 208, 72, 217, 92, 197, 128, 30, 148, 114, 229, 212, 50, 162, 21, 234, 241, 33, 90, 205, 245, 85, 148, 1, 77, 115, 110, 196, 236, 26, 51, 20, 113, 79, 204, 111, 83, 142, 18, 196, 222, 239, 47, 47, 26, 105, 225, 54, 30, 184, 225, 61, 177, 172, 226, 21, 135, 60, 12, 104, 86, 212, 52, 254, 90, 79, 77, 124, 156, 82, 127, 27, 15, 221, 240, 59, 245, 251, 174, 78, 253, 223, 41, 88, 235, 169, 137, 23, 99, 202, 16, 78, 210, 10, 240, 60, 255, 155, 151, 17, 159, 55, 40, 204, 126, 115, 160, 107, 19, 160, 248, 186, 67, 83, 202, 138, 81, 28, 51, 148, 231, 246, 225, 108, 207, 253, 185, 141, 146, 86, 54, 63, 229, 24, 35, 18, 195, 205, 227, 230, 73, 59, 213, 6, 186, 101, 141, 142, 39, 234, 152, 189, 129, 248, 152, 112, 154, 41, 189, 155, 181, 31, 237, 62, 140, 252, 3, 5, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
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

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 205, 147, 75, 78, 195, 48, 16, 134, 147, 180, 220, 131, 235, 176, 64, 28, 97, 228, 56, 3, 178, 136, 31, 204, 216, 133, 46, 219, 44, 216, 38, 237, 5, 16, 207, 166, 82, 133, 0, 33, 46, 192, 193, 48, 137, 120, 172, 104, 23, 168, 170, 55, 182, 199, 163, 255, 255, 52, 250, 157, 77, 219, 227, 96, 164, 87, 214, 112, 115, 185, 250, 58, 131, 17, 26, 175, 94, 101, 32, 66, 227, 225, 92, 121, 131, 204, 160, 76, 129, 23, 123, 15, 214, 73, 91, 32, 207, 39, 203, 67, 212, 150, 198, 7, 70, 249, 106, 145, 151, 86, 158, 130, 42, 146, 91, 21, 239, 179, 52, 91, 246, 21, 63, 118, 120, 223, 55, 78, 22, 253, 126, 228, 166, 63, 237, 215, 214, 85, 173, 117, 72, 226, 211, 185, 106, 117, 40, 193, 35, 105, 174, 95, 74, 101, 80, 16, 72, 171, 115, 101, 186, 103, 174, 111, 206, 64, 190, 239, 39, 127, 175, 244, 174, 67, 253, 47, 177, 145, 40, 3, 174, 19, 107, 102, 235, 149, 6, 27, 25, 38, 91, 155, 83, 178, 163, 115, 26, 110, 70, 255, 236, 72, 141, 132, 71, 112, 130, 98, 96, 163, 39, 207, 211, 108, 240, 228, 66, 94, 42, 249, 171, 90, 175, 8, 125, 32, 3, 29, 33, 55, 195, 71, 193, 140, 228, 65, 199, 88, 139, 19, 228, 250, 45, 38, 63, 194, 121, 18, 145, 184, 128, 239, 79, 81, 127, 0, 205, 96, 137, 59, 34, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 142, 187, 9, 128, 64, 16, 68, 119, 111, 207, 62, 108, 199, 64, 172, 194, 192, 192, 15, 98, 98, 120, 29, 236, 39, 48, 53, 18, 177, 14, 11, 51, 184, 220, 59, 193, 73, 134, 129, 7, 243, 72, 101, 219, 11, 11, 87, 221, 246, 227, 188, 86, 67, 183, 24, 40, 186, 35, 238, 112, 198, 110, 38, 5, 51, 230, 187, 132, 247, 96, 38, 36, 154, 166, 40, 137, 192, 103, 63, 248, 209, 207, 103, 188, 161, 35, 22, 207, 252, 0, 167, 131, 176, 229, 104, 1, 0, 0]");

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
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w2]
    CALL func: 1, predicate: 1, inputs: [w0, w1], outputs: [w3]
    ASSERT 0 = w2 - w3
    ";
    let main = Circuit::from_str(src).unwrap();

    let src = "
    private parameters: [w0, w1]
    public parameters: []
    return values: [w3]
    ASSERT 0 = w0 - w2 + 2
    CALL func: 2, predicate: 1, inputs: [w2, w1], outputs: [w3]
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
    insta::assert_compact_debug_snapshot!(bytes_msgpack,  @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 148, 203, 74, 3, 49, 20, 134, 103, 50, 47, 226, 82, 119, 138, 79, 32, 69, 112, 37, 46, 69, 144, 144, 102, 142, 18, 152, 201, 196, 147, 164, 234, 178, 42, 184, 157, 203, 11, 20, 21, 203, 8, 69, 84, 188, 237, 125, 145, 238, 92, 186, 113, 111, 44, 140, 23, 42, 237, 88, 42, 102, 117, 56, 28, 206, 229, 255, 126, 66, 14, 202, 45, 43, 185, 17, 137, 212, 197, 113, 175, 138, 169, 100, 49, 116, 110, 185, 69, 4, 105, 232, 174, 48, 18, 180, 166, 66, 134, 176, 23, 116, 19, 197, 147, 16, 116, 209, 62, 107, 176, 40, 58, 58, 17, 161, 127, 46, 164, 178, 70, 231, 158, 223, 77, 172, 121, 15, 51, 82, 42, 132, 80, 112, 102, 224, 176, 140, 109, 68, 13, 96, 172, 211, 155, 72, 72, 96, 72, 121, 18, 55, 133, 100, 131, 201, 233, 233, 14, 229, 79, 51, 222, 232, 231, 143, 158, 23, 76, 127, 222, 197, 146, 214, 128, 102, 3, 48, 25, 215, 50, 207, 199, 247, 35, 174, 102, 62, 92, 197, 254, 66, 103, 238, 97, 109, 249, 174, 221, 94, 223, 156, 93, 124, 94, 217, 127, 84, 89, 163, 255, 90, 188, 184, 162, 160, 214, 106, 222, 181, 66, 209, 114, 151, 82, 197, 208, 161, 114, 123, 233, 204, 187, 82, 182, 25, 9, 254, 53, 231, 247, 16, 140, 69, 73, 91, 44, 178, 160, 211, 75, 54, 184, 136, 198, 14, 39, 219, 118, 137, 95, 50, 207, 167, 173, 137, 87, 67, 19, 82, 75, 19, 242, 105, 15, 82, 217, 131, 252, 165, 61, 126, 96, 224, 252, 56, 12, 33, 253, 206, 32, 11, 38, 132, 224, 87, 16, 178, 255, 128, 224, 79, 108, 204, 90, 162, 120, 67, 162, 220, 59, 73, 220, 254, 6, 153, 59, 42, 164, 31, 223, 84, 250, 6, 44, 85, 242, 112, 180, 4, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 144, 161, 10, 2, 65, 16, 134, 103, 102, 95, 196, 168, 77, 241, 9, 228, 16, 76, 98, 20, 193, 32, 120, 237, 64, 217, 179, 24, 247, 13, 102, 102, 5, 235, 5, 147, 15, 32, 218, 125, 145, 107, 70, 139, 221, 69, 12, 38, 119, 47, 220, 228, 255, 255, 249, 230, 51, 234, 143, 149, 241, 238, 148, 173, 138, 226, 128, 10, 40, 228, 153, 239, 29, 248, 127, 248, 219, 48, 105, 141, 243, 168, 44, 115, 187, 91, 228, 118, 227, 89, 53, 222, 160, 144, 233, 175, 167, 182, 30, 84, 189, 235, 108, 124, 113, 110, 190, 236, 14, 31, 147, 253, 109, 43, 89, 253, 242, 207, 16, 50, 209, 25, 16, 16, 100, 14, 111, 106, 115, 4, 72, 64, 160, 232, 12, 125, 109, 145, 82, 170, 173, 224, 149, 197, 4, 106, 148, 118, 168, 49, 46, 238, 195, 0, 204, 111, 233, 240, 53, 48, 37, 2, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
}
