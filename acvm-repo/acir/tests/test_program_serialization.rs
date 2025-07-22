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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 205, 161, 13, 128, 48, 20, 69, 209, 247, 126, 25, 164, 18, 28, 132, 17, 8, 9, 138, 32, 49, 56, 6, 32, 56, 100, 71, 32, 44, 192, 20, 4, 198, 169, 67, 214, 212, 119, 131, 182, 87, 223, 228, 16, 5, 1, 165, 17, 143, 76, 30, 162, 235, 117, 220, 109, 115, 87, 239, 212, 63, 198, 204, 75, 217, 254, 195, 241, 109, 103, 103, 253, 229, 144, 161, 64, 40, 160, 2, 2, 113, 34, 137, 184, 150, 0, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}

#[test]
fn multi_scalar_mul_circuit() {
    let multi_scalar_mul: Opcode<FieldElement> =
        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::MultiScalarMul {
            points: vec![
                FunctionInput::Witness(Witness(1)),
                FunctionInput::Witness(Witness(2)),
                FunctionInput::Witness(Witness(3)),
            ],
            scalars: vec![FunctionInput::Witness(Witness(4)), FunctionInput::Witness(Witness(5))],
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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 1, 34, 0, 221, 255, 1, 0, 9, 1, 1, 8, 3, 1, 1, 1, 2, 1, 3, 2, 1, 4, 1, 5, 6, 7, 8, 5, 1, 2, 3, 4, 5, 0, 3, 6, 7, 8, 0, 0, 88, 58, 161, 140, 34, 0, 0, 0]");

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
        name: "invert_call".into(),
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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 133, 142, 177, 10, 128, 64, 12, 67, 147, 170, 71, 193, 205, 47, 240, 207, 68, 196, 65, 56, 28, 68, 252, 126, 115, 122, 243, 181, 67, 219, 33, 47, 9, 225, 236, 65, 128, 51, 218, 195, 80, 33, 27, 131, 209, 202, 55, 30, 231, 179, 95, 247, 178, 173, 57, 15, 46, 184, 11, 105, 23, 27, 170, 152, 180, 76, 254, 233, 15, 224, 23, 139, 122, 166, 82, 225, 5, 26, 73, 151, 197, 211, 0, 0, 0]");

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
        name: "complex_call".into(),
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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 141, 146, 193, 14, 194, 48, 12, 67, 237, 186, 235, 34, 141, 19, 95, 48, 9, 254, 11, 161, 137, 91, 17, 28, 249, 124, 82, 216, 121, 73, 78, 86, 228, 186, 207, 85, 9, 99, 69, 161, 192, 21, 199, 195, 208, 129, 68, 72, 137, 28, 153, 16, 69, 14, 159, 208, 19, 247, 137, 97, 19, 36, 162, 234, 212, 48, 195, 92, 22, 141, 67, 167, 237, 245, 124, 247, 199, 231, 182, 221, 123, 95, 44, 19, 35, 115, 222, 248, 178, 134, 97, 97, 42, 114, 29, 64, 241, 43, 25, 106, 2, 175, 129, 16, 234, 50, 239, 213, 68, 95, 2, 23, 224, 42, 254, 53, 252, 155, 253, 4, 203, 190, 201, 98, 38, 154, 79, 231, 193, 250, 5, 168, 83, 113, 51, 206, 2, 0, 0]");

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
    };
    let read = Opcode::MemoryOp {
        block_id: BlockId(0),
        op: MemOp::read_at_mem_index(FieldElement::one().into(), Witness(4)),
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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 206, 177, 1, 0, 32, 12, 195, 48, 39, 129, 63, 248, 255, 74, 184, 128, 118, 168, 103, 13, 22, 59, 193, 50, 6, 14, 255, 212, 33, 42, 73, 42, 209, 154, 97, 102, 102, 85, 130, 200, 121, 14, 46, 69, 85, 191, 45, 45, 1, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 49, 14, 64, 64, 16, 69, 103, 254, 196, 61, 148, 116, 196, 17, 68, 162, 18, 165, 70, 231, 0, 162, 83, 58, 130, 184, 128, 83, 8, 199, 217, 78, 169, 209, 155, 205, 246, 86, 97, 146, 201, 111, 126, 222, 203, 23, 18, 9, 24, 196, 250, 46, 133, 136, 16, 210, 251, 49, 194, 164, 171, 6, 147, 174, 241, 94, 23, 219, 52, 53, 109, 148, 157, 229, 120, 244, 115, 110, 238, 229, 210, 138, 248, 24, 196, 106, 179, 69, 124, 17, 146, 87, 232, 101, 32, 0, 96, 7, 194, 170, 117, 167, 213, 255, 97, 102, 31, 195, 25, 53, 31, 8, 23, 33, 144, 111, 1, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}
