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
    let circuit = Circuit::from_str(src).unwrap();

    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 144, 187, 74, 3, 81, 16, 134, 179, 187, 47, 98, 169, 157, 226, 19, 136, 8, 86, 98, 41, 130, 12, 39, 103, 71, 57, 176, 231, 226, 204, 57, 81, 203, 213, 194, 118, 55, 121, 129, 128, 133, 68, 8, 162, 226, 173, 247, 69, 210, 89, 218, 216, 59, 4, 12, 169, 146, 76, 53, 12, 63, 63, 243, 125, 249, 245, 232, 52, 57, 29, 141, 119, 220, 222, 142, 255, 119, 112, 202, 226, 240, 85, 39, 34, 116, 17, 46, 76, 116, 200, 12, 198, 149, 120, 89, 220, 251, 160, 125, 137, 220, 214, 15, 59, 204, 72, 241, 24, 201, 223, 140, 108, 170, 32, 34, 89, 110, 94, 42, 227, 80, 17, 104, 111, 187, 198, 169, 105, 249, 160, 255, 181, 214, 89, 60, 89, 182, 66, 38, 151, 204, 102, 121, 64, 147, 173, 225, 198, 199, 225, 222, 91, 93, 31, 157, 172, 111, 127, 239, 95, 125, 134, 118, 119, 242, 59, 248, 145, 80, 113, 119, 14, 122, 105, 85, 231, 57, 144, 233, 169, 136, 16, 20, 9, 174, 252, 206, 253, 44, 127, 10, 169, 91, 25, 61, 119, 108, 198, 132, 49, 145, 131, 158, 170, 146, 96, 23, 143, 106, 138, 13, 86, 156, 168, 51, 228, 230, 93, 180, 9, 99, 36, 37, 224, 37, 204, 140, 54, 127, 135, 11, 68, 245, 95, 1, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 204, 49, 14, 64, 48, 24, 64, 97, 109, 47, 98, 100, 35, 78, 32, 34, 49, 137, 81, 36, 54, 157, 73, 107, 49, 246, 6, 253, 219, 193, 218, 217, 1, 132, 221, 69, 186, 25, 45, 118, 61, 1, 222, 252, 229, 17, 5, 179, 33, 32, 150, 148, 115, 202, 198, 134, 178, 94, 75, 173, 14, 223, 123, 15, 161, 31, 6, 59, 19, 117, 37, 179, 177, 9, 183, 42, 95, 133, 168, 219, 32, 57, 139, 105, 31, 32, 179, 183, 190, 28, 34, 159, 27, 79, 33, 44, 129, 72, 249, 0, 38, 129, 97, 236, 172, 0, 0, 0]");

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
    let circuit = Circuit::from_str(src).unwrap();

    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 144, 205, 74, 3, 65, 16, 132, 201, 230, 71, 243, 90, 17, 188, 121, 242, 224, 113, 104, 103, 91, 105, 156, 244, 12, 221, 61, 49, 215, 21, 193, 235, 70, 193, 179, 39, 151, 28, 226, 31, 136, 175, 231, 16, 113, 200, 237, 163, 138, 42, 186, 171, 185, 219, 94, 101, 246, 70, 145, 117, 243, 176, 251, 103, 199, 176, 196, 151, 111, 159, 69, 144, 205, 221, 146, 49, 170, 58, 226, 22, 215, 243, 33, 38, 31, 91, 212, 77, 247, 177, 8, 224, 111, 22, 113, 125, 90, 114, 39, 16, 66, 247, 118, 150, 131, 209, 185, 135, 0, 82, 240, 254, 53, 69, 98, 211, 167, 110, 184, 248, 235, 24, 85, 106, 42, 141, 7, 221, 7, 244, 177, 74, 147, 74, 211, 109, 18, 108, 201, 131, 97, 213, 102, 67, 204, 150, 114, 233, 61, 58, 158, 127, 37, 161, 85, 113, 93, 2, 41, 87, 27, 138, 62, 143, 154, 241, 100, 58, 251, 76, 249, 50, 144, 63, 48, 250, 157, 160, 101, 97, 183, 130, 144, 113, 31, 127, 7, 85, 20, 115, 203, 210, 11, 215, 168, 253, 79, 249, 165, 172, 97, 2, 196, 216, 186, 58, 79, 255, 11, 38, 112, 144, 200, 44, 1, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 61, 198, 171, 10, 128, 48, 20, 0, 80, 116, 62, 127, 107, 130, 205, 100, 48, 143, 97, 24, 94, 38, 184, 13, 172, 251, 131, 61, 68, 171, 205, 38, 126, 162, 90, 110, 59, 36, 248, 227, 172, 189, 125, 40, 48, 62, 209, 121, 109, 141, 228, 13, 3, 176, 119, 103, 64, 139, 158, 51, 96, 203, 199, 45, 218, 107, 16, 90, 142, 74, 37, 168, 20, 69, 2, 50, 67, 229, 168, 34, 150, 85, 189, 39, 41, 201, 242, 194, 253, 118, 238, 5, 128, 123, 246, 56, 122, 0, 0, 0]");

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
    let circuit = Circuit::from_str(&src).unwrap();

    let program =
        Program { functions: vec![circuit], unconstrained_functions: vec![brillig_bytecode] };

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 146, 207, 78, 27, 49, 16, 198, 179, 251, 36, 125, 134, 246, 9, 218, 84, 72, 61, 244, 132, 122, 182, 28, 123, 178, 26, 201, 107, 187, 227, 217, 180, 233, 109, 147, 34, 174, 1, 137, 59, 2, 242, 79, 155, 128, 0, 33, 94, 128, 7, 195, 27, 72, 8, 145, 72, 144, 216, 211, 106, 108, 127, 223, 204, 111, 190, 180, 55, 109, 23, 86, 49, 58, 27, 142, 14, 231, 203, 127, 97, 101, 14, 167, 119, 170, 32, 2, 203, 226, 15, 178, 133, 16, 4, 90, 13, 127, 211, 177, 243, 202, 105, 8, 71, 101, 245, 141, 208, 24, 204, 154, 210, 152, 255, 103, 168, 27, 35, 180, 190, 224, 120, 50, 218, 71, 155, 25, 232, 79, 243, 194, 8, 6, 202, 195, 224, 214, 160, 5, 73, 66, 185, 188, 133, 86, 62, 89, 30, 63, 124, 106, 108, 255, 146, 228, 252, 183, 80, 59, 175, 53, 198, 174, 224, 149, 119, 238, 13, 164, 83, 79, 160, 81, 73, 222, 217, 198, 224, 93, 22, 201, 141, 39, 236, 68, 57, 225, 37, 69, 62, 81, 47, 28, 39, 233, 181, 47, 90, 6, 213, 90, 113, 48, 39, 224, 130, 172, 232, 72, 83, 64, 24, 92, 201, 16, 128, 88, 228, 145, 161, 204, 98, 225, 62, 98, 142, 182, 76, 50, 246, 162, 197, 203, 6, 122, 175, 55, 80, 161, 237, 212, 15, 85, 228, 59, 105, 117, 25, 106, 238, 39, 229, 176, 89, 63, 238, 87, 113, 7, 252, 60, 66, 57, 250, 142, 4, 138, 147, 73, 11, 89, 4, 252, 7, 229, 248, 135, 101, 200, 128, 206, 127, 125, 249, 60, 92, 116, 178, 27, 226, 54, 233, 244, 67, 210, 73, 57, 171, 83, 162, 37, 203, 166, 243, 221, 254, 237, 154, 131, 144, 90, 83, 100, 179, 116, 106, 204, 106, 151, 205, 106, 122, 233, 218, 237, 0, 188, 89, 79, 202, 106, 207, 17, 96, 102, 107, 131, 131, 201, 146, 224, 232, 9, 222, 108, 205, 40, 134, 99, 254, 19, 114, 71, 221, 175, 27, 142, 247, 235, 237, 44, 38, 18, 220, 245, 240, 146, 166, 225, 30, 130, 209, 171, 124, 191, 33, 115, 189, 56, 223, 34, 80, 94, 236, 179, 243, 101, 245, 28, 144, 26, 71, 111, 236, 29, 70, 158, 180, 26, 232, 98, 1, 121, 57, 246, 35, 154, 225, 87, 167, 163, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 144, 79, 10, 130, 64, 20, 198, 29, 187, 72, 103, 168, 19, 148, 17, 180, 104, 37, 173, 67, 244, 33, 3, 227, 140, 140, 67, 224, 114, 110, 48, 51, 38, 109, 131, 210, 22, 214, 45, 58, 88, 138, 52, 66, 139, 172, 183, 254, 254, 252, 222, 55, 49, 250, 116, 118, 181, 108, 150, 28, 19, 130, 99, 47, 32, 228, 232, 104, 89, 251, 152, 198, 4, 10, 165, 205, 115, 234, 124, 63, 132, 70, 37, 125, 98, 146, 18, 112, 11, 165, 198, 19, 13, 114, 149, 106, 187, 27, 76, 15, 192, 197, 62, 108, 177, 74, 89, 121, 140, 102, 162, 144, 245, 10, 115, 8, 5, 146, 183, 13, 21, 16, 3, 191, 236, 230, 179, 113, 134, 79, 191, 251, 159, 31, 201, 123, 183, 78, 20, 136, 192, 99, 105, 110, 99, 156, 33, 207, 130, 53, 107, 198, 1, 199, 180, 51, 148, 117, 255, 132, 150, 143, 45, 36, 140, 231, 139, 40, 226, 144, 101, 214, 111, 183, 169, 214, 24, 72, 244, 171, 78, 94, 125, 193, 82, 109, 134, 214, 55, 199, 11, 1, 67, 107, 124, 213, 1, 0, 0]");

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
    let circuit = Circuit::from_str(src).unwrap();

    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 205, 147, 75, 78, 195, 48, 16, 134, 147, 52, 7, 225, 58, 44, 16, 71, 24, 57, 206, 128, 44, 226, 7, 51, 118, 161, 203, 54, 11, 182, 73, 123, 1, 196, 179, 169, 84, 33, 64, 136, 11, 112, 48, 76, 34, 30, 43, 218, 5, 66, 245, 198, 246, 120, 244, 255, 159, 70, 191, 179, 89, 119, 20, 140, 244, 202, 26, 110, 47, 214, 159, 103, 48, 66, 227, 229, 139, 12, 68, 104, 60, 156, 41, 111, 144, 25, 148, 41, 241, 60, 191, 183, 78, 218, 18, 121, 49, 93, 29, 160, 182, 52, 217, 55, 202, 215, 203, 162, 178, 242, 4, 84, 153, 220, 168, 120, 159, 167, 217, 106, 168, 248, 137, 195, 187, 161, 113, 186, 28, 246, 67, 55, 251, 110, 191, 178, 174, 238, 172, 67, 18, 31, 206, 117, 167, 67, 5, 30, 73, 115, 243, 92, 41, 131, 130, 64, 90, 93, 40, 211, 63, 115, 115, 125, 10, 242, 109, 47, 249, 125, 165, 183, 61, 234, 95, 137, 141, 69, 21, 112, 147, 88, 59, 223, 172, 52, 218, 202, 48, 249, 183, 57, 37, 59, 58, 167, 124, 59, 250, 39, 71, 106, 44, 60, 130, 19, 20, 3, 27, 61, 121, 145, 102, 163, 71, 23, 138, 74, 201, 31, 213, 102, 77, 232, 3, 25, 232, 9, 185, 205, 31, 4, 51, 146, 7, 29, 99, 45, 142, 145, 155, 215, 152, 252, 8, 231, 73, 68, 226, 18, 190, 62, 69, 243, 14, 26, 99, 194, 150, 34, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 142, 59, 10, 128, 48, 16, 68, 55, 217, 61, 136, 215, 177, 16, 79, 97, 97, 225, 7, 177, 177, 204, 13, 246, 83, 216, 90, 137, 120, 14, 15, 102, 145, 222, 68, 200, 52, 195, 192, 131, 121, 168, 178, 31, 100, 225, 110, 186, 97, 90, 182, 122, 236, 87, 3, 117, 254, 140, 59, 92, 177, 219, 89, 193, 140, 249, 169, 224, 59, 46, 19, 18, 77, 83, 152, 68, 224, 183, 31, 20, 244, 163, 140, 55, 231, 145, 133, 152, 95, 174, 253, 29, 158, 104, 1, 0, 0]");

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
