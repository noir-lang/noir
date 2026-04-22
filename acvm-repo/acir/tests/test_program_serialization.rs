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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 144, 187, 74, 3, 81, 16, 134, 119, 55, 47, 98, 169, 157, 226, 19, 136, 8, 86, 98, 41, 130, 12, 39, 103, 71, 57, 176, 231, 226, 204, 57, 81, 203, 213, 194, 118, 55, 121, 129, 128, 133, 68, 8, 162, 226, 173, 247, 69, 210, 89, 218, 216, 59, 4, 12, 169, 146, 76, 53, 12, 63, 63, 243, 125, 197, 245, 232, 52, 57, 29, 141, 119, 220, 222, 142, 255, 119, 112, 202, 226, 240, 85, 39, 34, 116, 17, 46, 76, 116, 200, 12, 198, 149, 120, 153, 221, 251, 160, 125, 137, 220, 214, 15, 59, 204, 72, 241, 24, 201, 223, 140, 108, 170, 32, 34, 89, 110, 94, 42, 227, 80, 17, 104, 111, 187, 198, 169, 105, 249, 160, 255, 181, 150, 45, 158, 60, 95, 33, 83, 72, 102, 179, 60, 160, 201, 214, 112, 227, 227, 112, 239, 173, 174, 143, 78, 214, 183, 191, 247, 175, 62, 67, 187, 59, 249, 29, 252, 72, 168, 115, 119, 14, 122, 105, 85, 246, 28, 200, 244, 84, 68, 8, 138, 4, 87, 126, 231, 126, 94, 60, 133, 212, 173, 140, 158, 59, 54, 99, 194, 152, 200, 65, 79, 85, 73, 176, 59, 143, 106, 138, 13, 86, 156, 168, 51, 228, 230, 93, 180, 9, 99, 36, 37, 224, 37, 204, 140, 54, 127, 88, 203, 19, 149, 95, 1, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 144, 205, 74, 3, 65, 16, 132, 205, 230, 71, 125, 173, 8, 222, 60, 121, 240, 56, 180, 179, 173, 52, 78, 122, 134, 238, 158, 152, 235, 138, 224, 117, 163, 224, 217, 147, 75, 14, 241, 15, 196, 215, 115, 136, 56, 228, 246, 81, 69, 21, 221, 213, 220, 109, 174, 50, 123, 163, 200, 186, 126, 216, 254, 179, 99, 88, 224, 203, 183, 207, 34, 200, 230, 110, 201, 24, 85, 29, 113, 139, 171, 131, 33, 38, 31, 91, 212, 117, 247, 49, 15, 224, 111, 230, 113, 117, 90, 114, 39, 16, 66, 247, 118, 150, 131, 209, 185, 135, 0, 82, 240, 254, 53, 69, 98, 211, 167, 110, 184, 248, 235, 24, 85, 106, 42, 141, 7, 221, 5, 244, 177, 74, 147, 74, 211, 77, 18, 108, 201, 131, 97, 213, 102, 67, 204, 150, 114, 233, 61, 60, 58, 254, 74, 66, 203, 226, 186, 4, 82, 174, 54, 20, 125, 30, 53, 227, 201, 116, 246, 153, 242, 101, 32, 191, 103, 244, 91, 65, 203, 194, 110, 9, 33, 227, 46, 254, 14, 170, 40, 230, 22, 165, 23, 174, 81, 251, 159, 242, 75, 89, 195, 4, 136, 177, 117, 117, 158, 254, 23, 31, 134, 82, 11, 44, 1, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 146, 207, 78, 27, 49, 16, 198, 179, 251, 36, 125, 134, 246, 9, 218, 84, 72, 61, 244, 132, 122, 182, 28, 123, 178, 26, 201, 107, 187, 227, 217, 180, 233, 109, 147, 34, 174, 1, 137, 59, 2, 242, 79, 155, 128, 0, 33, 94, 128, 7, 195, 27, 72, 8, 145, 72, 144, 216, 211, 106, 108, 127, 223, 204, 111, 190, 180, 55, 109, 23, 86, 49, 58, 27, 142, 14, 231, 203, 127, 97, 101, 14, 167, 119, 170, 32, 2, 203, 226, 15, 178, 133, 16, 4, 90, 13, 127, 27, 99, 231, 149, 211, 16, 142, 202, 234, 27, 161, 49, 152, 53, 165, 49, 255, 207, 80, 55, 70, 104, 125, 193, 241, 100, 180, 143, 54, 51, 208, 159, 230, 133, 17, 12, 148, 135, 193, 173, 65, 11, 146, 132, 114, 121, 11, 173, 124, 178, 60, 126, 248, 212, 216, 254, 37, 201, 249, 111, 161, 118, 94, 139, 93, 21, 188, 242, 206, 189, 129, 116, 234, 9, 52, 42, 201, 59, 219, 24, 188, 203, 34, 185, 241, 132, 157, 40, 39, 188, 164, 200, 39, 234, 133, 227, 36, 189, 246, 69, 203, 160, 90, 43, 14, 230, 4, 92, 144, 21, 29, 105, 10, 8, 131, 43, 25, 2, 16, 139, 60, 50, 148, 89, 44, 220, 71, 204, 209, 150, 73, 198, 94, 180, 120, 217, 64, 239, 245, 6, 42, 180, 157, 250, 161, 138, 124, 39, 173, 46, 67, 205, 253, 164, 28, 54, 235, 199, 253, 42, 238, 128, 159, 71, 40, 71, 223, 145, 64, 113, 50, 105, 33, 139, 128, 255, 160, 28, 255, 176, 12, 25, 208, 249, 175, 47, 159, 135, 139, 78, 118, 67, 220, 38, 157, 126, 72, 58, 41, 103, 117, 74, 180, 100, 217, 116, 190, 219, 191, 93, 115, 16, 82, 107, 138, 108, 150, 78, 141, 89, 237, 178, 89, 77, 47, 93, 187, 29, 128, 55, 235, 73, 89, 237, 57, 2, 204, 108, 109, 112, 48, 89, 18, 28, 61, 193, 155, 173, 25, 197, 112, 204, 127, 66, 238, 168, 251, 117, 195, 241, 126, 189, 157, 197, 68, 130, 187, 30, 94, 210, 52, 220, 67, 48, 122, 149, 239, 55, 100, 174, 23, 231, 91, 4, 202, 139, 125, 118, 190, 172, 158, 3, 82, 227, 232, 141, 189, 195, 200, 147, 86, 3, 93, 44, 32, 47, 199, 126, 4, 221, 152, 224, 160, 163, 3, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 149, 205, 110, 211, 64, 20, 133, 237, 56, 5, 30, 35, 18, 60, 1, 60, 65, 9, 170, 96, 193, 170, 98, 61, 154, 216, 55, 214, 72, 227, 153, 97, 102, 92, 106, 118, 147, 128, 216, 230, 103, 201, 6, 209, 38, 78, 228, 4, 84, 42, 212, 23, 224, 193, 24, 135, 56, 56, 21, 73, 172, 166, 180, 89, 57, 87, 246, 253, 238, 61, 62, 103, 92, 235, 76, 219, 49, 243, 53, 225, 76, 245, 63, 205, 139, 107, 196, 112, 4, 95, 126, 250, 177, 148, 192, 52, 122, 71, 52, 3, 165, 16, 97, 1, 156, 58, 41, 23, 62, 15, 64, 245, 77, 246, 92, 18, 74, 73, 216, 196, 148, 126, 248, 74, 2, 103, 76, 152, 136, 181, 26, 152, 209, 161, 148, 56, 25, 118, 167, 81, 76, 145, 6, 25, 169, 222, 37, 37, 12, 176, 68, 62, 143, 90, 132, 225, 63, 200, 193, 175, 134, 179, 253, 231, 186, 103, 111, 145, 191, 243, 54, 231, 54, 80, 181, 187, 67, 121, 213, 80, 102, 124, 76, 88, 72, 97, 23, 114, 88, 69, 200, 42, 10, 220, 222, 232, 41, 143, 117, 110, 134, 97, 97, 134, 250, 193, 131, 124, 157, 72, 80, 120, 88, 92, 60, 154, 10, 9, 1, 241, 177, 222, 185, 98, 175, 18, 214, 253, 33, 36, 57, 177, 237, 144, 192, 210, 154, 216, 246, 83, 67, 183, 230, 93, 136, 184, 69, 137, 95, 170, 246, 230, 18, 116, 44, 25, 58, 193, 52, 6, 213, 251, 142, 149, 2, 169, 81, 100, 157, 142, 67, 91, 184, 178, 97, 176, 92, 45, 177, 29, 38, 64, 127, 115, 210, 89, 207, 201, 204, 78, 105, 87, 57, 69, 190, 141, 193, 164, 149, 104, 200, 227, 241, 217, 140, 154, 249, 211, 221, 204, 70, 69, 47, 151, 48, 227, 23, 68, 130, 175, 157, 73, 139, 104, 164, 200, 123, 48, 233, 43, 166, 33, 4, 121, 246, 230, 217, 211, 209, 98, 148, 157, 59, 122, 219, 90, 187, 123, 181, 118, 204, 44, 15, 115, 128, 53, 110, 114, 145, 116, 47, 75, 4, 132, 131, 64, 90, 113, 10, 82, 99, 150, 83, 174, 87, 157, 111, 188, 221, 86, 160, 175, 215, 221, 255, 167, 71, 99, 91, 107, 111, 175, 214, 91, 167, 174, 239, 249, 22, 171, 75, 237, 254, 83, 106, 111, 131, 212, 117, 147, 29, 113, 9, 36, 100, 57, 224, 227, 164, 112, 107, 186, 52, 234, 172, 68, 178, 241, 156, 190, 4, 44, 22, 17, 237, 164, 130, 19, 187, 136, 92, 189, 151, 243, 156, 235, 153, 249, 107, 136, 184, 76, 14, 215, 57, 143, 55, 212, 159, 92, 149, 87, 89, 168, 129, 116, 34, 96, 117, 20, 116, 178, 82, 177, 95, 156, 5, 163, 35, 2, 52, 88, 18, 215, 106, 235, 255, 86, 31, 153, 27, 15, 238, 94, 44, 90, 148, 71, 27, 220, 116, 180, 251, 113, 245, 126, 41, 63, 48, 231, 199, 154, 11, 147, 45, 79, 192, 220, 131, 27, 36, 92, 1, 127, 3, 207, 68, 96, 13, 42, 8, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 205, 147, 75, 78, 195, 48, 16, 134, 147, 180, 7, 225, 58, 44, 16, 71, 24, 57, 206, 128, 44, 226, 7, 51, 118, 161, 203, 54, 11, 182, 73, 123, 1, 196, 179, 169, 84, 33, 64, 136, 11, 112, 48, 76, 34, 30, 43, 218, 69, 85, 213, 27, 219, 227, 209, 255, 127, 26, 253, 206, 166, 237, 73, 48, 210, 43, 107, 184, 185, 90, 125, 159, 193, 8, 141, 215, 111, 50, 16, 161, 241, 112, 161, 188, 65, 102, 80, 166, 192, 203, 228, 209, 58, 105, 11, 228, 249, 100, 121, 132, 218, 210, 248, 208, 40, 95, 45, 242, 210, 202, 51, 80, 69, 114, 167, 226, 125, 150, 102, 203, 190, 226, 199, 14, 31, 250, 198, 201, 162, 223, 143, 221, 244, 183, 253, 198, 186, 170, 181, 14, 73, 124, 57, 87, 173, 14, 37, 120, 36, 205, 245, 107, 169, 12, 10, 2, 105, 117, 174, 76, 247, 204, 245, 237, 57, 200, 143, 131, 228, 255, 149, 222, 119, 168, 219, 18, 27, 137, 50, 224, 58, 177, 102, 182, 94, 105, 176, 145, 97, 178, 179, 57, 37, 123, 58, 167, 225, 102, 244, 47, 142, 212, 72, 120, 4, 39, 40, 6, 54, 122, 242, 60, 205, 6, 207, 46, 228, 165, 146, 127, 170, 245, 138, 208, 7, 50, 208, 17, 114, 51, 124, 18, 204, 72, 30, 116, 140, 181, 56, 69, 174, 223, 99, 242, 35, 156, 39, 17, 137, 11, 248, 249, 20, 245, 39, 133, 102, 125, 78, 34, 3, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack,  @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 148, 203, 74, 3, 49, 20, 134, 51, 233, 139, 184, 212, 157, 226, 19, 72, 17, 92, 137, 75, 17, 36, 164, 153, 163, 4, 102, 50, 241, 36, 169, 186, 172, 10, 110, 231, 242, 2, 69, 197, 50, 66, 17, 21, 111, 123, 95, 164, 59, 151, 110, 220, 27, 11, 245, 66, 165, 29, 181, 98, 86, 135, 195, 225, 92, 254, 239, 39, 116, 175, 220, 112, 74, 88, 153, 40, 83, 28, 118, 7, 49, 83, 60, 134, 246, 181, 112, 136, 160, 44, 219, 150, 86, 129, 49, 76, 170, 16, 118, 72, 39, 209, 34, 9, 193, 20, 173, 147, 58, 143, 162, 131, 35, 25, 6, 167, 82, 105, 103, 77, 78, 130, 78, 226, 236, 107, 152, 209, 82, 35, 132, 82, 112, 11, 251, 101, 236, 34, 102, 1, 99, 147, 94, 69, 82, 1, 71, 38, 146, 184, 33, 21, 239, 79, 78, 143, 183, 152, 120, 152, 34, 163, 95, 48, 122, 94, 109, 242, 243, 206, 22, 140, 1, 180, 107, 128, 201, 184, 150, 121, 62, 190, 31, 245, 53, 179, 225, 50, 246, 230, 218, 51, 119, 43, 139, 55, 173, 214, 234, 250, 244, 252, 227, 210, 238, 189, 206, 234, 189, 231, 226, 201, 23, 213, 42, 173, 70, 46, 53, 202, 166, 191, 148, 105, 142, 30, 149, 223, 203, 100, 228, 66, 187, 70, 36, 197, 199, 92, 208, 69, 176, 14, 21, 107, 242, 200, 129, 73, 207, 121, 255, 34, 22, 123, 156, 124, 211, 39, 190, 201, 60, 159, 180, 38, 164, 130, 38, 180, 146, 38, 244, 221, 30, 116, 96, 15, 250, 151, 246, 248, 130, 129, 247, 227, 48, 132, 244, 51, 131, 172, 246, 91, 8, 217, 127, 64, 8, 126, 108, 204, 74, 162, 144, 33, 81, 110, 189, 36, 126, 127, 139, 220, 31, 21, 178, 183, 111, 42, 125, 1, 186, 80, 103, 77, 180, 4, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 144, 161, 10, 2, 65, 16, 134, 103, 102, 95, 196, 168, 77, 241, 9, 228, 16, 76, 98, 20, 193, 32, 120, 237, 64, 217, 179, 24, 247, 13, 102, 102, 5, 235, 5, 147, 15, 32, 218, 125, 145, 107, 70, 139, 221, 69, 12, 38, 119, 47, 220, 228, 255, 255, 249, 230, 51, 234, 143, 149, 241, 238, 148, 173, 138, 226, 128, 10, 40, 228, 153, 239, 29, 248, 127, 248, 219, 48, 105, 141, 243, 168, 44, 115, 187, 91, 228, 118, 227, 89, 53, 222, 160, 144, 233, 175, 167, 182, 30, 84, 189, 235, 108, 124, 113, 110, 190, 236, 14, 31, 147, 253, 109, 43, 89, 253, 242, 207, 16, 50, 209, 25, 16, 16, 100, 14, 111, 106, 115, 4, 72, 64, 160, 232, 12, 125, 109, 145, 82, 170, 173, 224, 149, 197, 4, 106, 148, 118, 168, 49, 46, 238, 195, 0, 204, 111, 233, 240, 53, 48, 37, 2, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
}
