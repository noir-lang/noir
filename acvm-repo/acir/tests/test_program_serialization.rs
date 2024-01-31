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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 187, 13, 192, 32, 12, 68, 249, 100, 32, 27,
        219, 96, 119, 89, 37, 40, 176, 255, 8, 17, 18, 5, 74, 202, 240, 154, 235, 158, 238, 238,
        112, 206, 121, 247, 37, 206, 60, 103, 194, 63, 208, 111, 116, 133, 197, 69, 144, 153, 91,
        73, 13, 9, 47, 72, 86, 85, 128, 165, 102, 69, 69, 81, 185, 147, 18, 53, 101, 45, 86, 173,
        128, 33, 83, 195, 46, 70, 125, 202, 226, 190, 94, 16, 166, 103, 108, 13, 203, 151, 254,
        245, 233, 224, 1, 1, 52, 166, 127, 120, 1, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 138, 91, 10, 0, 48, 12, 194, 178, 215, 215, 46, 189,
        163, 175, 165, 10, 21, 36, 10, 57, 192, 160, 146, 188, 226, 139, 78, 113, 69, 183, 190, 61,
        111, 218, 182, 231, 124, 122, 8, 177, 65, 92, 0, 0, 0,
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
        138, 10, 34, 65, 84, 198, 15, 28, 82, 145, 178, 182, 86, 191, 238, 183, 24, 131, 205, 79,
        203, 0, 166, 242, 158, 93, 92, 0, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 210, 87, 78, 2, 1, 20, 134, 209, 177, 247, 222, 123,
        71, 68, 68, 68, 68, 68, 68, 68, 68, 68, 221, 133, 251, 95, 130, 145, 27, 206, 36, 78, 50,
        57, 16, 94, 200, 253, 191, 159, 36, 73, 134, 146, 193, 19, 142, 243, 183, 255, 14, 179,
        233, 247, 145, 254, 59, 217, 127, 71, 57, 198, 113, 78, 48, 125, 167, 56, 205, 25, 206,
        114, 142, 243, 92, 224, 34, 151, 184, 204, 21, 174, 114, 141, 235, 220, 224, 38, 183, 184,
        205, 29, 238, 114, 143, 251, 60, 224, 33, 143, 120, 204, 19, 158, 242, 140, 25, 158, 51,
        203, 11, 230, 120, 201, 60, 175, 88, 224, 53, 139, 188, 97, 137, 183, 44, 243, 142, 21,
        222, 179, 202, 7, 214, 248, 200, 58, 159, 216, 224, 51, 155, 124, 97, 235, 223, 142, 241,
        188, 250, 222, 230, 27, 59, 124, 103, 151, 31, 236, 241, 147, 95, 252, 246, 57, 158, 104,
        47, 186, 139, 214, 162, 179, 104, 44, 250, 74, 219, 154, 242, 63, 162, 165, 232, 40, 26,
        138, 126, 162, 157, 232, 38, 154, 137, 94, 162, 149, 232, 36, 26, 137, 62, 162, 141, 232,
        34, 154, 136, 30, 162, 133, 232, 32, 26, 136, 253, 99, 251, 195, 100, 176, 121, 236, 29,
        91, 159, 218, 56, 99, 219, 172, 77, 115, 182, 204, 219, 176, 96, 187, 162, 205, 74, 182,
        42, 219, 168, 98, 155, 170, 77, 106, 182, 168, 219, 160, 225, 246, 77, 55, 111, 185, 113,
        219, 109, 59, 110, 218, 117, 203, 158, 27, 166, 55, 75, 239, 150, 184, 101, 250, 252, 1,
        55, 204, 92, 74, 220, 3, 0, 0,
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
        234, 175, 216, 63, 232, 207, 116, 232, 210, 161, 136, 223, 175, 98, 132, 27, 212, 69, 31,
        132, 28, 23, 8, 119, 59, 0, 131, 204, 66, 154, 41, 222, 173, 219, 142, 113, 153, 121, 191,
        44, 231, 21, 237, 144, 88, 43, 249, 11, 71, 156, 77, 245, 251, 249, 231, 119, 189, 214,
        204, 89, 187, 11, 25, 130, 54, 1, 36, 1, 124, 242, 107, 1, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 83, 65, 10, 132, 48, 12, 76, 218, 237, 174, 123,
        242, 11, 130, 62, 160, 250, 2, 255, 34, 222, 20, 61, 250, 124, 11, 78, 49, 4, 193, 131, 21,
        52, 16, 210, 132, 105, 50, 77, 210, 140, 136, 152, 54, 177, 65, 13, 206, 12, 95, 74, 196,
        181, 176, 254, 154, 212, 156, 46, 151, 191, 139, 163, 121, 1, 71, 123, 3, 199, 184, 15, 15,
        157, 119, 202, 185, 36, 237, 159, 61, 248, 63, 159, 160, 46, 232, 23, 254, 15, 54, 67, 156,
        96, 11, 213, 119, 82, 248, 116, 179, 104, 188, 163, 125, 15, 89, 213, 253, 139, 154, 221,
        52, 206, 67, 191, 88, 5, 213, 52, 75, 113, 174, 96, 205, 201, 157, 24, 207, 197, 211, 157,
        6, 50, 18, 233, 158, 72, 89, 1, 53, 215, 75, 175, 196, 4, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 146, 49, 14, 0, 32, 8, 3, 139, 192, 127, 240, 7,
        254, 255, 85, 198, 136, 9, 131, 155, 48, 216, 165, 76, 77, 57, 80, 0, 140, 45, 117, 111,
        238, 228, 179, 224, 174, 225, 110, 111, 234, 213, 185, 148, 156, 203, 121, 89, 86, 13, 215,
        126, 131, 43, 153, 187, 115, 40, 185, 62, 153, 3, 136, 83, 60, 30, 96, 2, 12, 235, 225,
        124, 14, 3, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}
