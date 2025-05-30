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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 65, 14, 128, 32, 12, 4, 65, 124, 80, 75, 91, 104, 111, 126, 69, 34, 252, 255, 9, 106, 228, 64, 194, 81, 38, 105, 182, 167, 201, 102, 189, 251, 216, 159, 243, 110, 38, 244, 60, 122, 194, 63, 208, 47, 116, 109, 131, 139, 32, 49, 215, 28, 43, 18, 158, 16, 173, 168, 0, 75, 73, 138, 138, 162, 114, 69, 37, 170, 202, 154, 173, 88, 6, 67, 166, 138, 77, 140, 90, 151, 133, 117, 189, 224, 117, 108, 221, 229, 135, 223, 13, 27, 135, 121, 106, 119, 3, 58, 173, 124, 163, 140, 1, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 81, 203, 10, 128, 48, 12, 179, 243, 193, 192, 155, 95, 178, 253, 129, 63, 227, 193, 139, 7, 17, 191, 223, 137, 45, 4, 201, 188, 216, 64, 73, 27, 182, 146, 108, 210, 60, 136, 165, 68, 251, 78, 217, 102, 132, 105, 179, 114, 250, 135, 44, 126, 187, 18, 250, 13, 239, 70, 80, 252, 8, 214, 195, 131, 160, 126, 115, 235, 104, 54, 18, 127, 142, 251, 243, 64, 50, 6, 146, 119, 44, 101, 103, 215, 237, 92, 246, 131, 125, 59, 222, 168, 205, 53, 125, 34, 186, 57, 185, 0, 144, 108, 110, 185, 127, 2, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 85, 205, 10, 194, 48, 12, 78, 219, 57, 11, 222, 124, 2, 65, 31, 160, 83, 188, 239, 93, 196, 155, 162, 71, 31, 223, 5, 19, 136, 89, 217, 14, 38, 162, 31, 148, 172, 35, 249, 242, 79, 3, 188, 144, 135, 19, 232, 187, 33, 25, 73, 226, 255, 4, 239, 96, 221, 158, 100, 249, 12, 93, 176, 227, 42, 94, 49, 198, 63, 136, 49, 57, 196, 8, 162, 255, 63, 216, 111, 203, 190, 152, 214, 47, 85, 246, 7, 119, 107, 49, 156, 150, 238, 75, 146, 89, 23, 26, 141, 34, 140, 23, 79, 130, 135, 103, 165, 73, 148, 227, 222, 38, 161, 67, 86, 126, 141, 249, 11, 23, 101, 3, 211, 249, 250, 230, 185, 47, 204, 159, 92, 248, 75, 199, 252, 141, 83, 159, 90, 85, 47, 153, 139, 244, 139, 115, 195, 3, 120, 186, 93, 239, 151, 243, 35, 85, 76, 181, 57, 98, 171, 238, 187, 74, 201, 230, 56, 24, 242, 157, 153, 210, 15, 21, 253, 57, 155, 111, 141, 138, 211, 74, 28, 215, 48, 6, 251, 122, 2, 22, 208, 240, 227, 188, 7, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 144, 75, 10, 0, 32, 8, 68, 253, 117, 31, 187, 65, 247, 63, 85, 65, 10, 82, 203, 116, 209, 128, 60, 221, 12, 227, 32, 108, 181, 53, 108, 187, 147, 140, 24, 118, 231, 169, 97, 212, 55, 245, 106, 95, 76, 246, 229, 60, 47, 45, 238, 86, 127, 235, 86, 146, 127, 231, 144, 147, 194, 29, 179, 11, 220, 154, 50, 208, 200, 5, 36, 3, 0, 0]");

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

    insta::assert_compact_debug_snapshot!(bytes, @"[31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 146, 81, 10, 195, 48, 8, 134, 77, 164, 247, 209, 152, 52, 230, 109, 87, 89, 88, 122, 255, 35, 172, 99, 41, 11, 89, 161, 15, 77, 31, 250, 193, 143, 34, 34, 250, 35, 194, 23, 172, 250, 48, 173, 50, 171, 44, 252, 48, 85, 176, 213, 143, 154, 16, 58, 182, 198, 71, 141, 116, 14, 182, 205, 44, 161, 217, 251, 18, 93, 97, 225, 39, 185, 148, 53, 144, 15, 121, 86, 86, 14, 26, 94, 78, 69, 138, 122, 141, 41, 167, 72, 137, 189, 20, 94, 66, 146, 165, 14, 195, 113, 123, 17, 52, 38, 180, 185, 129, 127, 176, 51, 240, 42, 175, 96, 160, 87, 118, 220, 94, 110, 170, 183, 218, 230, 238, 221, 39, 234, 191, 172, 207, 177, 171, 153, 155, 153, 106, 96, 236, 3, 30, 249, 181, 199, 27, 99, 149, 130, 253, 11, 4, 0, 0]");

    let program_de = Program::deserialize_program(&bytes).unwrap();
    assert_eq!(program_de, program);
}
