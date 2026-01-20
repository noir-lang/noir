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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 144, 61, 75, 3, 65, 16, 134, 115, 151, 63, 98, 169, 157, 226, 47, 16, 17, 172, 196, 82, 4, 89, 38, 123, 163, 44, 220, 126, 56, 179, 27, 176, 60, 5, 235, 187, 228, 15, 4, 44, 36, 66, 144, 36, 144, 143, 62, 127, 36, 93, 202, 52, 233, 179, 4, 18, 82, 37, 153, 106, 24, 30, 94, 230, 121, 211, 207, 238, 107, 48, 210, 43, 107, 184, 250, 238, 109, 119, 97, 64, 99, 231, 215, 58, 105, 51, 228, 170, 248, 187, 97, 70, 242, 207, 72, 246, 171, 171, 67, 46, 60, 146, 230, 114, 152, 43, 131, 64, 66, 90, 221, 80, 6, 54, 41, 237, 214, 236, 172, 118, 120, 146, 228, 4, 38, 141, 204, 101, 246, 64, 243, 171, 206, 197, 228, 241, 110, 84, 20, 79, 47, 231, 215, 139, 251, 143, 169, 171, 110, 231, 171, 246, 50, 66, 245, 159, 119, 33, 143, 70, 213, 6, 142, 84, 19, 60, 10, 7, 20, 189, 226, 239, 220, 74, 210, 190, 11, 141, 92, 201, 189, 99, 217, 35, 244, 129, 140, 104, 66, 30, 162, 118, 253, 31, 54, 218, 66, 35, 51, 188, 33, 151, 227, 216, 79, 116, 244, 4, 81, 60, 19, 187, 234, 202, 53, 135, 201, 70, 16, 72, 1, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 204, 49, 14, 64, 48, 24, 64, 97, 109, 47, 98, 100, 35, 78, 32, 34, 49, 137, 81, 36, 54, 157, 73, 107, 49, 246, 6, 253, 219, 196, 220, 217, 1, 132, 221, 69, 186, 25, 45, 118, 61, 1, 222, 252, 229, 17, 5, 179, 1, 177, 164, 156, 83, 54, 54, 148, 245, 90, 106, 117, 248, 222, 123, 8, 253, 48, 216, 153, 168, 43, 153, 141, 77, 184, 85, 249, 42, 68, 221, 6, 201, 89, 76, 251, 0, 153, 189, 245, 229, 16, 249, 220, 120, 10, 97, 9, 68, 202, 7, 63, 182, 26, 15, 171, 0, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 144, 189, 138, 66, 49, 16, 133, 241, 250, 179, 187, 175, 229, 194, 118, 91, 109, 177, 101, 24, 115, 71, 9, 198, 36, 204, 76, 196, 246, 138, 88, 95, 21, 172, 173, 188, 88, 248, 219, 248, 122, 6, 193, 96, 247, 49, 103, 206, 97, 206, 20, 243, 195, 48, 58, 45, 198, 59, 94, 45, 143, 47, 86, 14, 38, 184, 107, 124, 208, 190, 68, 94, 85, 151, 190, 5, 61, 238, 251, 217, 79, 90, 248, 6, 107, 171, 211, 111, 180, 98, 254, 52, 88, 160, 132, 139, 125, 240, 198, 9, 111, 170, 230, 223, 136, 67, 230, 86, 166, 34, 83, 187, 225, 167, 129, 215, 121, 212, 201, 212, 61, 4, 194, 210, 104, 16, 204, 179, 94, 227, 163, 132, 152, 114, 63, 62, 191, 110, 129, 204, 52, 169, 42, 0, 165, 243, 4, 137, 183, 173, 162, 221, 233, 246, 174, 33, 14, 172, 209, 111, 66, 125, 36, 148, 72, 78, 77, 193, 70, 124, 218, 207, 192, 140, 36, 106, 146, 114, 97, 132, 92, 223, 83, 151, 84, 91, 8, 140, 195, 82, 229, 63, 212, 15, 176, 100, 253, 143, 21, 1, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 61, 198, 187, 10, 128, 32, 20, 0, 80, 202, 158, 191, 101, 208, 214, 212, 208, 44, 210, 32, 93, 12, 82, 161, 213, 63, 240, 17, 206, 109, 109, 209, 39, 86, 203, 221, 14, 9, 62, 157, 222, 62, 20, 24, 95, 232, 186, 247, 70, 242, 142, 1, 216, 123, 48, 160, 197, 200, 25, 176, 237, 227, 17, 237, 53, 9, 45, 103, 165, 50, 84, 142, 34, 1, 89, 160, 74, 84, 21, 235, 166, 77, 89, 78, 138, 178, 114, 191, 157, 123, 1, 110, 20, 217, 141, 121, 0, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 146, 205, 78, 2, 49, 20, 133, 153, 121, 18, 159, 65, 159, 64, 49, 36, 46, 92, 17, 215, 77, 153, 185, 76, 110, 210, 105, 107, 123, 135, 4, 119, 3, 234, 26, 72, 220, 27, 249, 207, 128, 6, 89, 240, 2, 62, 152, 29, 254, 37, 17, 76, 236, 170, 185, 109, 207, 185, 247, 235, 241, 27, 227, 106, 34, 3, 66, 37, 109, 251, 121, 186, 217, 51, 201, 99, 120, 29, 42, 29, 168, 16, 108, 59, 205, 174, 12, 10, 129, 81, 145, 11, 241, 248, 134, 97, 97, 128, 82, 39, 228, 78, 6, 101, 148, 145, 128, 230, 56, 78, 4, 35, 48, 177, 109, 205, 5, 74, 224, 134, 5, 42, 174, 160, 228, 43, 237, 206, 215, 89, 225, 248, 242, 188, 238, 61, 11, 78, 94, 43, 12, 85, 66, 91, 239, 88, 11, 240, 199, 218, 64, 136, 1, 167, 147, 109, 180, 254, 100, 225, 125, 106, 131, 53, 39, 199, 52, 55, 14, 132, 211, 179, 29, 207, 159, 233, 164, 34, 48, 216, 43, 182, 166, 6, 40, 49, 146, 213, 184, 72, 192, 182, 62, 184, 181, 96, 136, 197, 96, 45, 143, 92, 97, 225, 120, 58, 91, 50, 220, 245, 18, 178, 29, 234, 198, 79, 212, 25, 202, 90, 254, 48, 112, 124, 71, 149, 58, 65, 206, 253, 37, 237, 23, 243, 199, 205, 204, 253, 1, 173, 71, 72, 7, 215, 104, 32, 32, 111, 84, 65, 98, 22, 31, 32, 29, 222, 72, 130, 8, 76, 247, 238, 226, 188, 191, 236, 228, 52, 196, 99, 210, 254, 191, 164, 189, 116, 146, 167, 36, 228, 196, 139, 74, 215, 155, 243, 61, 7, 198, 195, 208, 56, 54, 27, 167, 194, 36, 119, 57, 172, 250, 239, 170, 90, 181, 64, 135, 117, 47, 205, 74, 202, 0, 70, 50, 55, 120, 26, 109, 8, 14, 86, 240, 38, 123, 70, 46, 28, 211, 91, 136, 149, 169, 95, 30, 56, 46, 246, 219, 89, 78, 196, 168, 174, 97, 151, 166, 126, 9, 65, 132, 219, 124, 255, 34, 51, 91, 158, 31, 17, 72, 123, 101, 82, 58, 205, 214, 1, 201, 113, 52, 134, 90, 161, 227, 105, 182, 3, 245, 150, 144, 55, 99, 127, 3, 222, 119, 30, 14, 140, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 144, 63, 14, 130, 48, 20, 198, 41, 94, 196, 51, 232, 9, 20, 99, 226, 224, 68, 156, 13, 129, 23, 210, 164, 180, 164, 52, 38, 140, 189, 65, 91, 196, 217, 68, 193, 1, 189, 133, 7, 19, 66, 44, 137, 131, 232, 155, 191, 63, 191, 247, 77, 140, 62, 157, 181, 108, 150, 28, 19, 130, 99, 47, 32, 228, 232, 104, 89, 251, 152, 198, 4, 10, 165, 205, 115, 234, 124, 63, 132, 70, 37, 125, 98, 146, 18, 112, 11, 165, 198, 19, 13, 114, 149, 106, 187, 27, 76, 15, 192, 197, 62, 108, 177, 74, 89, 121, 140, 102, 162, 144, 245, 10, 115, 8, 5, 146, 183, 13, 21, 16, 3, 191, 236, 230, 179, 113, 134, 79, 191, 251, 159, 31, 201, 123, 183, 78, 20, 136, 192, 99, 105, 110, 99, 156, 33, 207, 130, 53, 107, 198, 1, 199, 180, 51, 148, 117, 255, 132, 150, 143, 45, 36, 140, 231, 139, 40, 226, 144, 101, 214, 111, 183, 169, 214, 24, 72, 244, 171, 78, 94, 125, 193, 82, 109, 134, 214, 55, 199, 11, 140, 230, 138, 87, 212, 1, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 85, 205, 110, 211, 64, 24, 180, 227, 20, 120, 140, 72, 240, 4, 240, 4, 37, 168, 130, 3, 167, 138, 243, 106, 99, 127, 177, 86, 90, 239, 46, 187, 235, 10, 115, 219, 4, 56, 231, 231, 200, 5, 209, 38, 78, 228, 4, 84, 122, 232, 11, 240, 96, 172, 67, 156, 58, 85, 147, 88, 77, 127, 114, 114, 62, 217, 51, 243, 141, 103, 214, 181, 206, 180, 29, 51, 95, 19, 206, 84, 255, 219, 188, 184, 70, 12, 71, 240, 35, 229, 194, 231, 1, 168, 190, 201, 94, 75, 66, 41, 9, 155, 152, 210, 47, 63, 73, 224, 140, 9, 19, 177, 86, 3, 51, 58, 148, 18, 39, 195, 238, 52, 138, 41, 210, 32, 35, 213, 187, 160, 132, 1, 150, 200, 231, 81, 139, 48, 252, 31, 123, 240, 183, 225, 108, 255, 185, 238, 233, 71, 228, 239, 188, 205, 185, 11, 170, 218, 195, 81, 121, 213, 168, 204, 248, 152, 176, 144, 194, 46, 202, 97, 21, 35, 171, 56, 112, 119, 210, 83, 30, 235, 60, 12, 195, 34, 12, 245, 131, 39, 249, 58, 145, 160, 240, 180, 184, 120, 54, 21, 18, 2, 226, 99, 189, 115, 197, 94, 37, 90, 247, 143, 144, 228, 196, 194, 33, 129, 165, 77, 171, 197, 83, 67, 183, 230, 157, 139, 184, 69, 137, 95, 154, 246, 230, 18, 116, 44, 25, 58, 193, 52, 6, 213, 251, 141, 149, 2, 169, 81, 4, 74, 225, 208, 14, 46, 109, 234, 45, 175, 150, 216, 138, 9, 208, 85, 33, 58, 235, 133, 152, 89, 149, 118, 149, 79, 200, 183, 53, 152, 180, 18, 13, 121, 61, 190, 155, 81, 51, 127, 186, 155, 217, 170, 232, 229, 18, 102, 252, 134, 72, 240, 181, 51, 105, 17, 141, 20, 249, 12, 38, 125, 199, 52, 132, 32, 79, 63, 188, 122, 57, 90, 72, 217, 185, 163, 183, 13, 218, 221, 11, 218, 49, 179, 188, 204, 1, 214, 184, 201, 69, 210, 189, 40, 49, 32, 28, 4, 210, 154, 83, 48, 53, 102, 57, 203, 245, 169, 243, 139, 183, 219, 10, 244, 245, 185, 123, 127, 126, 52, 182, 65, 123, 123, 65, 111, 85, 93, 223, 243, 45, 86, 183, 218, 189, 209, 106, 111, 131, 213, 117, 147, 29, 113, 9, 36, 100, 57, 193, 215, 73, 145, 214, 116, 25, 212, 89, 137, 201, 214, 115, 250, 22, 176, 88, 84, 180, 147, 10, 78, 236, 34, 114, 245, 94, 206, 114, 94, 207, 204, 223, 67, 196, 101, 114, 184, 206, 243, 124, 195, 252, 197, 101, 121, 149, 133, 27, 72, 39, 2, 86, 71, 65, 39, 43, 13, 251, 197, 89, 48, 58, 34, 64, 131, 37, 227, 218, 108, 253, 223, 234, 35, 115, 107, 225, 238, 249, 2, 162, 44, 109, 112, 91, 105, 143, 147, 234, 253, 90, 126, 96, 206, 142, 53, 23, 38, 91, 158, 128, 121, 6, 55, 88, 184, 34, 252, 7, 159, 7, 118, 138, 19, 8, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 82, 75, 78, 195, 48, 20, 140, 235, 22, 56, 70, 36, 56, 1, 156, 160, 4, 85, 176, 96, 85, 177, 70, 86, 98, 69, 150, 220, 56, 114, 188, 32, 75, 223, 192, 31, 96, 195, 6, 9, 210, 10, 181, 220, 130, 131, 209, 32, 156, 170, 45, 144, 88, 196, 43, 255, 102, 222, 188, 121, 3, 141, 126, 124, 214, 114, 121, 206, 9, 165, 36, 141, 16, 165, 247, 129, 145, 213, 152, 115, 84, 90, 171, 180, 249, 8, 131, 191, 23, 0, 173, 95, 130, 110, 68, 131, 190, 136, 96, 59, 145, 156, 79, 73, 150, 82, 108, 149, 237, 210, 98, 23, 245, 253, 8, 179, 206, 252, 225, 232, 160, 22, 57, 203, 41, 62, 116, 155, 35, 171, 84, 123, 21, 11, 6, 80, 169, 181, 83, 171, 152, 213, 168, 187, 219, 120, 61, 215, 39, 89, 69, 44, 43, 132, 149, 243, 11, 194, 113, 44, 2, 185, 184, 202, 4, 78, 49, 127, 185, 57, 59, 109, 165, 133, 187, 120, 224, 135, 15, 228, 170, 142, 87, 130, 4, 138, 88, 94, 54, 52, 225, 70, 79, 67, 252, 63, 165, 225, 46, 30, 250, 225, 247, 234, 15, 125, 157, 250, 185, 83, 176, 209, 211, 16, 47, 39, 140, 99, 146, 102, 53, 224, 97, 241, 61, 47, 43, 223, 46, 49, 202, 191, 114, 96, 26, 19, 160, 124, 191, 198, 51, 198, 203, 113, 146, 112, 92, 20, 238, 225, 248, 151, 251, 19, 23, 37, 163, 93, 126, 170, 9, 193, 52, 129, 219, 199, 237, 147, 241, 170, 13, 76, 199, 26, 61, 79, 212, 51, 123, 35, 249, 58, 21, 44, 215, 102, 63, 107, 159, 173, 62, 187, 243, 2, 5, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 205, 82, 75, 78, 2, 65, 16, 157, 25, 56, 136, 215, 113, 97, 60, 66, 167, 167, 167, 52, 29, 251, 103, 85, 55, 145, 37, 76, 226, 186, 7, 46, 96, 252, 50, 36, 196, 168, 11, 47, 224, 193, 108, 153, 8, 172, 128, 5, 33, 212, 166, 190, 121, 239, 165, 242, 138, 113, 123, 21, 140, 240, 210, 26, 106, 238, 23, 255, 53, 51, 92, 195, 195, 155, 117, 194, 86, 64, 211, 209, 252, 2, 180, 197, 225, 185, 145, 190, 158, 149, 202, 138, 27, 38, 171, 236, 89, 166, 126, 146, 23, 243, 110, 226, 135, 14, 94, 187, 195, 209, 172, 203, 151, 110, 188, 62, 127, 180, 174, 110, 173, 3, 228, 127, 20, 117, 171, 131, 98, 30, 80, 83, 252, 82, 210, 0, 71, 38, 172, 46, 165, 89, 174, 41, 62, 221, 50, 241, 115, 150, 109, 143, 252, 69, 154, 10, 238, 14, 5, 54, 224, 42, 192, 46, 176, 102, 178, 27, 169, 183, 23, 97, 118, 180, 63, 101, 39, 250, 167, 254, 126, 234, 63, 29, 202, 1, 247, 192, 28, 199, 228, 204, 196, 73, 211, 188, 232, 125, 184, 80, 42, 41, 54, 166, 113, 129, 224, 3, 26, 182, 84, 72, 77, 255, 157, 19, 1, 122, 166, 129, 136, 95, 3, 197, 239, 100, 241, 36, 206, 35, 79, 138, 43, 182, 114, 127, 252, 5, 89, 70, 229, 125, 11, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 142, 59, 10, 128, 48, 16, 68, 119, 179, 57, 136, 215, 177, 16, 79, 97, 97, 225, 7, 177, 177, 204, 13, 246, 3, 214, 86, 34, 158, 195, 131, 89, 164, 55, 17, 50, 205, 48, 240, 96, 30, 169, 236, 135, 133, 187, 233, 134, 105, 217, 234, 177, 95, 13, 20, 221, 25, 119, 184, 98, 183, 179, 130, 25, 243, 83, 193, 119, 48, 19, 18, 77, 83, 148, 68, 224, 183, 31, 20, 244, 243, 25, 111, 232, 136, 197, 51, 191, 21, 82, 121, 98, 103, 1, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack,  @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 148, 205, 74, 3, 49, 16, 199, 179, 217, 23, 241, 168, 55, 197, 39, 144, 34, 120, 18, 143, 34, 72, 72, 179, 81, 2, 217, 36, 78, 146, 130, 199, 85, 241, 188, 31, 47, 80, 84, 44, 43, 20, 81, 193, 143, 187, 47, 210, 155, 71, 47, 222, 141, 133, 250, 65, 181, 93, 164, 98, 78, 195, 48, 204, 204, 239, 63, 127, 130, 15, 234, 29, 175, 152, 19, 90, 217, 234, 184, 63, 138, 137, 162, 41, 239, 246, 180, 97, 58, 225, 182, 202, 206, 90, 84, 202, 163, 19, 145, 68, 231, 66, 25, 239, 108, 137, 162, 158, 246, 238, 45, 44, 112, 109, 128, 39, 130, 81, 199, 15, 235, 212, 75, 226, 56, 164, 54, 191, 145, 66, 113, 10, 132, 233, 180, 45, 20, 29, 142, 200, 79, 247, 8, 123, 156, 67, 147, 95, 52, 121, 94, 60, 251, 121, 23, 43, 214, 114, 112, 91, 28, 244, 180, 150, 101, 57, 189, 31, 14, 53, 139, 201, 58, 12, 150, 186, 11, 247, 27, 171, 183, 89, 182, 185, 61, 191, 252, 180, 182, 255, 96, 138, 214, 224, 165, 122, 14, 69, 113, 163, 213, 208, 181, 1, 209, 9, 164, 196, 80, 8, 55, 9, 123, 217, 2, 93, 25, 223, 150, 130, 125, 206, 69, 125, 224, 206, 131, 34, 29, 42, 61, 183, 249, 37, 29, 18, 145, 148, 91, 75, 119, 67, 226, 167, 227, 150, 179, 134, 71, 13, 224, 113, 35, 120, 252, 225, 3, 60, 242, 1, 254, 75, 31, 124, 35, 118, 48, 222, 184, 218, 249, 87, 177, 139, 184, 177, 218, 197, 127, 168, 29, 253, 218, 106, 141, 232, 209, 24, 253, 93, 96, 15, 251, 59, 160, 1, 42, 33, 239, 63, 76, 254, 10, 36, 156, 46, 45, 111, 4, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 144, 161, 10, 2, 65, 20, 69, 223, 188, 249, 17, 163, 54, 197, 47, 144, 69, 48, 137, 81, 4, 131, 224, 182, 5, 101, 214, 98, 156, 63, 120, 239, 141, 108, 158, 96, 242, 3, 68, 187, 63, 178, 205, 104, 177, 59, 136, 193, 180, 51, 27, 246, 230, 123, 47, 135, 163, 197, 85, 222, 217, 115, 182, 41, 138, 147, 18, 80, 140, 142, 232, 209, 131, 230, 168, 255, 133, 78, 91, 92, 38, 101, 153, 155, 195, 42, 55, 59, 71, 34, 241, 5, 134, 206, 112, 59, 55, 245, 200, 15, 110, 139, 233, 213, 218, 229, 186, 63, 126, 206, 142, 247, 61, 103, 245, 219, 189, 66, 73, 71, 111, 128, 129, 21, 81, 229, 165, 61, 1, 36, 16, 96, 244, 6, 127, 178, 80, 48, 85, 86, 208, 74, 172, 3, 52, 119, 3, 173, 226, 218, 190, 8, 64, 244, 1, 164, 224, 142, 129, 34, 2, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
}
