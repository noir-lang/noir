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
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, BlockId, FunctionInput, MemOp},
        Circuit, Opcode, Program, PublicInputs,
    },
    native_types::{Expression, Witness},
};
use acir_field::FieldElement;
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

    let circuit = Circuit {
        current_witness_index: 4,
        opcodes: vec![addition],
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        return_values: PublicInputs([Witness(3)].into()),
        ..Circuit::default()
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
fn fixed_base_scalar_mul_circuit() {
    let fixed_base_scalar_mul = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::FixedBaseScalarMul {
        low: FunctionInput { witness: Witness(1), num_bits: 128 },
        high: FunctionInput { witness: Witness(2), num_bits: 128 },
        outputs: (Witness(3), Witness(4)),
    });

    let circuit = Circuit {
        current_witness_index: 5,
        opcodes: vec![fixed_base_scalar_mul],
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(3), Witness(4)])),
        ..Circuit::default()
    };
    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 85, 138, 81, 10, 0, 48, 8, 66, 87, 219, 190, 118, 233,
        29, 61, 35, 3, 19, 228, 137, 60, 91, 149, 139, 26, 119, 242, 145, 31, 117, 114, 163, 135,
        142, 139, 219, 91, 127, 117, 71, 2, 117, 84, 50, 98, 113, 0, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn pedersen_circuit() {
    let pedersen = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::PedersenCommitment {
        inputs: vec![FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() }],
        outputs: (Witness(2), Witness(3)),
        domain_separator: 0,
    });

    let circuit = Circuit {
        current_witness_index: 4,
        opcodes: vec![pedersen],
        private_parameters: BTreeSet::from([Witness(1)]),
        return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
        ..Circuit::default()
    };
    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 74, 9, 10, 0, 0, 4, 115, 149, 255, 127, 88, 8, 133,
        213, 218, 137, 80, 144, 32, 182, 79, 213, 151, 173, 61, 5, 121, 245, 91, 103, 255, 191, 3,
        7, 16, 26, 112, 158, 113, 0, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn schnorr_verify_circuit() {
    let public_key_x =
        FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() };
    let public_key_y =
        FunctionInput { witness: Witness(2), num_bits: FieldElement::max_num_bits() };
    let signature =
        (3..(3 + 64)).map(|i| FunctionInput { witness: Witness(i), num_bits: 8 }).collect();
    let message = ((3 + 64)..(3 + 64 + 10))
        .map(|i| FunctionInput { witness: Witness(i), num_bits: 8 })
        .collect();
    let output = Witness(3 + 64 + 10);
    let last_input = output.witness_index() - 1;

    let schnorr = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::SchnorrVerify {
        public_key_x,
        public_key_y,
        signature,
        message,
        output,
    });

    let circuit = Circuit {
        current_witness_index: 100,
        opcodes: vec![schnorr],
        private_parameters: BTreeSet::from_iter((1..=last_input).map(Witness)),
        return_values: PublicInputs(BTreeSet::from([output])),
        ..Circuit::default()
    };
    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 85, 210, 135, 74, 66, 1, 24, 134, 97, 219, 123, 239,
        189, 247, 222, 187, 108, 153, 153, 221, 69, 247, 127, 9, 145, 31, 62, 130, 29, 56, 60, 133,
        32, 242, 127, 111, 75, 161, 254, 252, 212, 222, 22, 127, 199, 78, 254, 214, 222, 86, 22,
        125, 222, 86, 123, 187, 107, 111, 59, 59, 216, 201, 46, 54, 222, 30, 246, 178, 143, 253,
        28, 224, 32, 135, 56, 204, 17, 142, 114, 140, 227, 156, 224, 36, 167, 56, 205, 25, 206,
        114, 142, 243, 92, 224, 34, 151, 184, 204, 21, 174, 114, 141, 235, 220, 224, 38, 183, 184,
        205, 29, 238, 114, 143, 251, 60, 224, 33, 143, 120, 204, 19, 158, 242, 140, 231, 188, 224,
        37, 175, 120, 205, 27, 222, 242, 142, 247, 124, 224, 35, 159, 88, 228, 51, 95, 154, 118,
        204, 243, 234, 255, 55, 190, 179, 196, 15, 150, 249, 201, 10, 191, 88, 229, 183, 239, 173,
        50, 253, 165, 189, 244, 150, 214, 210, 89, 26, 107, 244, 213, 227, 183, 164, 167, 180, 148,
        142, 210, 80, 250, 73, 59, 233, 38, 205, 164, 151, 180, 146, 78, 210, 72, 250, 72, 27, 233,
        34, 77, 164, 135, 180, 144, 14, 210, 64, 246, 95, 46, 212, 119, 207, 230, 217, 59, 91, 103,
        231, 108, 156, 125, 183, 237, 186, 107, 207, 125, 59, 30, 218, 239, 216, 110, 167, 246, 58,
        183, 211, 165, 125, 174, 237, 114, 107, 143, 123, 59, 60, 186, 127, 209, 221, 95, 220, 249,
        205, 125, 75, 238, 90, 118, 207, 138, 59, 54, 110, 214, 184, 91, 161, 233, 158, 255, 158,
        63, 46, 139, 64, 203, 241, 3, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn simple_brillig_foreign_call() {
    let w_input = Witness(1);
    let w_inverted = Witness(2);

    let brillig_data = Brillig {
        inputs: vec![
            BrilligInputs::Single(w_input.into()), // Input Register 0,
        ],
        // This tells the BrilligSolver which witnesses its output values correspond to
        outputs: vec![
            BrilligOutputs::Simple(w_inverted), // Output Register 1
        ],
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
        predicate: None,
    };

    let opcodes = vec![Opcode::Brillig(brillig_data)];
    let circuit = Circuit {
        current_witness_index: 8,
        opcodes,
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        ..Circuit::default()
    };
    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 61, 10, 192, 32, 12, 133, 19, 11, 165, 116,
        235, 77, 236, 13, 122, 153, 14, 93, 58, 136, 120, 124, 241, 47, 129, 12, 42, 130, 126, 16,
        18, 146, 16, 222, 11, 66, 225, 136, 129, 84, 111, 162, 150, 112, 239, 161, 172, 231, 184,
        113, 221, 45, 45, 245, 42, 242, 144, 216, 43, 250, 153, 83, 204, 191, 223, 189, 198, 246,
        92, 39, 60, 244, 63, 195, 59, 87, 99, 150, 165, 113, 83, 193, 0, 1, 19, 247, 29, 5, 160, 1,
        0, 0,
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

    let brillig_data = Brillig {
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
        bytecode: vec![
            brillig::Opcode::CalldataCopy {
                destination_address: MemoryAddress(32),
                size: 3,
                offset: 0,
            },
            brillig::Opcode::Const {
                destination: MemoryAddress(0),
                value: FieldElement::from(32_usize),
                bit_size: 64,
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
        predicate: None,
    };

    let opcodes = vec![Opcode::Brillig(brillig_data)];
    let circuit = Circuit {
        current_witness_index: 8,
        opcodes,
        private_parameters: BTreeSet::from([Witness(1), Witness(2), Witness(3)]),
        ..Circuit::default()
    };
    let program = Program { functions: vec![circuit], unconstrained_functions: vec![] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 84, 75, 10, 132, 48, 12, 77, 218, 209, 145, 217,
        205, 13, 6, 198, 3, 84, 79, 224, 93, 196, 157, 162, 75, 79, 47, 22, 124, 197, 16, 186, 17,
        43, 104, 32, 36, 109, 126, 143, 36, 45, 211, 70, 133, 103, 134, 110, 61, 27, 232, 140, 179,
        164, 224, 215, 64, 186, 115, 84, 113, 186, 92, 238, 42, 140, 230, 1, 24, 237, 5, 24, 195,
        62, 220, 116, 222, 41, 231, 146, 180, 127, 54, 242, 126, 94, 158, 51, 207, 57, 206, 111,
        200, 2, 247, 4, 219, 79, 245, 157, 132, 31, 137, 89, 52, 73, 176, 214, 46, 167, 125, 23,
        89, 213, 254, 8, 156, 237, 56, 76, 125, 55, 91, 229, 170, 161, 254, 133, 94, 42, 59, 171,
        184, 69, 197, 46, 66, 202, 47, 40, 86, 39, 220, 155, 3, 185, 191, 180, 183, 55, 163, 72,
        98, 70, 66, 221, 251, 40, 173, 255, 35, 68, 62, 61, 5, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn memory_op_circuit() {
    let init = vec![Witness(1), Witness(2)];

    let memory_init = Opcode::MemoryInit { block_id: BlockId(0), init };
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 81, 57, 14, 0, 32, 8, 147, 195, 255, 224, 15, 252,
        255, 171, 212, 200, 208, 129, 77, 24, 108, 66, 90, 150, 166, 20, 106, 23, 125, 143, 128,
        62, 96, 103, 114, 173, 45, 198, 116, 182, 55, 140, 106, 95, 74, 246, 149, 60, 47, 171, 46,
        215, 126, 43, 87, 179, 111, 23, 8, 202, 176, 99, 248, 240, 9, 11, 137, 33, 212, 110, 35, 3,
        0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 205, 146, 65, 10, 3, 33, 12, 69, 163, 46, 230, 58, 137,
        209, 49, 238, 122, 149, 74, 157, 251, 31, 161, 83, 154, 161, 86, 132, 89, 212, 194, 124,
        248, 24, 36, 132, 228, 241, 29, 188, 229, 212, 47, 45, 187, 205, 110, 11, 31, 25, 53, 28,
        255, 103, 77, 14, 58, 29, 141, 55, 125, 241, 55, 145, 109, 102, 49, 174, 33, 212, 228, 43,
        49, 221, 209, 231, 34, 17, 67, 44, 171, 144, 80, 148, 248, 240, 194, 92, 37, 72, 202, 37,
        39, 204, 20, 184, 210, 22, 51, 111, 58, 204, 205, 219, 11, 161, 129, 208, 214, 6, 6, 114,
        29, 193, 127, 193, 130, 137, 176, 236, 188, 189, 252, 162, 183, 218, 230, 238, 97, 138,
        250, 152, 245, 245, 87, 220, 12, 140, 113, 95, 153, 170, 129, 185, 17, 60, 3, 54, 212, 19,
        104, 145, 195, 151, 14, 4, 0, 0,
    ];
    assert_eq!(bytes, expected_serialization);
}
