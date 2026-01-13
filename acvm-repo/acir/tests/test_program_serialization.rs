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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 146, 75, 78, 195, 48, 16, 134, 155, 172, 56, 6, 103, 128, 19, 64, 17, 18, 11, 86, 136, 181, 229, 38, 211, 104, 36, 199, 54, 227, 73, 161, 236, 210, 10, 177, 45, 72, 236, 17, 208, 7, 74, 11, 2, 132, 184, 0, 7, 195, 233, 155, 74, 180, 11, 188, 178, 198, 158, 249, 231, 255, 102, 194, 214, 115, 61, 211, 17, 163, 209, 238, 230, 122, 52, 187, 11, 45, 83, 184, 255, 140, 50, 34, 208, 44, 206, 145, 53, 56, 39, 80, 199, 112, 177, 213, 55, 54, 50, 49, 184, 155, 188, 216, 39, 84, 10, 147, 170, 84, 170, 253, 128, 113, 165, 135, 218, 102, 236, 95, 122, 39, 168, 19, 5, 237, 231, 52, 83, 130, 129, 82, 215, 249, 80, 168, 65, 146, 136, 76, 90, 67, 45, 39, 146, 183, 223, 219, 149, 245, 39, 8, 30, 207, 68, 180, 241, 91, 165, 111, 50, 158, 107, 167, 86, 65, 248, 110, 9, 27, 146, 65, 88, 73, 222, 142, 239, 194, 221, 6, 225, 155, 205, 106, 10, 163, 165, 96, 103, 68, 192, 25, 105, 209, 144, 42, 3, 215, 121, 149, 206, 1, 177, 72, 189, 101, 153, 248, 192, 151, 167, 226, 155, 101, 146, 222, 65, 44, 22, 192, 90, 191, 129, 21, 168, 27, 101, 98, 228, 113, 12, 106, 77, 134, 18, 211, 93, 222, 173, 150, 201, 237, 194, 35, 227, 169, 241, 188, 119, 128, 4, 17, 7, 131, 26, 178, 112, 120, 9, 121, 255, 72, 51, 36, 64, 143, 167, 187, 59, 221, 113, 39, 155, 61, 175, 43, 29, 254, 171, 116, 144, 15, 203, 161, 198, 146, 101, 213, 216, 102, 251, 99, 73, 65, 200, 56, 38, 207, 102, 166, 84, 25, 150, 42, 171, 209, 240, 197, 212, 235, 14, 120, 53, 30, 228, 197, 161, 33, 192, 68, 151, 2, 87, 131, 25, 193, 222, 4, 222, 112, 73, 200, 207, 114, 116, 12, 169, 161, 230, 222, 138, 226, 215, 114, 59, 99, 71, 130, 155, 22, 22, 195, 239, 30, 34, 168, 120, 190, 142, 127, 148, 121, 27, 191, 175, 41, 144, 63, 157, 176, 177, 121, 49, 93, 144, 18, 71, 171, 111, 13, 122, 158, 52, 55, 244, 52, 134, 60, 179, 253, 3, 17, 108, 54, 103, 82, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 143, 65, 10, 130, 64, 20, 134, 29, 219, 116, 140, 206, 80, 39, 40, 35, 104, 209, 74, 90, 135, 232, 99, 24, 24, 103, 100, 28, 2, 151, 115, 3, 103, 68, 218, 6, 165, 45, 172, 91, 116, 176, 20, 105, 132, 22, 89, 111, 253, 190, 239, 255, 255, 137, 209, 167, 243, 84, 171, 102, 37, 8, 165, 4, 123, 1, 165, 133, 163, 85, 237, 19, 134, 41, 20, 185, 54, 207, 153, 243, 253, 16, 26, 125, 233, 141, 113, 66, 193, 53, 200, 205, 243, 86, 219, 16, 118, 4, 33, 15, 97, 155, 88, 170, 202, 227, 44, 149, 133, 170, 215, 68, 64, 40, 145, 186, 109, 153, 4, 12, 226, 178, 95, 204, 199, 245, 159, 188, 251, 31, 143, 212, 189, 27, 30, 5, 50, 240, 120, 146, 89, 141, 51, 248, 108, 177, 102, 195, 5, 16, 204, 58, 160, 172, 251, 17, 90, 61, 118, 16, 115, 145, 45, 163, 72, 64, 154, 90, 222, 206, 174, 54, 4, 104, 244, 235, 159, 186, 250, 146, 39, 218, 12, 169, 239, 30, 47, 106, 71, 214, 240, 176, 1, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 149, 203, 110, 211, 64, 20, 134, 237, 56, 133, 62, 70, 36, 120, 2, 120, 130, 18, 84, 193, 130, 85, 197, 122, 52, 177, 79, 172, 145, 198, 51, 195, 204, 184, 212, 236, 38, 17, 98, 155, 203, 146, 13, 162, 77, 156, 200, 9, 168, 84, 168, 47, 192, 131, 49, 78, 227, 212, 169, 154, 139, 154, 94, 188, 26, 255, 178, 207, 119, 230, 159, 255, 216, 149, 214, 184, 25, 51, 95, 19, 206, 84, 247, 219, 180, 88, 35, 134, 35, 248, 241, 215, 143, 165, 4, 166, 209, 103, 162, 25, 40, 133, 8, 11, 224, 100, 63, 229, 194, 231, 1, 168, 174, 201, 222, 72, 66, 41, 9, 235, 152, 210, 246, 79, 18, 56, 67, 194, 68, 172, 85, 207, 12, 14, 164, 196, 73, 191, 61, 142, 98, 138, 52, 200, 72, 117, 46, 40, 97, 128, 37, 242, 121, 212, 32, 12, 95, 33, 123, 255, 106, 206, 250, 203, 117, 79, 63, 33, 127, 227, 99, 206, 125, 160, 42, 143, 135, 242, 182, 67, 153, 225, 17, 97, 33, 133, 77, 200, 254, 54, 70, 110, 227, 192, 253, 181, 158, 242, 88, 231, 97, 232, 23, 97, 168, 238, 61, 203, 183, 19, 9, 10, 207, 139, 197, 254, 31, 33, 201, 49, 214, 128, 4, 150, 54, 115, 118, 135, 170, 239, 86, 188, 115, 17, 55, 40, 241, 75, 106, 103, 42, 65, 199, 146, 161, 99, 76, 99, 80, 157, 223, 88, 41, 144, 26, 69, 54, 152, 56, 180, 194, 165, 205, 174, 117, 66, 75, 108, 237, 9, 208, 117, 172, 91, 203, 177, 158, 88, 223, 44, 249, 4, 249, 54, 181, 163, 70, 162, 33, 79, 243, 119, 51, 168, 231, 111, 183, 51, 155, 108, 61, 183, 213, 12, 223, 18, 9, 190, 118, 70, 13, 162, 145, 34, 95, 192, 164, 239, 153, 134, 16, 228, 233, 199, 215, 175, 6, 179, 86, 54, 58, 225, 173, 43, 237, 238, 84, 218, 49, 147, 124, 246, 2, 172, 113, 157, 139, 164, 125, 81, 34, 32, 28, 4, 210, 154, 83, 144, 106, 147, 156, 114, 83, 117, 126, 241, 102, 83, 129, 190, 169, 187, 15, 231, 71, 109, 93, 105, 111, 167, 210, 107, 187, 174, 238, 120, 138, 219, 91, 237, 222, 106, 181, 183, 194, 234, 170, 201, 14, 185, 4, 18, 178, 28, 240, 117, 84, 164, 53, 157, 7, 117, 82, 34, 217, 105, 26, 191, 3, 44, 102, 19, 213, 74, 5, 39, 118, 35, 114, 113, 46, 103, 57, 215, 51, 211, 15, 16, 113, 153, 28, 44, 115, 94, 172, 208, 95, 94, 150, 183, 50, 115, 3, 233, 68, 192, 98, 114, 91, 89, 73, 236, 22, 163, 59, 56, 36, 64, 131, 57, 113, 73, 91, 190, 91, 252, 19, 238, 220, 184, 123, 62, 43, 81, 110, 173, 119, 215, 214, 158, 38, 213, 187, 77, 249, 158, 57, 59, 210, 92, 152, 108, 254, 5, 204, 51, 184, 194, 194, 5, 240, 63, 28, 174, 123, 214, 217, 7, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 82, 65, 78, 195, 48, 16, 180, 235, 22, 250, 140, 72, 240, 2, 120, 65, 9, 170, 224, 192, 169, 226, 140, 172, 196, 138, 44, 185, 113, 228, 248, 64, 142, 254, 65, 108, 11, 113, 224, 130, 4, 105, 133, 90, 126, 193, 195, 104, 16, 78, 213, 22, 72, 44, 226, 147, 119, 237, 153, 157, 221, 29, 100, 244, 227, 243, 88, 171, 213, 133, 160, 140, 209, 36, 196, 140, 89, 96, 84, 53, 17, 2, 23, 214, 150, 218, 124, 4, 224, 239, 3, 97, 235, 23, 208, 141, 104, 208, 23, 17, 106, 39, 82, 139, 25, 77, 19, 70, 108, 105, 187, 180, 216, 69, 125, 63, 194, 172, 27, 254, 112, 116, 84, 139, 156, 103, 140, 28, 187, 203, 216, 194, 1, 42, 203, 205, 16, 214, 17, 175, 19, 247, 119, 209, 102, 101, 79, 170, 10, 121, 154, 75, 171, 22, 151, 84, 144, 72, 2, 181, 188, 78, 37, 73, 136, 120, 185, 61, 63, 107, 45, 138, 246, 241, 208, 15, 15, 212, 186, 118, 78, 140, 37, 14, 121, 86, 52, 52, 193, 86, 79, 67, 252, 63, 165, 193, 62, 30, 249, 225, 15, 234, 15, 125, 39, 245, 115, 167, 112, 171, 167, 33, 94, 77, 185, 32, 52, 73, 107, 192, 195, 242, 123, 95, 86, 189, 93, 17, 156, 125, 173, 216, 52, 67, 64, 234, 253, 134, 204, 185, 40, 38, 113, 44, 72, 158, 187, 135, 147, 95, 242, 167, 206, 37, 70, 59, 107, 84, 83, 74, 88, 140, 118, 195, 221, 200, 120, 213, 134, 166, 99, 141, 158, 55, 234, 233, 189, 145, 122, 157, 73, 158, 105, 115, 232, 181, 79, 89, 23, 97, 253, 222, 4, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack,  @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 211, 205, 74, 3, 49, 16, 0, 224, 36, 251, 34, 30, 245, 166, 248, 4, 82, 4, 79, 226, 81, 4, 9, 105, 54, 74, 32, 155, 196, 252, 84, 61, 174, 61, 120, 221, 159, 23, 40, 42, 150, 10, 69, 84, 252, 187, 251, 34, 189, 121, 244, 226, 221, 88, 88, 89, 169, 212, 181, 8, 205, 41, 76, 134, 48, 243, 77, 130, 78, 6, 123, 94, 82, 199, 149, 180, 229, 233, 176, 218, 99, 73, 18, 214, 187, 167, 222, 24, 38, 29, 62, 228, 78, 50, 107, 49, 151, 49, 59, 138, 250, 74, 83, 21, 51, 91, 166, 23, 45, 34, 68, 247, 140, 199, 240, 146, 75, 237, 157, 45, 0, 236, 43, 239, 62, 183, 57, 154, 126, 28, 165, 87, 107, 214, 50, 227, 118, 152, 81, 221, 65, 226, 5, 118, 204, 36, 54, 187, 19, 92, 50, 98, 48, 85, 73, 155, 75, 50, 46, 173, 40, 94, 22, 192, 244, 5, 81, 200, 89, 142, 55, 205, 104, 165, 183, 244, 180, 181, 254, 144, 166, 219, 187, 139, 171, 175, 27, 199, 207, 58, 111, 141, 222, 203, 183, 144, 20, 157, 31, 96, 250, 235, 85, 224, 86, 27, 222, 33, 142, 97, 77, 76, 128, 8, 117, 217, 28, 220, 104, 223, 22, 156, 214, 99, 112, 104, 152, 243, 70, 226, 14, 17, 158, 217, 236, 154, 140, 59, 194, 73, 192, 34, 251, 33, 240, 71, 209, 226, 191, 77, 64, 3, 19, 212, 200, 164, 54, 77, 84, 77, 19, 213, 166, 249, 3, 89, 152, 246, 164, 89, 246, 157, 44, 143, 102, 52, 131, 149, 89, 62, 15, 51, 56, 243, 59, 106, 132, 2, 38, 80, 30, 3, 73, 168, 223, 25, 18, 154, 138, 241, 215, 159, 205, 62, 0, 203, 73, 65, 11, 193, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 144, 45, 10, 2, 81, 20, 133, 239, 125, 111, 35, 70, 109, 138, 43, 144, 65, 48, 137, 81, 4, 131, 224, 180, 1, 229, 141, 197, 248, 118, 112, 127, 130, 117, 130, 201, 5, 136, 118, 55, 50, 205, 104, 177, 123, 17, 4, 147, 207, 9, 115, 218, 129, 195, 225, 227, 243, 162, 135, 202, 107, 60, 102, 171, 162, 80, 20, 64, 118, 223, 197, 199, 211, 168, 44, 243, 176, 91, 228, 97, 163, 36, 114, 235, 192, 239, 160, 179, 77, 127, 61, 13, 245, 160, 234, 93, 102, 227, 115, 140, 243, 101, 119, 120, 159, 236, 175, 91, 206, 234, 167, 62, 108, 228, 147, 55, 192, 192, 72, 100, 112, 210, 28, 1, 254, 64, 112, 201, 155, 143, 8, 39, 206, 68, 152, 13, 98, 111, 64, 200, 237, 0, 97, 218, 201, 155, 1, 136, 94, 214, 235, 74, 112, 182, 1, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
}
