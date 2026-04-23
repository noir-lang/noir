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
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 204, 49, 14, 64, 48, 24, 64, 225, 86, 47, 98, 100, 35, 78, 32, 34, 49, 137, 81, 36, 54, 157, 73, 107, 49, 246, 6, 253, 219, 193, 218, 217, 1, 132, 221, 69, 186, 25, 45, 118, 61, 1, 222, 252, 229, 17, 5, 179, 65, 32, 150, 148, 115, 202, 198, 134, 178, 94, 75, 173, 14, 31, 189, 135, 241, 15, 227, 57, 19, 117, 37, 179, 177, 9, 183, 42, 95, 133, 168, 219, 32, 57, 139, 105, 31, 32, 179, 183, 190, 28, 34, 159, 27, 164, 176, 39, 129, 72, 249, 0, 202, 250, 172, 64, 172, 0, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 61, 198, 187, 14, 64, 48, 20, 0, 80, 212, 243, 183, 72, 108, 38, 131, 185, 105, 12, 141, 155, 74, 180, 77, 172, 253, 131, 62, 132, 213, 102, 19, 159, 136, 229, 110, 135, 120, 119, 156, 145, 51, 79, 13, 148, 77, 245, 188, 182, 90, 176, 134, 2, 152, 187, 211, 160, 120, 207, 40, 208, 229, 227, 22, 204, 53, 112, 37, 70, 41, 99, 84, 130, 34, 30, 153, 162, 50, 84, 30, 138, 178, 218, 227, 132, 164, 89, 110, 127, 91, 251, 2, 11, 11, 246, 224, 122, 0, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 144, 65, 10, 194, 48, 16, 69, 155, 122, 17, 207, 160, 39, 208, 138, 224, 194, 85, 113, 45, 165, 29, 74, 32, 77, 74, 26, 132, 46, 115, 131, 36, 181, 184, 21, 180, 117, 81, 189, 133, 7, 179, 165, 152, 130, 11, 171, 179, 27, 152, 255, 255, 155, 63, 49, 250, 116, 118, 180, 108, 150, 28, 19, 130, 99, 47, 32, 228, 216, 238, 181, 143, 105, 76, 160, 80, 218, 60, 167, 206, 247, 65, 104, 244, 164, 119, 76, 82, 2, 110, 161, 212, 184, 163, 65, 174, 82, 109, 118, 131, 233, 1, 184, 216, 135, 45, 86, 41, 43, 143, 209, 76, 20, 178, 94, 97, 14, 161, 64, 242, 182, 161, 2, 98, 224, 151, 221, 124, 54, 206, 240, 169, 119, 255, 211, 35, 121, 239, 218, 137, 2, 17, 120, 44, 205, 173, 141, 51, 248, 89, 176, 102, 205, 56, 224, 152, 118, 130, 178, 238, 159, 208, 242, 177, 133, 132, 241, 124, 17, 69, 28, 178, 204, 234, 109, 55, 213, 26, 3, 137, 126, 189, 147, 87, 95, 176, 84, 155, 33, 245, 205, 241, 2, 230, 146, 118, 152, 213, 1, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 82, 75, 78, 195, 48, 20, 180, 235, 22, 56, 70, 36, 56, 1, 156, 160, 4, 85, 176, 96, 85, 177, 70, 81, 98, 69, 150, 220, 56, 114, 188, 32, 75, 223, 192, 31, 16, 11, 54, 72, 144, 86, 168, 229, 22, 28, 140, 6, 225, 84, 109, 129, 196, 34, 94, 249, 55, 243, 230, 205, 27, 100, 244, 227, 51, 208, 114, 121, 206, 9, 165, 36, 13, 35, 74, 239, 129, 145, 213, 152, 243, 168, 180, 86, 105, 243, 17, 128, 191, 23, 132, 173, 95, 64, 55, 162, 65, 95, 68, 168, 157, 72, 206, 167, 36, 75, 41, 182, 202, 118, 105, 177, 139, 250, 126, 132, 89, 103, 254, 112, 116, 80, 139, 156, 229, 20, 31, 186, 205, 145, 85, 170, 189, 138, 133, 3, 164, 212, 218, 169, 85, 204, 106, 212, 221, 109, 188, 158, 235, 147, 172, 66, 150, 21, 194, 202, 249, 5, 225, 56, 22, 64, 46, 174, 50, 129, 83, 204, 95, 110, 206, 78, 91, 105, 209, 46, 30, 250, 225, 129, 92, 213, 241, 74, 34, 17, 133, 44, 47, 27, 154, 96, 163, 167, 33, 254, 159, 210, 96, 23, 143, 252, 240, 123, 245, 135, 190, 78, 253, 220, 41, 220, 232, 105, 136, 151, 19, 198, 49, 73, 179, 26, 240, 176, 248, 158, 151, 149, 111, 151, 56, 202, 191, 114, 96, 26, 19, 144, 124, 191, 198, 51, 198, 203, 113, 146, 112, 92, 20, 238, 225, 248, 151, 251, 19, 23, 37, 163, 93, 126, 170, 9, 193, 52, 65, 219, 199, 237, 147, 241, 170, 13, 77, 199, 26, 61, 79, 212, 51, 123, 35, 249, 58, 21, 44, 215, 102, 63, 107, 159, 204, 216, 181, 142, 3, 5, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
}

#[test]
fn memory_op_circuit() {
    let src = "
    private parameters: [w1, w2, w3]
    public parameters: []
    return values: [w4]
    INIT b0 = [w1, w2]
    WRITE b0[w5] = w3
    READ w4 = b0[w5]
    ";
    let circuit = Circuit::from_str(src).unwrap();

    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes_msgpack =
        Program::serialize_program_with_format(&program, SerializationFormat::Msgpack);
    insta::assert_compact_debug_snapshot!(bytes_msgpack, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 147, 75, 78, 195, 48, 16, 134, 211, 180, 220, 131, 235, 176, 64, 28, 97, 228, 56, 3, 178, 136, 31, 204, 216, 133, 46, 219, 44, 216, 38, 237, 5, 16, 80, 104, 42, 85, 8, 16, 226, 2, 28, 12, 147, 136, 199, 138, 102, 129, 144, 234, 141, 237, 153, 209, 124, 191, 70, 243, 167, 179, 230, 56, 24, 233, 149, 53, 92, 95, 110, 62, 223, 96, 132, 198, 171, 23, 25, 136, 208, 120, 56, 87, 222, 32, 51, 40, 147, 227, 69, 114, 111, 157, 180, 57, 242, 98, 186, 62, 68, 109, 105, 114, 96, 148, 47, 87, 89, 97, 229, 41, 168, 60, 185, 85, 241, 63, 31, 164, 235, 46, 226, 39, 14, 239, 186, 194, 233, 170, 187, 143, 220, 236, 187, 252, 218, 186, 178, 177, 14, 73, 124, 144, 203, 70, 135, 2, 60, 146, 230, 234, 185, 80, 6, 5, 129, 180, 58, 83, 166, 77, 115, 117, 115, 6, 242, 109, 63, 249, 253, 12, 150, 173, 212, 109, 205, 234, 249, 246, 78, 123, 189, 128, 201, 114, 44, 138, 128, 127, 1, 28, 246, 3, 254, 219, 44, 147, 29, 158, 229, 168, 31, 240, 201, 145, 26, 11, 143, 224, 4, 197, 197, 143, 76, 94, 12, 210, 225, 163, 11, 89, 161, 228, 143, 104, 181, 33, 244, 129, 12, 180, 10, 185, 30, 61, 8, 102, 36, 15, 58, 218, 67, 156, 32, 87, 175, 209, 65, 81, 156, 39, 17, 21, 231, 240, 101, 174, 234, 29, 87, 125, 68, 249, 106, 3, 0, 0]");

    let bytes_default = Program::serialize_program(&program);
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 189, 143, 171, 13, 128, 64, 16, 68, 247, 110, 143, 62, 104, 7, 65, 168, 2, 129, 224, 19, 130, 65, 94, 7, 251, 17, 88, 20, 33, 212, 65, 97, 136, 243, 236, 97, 24, 51, 153, 100, 242, 146, 135, 194, 219, 14, 26, 175, 186, 237, 199, 121, 173, 134, 110, 81, 16, 231, 143, 180, 227, 153, 186, 153, 4, 84, 137, 238, 18, 222, 227, 148, 88, 236, 87, 97, 94, 32, 15, 132, 54, 232, 179, 3, 252, 236, 16, 50, 64, 206, 35, 113, 32, 122, 0, 237, 231, 14, 51, 176, 1, 0, 0]");

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
    insta::assert_compact_debug_snapshot!(bytes_default, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 144, 173, 10, 2, 65, 20, 133, 239, 220, 121, 17, 163, 54, 197, 39, 144, 69, 48, 137, 81, 4, 131, 224, 182, 5, 101, 214, 98, 156, 55, 184, 63, 130, 117, 131, 201, 7, 16, 237, 190, 200, 54, 163, 197, 238, 40, 6, 147, 51, 134, 189, 249, 156, 195, 119, 63, 43, 186, 175, 64, 253, 33, 91, 20, 197, 206, 8, 24, 70, 37, 186, 182, 224, 247, 153, 239, 134, 77, 107, 28, 7, 101, 153, 187, 205, 44, 119, 43, 37, 145, 120, 3, 67, 166, 187, 28, 187, 186, 87, 117, 206, 147, 225, 201, 251, 233, 188, 221, 191, 141, 182, 151, 53, 103, 245, 67, 239, 33, 100, 163, 51, 192, 192, 134, 40, 188, 41, 255, 35, 64, 2, 2, 70, 103, 240, 99, 11, 5, 83, 109, 5, 175, 196, 246, 69, 205, 205, 80, 155, 184, 184, 55, 3, 16, 61, 1, 97, 127, 171, 83, 37, 2, 0, 0]");

    assert_deserialization(&program, [bytes_msgpack, bytes_default]);
}
