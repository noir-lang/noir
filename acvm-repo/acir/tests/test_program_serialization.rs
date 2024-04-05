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
    let program = Program { functions: vec![circuit] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 75, 14, 128, 32, 12, 68, 249, 120, 160, 150,
        182, 208, 238, 188, 138, 68, 184, 255, 17, 212, 200, 130, 196, 165, 188, 164, 153, 174, 94,
        38, 227, 221, 203, 118, 159, 119, 95, 226, 200, 125, 36, 252, 3, 253, 66, 87, 152, 92, 4,
        153, 185, 149, 212, 144, 240, 128, 100, 85, 5, 88, 106, 86, 84, 20, 149, 51, 41, 81, 83,
        214, 98, 213, 10, 24, 50, 53, 236, 98, 212, 135, 44, 174, 235, 5, 143, 35, 12, 151, 159,
        126, 55, 109, 28, 231, 145, 47, 245, 105, 191, 143, 133, 1, 0, 0,
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
    let program = Program { functions: vec![circuit] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 138, 81, 10, 0, 48, 8, 66, 87, 219, 190, 118, 233,
        29, 61, 43, 3, 5, 121, 34, 207, 86, 231, 162, 198, 157, 124, 228, 71, 157, 220, 232, 161,
        227, 226, 206, 214, 95, 221, 74, 0, 116, 58, 13, 182, 105, 0, 0, 0,
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
    let program = Program { functions: vec![circuit] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 74, 7, 6, 0, 0, 8, 108, 209, 255, 63, 156, 54, 233,
        56, 55, 17, 26, 18, 196, 241, 169, 250, 178, 141, 167, 32, 159, 254, 234, 238, 255, 87,
        112, 52, 63, 63, 101, 105, 0, 0, 0,
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
    let program = Program { functions: vec![circuit] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 210, 7, 78, 2, 1, 20, 69, 81, 236, 189, 247, 222,
        123, 239, 93, 177, 33, 34, 238, 194, 253, 47, 193, 200, 147, 67, 194, 36, 147, 163, 33, 33,
        228, 191, 219, 82, 168, 63, 63, 181, 183, 197, 223, 177, 147, 191, 181, 183, 149, 69, 159,
        183, 213, 222, 238, 218, 219, 206, 14, 118, 178, 139, 141, 183, 135, 189, 236, 99, 63, 7,
        56, 200, 33, 14, 115, 132, 163, 28, 227, 56, 39, 56, 201, 41, 78, 115, 134, 179, 156, 227,
        60, 23, 184, 200, 37, 46, 115, 133, 171, 92, 227, 58, 55, 184, 201, 45, 110, 115, 135, 187,
        220, 227, 62, 15, 120, 200, 35, 30, 243, 132, 167, 60, 227, 57, 47, 120, 201, 43, 94, 243,
        134, 183, 188, 227, 61, 31, 248, 200, 39, 22, 249, 204, 151, 166, 29, 243, 188, 250, 255,
        141, 239, 44, 241, 131, 101, 126, 178, 194, 47, 86, 249, 237, 123, 171, 76, 127, 105, 47,
        189, 165, 181, 116, 150, 198, 26, 125, 245, 248, 45, 233, 41, 45, 165, 163, 52, 148, 126,
        210, 78, 186, 73, 51, 233, 37, 173, 164, 147, 52, 146, 62, 210, 70, 186, 72, 19, 233, 33,
        45, 164, 131, 52, 144, 253, 151, 11, 245, 221, 179, 121, 246, 206, 214, 217, 57, 27, 103,
        223, 109, 187, 238, 218, 115, 223, 142, 135, 246, 59, 182, 219, 169, 189, 206, 237, 116,
        105, 159, 107, 187, 220, 218, 227, 222, 14, 143, 238, 95, 116, 247, 23, 119, 126, 115, 223,
        146, 187, 150, 221, 179, 226, 142, 141, 155, 53, 238, 86, 104, 186, 231, 255, 243, 7, 100,
        141, 232, 192, 233, 3, 0, 0,
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
    let program = Program { functions: vec![circuit] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 61, 10, 192, 48, 8, 133, 53, 133, 82, 186,
        245, 38, 233, 13, 122, 153, 14, 93, 58, 132, 144, 227, 135, 252, 41, 56, 36, 46, 201, 7,
        162, 168, 200, 123, 34, 52, 142, 28, 72, 245, 38, 106, 9, 247, 30, 202, 118, 142, 27, 215,
        221, 178, 82, 175, 33, 15, 133, 189, 163, 159, 57, 197, 252, 251, 195, 235, 188, 230, 186,
        16, 65, 255, 12, 239, 92, 131, 89, 149, 198, 77, 3, 10, 9, 119, 8, 198, 242, 152, 1, 0, 0,
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
    let program = Program { functions: vec![circuit] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 84, 93, 10, 131, 48, 12, 78, 218, 233, 100, 111,
        187, 193, 96, 59, 64, 231, 9, 188, 139, 248, 166, 232, 163, 167, 23, 11, 126, 197, 24, 250,
        34, 86, 208, 64, 72, 218, 252, 125, 36, 105, 153, 22, 42, 60, 51, 116, 235, 217, 64, 103,
        156, 37, 5, 191, 10, 210, 29, 163, 63, 167, 203, 229, 206, 194, 104, 110, 128, 209, 158,
        128, 49, 236, 195, 69, 231, 157, 114, 46, 73, 251, 103, 35, 239, 231, 225, 57, 243, 156,
        227, 252, 132, 44, 112, 79, 176, 125, 84, 223, 73, 248, 145, 152, 69, 149, 4, 107, 233,
        114, 90, 119, 145, 85, 237, 151, 192, 89, 247, 221, 208, 54, 163, 85, 174, 26, 234, 87,
        232, 63, 101, 103, 21, 55, 169, 216, 73, 72, 249, 5, 197, 234, 132, 123, 179, 35, 247, 155,
        214, 246, 102, 20, 73, 204, 72, 168, 123, 191, 161, 25, 66, 136, 159, 187, 53, 5, 0, 0,
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
    let program = Program { functions: vec![circuit] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 81, 201, 13, 0, 32, 8, 147, 195, 125, 112, 3, 247,
        159, 74, 141, 60, 106, 226, 79, 120, 216, 132, 180, 124, 154, 82, 168, 108, 212, 57, 2,
        122, 129, 157, 201, 181, 150, 59, 186, 179, 189, 161, 101, 251, 82, 176, 175, 196, 121, 89,
        118, 185, 246, 91, 185, 26, 125, 187, 64, 80, 134, 29, 195, 31, 79, 24, 2, 250, 167, 252,
        27, 3, 0, 0,
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
    let nested_call =
        Opcode::Call { id: 1, inputs: vec![Witness(0), Witness(1)], outputs: vec![Witness(2)] };
    let nested_call_two =
        Opcode::Call { id: 1, inputs: vec![Witness(0), Witness(1)], outputs: vec![Witness(3)] };

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
    let call =
        Opcode::Call { id: 2, inputs: vec![Witness(2), Witness(1)], outputs: vec![Witness(3)] };

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

    let program = Program { functions: vec![main, nested_call, inner_call] };

    let bytes = Program::serialize_program(&program);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 205, 146, 97, 10, 195, 32, 12, 133, 163, 66, 207, 147,
        24, 109, 227, 191, 93, 101, 50, 123, 255, 35, 172, 99, 25, 83, 17, 250, 99, 14, 250, 224,
        97, 144, 16, 146, 143, 231, 224, 45, 167, 126, 105, 57, 108, 14, 91, 248, 202, 168, 65,
        255, 207, 122, 28, 180, 250, 244, 221, 244, 197, 223, 68, 182, 154, 197, 184, 134, 80, 54,
        95, 136, 233, 142, 62, 101, 137, 24, 98, 94, 133, 132, 162, 196, 135, 23, 230, 34, 65, 182,
        148, 211, 134, 137, 2, 23, 218, 99, 226, 93, 135, 185, 121, 123, 33, 84, 12, 234, 218, 192,
        64, 174, 3, 248, 47, 88, 48, 17, 150, 157, 183, 151, 95, 244, 86, 91, 221, 61, 10, 81, 31,
        178, 190, 110, 194, 102, 96, 76, 251, 202, 80, 13, 204, 77, 224, 25, 176, 70, 79, 197, 128,
        18, 64, 3, 4, 0, 0,
    ];
    assert_eq!(bytes, expected_serialization);
}
