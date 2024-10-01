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
        brillig::{BrilligBytecode, BrilligFunctionId, BrilligInputs, BrilligOutputs},
        opcodes::{AcirFunctionId, BlackBoxFuncCall, BlockId, FunctionInput, MemOp},
        Circuit, Opcode, Program, PublicInputs,
    },
    native_types::{Expression, Witness},
};
use acir_field::{AcirField, FieldElement};
use brillig::{BitSize, HeapArray, HeapValueType, IntegerBitSize, MemoryAddress, ValueOrArray};

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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 141, 11, 10, 0, 32, 8, 67, 43, 181, 15, 221, 255,
        186, 145, 210, 130, 149, 240, 112, 234, 212, 156, 78, 12, 39, 67, 71, 158, 142, 80, 29, 44,
        228, 66, 90, 168, 119, 189, 74, 115, 131, 174, 78, 115, 58, 124, 70, 254, 130, 59, 74, 253,
        68, 255, 255, 221, 39, 54, 29, 134, 27, 102, 193, 0, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 85, 211, 103, 78, 2, 81, 24, 70, 225, 193, 130, 96, 239,
        189, 96, 239, 189, 35, 34, 34, 34, 34, 238, 130, 253, 47, 129, 192, 9, 223, 36, 7, 146,
        201, 60, 209, 31, 144, 123, 207, 155, 73, 250, 159, 118, 239, 201, 132, 121, 103, 227, 205,
        211, 137, 247, 144, 60, 220, 123, 114, 225, 17, 121, 84, 206, 202, 99, 114, 78, 206, 203,
        227, 242, 132, 60, 41, 79, 201, 211, 242, 140, 60, 43, 207, 201, 243, 242, 130, 188, 40,
        47, 201, 203, 242, 138, 188, 42, 175, 201, 235, 242, 134, 188, 41, 111, 201, 219, 242, 142,
        92, 144, 119, 229, 61, 121, 95, 62, 144, 15, 229, 35, 249, 88, 62, 145, 79, 229, 51, 249,
        92, 190, 144, 47, 229, 43, 249, 90, 190, 145, 111, 229, 59, 249, 94, 126, 144, 31, 229, 39,
        249, 89, 126, 145, 95, 229, 162, 252, 38, 151, 228, 119, 185, 44, 127, 200, 21, 249, 83,
        174, 134, 233, 52, 137, 191, 125, 233, 255, 53, 249, 91, 174, 203, 63, 114, 67, 254, 149,
        155, 242, 159, 220, 10, 255, 199, 247, 183, 244, 59, 216, 38, 155, 100, 139, 108, 144, 237,
        165, 155, 203, 199, 111, 102, 83, 108, 137, 13, 177, 29, 54, 195, 86, 216, 8, 219, 96, 19,
        108, 129, 13, 208, 62, 205, 211, 58, 141, 211, 54, 77, 211, 50, 13, 211, 46, 205, 22, 146,
        126, 163, 180, 73, 147, 180, 72, 131, 180, 71, 115, 180, 70, 99, 180, 69, 83, 180, 68, 67,
        180, 67, 51, 180, 66, 35, 180, 65, 19, 180, 64, 3, 220, 61, 119, 206, 93, 115, 199, 197,
        184, 211, 82, 220, 97, 57, 238, 172, 18, 119, 84, 141, 187, 168, 197, 217, 215, 227, 172,
        27, 113, 182, 205, 56, 203, 244, 204, 210, 115, 75, 116, 158, 3, 159, 46, 43, 32, 188, 53,
        25, 5, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn simple_brillig_foreign_call() {
    let w_input = Witness(1);
    let w_inverted = Witness(2);

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            brillig::Opcode::Const {
                destination: MemoryAddress(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(1_usize),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0_usize),
            },
            brillig::Opcode::CalldataCopy {
                destination_address: MemoryAddress(0),
                size_address: MemoryAddress(0),
                offset_address: MemoryAddress(1),
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 81, 73, 10, 192, 48, 8, 140, 165, 91, 160, 183,
        126, 196, 254, 160, 159, 233, 161, 151, 30, 74, 200, 251, 19, 136, 130, 132, 196, 75, 28,
        16, 199, 17, 212, 65, 112, 5, 123, 14, 32, 190, 80, 230, 90, 130, 181, 155, 50, 142, 225,
        2, 187, 89, 40, 239, 157, 106, 2, 82, 116, 138, 51, 118, 239, 171, 222, 108, 232, 218, 139,
        125, 198, 179, 113, 83, 188, 29, 57, 86, 226, 239, 23, 159, 63, 104, 63, 238, 213, 45, 237,
        108, 244, 18, 195, 174, 252, 193, 92, 2, 0, 0,
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
            brillig::Opcode::Const {
                destination: MemoryAddress(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(3_usize),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0_usize),
            },
            brillig::Opcode::CalldataCopy {
                destination_address: MemoryAddress(32),
                size_address: MemoryAddress(0),
                offset_address: MemoryAddress(1),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress(0),
                value: FieldElement::from(32_usize),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress(3),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(1_usize),
            },
            brillig::Opcode::Const {
                destination: MemoryAddress(4),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(3_usize),
            },
            brillig::Opcode::CalldataCopy {
                destination_address: MemoryAddress(1),
                size_address: MemoryAddress(3),
                offset_address: MemoryAddress(4),
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 85, 81, 14, 194, 48, 8, 133, 118, 186, 53, 241,
        207, 11, 152, 232, 1, 58, 189, 128, 119, 49, 254, 105, 244, 211, 227, 59, 50, 154, 49, 214,
        100, 31, 163, 201, 246, 146, 133, 174, 5, 10, 15, 72, 17, 122, 52, 221, 135, 188, 222, 177,
        116, 44, 105, 223, 195, 24, 73, 247, 206, 50, 46, 67, 139, 118, 190, 98, 169, 24, 221, 6,
        98, 244, 5, 98, 4, 81, 255, 21, 214, 219, 178, 46, 166, 252, 249, 204, 252, 84, 208, 207,
        215, 158, 255, 107, 150, 141, 38, 154, 140, 28, 76, 7, 111, 132, 212, 61, 65, 201, 116, 86,
        217, 101, 115, 11, 226, 62, 99, 223, 145, 88, 56, 205, 228, 102, 127, 239, 53, 6, 69, 184,
        97, 78, 109, 96, 127, 37, 106, 81, 11, 126, 100, 103, 17, 14, 48, 116, 213, 227, 243, 254,
        190, 158, 63, 175, 40, 149, 102, 132, 179, 88, 95, 212, 57, 42, 59, 109, 43, 33, 31, 140,
        156, 46, 102, 244, 230, 124, 31, 97, 104, 141, 244, 48, 253, 1, 180, 46, 168, 159, 181, 6,
        0, 0,
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
