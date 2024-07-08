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
        brillig::{BrilligBytecode, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, BlockId, ConstantOrWitnessEnum, FunctionInput, MemOp},
        Circuit, Opcode, Program, PublicInputs,
    },
    native_types::{Expression, Witness},
};
use acir_field::{AcirField, FieldElement};
use brillig::{HeapArray, HeapValueType, MemoryAddress, ValueOrArray};

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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 65, 14, 128, 32, 12, 4, 65, 124, 80, 75, 91,
        104, 111, 126, 69, 34, 252, 255, 9, 106, 228, 64, 162, 55, 153, 164, 217, 158, 38, 155,
        245, 238, 97, 189, 206, 187, 55, 161, 231, 214, 19, 254, 129, 126, 162, 107, 25, 92, 4,
        137, 185, 230, 88, 145, 112, 135, 104, 69, 5, 88, 74, 82, 84, 20, 149, 35, 42, 81, 85, 214,
        108, 197, 50, 24, 50, 85, 108, 98, 212, 186, 44, 204, 235, 5, 183, 99, 233, 46, 63, 252,
        110, 216, 56, 184, 15, 78, 146, 74, 173, 20, 141, 1, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn multi_scalar_mul_circuit() {
    let multi_scalar_mul: Opcode<FieldElement> =
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::MultiScalarMul {
            points: vec![
                FunctionInput::witness(Witness(1), 128),
                FunctionInput::witness(Witness(2), 128),
                FunctionInput::witness(Witness(3), 1),
            ],
            scalars: vec![
                FunctionInput::witness(Witness(4), 128),
                FunctionInput::witness(Witness(5), 128),
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 141, 11, 10, 0, 32, 8, 67, 43, 181, 15, 116, 232,
        142, 158, 210, 130, 149, 240, 112, 234, 212, 156, 78, 12, 39, 67, 71, 158, 142, 80, 29, 44,
        228, 66, 90, 168, 119, 189, 74, 115, 131, 174, 78, 115, 58, 124, 70, 254, 130, 59, 74, 253,
        68, 255, 255, 221, 39, 54, 221, 93, 91, 132, 193, 0, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn schnorr_verify_circuit() {
    let public_key_x = FunctionInput::witness(Witness(1), FieldElement::max_num_bits());
    let public_key_y = FunctionInput::witness(Witness(2), FieldElement::max_num_bits());
    let signature: [FunctionInput<FieldElement>; 64] = (3..(3 + 64))
        .map(|i| FunctionInput::witness(Witness(i), 8))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let message =
        ((3 + 64)..(3 + 64 + 10)).map(|i| FunctionInput::witness(Witness(i), 8)).collect();
    let output = Witness(3 + 64 + 10);
    let last_input = output.witness_index() - 1;

    let schnorr = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::SchnorrVerify {
        public_key_x,
        public_key_y,
        signature: Box::new(signature),
        message,
        output,
    });

    let circuit: Circuit<FieldElement> = Circuit {
        current_witness_index: 100,
        opcodes: vec![schnorr],
        private_parameters: BTreeSet::from_iter((1..=last_input).map(Witness)),
        return_values: PublicInputs(BTreeSet::from([output])),
        ..Circuit::default()
    };
    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 85, 211, 103, 78, 2, 81, 24, 70, 225, 193, 6, 216, 123,
        47, 216, 123, 239, 136, 136, 136, 136, 136, 187, 96, 255, 75, 32, 112, 194, 55, 201, 129,
        100, 50, 79, 244, 7, 228, 222, 243, 102, 146, 254, 167, 221, 123, 50, 97, 222, 217, 120,
        243, 116, 226, 61, 36, 15, 247, 158, 92, 120, 68, 30, 149, 199, 228, 172, 156, 147, 243,
        242, 184, 60, 33, 79, 202, 83, 242, 180, 60, 35, 207, 202, 115, 242, 188, 188, 32, 47, 202,
        75, 242, 178, 188, 34, 175, 202, 107, 242, 186, 188, 33, 111, 202, 91, 242, 182, 188, 35,
        23, 228, 93, 121, 79, 222, 151, 15, 228, 67, 249, 72, 62, 150, 79, 228, 83, 249, 76, 62,
        151, 47, 228, 75, 249, 74, 190, 150, 111, 228, 91, 249, 78, 190, 151, 31, 228, 71, 249, 73,
        126, 150, 95, 228, 87, 185, 40, 191, 201, 37, 249, 93, 46, 203, 31, 114, 69, 254, 148, 171,
        97, 58, 77, 226, 111, 95, 250, 127, 77, 254, 150, 235, 242, 143, 220, 144, 127, 229, 166,
        252, 39, 183, 194, 255, 241, 253, 45, 253, 14, 182, 201, 38, 217, 34, 27, 100, 123, 233,
        230, 242, 241, 155, 217, 20, 91, 98, 67, 108, 135, 205, 176, 21, 54, 194, 54, 216, 4, 91,
        96, 3, 180, 79, 243, 180, 78, 227, 180, 77, 211, 180, 76, 195, 180, 75, 179, 133, 164, 223,
        40, 109, 210, 36, 45, 210, 32, 237, 209, 28, 173, 209, 24, 109, 209, 20, 45, 209, 16, 237,
        208, 12, 173, 208, 8, 109, 208, 4, 45, 208, 0, 119, 207, 157, 115, 215, 220, 113, 49, 238,
        180, 20, 119, 88, 142, 59, 171, 196, 29, 85, 227, 46, 106, 113, 246, 245, 56, 235, 70, 156,
        109, 51, 206, 50, 61, 179, 244, 220, 18, 157, 231, 192, 167, 11, 75, 28, 99, 152, 25, 5, 0,
        0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn simple_brillig_foreign_call() {
    let w_input = Witness(1);
    let w_inverted = Witness(2);

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            brillig::Opcode::CalldataCopy {
                destination_address: MemoryAddress(0),
                size: 1,
                offset: 0,
            },
            brillig::Opcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(0))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(0))],
                input_value_types: vec![HeapValueType::field()],
            },
            brillig::Opcode::Stop { return_data_offset: 0, return_data_size: 1 },
        ],
    };

    let opcodes = vec![Opcode::BrilligCall {
        id: 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 193, 10, 192, 32, 8, 134, 117, 99, 99, 236,
        182, 55, 105, 111, 176, 151, 217, 161, 75, 135, 136, 30, 63, 42, 82, 144, 8, 47, 245, 65,
        252, 230, 47, 162, 34, 52, 174, 242, 144, 226, 131, 148, 255, 18, 206, 125, 164, 102, 142,
        23, 215, 245, 50, 114, 222, 173, 15, 80, 38, 65, 217, 108, 39, 61, 7, 30, 115, 11, 223,
        186, 248, 251, 160, 221, 170, 146, 64, 191, 39, 215, 60, 3, 47, 3, 99, 171, 188, 84, 164,
        1, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
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
            brillig::Opcode::CalldataCopy {
                destination_address: MemoryAddress(32),
                size: 3,
                offset: 0,
            },
            brillig::Opcode::Const {
                destination: MemoryAddress(0),
                value: FieldElement::from(32_usize),
                bit_size: 32,
            },
            brillig::Opcode::CalldataCopy {
                destination_address: MemoryAddress(1),
                size: 1,
                offset: 3,
            },
            // Oracles are named 'foreign calls' in brillig
            brillig::Opcode::ForeignCall {
                function: "complex".into(),
                inputs: vec![
                    ValueOrArray::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
                    ValueOrArray::MemoryAddress(MemoryAddress::from(1)),
                ],
                input_value_types: vec![
                    HeapValueType::Array { size: 3, value_types: vec![HeapValueType::field()] },
                    HeapValueType::field(),
                ],
                destinations: vec![
                    ValueOrArray::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
                    ValueOrArray::MemoryAddress(MemoryAddress::from(35)),
                    ValueOrArray::MemoryAddress(MemoryAddress::from(36)),
                ],
                destination_value_types: vec![
                    HeapValueType::Array { size: 3, value_types: vec![HeapValueType::field()] },
                    HeapValueType::field(),
                    HeapValueType::field(),
                ],
            },
            brillig::Opcode::Stop { return_data_offset: 32, return_data_size: 5 },
        ],
    };

    let opcodes = vec![Opcode::BrilligCall {
        id: 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 84, 75, 10, 131, 64, 12, 77, 102, 90, 43, 221, 245,
        6, 133, 246, 0, 211, 158, 192, 187, 136, 59, 69, 151, 158, 94, 116, 48, 131, 241, 233, 70,
        28, 65, 3, 195, 155, 79, 62, 47, 9, 25, 166, 81, 210, 97, 177, 236, 239, 130, 70, 208, 223,
        91, 154, 75, 208, 205, 4, 221, 62, 249, 113, 60, 95, 238, 40, 142, 230, 2, 28, 237, 1, 28,
        73, 245, 255, 132, 253, 142, 217, 151, 168, 245, 179, 43, 243, 115, 163, 113, 190, 18, 57,
        63, 4, 83, 44, 180, 55, 50, 180, 28, 188, 153, 224, 196, 122, 175, 111, 112, 68, 24, 65,
        116, 178, 40, 89, 254, 93, 162, 120, 48, 196, 126, 170, 12, 243, 186, 106, 202, 162, 181,
        160, 138, 84, 63, 106, 255, 133, 119, 6, 187, 14, 108, 59, 133, 250, 243, 90, 139, 19, 238,
        205, 6, 223, 47, 154, 202, 27, 74, 222, 3, 234, 73, 242, 82, 65, 5, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 82, 65, 10, 0, 32, 8, 211, 180, 255, 216, 15, 250,
        255, 171, 10, 82, 176, 232, 150, 30, 26, 200, 118, 144, 49, 135, 8, 11, 117, 14, 169, 102,
        229, 162, 140, 78, 219, 206, 137, 174, 44, 111, 104, 217, 190, 24, 236, 75, 113, 94, 146,
        93, 174, 252, 86, 46, 71, 223, 78, 46, 104, 129, 253, 155, 45, 60, 195, 5, 3, 89, 11, 161,
        73, 39, 3, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
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
        id: 1,
        inputs: vec![Witness(0), Witness(1)],
        outputs: vec![Witness(2)],
        predicate: None,
    };
    let nested_call_two = Opcode::Call {
        id: 1,
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
        id: 2,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 205, 146, 97, 10, 195, 32, 12, 133, 163, 66, 207, 147,
        24, 173, 241, 223, 174, 50, 153, 189, 255, 17, 214, 177, 148, 57, 17, 250, 99, 14, 250,
        224, 97, 144, 16, 146, 143, 231, 224, 45, 167, 126, 105, 217, 109, 118, 91, 248, 200, 168,
        225, 248, 63, 107, 114, 208, 233, 104, 188, 233, 139, 191, 137, 108, 51, 139, 113, 13, 161,
        38, 95, 137, 233, 142, 62, 23, 137, 24, 98, 89, 133, 132, 162, 196, 135, 23, 230, 42, 65,
        82, 46, 57, 97, 166, 192, 149, 182, 152, 121, 211, 97, 110, 222, 94, 8, 13, 132, 182, 54,
        48, 144, 235, 8, 254, 11, 22, 76, 132, 101, 231, 237, 229, 23, 189, 213, 54, 119, 15, 83,
        212, 199, 172, 175, 191, 226, 102, 96, 140, 251, 202, 84, 13, 204, 141, 224, 25, 176, 161,
        158, 53, 121, 144, 73, 14, 4, 0, 0,
    ];
    assert_eq!(bytes, expected_serialization);
}
