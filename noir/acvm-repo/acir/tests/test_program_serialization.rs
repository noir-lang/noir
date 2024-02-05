//! This integration test defines a set of circuits which are used in order to test the acvm_js package.
//!
//! The acvm_js test suite contains serialized [circuits][`Circuit`] which must be kept in sync with the format
//! outputted from the [ACIR crate][acir].
//! Breaking changes to the serialization format then require refreshing acvm_js's test suite.
//! This file contains Rust definitions of these circuits and outputs the updated serialized format.
//!
//! These tests also check this circuit serialization against an expected value, erroring if the serialization changes.
//! Generally in this situation we just need to refresh the `expected_serialization` variables to match the
//! actual output, **HOWEVER** note that this results in a breaking change to the ACIR format.

use std::collections::BTreeSet;

use acir::{
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, BlockId, FunctionInput, MemOp},
        Circuit, Opcode, PublicInputs,
    },
    native_types::{Expression, Witness},
};
use acir_field::FieldElement;
use brillig::{HeapArray, MemoryAddress, ValueOrArray};

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

    let bytes = Circuit::serialize_circuit(&circuit);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 59, 18, 128, 32, 12, 68, 249, 120, 160, 132,
        36, 144, 116, 94, 69, 70, 184, 255, 17, 28, 29, 10, 70, 75, 121, 205, 118, 111, 118, 119,
        115, 206, 121, 247, 37, 142, 220, 71, 194, 63, 208, 47, 116, 133, 201, 69, 144, 153, 91,
        73, 13, 9, 15, 72, 86, 85, 128, 165, 102, 69, 69, 81, 57, 147, 18, 53, 101, 45, 86, 173,
        128, 33, 83, 195, 46, 70, 125, 200, 226, 186, 94, 16, 134, 231, 222, 26, 166, 47, 253, 235,
        211, 135, 11, 47, 121, 122, 165, 121, 1, 0, 0,
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

    let bytes = Circuit::serialize_circuit(&circuit);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 138, 91, 10, 0, 32, 12, 195, 226, 235, 203, 75, 123,
        116, 39, 182, 99, 133, 146, 22, 178, 128, 198, 207, 227, 22, 79, 180, 139, 35, 58, 245,
        237, 121, 83, 182, 189, 204, 5, 167, 198, 147, 98, 93, 0, 0, 0,
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

    let bytes = Circuit::serialize_circuit(&circuit);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 138, 9, 10, 0, 64, 8, 2, 103, 15, 232, 255, 31, 142,
        138, 34, 34, 65, 84, 198, 15, 28, 82, 145, 178, 182, 86, 191, 238, 183, 24, 131, 205, 79,
        203, 0, 162, 119, 234, 237, 93, 0, 0, 0,
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

    let bytes = Circuit::serialize_circuit(&circuit);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 210, 233, 50, 66, 1, 24, 199, 225, 99, 223, 247,
        125, 223, 146, 36, 73, 146, 36, 73, 146, 132, 187, 112, 255, 151, 96, 244, 78, 79, 198,
        153, 57, 243, 212, 244, 165, 121, 255, 191, 239, 36, 73, 134, 146, 254, 19, 142, 243, 167,
        247, 14, 179, 225, 247, 145, 222, 59, 217, 123, 71, 57, 198, 113, 78, 112, 240, 78, 113,
        154, 51, 156, 229, 28, 231, 185, 192, 69, 46, 113, 153, 43, 92, 229, 26, 215, 185, 193, 77,
        110, 113, 155, 59, 220, 229, 30, 247, 121, 192, 67, 30, 241, 152, 39, 76, 241, 148, 105,
        158, 49, 195, 115, 102, 121, 193, 28, 47, 153, 231, 21, 11, 188, 102, 145, 55, 44, 241,
        150, 101, 222, 177, 194, 123, 86, 249, 192, 26, 31, 89, 231, 19, 27, 124, 102, 243, 223,
        142, 241, 188, 248, 222, 226, 43, 219, 124, 99, 135, 239, 236, 242, 131, 159, 252, 242, 57,
        158, 104, 47, 186, 139, 214, 162, 179, 104, 44, 250, 26, 180, 53, 229, 127, 68, 75, 209,
        81, 52, 20, 253, 68, 59, 209, 77, 52, 19, 189, 68, 43, 209, 73, 52, 18, 125, 68, 27, 209,
        69, 52, 17, 61, 68, 11, 209, 65, 52, 16, 251, 199, 246, 135, 73, 127, 243, 216, 59, 182,
        78, 217, 56, 109, 219, 140, 77, 179, 182, 204, 217, 48, 111, 187, 130, 205, 138, 182, 42,
        217, 168, 108, 155, 138, 77, 170, 182, 168, 217, 160, 238, 246, 13, 55, 111, 186, 113, 203,
        109, 219, 110, 218, 113, 203, 174, 27, 14, 110, 54, 184, 91, 226, 150, 127, 207, 47, 78,
        22, 245, 106, 221, 3, 0, 0,
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
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(0))],
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

    let bytes = Circuit::serialize_circuit(&circuit);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 177, 10, 192, 32, 12, 68, 207, 148, 150, 118,
        234, 175, 216, 63, 232, 207, 116, 232, 226, 32, 226, 247, 171, 24, 225, 6, 113, 209, 7, 33,
        199, 5, 194, 221, 9, 192, 160, 178, 145, 102, 154, 247, 234, 182, 115, 60, 102, 221, 47,
        203, 121, 69, 59, 20, 246, 78, 254, 198, 149, 231, 80, 253, 187, 248, 249, 48, 106, 205,
        220, 189, 187, 144, 33, 24, 144, 0, 93, 119, 243, 238, 108, 1, 0, 0,
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
                value: brillig::Value::from(32_usize),
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
                destinations: vec![
                    ValueOrArray::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
                    ValueOrArray::MemoryAddress(MemoryAddress::from(35)),
                    ValueOrArray::MemoryAddress(MemoryAddress::from(36)),
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

    let bytes = Circuit::serialize_circuit(&circuit);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 83, 81, 10, 131, 48, 12, 125, 105, 215, 205, 125,
        237, 10, 131, 237, 0, 221, 78, 224, 93, 196, 63, 69, 63, 61, 190, 5, 95, 177, 6, 193, 15,
        43, 104, 32, 164, 9, 175, 201, 107, 146, 22, 0, 4, 147, 216, 160, 134, 103, 161, 159, 74,
        196, 149, 180, 126, 159, 252, 36, 95, 46, 127, 20, 71, 115, 1, 142, 246, 0, 142, 113, 31,
        78, 58, 239, 156, 115, 201, 218, 63, 187, 242, 127, 110, 65, 93, 208, 59, 253, 7, 109, 193,
        56, 104, 223, 170, 239, 80, 120, 16, 83, 102, 225, 250, 247, 14, 243, 46, 138, 170, 253,
        76, 234, 86, 93, 219, 55, 245, 96, 21, 84, 83, 253, 36, 231, 47, 173, 217, 184, 19, 227,
        47, 204, 207, 119, 26, 40, 76, 164, 251, 178, 144, 17, 127, 189, 34, 151, 201, 4, 0, 0,
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
    let bytes = Circuit::serialize_circuit(&circuit);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 146, 193, 13, 0, 32, 8, 3, 171, 192, 62, 184, 129,
        251, 79, 101, 140, 152, 96, 226, 79, 120, 216, 79, 121, 53, 229, 64, 0, 16, 150, 196, 188,
        154, 23, 155, 25, 119, 117, 115, 125, 83, 203, 206, 45, 193, 185, 20, 151, 165, 217, 112,
        245, 55, 184, 28, 185, 59, 185, 146, 243, 147, 201, 129, 216, 197, 143, 3, 12, 77, 66, 200,
        219, 15, 3, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}
