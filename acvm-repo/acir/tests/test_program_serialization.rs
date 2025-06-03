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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 143, 49, 14, 64, 64, 16, 69, 255, 44, 7, 81, 210, 17, 71, 16, 137, 74, 148, 26, 157, 3, 136, 78, 185, 71, 16, 23, 112, 10, 225, 56, 219, 41, 53, 122, 194, 110, 76, 162, 217, 125, 201, 100, 102, 146, 159, 153, 255, 9, 47, 254, 93, 132, 63, 158, 238, 1, 172, 32, 114, 208, 10, 166, 141, 219, 178, 87, 201, 28, 173, 85, 190, 72, 89, 55, 97, 186, 23, 195, 214, 141, 153, 58, 167, 67, 251, 176, 188, 251, 32, 204, 19, 54, 155, 29, 248, 114, 113, 46, 164, 199, 252, 137, 12, 1, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 141, 11, 10, 0, 32, 8, 67, 87, 246, 189, 255, 133, 91, 52, 97, 36, 60, 230, 111, 90, 240, 98, 147, 162, 252, 234, 34, 97, 117, 82, 165, 161, 60, 231, 77, 218, 201, 32, 83, 55, 160, 30, 204, 31, 218, 207, 62, 236, 215, 239, 245, 56, 194, 131, 194, 221, 172, 0, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 80, 203, 10, 128, 48, 12, 91, 124, 81, 240, 230, 23, 248, 115, 30, 188, 120, 16, 241, 251, 157, 216, 64, 29, 221, 216, 2, 37, 109, 104, 104, 40, 194, 7, 137, 5, 237, 7, 101, 206, 22, 212, 214, 80, 5, 160, 126, 247, 119, 175, 75, 27, 88, 177, 96, 30, 149, 37, 209, 95, 238, 27, 194, 136, 115, 191, 193, 143, 201, 17, 109, 126, 230, 154, 99, 113, 119, 63, 238, 237, 188, 188, 183, 91, 71, 110, 206, 233, 139, 163, 51, 201, 3, 1, 24, 56, 224, 255, 1, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 84, 209, 14, 194, 48, 8, 132, 178, 205, 38, 190, 249, 5, 77, 244, 207, 140, 111, 26, 125, 244, 243, 29, 17, 150, 14, 171, 150, 110, 187, 164, 161, 109, 238, 14, 58, 200, 16, 222, 136, 227, 66, 217, 119, 18, 131, 68, 190, 39, 152, 67, 185, 9, 170, 128, 88, 207, 109, 206, 17, 96, 251, 28, 4, 254, 28, 12, 114, 230, 113, 124, 47, 207, 187, 93, 245, 107, 205, 121, 255, 121, 54, 250, 113, 13, 114, 222, 73, 140, 37, 81, 128, 207, 193, 153, 85, 35, 113, 111, 77, 76, 226, 4, 117, 245, 70, 227, 235, 212, 79, 143, 250, 198, 87, 223, 133, 117, 38, 213, 83, 155, 30, 85, 223, 181, 233, 105, 200, 116, 211, 101, 182, 87, 95, 238, 139, 54, 248, 124, 191, 61, 174, 151, 39, 21, 164, 86, 206, 56, 154, 243, 201, 240, 176, 194, 67, 145, 255, 135, 126, 241, 177, 192, 255, 167, 89, 171, 149, 141, 35, 215, 31, 10, 151, 234, 245, 2, 105, 20, 97, 90, 156, 5, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 165, 79, 91, 10, 0, 32, 8, 243, 213, 61, 186, 255, 41, 43, 82, 144, 232, 195, 199, 64, 54, 197, 161, 67, 184, 24, 187, 88, 181, 49, 41, 163, 211, 198, 47, 38, 132, 128, 93, 31, 38, 125, 28, 223, 237, 102, 171, 250, 202, 217, 4, 114, 191, 177, 187, 67, 174, 183, 217, 129, 124, 124, 11, 25, 166, 104, 137, 36, 2, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 145, 61, 10, 131, 64, 16, 133, 231, 7, 239, 145, 50, 233, 18, 114, 132, 16, 72, 21, 82, 166, 177, 243, 0, 98, 103, 233, 17, 196, 11, 120, 10, 209, 227, 216, 89, 218, 216, 171, 184, 171, 235, 176, 176, 107, 225, 7, 15, 134, 225, 49, 59, 111, 135, 97, 129, 149, 102, 130, 73, 56, 137, 96, 3, 149, 64, 247, 93, 38, 6, 129, 54, 94, 192, 11, 36, 195, 123, 143, 190, 73, 251, 40, 111, 245, 239, 93, 101, 217, 63, 188, 62, 187, 79, 218, 196, 249, 171, 29, 138, 94, 61, 230, 57, 119, 93, 84, 214, 104, 241, 49, 236, 3, 106, 142, 102, 49, 189, 174, 44, 228, 63, 151, 2, 229, 39, 99, 47, 235, 17, 228, 149, 100, 205, 162, 103, 251, 140, 51, 67, 35, 28, 59, 160, 43, 143, 141, 17, 210, 152, 112, 189, 235, 2, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}
