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

use std::collections::BTreeSet;

use acir::{
    circuit::{
        Circuit, Opcode, Program, PublicInputs,
        brillig::{BrilligBytecode, BrilligFunctionId, BrilligInputs, BrilligOutputs},
        opcodes::{AcirFunctionId, BlackBoxFuncCall, BlockId, FunctionInput, MemOp},
    },
    native_types::{Expression, Witness},
};
use acir_field::{AcirField, FieldElement};
use brillig::{
    BitSize, HeapArray, HeapValueType, HeapVector, IntegerBitSize, MemoryAddress, ValueOrArray,
};

#[test]
fn addition_circuit() {
    let addition = Opcode::AssertZero(Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![
            (FieldElement::one(), Witness(1)),
            (FieldElement::one(), Witness(2)),
            (-FieldElement::one(), Witness(3)),
        ],
        q_c: FieldElement::zero(),
    });

    let circuit: Circuit<FieldElement> = Circuit {
        current_witness_index: 4,
        opcodes: vec![addition],
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        return_values: PublicInputs([Witness(3)].into()),
        ..Circuit::<FieldElement>::default()
    };
    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 145, 189, 78, 195, 48, 20, 70, 213, 150, 7, 178,
        227, 164, 113, 54, 120, 8, 22, 22, 203, 63, 55, 96, 41, 177, 195, 181, 93, 186, 2, 3, 107,
        90, 30, 129, 1, 5, 9, 241, 39, 196, 179, 240, 54, 88, 21, 66, 236, 237, 29, 239, 112, 244,
        233, 156, 249, 205, 212, 38, 167, 163, 245, 46, 108, 238, 62, 117, 66, 4, 23, 197, 149,
        141, 14, 66, 16, 214, 25, 88, 31, 61, 250, 65, 123, 3, 97, 115, 253, 116, 18, 2, 96, 60, 3,
        244, 183, 83, 159, 58, 17, 1, 251, 48, 126, 116, 214, 129, 68, 161, 125, 175, 172, 147, 59,
        218, 253, 246, 251, 152, 236, 119, 116, 118, 0, 198, 60, 51, 24, 89, 150, 37, 212, 5, 80,
        70, 37, 41, 26, 197, 43, 82, 86, 106, 201, 41, 167, 21, 175, 76, 193, 25, 3, 94, 242, 186,
        81, 77, 77, 26, 90, 50, 160, 109, 213, 176, 246, 23, 178, 120, 184, 20, 122, 239, 41, 228,
        21, 214, 3, 102, 173, 89, 79, 54, 108, 226, 197, 116, 234, 148, 79, 217, 177, 121, 31, 208,
        174, 100, 4, 49, 72, 148, 61, 100, 173, 97, 59, 155, 191, 13, 73, 117, 86, 255, 123, 142,
        207, 8, 49, 161, 19, 43, 217, 165, 92, 100, 241, 34, 119, 69, 68, 159, 185, 242, 28, 194,
        248, 149, 115, 102, 253, 17, 101, 110, 98, 196, 95, 221, 241, 7, 105, 165, 0, 5, 235, 1, 0,
        0,
    ];

    assert_eq!(bytes, expected_serialization);

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}

#[test]
fn multi_scalar_mul_circuit() {
    let multi_scalar_mul: Opcode<FieldElement> =
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::MultiScalarMul {
            points: vec![
                FunctionInput::witness(Witness(1), FieldElement::max_num_bits()),
                FunctionInput::witness(Witness(2), FieldElement::max_num_bits()),
                FunctionInput::witness(Witness(3), 1),
            ],
            scalars: vec![
                FunctionInput::witness(Witness(4), FieldElement::max_num_bits()),
                FunctionInput::witness(Witness(5), FieldElement::max_num_bits()),
            ],
            outputs: (Witness(6), Witness(7), Witness(8)),
        });

    let circuit = Circuit {
        current_witness_index: 9,
        opcodes: vec![multi_scalar_mul],
        private_parameters: BTreeSet::from([
            Witness(1),
            Witness(2),
            Witness(3),
            Witness(4),
            Witness(5),
        ]),
        return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(6), Witness(7), Witness(8)])),
        ..Circuit::default()
    };
    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 125, 144, 77, 74, 3, 65, 16, 133, 153, 73, 226, 207,
        181, 34, 184, 115, 37, 226, 178, 233, 233, 46, 181, 177, 167, 186, 169, 170, 142, 179, 157,
        184, 112, 59, 49, 120, 4, 29, 179, 80, 163, 32, 94, 194, 51, 137, 77, 2, 33, 100, 145, 93,
        81, 95, 189, 87, 188, 87, 78, 23, 87, 9, 141, 184, 128, 60, 123, 248, 54, 137, 8, 80, 212,
        157, 19, 4, 102, 229, 208, 66, 115, 220, 135, 104, 130, 5, 158, 181, 203, 177, 215, 230,
        118, 28, 154, 211, 44, 58, 209, 222, 183, 239, 103, 201, 139, 59, 55, 218, 107, 202, 227,
        253, 75, 12, 14, 133, 231, 211, 103, 135, 49, 73, 219, 95, 174, 173, 138, 87, 76, 181, 170,
        156, 240, 239, 223, 46, 43, 247, 176, 193, 134, 21, 61, 175, 190, 240, 227, 238, 205, 112,
        143, 126, 180, 197, 250, 144, 36, 51, 158, 31, 28, 30, 45, 161, 137, 148, 121, 14, 158,
        211, 90, 185, 89, 92, 96, 21, 82, 206, 107, 191, 34, 185, 137, 22, 80, 81, 147, 174, 65,
        128, 248, 169, 40, 7, 195, 209, 103, 76, 149, 119, 102, 107, 223, 189, 17, 72, 34, 84, 19,
        237, 19, 172, 140, 63, 52, 51, 144, 168, 58, 155, 235, 107, 224, 238, 39, 87, 149, 219, 21,
        210, 14, 193, 170, 77, 221, 221, 63, 3, 121, 88, 245, 124, 1, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization);

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

    let opcodes = vec![Opcode::BrilligCall {
        id: BrilligFunctionId(0),
        inputs: vec![
            BrilligInputs::Single(w_input.into()), // Input Register 0,
        ],
        // This tells the BrilligSolver which witnesses its output values correspond to
        outputs: vec![
            BrilligOutputs::Simple(w_inverted), // Output Register 1
        ],
        predicate: None,
    }];

    let circuit: Circuit<FieldElement> = Circuit {
        current_witness_index: 8,
        opcodes,
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        ..Circuit::default()
    };
    let program =
        Program { functions: vec![circuit], unconstrained_functions: vec![brillig_bytecode] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 147, 207, 78, 27, 49, 16, 198, 201, 158, 250, 56,
        208, 190, 64, 219, 84, 72, 61, 244, 132, 56, 91, 222, 245, 100, 59, 146, 215, 118, 237,
        113, 74, 122, 51, 105, 213, 107, 64, 226, 5, 128, 252, 83, 194, 127, 33, 174, 188, 6, 111,
        195, 108, 32, 33, 138, 4, 151, 136, 189, 237, 216, 154, 239, 155, 223, 55, 206, 246, 199,
        173, 104, 10, 66, 107, 194, 193, 255, 219, 34, 122, 15, 134, 196, 111, 36, 3, 33, 8, 52,
        10, 246, 62, 12, 173, 43, 172, 130, 112, 144, 38, 95, 61, 106, 141, 101, 83, 106, 253, 247,
        24, 213, 198, 0, 141, 139, 196, 39, 131, 29, 52, 165, 134, 238, 184, 138, 90, 16, 248, 42,
        244, 110, 52, 26, 144, 94, 20, 182, 202, 209, 200, 39, 141, 195, 135, 207, 155, 235, 125,
        91, 141, 147, 95, 162, 88, 187, 205, 230, 208, 70, 90, 120, 175, 156, 134, 108, 236, 60,
        40, 44, 36, 193, 253, 37, 236, 241, 79, 8, 236, 153, 97, 40, 250, 57, 222, 53, 185, 141,
        140, 67, 93, 59, 143, 109, 190, 35, 156, 244, 178, 2, 158, 53, 28, 54, 178, 43, 23, 115,
        141, 197, 82, 177, 119, 230, 129, 162, 55, 162, 45, 117, 132, 208, 187, 144, 33, 128, 39,
        81, 113, 91, 89, 114, 225, 142, 193, 51, 18, 242, 146, 57, 41, 241, 146, 67, 26, 229, 29,
        130, 26, 249, 81, 234, 55, 235, 43, 221, 9, 227, 167, 103, 136, 105, 240, 13, 61, 20, 212,
        24, 229, 72, 34, 224, 31, 72, 195, 239, 134, 160, 4, 127, 178, 251, 233, 99, 127, 166, 183,
        62, 159, 183, 164, 179, 119, 149, 222, 74, 211, 122, 193, 148, 36, 217, 180, 174, 211, 189,
        89, 114, 32, 164, 82, 117, 48, 115, 39, 27, 211, 218, 197, 106, 53, 59, 183, 173, 86, 0,
        90, 173, 55, 210, 100, 219, 122, 192, 210, 212, 2, 255, 70, 115, 230, 188, 198, 109, 206,
        102, 186, 36, 196, 49, 156, 253, 128, 202, 250, 206, 151, 21, 197, 187, 101, 59, 179, 137,
        5, 117, 28, 188, 44, 82, 127, 27, 65, 171, 197, 211, 120, 165, 205, 213, 236, 252, 141, 6,
        233, 116, 135, 172, 75, 147, 231, 53, 170, 113, 236, 15, 157, 69, 230, 237, 23, 3, 157,
        206, 66, 152, 143, 253, 8, 241, 88, 190, 153, 207, 3, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization);

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}

#[test]
fn complex_brillig_foreign_call() {
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let a = Witness(1);
    let b = Witness(2);
    let c = Witness(3);

    let a_times_2 = Witness(4);
    let b_times_3 = Witness(5);
    let c_times_4 = Witness(6);
    let a_plus_b_plus_c = Witness(7);
    let a_plus_b_plus_c_times_2 = Witness(8);

    let brillig_bytecode = BrilligBytecode {
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

    let opcodes = vec![Opcode::BrilligCall {
        id: BrilligFunctionId(0),
        inputs: vec![
            // Input 0,1,2
            BrilligInputs::Array(vec![
                Expression::from(a),
                Expression::from(b),
                Expression::from(c),
            ]),
            // Input 3
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, a), (fe_1, b), (fe_1, c)],
                q_c: fe_0,
            }),
        ],
        // This tells the BrilligSolver which witnesses its output values correspond to
        outputs: vec![
            BrilligOutputs::Array(vec![a_times_2, b_times_3, c_times_4]), // Output 0,1,2
            BrilligOutputs::Simple(a_plus_b_plus_c),                      // Output 3
            BrilligOutputs::Simple(a_plus_b_plus_c_times_2),              // Output 4
        ],
        predicate: None,
    }];

    let circuit = Circuit {
        current_witness_index: 8,
        opcodes,
        private_parameters: BTreeSet::from([Witness(1), Witness(2), Witness(3)]),
        ..Circuit::default()
    };
    let program =
        Program { functions: vec![circuit], unconstrained_functions: vec![brillig_bytecode] };

    let bytes = Program::serialize_program(&program);
    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 85, 205, 110, 19, 49, 24, 204, 38, 1, 250, 24, 72,
        240, 0, 105, 43, 238, 148, 160, 10, 14, 156, 170, 158, 45, 103, 253, 37, 88, 242, 218, 198,
        246, 150, 132, 155, 19, 16, 215, 252, 28, 185, 209, 230, 103, 181, 41, 80, 42, 212, 43,
        175, 193, 219, 224, 13, 217, 176, 169, 72, 64, 77, 66, 219, 61, 237, 126, 178, 60, 227, 25,
        207, 108, 190, 25, 85, 67, 238, 27, 42, 184, 238, 188, 255, 230, 135, 74, 1, 55, 232, 53,
        53, 28, 180, 70, 148, 19, 168, 111, 13, 133, 244, 5, 1, 221, 177, 241, 19, 69, 25, 163,
        181, 50, 102, 236, 237, 71, 74, 114, 3, 202, 101, 104, 116, 215, 246, 247, 148, 194, 141,
        94, 43, 10, 66, 134, 12, 168, 64, 183, 207, 25, 229, 128, 21, 242, 69, 80, 161, 28, 255,
        194, 232, 254, 120, 92, 90, 237, 217, 246, 142, 95, 33, 127, 229, 109, 74, 255, 131, 106,
        254, 246, 80, 45, 172, 135, 170, 29, 28, 80, 94, 99, 240, 55, 202, 189, 117, 92, 132, 117,
        56, 116, 115, 164, 27, 138, 208, 36, 97, 234, 165, 97, 42, 222, 185, 155, 200, 25, 72, 6,
        247, 210, 151, 173, 72, 42, 32, 212, 199, 6, 190, 127, 129, 186, 251, 208, 218, 233, 233,
        18, 75, 204, 203, 232, 144, 87, 68, 232, 50, 75, 190, 74, 69, 143, 220, 26, 36, 177, 194,
        1, 56, 31, 116, 207, 203, 23, 206, 100, 88, 97, 212, 207, 76, 219, 167, 10, 76, 168, 56,
        58, 194, 44, 4, 221, 254, 140, 181, 6, 101, 80, 224, 246, 197, 53, 55, 184, 112, 245, 224,
        252, 50, 10, 59, 19, 9, 250, 221, 22, 118, 84, 105, 24, 72, 138, 225, 131, 237, 151, 147,
        37, 173, 216, 149, 132, 153, 58, 108, 7, 79, 169, 2, 223, 228, 70, 21, 106, 144, 166, 111,
        192, 14, 159, 115, 3, 53, 80, 199, 135, 187, 59, 253, 9, 222, 202, 162, 237, 46, 131, 246,
        54, 10, 93, 178, 227, 164, 6, 9, 54, 184, 44, 100, 163, 117, 158, 97, 128, 48, 33, 137, 51,
        41, 147, 251, 227, 132, 197, 229, 105, 238, 147, 168, 86, 53, 152, 203, 115, 239, 218, 244,
        220, 41, 45, 131, 46, 108, 84, 207, 237, 101, 208, 197, 13, 223, 162, 127, 183, 210, 251,
        163, 149, 133, 5, 86, 22, 109, 188, 47, 20, 208, 26, 79, 0, 222, 141, 210, 248, 12, 93, 23,
        186, 52, 215, 199, 25, 36, 23, 252, 232, 25, 96, 57, 9, 127, 115, 40, 5, 117, 7, 85, 51,
        223, 79, 18, 220, 130, 61, 125, 1, 129, 80, 141, 189, 121, 156, 7, 11, 230, 15, 47, 178,
        71, 153, 168, 133, 76, 67, 194, 172, 100, 154, 113, 102, 216, 73, 91, 166, 191, 79, 129,
        145, 41, 226, 220, 108, 254, 107, 246, 251, 191, 50, 113, 239, 108, 178, 69, 150, 90, 247,
        170, 212, 110, 102, 106, 54, 219, 66, 143, 236, 201, 129, 17, 210, 198, 211, 26, 79, 238,
        240, 2, 11, 102, 132, 126, 2, 214, 167, 16, 147, 245, 9, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization);

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}

#[test]
fn memory_op_circuit() {
    let init = vec![Witness(1), Witness(2)];

    let memory_init = Opcode::MemoryInit {
        block_id: BlockId(0),
        init,
        block_type: acir::circuit::opcodes::BlockType::Memory,
    };
    let write = Opcode::MemoryOp {
        block_id: BlockId(0),
        op: MemOp::write_to_mem_index(FieldElement::from(1u128).into(), Witness(3).into()),
        predicate: None,
    };
    let read = Opcode::MemoryOp {
        block_id: BlockId(0),
        op: MemOp::read_at_mem_index(FieldElement::one().into(), Witness(4)),
        predicate: None,
    };

    let circuit = Circuit {
        current_witness_index: 5,
        opcodes: vec![memory_init, write, read],
        private_parameters: BTreeSet::from([Witness(1), Witness(2), Witness(3)]),
        return_values: PublicInputs([Witness(4)].into()),
        ..Circuit::default()
    };
    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 221, 84, 59, 78, 3, 49, 16, 37, 155, 112, 159, 229, 6,
        180, 20, 136, 138, 218, 242, 218, 3, 88, 172, 63, 204, 216, 33, 41, 195, 22, 180, 187, 201,
        13, 248, 198, 145, 16, 63, 33, 90, 174, 193, 109, 48, 187, 18, 74, 71, 145, 8, 161, 184,
        177, 102, 252, 228, 55, 243, 158, 102, 178, 139, 120, 20, 140, 240, 202, 26, 106, 46, 223,
        68, 64, 4, 227, 217, 185, 242, 6, 136, 152, 50, 18, 70, 219, 247, 214, 9, 43, 129, 102,
        147, 197, 62, 104, 139, 227, 61, 163, 124, 53, 47, 74, 43, 78, 153, 146, 91, 55, 42, 197,
        211, 94, 182, 232, 50, 126, 236, 224, 174, 3, 78, 230, 221, 125, 224, 150, 224, 87, 214,
        85, 209, 58, 64, 254, 77, 91, 69, 29, 74, 230, 1, 53, 213, 175, 165, 50, 192, 145, 9, 171,
        11, 101, 218, 103, 170, 175, 207, 152, 248, 220, 205, 87, 59, 59, 183, 109, 43, 127, 69,
        54, 228, 101, 128, 223, 200, 154, 233, 234, 76, 253, 181, 20, 156, 71, 135, 32, 149, 224,
        30, 62, 254, 141, 101, 249, 134, 90, 54, 88, 187, 101, 79, 48, 74, 1, 81, 170, 48, 77, 174,
        244, 39, 241, 208, 20, 54, 36, 245, 228, 139, 67, 53, 76, 24, 230, 56, 114, 13, 169, 51,
        154, 245, 178, 254, 179, 11, 69, 169, 196, 82, 182, 126, 64, 240, 1, 13, 107, 117, 160,
        102, 240, 200, 137, 0, 61, 211, 233, 99, 126, 12, 84, 191, 167, 61, 145, 36, 240, 200, 147,
        46, 146, 253, 172, 141, 250, 11, 130, 86, 177, 6, 68, 4, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization);

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
    let nested_call = Opcode::Call {
        id: AcirFunctionId(1),
        inputs: vec![Witness(0), Witness(1)],
        outputs: vec![Witness(2)],
        predicate: None,
    };
    let nested_call_two = Opcode::Call {
        id: AcirFunctionId(1),
        inputs: vec![Witness(0), Witness(1)],
        outputs: vec![Witness(3)],
        predicate: None,
    };

    let assert_nested_call_results = Opcode::AssertZero(Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![
            (FieldElement::one(), Witness(2)),
            (-FieldElement::one(), Witness(3)),
        ],
        q_c: FieldElement::zero(),
    });

    let main = Circuit {
        current_witness_index: 3,
        private_parameters: BTreeSet::from([Witness(0)]),
        public_parameters: PublicInputs([Witness(1)].into()),
        opcodes: vec![nested_call, nested_call_two, assert_nested_call_results],
        ..Circuit::default()
    };

    let call_parameter_addition = Opcode::AssertZero(Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![
            (FieldElement::one(), Witness(0)),
            (-FieldElement::one(), Witness(2)),
        ],
        q_c: FieldElement::one() + FieldElement::one(),
    });
    let call = Opcode::Call {
        id: AcirFunctionId(2),
        inputs: vec![Witness(2), Witness(1)],
        outputs: vec![Witness(3)],
        predicate: None,
    };

    let nested_call = Circuit {
        current_witness_index: 3,
        private_parameters: BTreeSet::from([Witness(0), Witness(1)]),
        return_values: PublicInputs([Witness(3)].into()),
        opcodes: vec![call_parameter_addition, call],
        ..Circuit::default()
    };

    let assert_param_equality = Opcode::AssertZero(Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![
            (FieldElement::one(), Witness(0)),
            (-FieldElement::one(), Witness(1)),
        ],
        q_c: FieldElement::zero(),
    });

    let inner_call = Circuit {
        current_witness_index: 1,
        private_parameters: BTreeSet::from([Witness(0), Witness(1)]),
        return_values: PublicInputs([Witness(0)].into()),
        opcodes: vec![assert_param_equality],
        ..Circuit::default()
    };

    let program =
        Program { functions: vec![main, nested_call, inner_call], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 148, 75, 78, 195, 48, 16, 64, 243, 185, 80, 190,
        109, 178, 3, 113, 6, 54, 108, 44, 39, 158, 128, 165, 196, 9, 254, 148, 110, 249, 72, 108,
        147, 244, 6, 128, 168, 130, 132, 248, 9, 177, 229, 26, 220, 6, 83, 81, 40, 42, 130, 162,
        86, 32, 188, 178, 70, 163, 209, 248, 189, 241, 88, 7, 93, 166, 88, 42, 105, 201, 196, 232,
        248, 62, 85, 156, 3, 147, 104, 143, 74, 6, 66, 32, 202, 8, 12, 237, 113, 89, 165, 37, 1,
        49, 218, 63, 219, 192, 121, 126, 116, 66, 137, 121, 78, 89, 165, 164, 104, 13, 115, 92, 42,
        249, 114, 109, 172, 174, 226, 64, 104, 138, 37, 60, 126, 157, 105, 207, 102, 94, 172, 11,
        1, 92, 110, 1, 47, 15, 187, 66, 229, 72, 2, 47, 68, 125, 151, 83, 6, 152, 163, 180, 44, 18,
        202, 240, 164, 195, 182, 125, 90, 115, 150, 59, 174, 165, 107, 248, 78, 47, 8, 160, 239,
        129, 235, 187, 216, 241, 226, 36, 10, 157, 32, 76, 122, 145, 27, 185, 97, 20, 18, 47, 242,
        125, 136, 130, 168, 31, 39, 113, 223, 137, 221, 192, 7, 55, 11, 99, 63, 123, 45, 98, 159,
        238, 162, 116, 233, 86, 156, 107, 24, 106, 14, 66, 232, 167, 105, 226, 68, 238, 116, 155,
        44, 41, 149, 102, 78, 110, 43, 78, 7, 26, 15, 170, 48, 199, 5, 104, 36, 162, 49, 110, 42,
        149, 228, 52, 157, 141, 153, 151, 28, 164, 226, 12, 13, 112, 174, 64, 212, 87, 120, 2, 19,
        21, 186, 44, 222, 214, 129, 239, 156, 182, 191, 205, 223, 88, 1, 127, 107, 37, 252, 189,
        247, 33, 181, 166, 67, 106, 125, 62, 164, 63, 51, 165, 71, 125, 94, 85, 253, 209, 84, 99,
        47, 170, 202, 156, 170, 106, 254, 163, 42, 243, 47, 190, 202, 66, 2, 140, 57, 1, 15, 122,
        19, 106, 116, 146, 99, 205, 147, 160, 183, 197, 88, 63, 3, 120, 190, 183, 13, 38, 5, 0, 0,
    ];
    assert_eq!(bytes, expected_serialization);

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}
