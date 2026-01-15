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
    BRILLIG CALL func: 0, inputs: [{w_input}], outputs: [{w_inverted}]
    "
    );
    let circuit = Circuit::from_str(&src).unwrap();

    let program =
        Program { functions: vec![circuit], unconstrained_functions: vec![brillig_bytecode] };

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 146, 205, 78, 2, 49, 20, 133, 153, 121, 18, 159, 65, 159, 64, 49, 36, 46, 92, 17, 215, 77, 153, 185, 76, 110, 210, 105, 107, 123, 135, 4, 119, 3, 234, 26, 72, 220, 27, 249, 207, 128, 6, 89, 176, 117, 225, 131, 217, 225, 95, 18, 97, 97, 87, 205, 109, 123, 206, 61, 95, 175, 223, 24, 87, 19, 25, 16, 42, 105, 219, 207, 211, 205, 158, 73, 30, 195, 235, 80, 233, 64, 133, 96, 219, 105, 118, 101, 80, 8, 140, 138, 92, 136, 199, 55, 12, 11, 3, 148, 58, 33, 119, 50, 40, 163, 140, 4, 52, 199, 113, 34, 24, 129, 137, 109, 107, 46, 80, 2, 55, 44, 80, 113, 5, 37, 95, 105, 119, 190, 207, 10, 199, 151, 231, 117, 239, 89, 112, 242, 90, 97, 168, 18, 218, 122, 199, 90, 128, 63, 214, 6, 66, 12, 56, 193, 215, 167, 54, 88, 115, 27, 166, 185, 113, 17, 92, 67, 182, 227, 249, 51, 157, 84, 4, 6, 123, 197, 214, 212, 0, 37, 70, 178, 26, 23, 9, 216, 214, 7, 183, 22, 12, 177, 24, 172, 229, 145, 43, 44, 28, 9, 215, 55, 25, 238, 194, 132, 108, 7, 169, 241, 27, 82, 134, 178, 150, 63, 12, 28, 153, 81, 165, 78, 144, 19, 123, 73, 251, 197, 252, 113, 51, 115, 244, 104, 205, 32, 29, 92, 163, 129, 128, 188, 81, 5, 137, 89, 124, 128, 116, 120, 35, 9, 34, 48, 221, 187, 139, 243, 254, 178, 147, 211, 241, 143, 73, 251, 255, 146, 246, 210, 73, 254, 191, 33, 39, 94, 84, 186, 222, 156, 239, 57, 48, 30, 134, 198, 177, 217, 56, 21, 38, 185, 203, 97, 213, 127, 87, 213, 170, 5, 58, 172, 123, 105, 86, 82, 6, 48, 146, 185, 193, 211, 104, 67, 112, 176, 130, 55, 217, 51, 114, 223, 58, 189, 133, 88, 153, 250, 229, 129, 227, 98, 191, 157, 101, 34, 70, 117, 13, 187, 57, 232, 151, 16, 68, 184, 157, 204, 63, 100, 102, 203, 243, 35, 2, 105, 175, 76, 74, 167, 217, 122, 64, 114, 28, 141, 161, 86, 232, 120, 154, 109, 160, 222, 18, 242, 38, 246, 15, 121, 255, 75, 13, 70, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 143, 49, 14, 130, 48, 20, 134, 41, 94, 196, 51, 232, 9, 20, 67, 226, 224, 68, 156, 13, 129, 23, 210, 164, 180, 164, 52, 38, 140, 189, 1, 45, 226, 108, 162, 224, 128, 222, 193, 193, 131, 9, 33, 150, 196, 65, 244, 205, 239, 251, 254, 255, 159, 104, 117, 60, 41, 217, 44, 57, 38, 4, 71, 142, 79, 200, 193, 82, 178, 246, 48, 141, 8, 20, 185, 210, 207, 169, 245, 253, 16, 26, 125, 233, 141, 113, 66, 192, 126, 104, 100, 231, 121, 235, 109, 48, 221, 3, 23, 187, 160, 141, 44, 101, 229, 48, 154, 138, 66, 214, 43, 204, 33, 16, 72, 94, 215, 84, 64, 4, 252, 188, 157, 207, 198, 253, 159, 188, 253, 31, 143, 228, 173, 91, 30, 250, 194, 119, 88, 146, 25, 141, 53, 248, 76, 177, 198, 101, 28, 112, 68, 59, 160, 172, 251, 17, 74, 222, 55, 16, 51, 158, 45, 194, 144, 67, 154, 26, 222, 236, 174, 92, 12, 36, 252, 245, 79, 94, 60, 193, 18, 165, 135, 212, 119, 143, 23, 119, 248, 157, 100, 176, 1, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 85, 203, 142, 211, 48, 20, 77, 154, 14, 240, 25, 149, 224, 11, 224, 11, 134, 162, 17, 44, 88, 141, 88, 91, 110, 114, 27, 89, 114, 108, 99, 59, 35, 194, 206, 45, 176, 238, 99, 201, 6, 49, 211, 166, 85, 90, 208, 48, 139, 217, 178, 224, 195, 176, 75, 83, 210, 17, 125, 104, 58, 64, 86, 55, 71, 201, 61, 231, 30, 159, 155, 212, 58, 211, 118, 202, 66, 77, 56, 83, 253, 15, 243, 178, 70, 12, 39, 240, 41, 231, 34, 228, 17, 168, 190, 41, 158, 74, 66, 41, 137, 155, 152, 210, 119, 159, 73, 228, 141, 9, 19, 169, 86, 3, 51, 58, 150, 18, 103, 195, 238, 52, 73, 41, 210, 32, 19, 213, 187, 162, 132, 1, 150, 40, 228, 73, 139, 48, 252, 171, 247, 224, 71, 195, 219, 126, 249, 254, 249, 107, 20, 238, 124, 204, 187, 11, 170, 218, 191, 163, 10, 246, 163, 50, 227, 83, 194, 98, 10, 187, 40, 135, 251, 24, 185, 143, 3, 119, 39, 61, 231, 169, 118, 97, 24, 150, 97, 168, 31, 221, 115, 227, 36, 130, 194, 253, 178, 120, 48, 21, 18, 34, 18, 98, 13, 223, 191, 9, 73, 206, 108, 129, 4, 150, 54, 103, 118, 88, 53, 244, 107, 193, 165, 72, 91, 148, 132, 21, 180, 55, 151, 160, 83, 201, 208, 25, 166, 41, 168, 222, 87, 172, 20, 72, 141, 18, 80, 10, 199, 22, 184, 182, 121, 181, 166, 104, 137, 173, 83, 17, 250, 29, 229, 206, 122, 148, 103, 214, 66, 43, 226, 13, 10, 109, 128, 39, 173, 76, 131, 11, 246, 71, 51, 106, 186, 183, 187, 133, 13, 185, 94, 58, 108, 198, 207, 136, 132, 80, 123, 147, 22, 209, 72, 145, 183, 96, 242, 23, 76, 67, 12, 242, 252, 213, 147, 199, 163, 133, 148, 157, 166, 4, 219, 90, 251, 7, 181, 246, 204, 204, 173, 97, 132, 53, 110, 114, 145, 117, 175, 42, 12, 8, 71, 145, 180, 230, 148, 76, 141, 153, 99, 185, 137, 122, 95, 120, 187, 173, 64, 223, 196, 253, 191, 231, 71, 99, 91, 235, 224, 160, 214, 91, 85, 215, 15, 60, 197, 253, 173, 246, 255, 104, 117, 176, 193, 234, 186, 41, 78, 184, 4, 18, 51, 71, 240, 126, 82, 166, 53, 95, 6, 117, 86, 97, 178, 139, 53, 125, 14, 88, 44, 150, 171, 147, 11, 78, 236, 32, 114, 117, 46, 23, 142, 55, 48, 243, 151, 144, 112, 153, 29, 175, 243, 60, 220, 128, 63, 186, 174, 142, 178, 112, 3, 233, 76, 192, 106, 137, 59, 69, 5, 236, 151, 91, 60, 58, 33, 64, 163, 37, 227, 26, 182, 126, 183, 250, 61, 220, 90, 184, 127, 185, 104, 81, 149, 54, 184, 173, 180, 255, 147, 234, 195, 182, 252, 200, 92, 156, 106, 46, 76, 177, 252, 2, 186, 12, 110, 176, 112, 69, 248, 19, 241, 193, 224, 218, 205, 7, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 82, 65, 78, 195, 48, 16, 140, 235, 22, 120, 70, 36, 120, 1, 188, 160, 4, 85, 112, 224, 84, 113, 70, 86, 98, 69, 150, 220, 56, 114, 124, 32, 71, 255, 32, 182, 129, 11, 23, 36, 72, 43, 212, 242, 7, 14, 60, 140, 6, 225, 84, 109, 105, 19, 171, 241, 201, 187, 246, 204, 206, 238, 14, 212, 234, 249, 85, 201, 249, 37, 39, 148, 146, 56, 64, 148, 62, 122, 90, 150, 67, 206, 81, 110, 76, 161, 244, 183, 239, 237, 63, 0, 52, 126, 241, 218, 17, 245, 186, 34, 130, 205, 68, 114, 58, 38, 73, 76, 177, 41, 76, 155, 22, 219, 168, 239, 70, 152, 177, 195, 239, 15, 142, 42, 145, 147, 148, 226, 99, 123, 57, 249, 50, 160, 7, 139, 98, 57, 133, 69, 200, 170, 204, 195, 125, 184, 220, 217, 139, 44, 3, 150, 100, 194, 200, 233, 21, 225, 56, 20, 158, 156, 221, 36, 2, 199, 152, 191, 221, 93, 156, 55, 86, 133, 155, 120, 224, 134, 247, 228, 162, 178, 78, 132, 4, 10, 88, 154, 215, 52, 254, 74, 79, 77, 124, 152, 82, 127, 19, 15, 221, 240, 91, 245, 251, 174, 147, 250, 191, 83, 176, 210, 83, 19, 207, 71, 140, 99, 18, 39, 21, 224, 105, 246, 183, 47, 35, 63, 174, 49, 74, 127, 119, 172, 235, 33, 64, 249, 121, 139, 39, 140, 231, 195, 40, 226, 56, 203, 236, 195, 233, 142, 252, 153, 181, 137, 86, 214, 27, 229, 136, 96, 26, 193, 245, 112, 61, 210, 78, 181, 129, 110, 89, 163, 227, 141, 58, 122, 111, 32, 223, 199, 130, 165, 74, 111, 123, 237, 7, 252, 107, 218, 230, 222, 4, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_msgpack,  @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 197, 147, 189, 74, 4, 49, 20, 133, 51, 153, 23, 177, 212, 78, 241, 9, 100, 17, 172, 196, 82, 4, 9, 217, 76, 148, 64, 38, 137, 55, 201, 130, 229, 168, 88, 207, 207, 11, 44, 42, 46, 35, 44, 162, 130, 63, 157, 133, 47, 178, 157, 165, 141, 189, 113, 97, 117, 101, 215, 117, 16, 193, 84, 151, 203, 33, 156, 239, 156, 4, 31, 212, 59, 94, 49, 39, 180, 178, 213, 113, 127, 52, 19, 69, 83, 222, 237, 105, 195, 116, 194, 109, 149, 157, 181, 168, 148, 71, 39, 34, 137, 206, 133, 50, 222, 217, 18, 69, 61, 237, 221, 251, 88, 224, 218, 0, 79, 4, 163, 142, 63, 206, 86, 198, 227, 202, 139, 21, 107, 57, 184, 45, 14, 250, 176, 78, 189, 36, 142, 67, 106, 243, 27, 41, 20, 167, 64, 152, 78, 219, 66, 209, 161, 179, 178, 124, 154, 67, 179, 79, 132, 131, 102, 49, 89, 135, 193, 82, 119, 225, 126, 99, 245, 54, 203, 54, 183, 231, 151, 159, 215, 246, 31, 76, 209, 26, 188, 86, 47, 65, 20, 159, 238, 17, 246, 227, 85, 232, 218, 128, 232, 4, 143, 196, 80, 8, 57, 4, 95, 182, 64, 87, 198, 183, 165, 96, 227, 187, 168, 15, 220, 121, 80, 164, 67, 165, 231, 54, 191, 164, 67, 34, 146, 114, 107, 233, 110, 88, 124, 23, 104, 249, 215, 240, 168, 1, 60, 110, 4, 143, 63, 27, 196, 163, 6, 241, 244, 6, 167, 196, 20, 202, 158, 204, 41, 255, 26, 83, 17, 55, 206, 169, 248, 143, 156, 162, 95, 63, 146, 70, 244, 104, 130, 254, 46, 176, 7, 255, 14, 104, 128, 74, 200, 199, 127, 204, 223, 0, 164, 60, 113, 89, 157, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 144, 161, 10, 2, 65, 20, 69, 223, 188, 249, 17, 163, 54, 197, 47, 144, 69, 48, 137, 81, 4, 131, 224, 182, 5, 101, 214, 98, 156, 63, 120, 239, 141, 108, 158, 96, 242, 3, 68, 219, 6, 127, 100, 155, 209, 98, 119, 20, 131, 201, 217, 13, 123, 219, 133, 203, 229, 112, 180, 184, 194, 59, 123, 76, 86, 89, 118, 80, 2, 138, 177, 252, 109, 186, 180, 167, 81, 158, 167, 102, 183, 72, 205, 198, 145, 200, 173, 3, 255, 163, 48, 108, 250, 235, 169, 169, 6, 190, 119, 153, 141, 207, 214, 206, 151, 221, 225, 125, 178, 191, 110, 57, 169, 158, 238, 17, 70, 58, 122, 3, 12, 172, 136, 10, 47, 205, 9, 160, 6, 1, 70, 111, 240, 43, 2, 5, 223, 34, 130, 14, 98, 29, 128, 184, 29, 32, 21, 87, 242, 65, 0, 162, 23, 65, 214, 127, 213, 182, 1, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
}
