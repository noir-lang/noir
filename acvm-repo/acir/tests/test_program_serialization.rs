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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 141, 47, 10, 128, 48, 24, 197, 221, 52, 140, 47,
        46, 46, 201, 210, 108, 254, 57, 130, 8, 38, 49, 90, 108, 30, 64, 108, 198, 29, 65, 188,
        128, 8, 94, 65, 212, 219, 172, 25, 45, 118, 63, 88, 31, 254, 224, 133, 7, 191, 199, 131,
        139, 176, 128, 111, 4, 86, 194, 21, 72, 8, 137, 231, 134, 83, 246, 223, 164, 214, 196, 242,
        204, 175, 201, 167, 254, 28, 203, 59, 83, 109, 163, 245, 94, 212, 71, 180, 36, 102, 168,
        186, 24, 77, 95, 88, 207, 137, 160, 224, 73, 252, 199, 208, 20, 55, 31, 228, 93, 253, 226,
        189, 0, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 45, 204, 177, 13, 128, 48, 12, 68, 81, 236, 56, 228,
        226, 42, 114, 153, 146, 17, 24, 1, 33, 10, 134, 97, 255, 10, 113, 64, 138, 39, 125, 157,
        44, 251, 133, 26, 71, 236, 231, 230, 213, 45, 20, 210, 110, 25, 169, 111, 226, 203, 212,
        36, 254, 209, 56, 142, 204, 204, 174, 152, 169, 16, 186, 250, 180, 240, 3, 41, 37, 50, 202,
        43, 111, 168, 16, 30, 169, 242, 72, 33, 112, 0, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 142, 177, 10, 131, 64, 12, 134, 47, 161, 148, 240,
        211, 150, 146, 241, 166, 227, 166, 34, 165, 15, 209, 7, 40, 165, 83, 113, 119, 208, 193,
        65, 196, 7, 210, 23, 53, 158, 186, 170, 129, 192, 79, 248, 146, 124, 168, 68, 52, 207, 254,
        250, 197, 71, 31, 136, 8, 228, 182, 75, 89, 200, 27, 183, 131, 57, 127, 2, 11, 123, 134,
        139, 182, 97, 205, 58, 16, 94, 191, 167, 141, 73, 249, 30, 142, 28, 89, 120, 94, 249, 61,
        57, 220, 222, 23, 227, 39, 73, 123, 45, 132, 88, 5, 156, 203, 186, 43, 154, 86, 39, 161,
        164, 101, 74, 41, 103, 41, 227, 218, 19, 32, 179, 150, 240, 8, 59, 173, 170, 236, 18, 1, 0,
        0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 77, 10, 2, 49, 12, 133, 39, 141, 63, 33, 168,
        72, 150, 93, 13, 163, 11, 25, 196, 67, 120, 0, 23, 238, 196, 165, 184, 17, 69, 151, 158,
        71, 240, 18, 130, 151, 112, 237, 33, 4, 47, 96, 218, 113, 6, 113, 97, 139, 76, 33, 105,
        105, 191, 190, 188, 132, 111, 72, 36, 87, 204, 47, 40, 79, 144, 7, 200, 76, 70, 156, 113,
        10, 201, 239, 37, 134, 192, 42, 151, 132, 184, 120, 61, 83, 179, 30, 70, 233, 157, 129, 79,
        16, 223, 115, 124, 55, 245, 250, 180, 61, 233, 176, 161, 134, 70, 83, 163, 101, 221, 169,
        237, 51, 89, 195, 73, 166, 222, 52, 140, 6, 202, 29, 121, 50, 31, 235, 147, 202, 247, 83,
        39, 143, 1, 249, 55, 15, 37, 31, 178, 195, 189, 169, 179, 147, 170, 253, 196, 234, 191,
        239, 122, 33, 129, 55, 143, 37, 15, 113, 124, 35, 186, 159, 194, 31, 248, 241, 186, 177,
        45, 55, 11, 110, 175, 246, 187, 195, 118, 125, 20, 146, 150, 51, 219, 71, 113, 243, 27,
        248, 60, 180, 44, 196, 122, 98, 189, 183, 126, 47, 114, 86, 209, 153, 227, 32, 255, 224,
        114, 191, 255, 217, 124, 53, 236, 102, 136, 239, 158, 128, 153, 138, 18, 4, 47, 138, 198,
        220, 228, 180, 3, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 49, 10, 128, 64, 12, 4, 147, 156, 66, 8, 98,
        145, 50, 149, 4, 11, 107, 31, 226, 11, 5, 193, 23, 250, 4, 115, 165, 141, 151, 226, 2, 219,
        77, 118, 151, 149, 135, 120, 212, 217, 39, 37, 198, 16, 25, 9, 232, 141, 118, 161, 158, 40,
        171, 185, 44, 8, 255, 167, 41, 202, 14, 221, 36, 227, 70, 92, 170, 31, 180, 184, 111, 201,
        38, 222, 187, 228, 144, 73, 173, 107, 122, 44, 27, 162, 80, 217, 227, 239, 5, 165, 80, 70,
        173, 115, 1, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 144, 161, 10, 2, 65, 16, 134, 119, 102, 203, 50,
        200, 33, 19, 55, 201, 6, 57, 77, 186, 62, 130, 8, 38, 49, 90, 108, 62, 128, 24, 228, 180,
        93, 179, 138, 213, 96, 244, 9, 68, 31, 231, 154, 209, 98, 119, 118, 175, 25, 206, 45, 254,
        48, 225, 135, 127, 134, 111, 126, 186, 130, 209, 156, 249, 150, 1, 86, 140, 6, 44, 26, 252,
        242, 154, 247, 84, 112, 78, 142, 58, 160, 154, 197, 97, 59, 38, 197, 188, 78, 239, 106,
        124, 92, 63, 118, 211, 231, 40, 95, 46, 202, 242, 54, 153, 223, 123, 151, 97, 181, 153,
        173, 6, 146, 212, 182, 206, 53, 202, 34, 41, 167, 250, 66, 66, 231, 128, 90, 208, 150, 187,
        73, 40, 42, 25, 4, 3, 8, 254, 186, 215, 246, 153, 60, 135, 113, 98, 45, 53, 154, 19, 231,
        197, 209, 1, 164, 179, 127, 224, 65, 122, 79, 17, 70, 125, 0, 65, 207, 145, 175, 213, 1, 0,
        0,
    ];
    assert_eq!(bytes, expected_serialization);

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}
