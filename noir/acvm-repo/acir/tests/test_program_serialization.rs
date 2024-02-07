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

    let bytes = Circuit::serialize_circuit(&circuit);

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 208, 49, 14, 192, 32, 8, 5, 80, 212, 30, 8, 4, 20,
        182, 94, 165, 166, 122, 255, 35, 52, 77, 28, 76, 58, 214, 191, 124, 166, 23, 242, 15, 0, 8,
        240, 77, 154, 125, 206, 198, 127, 161, 176, 209, 138, 139, 197, 88, 68, 122, 205, 157, 152,
        46, 204, 222, 76, 81, 180, 21, 35, 35, 53, 189, 179, 49, 119, 19, 171, 222, 188, 162, 147,
        112, 167, 161, 206, 99, 98, 105, 223, 95, 248, 26, 113, 90, 97, 185, 97, 217, 56, 173, 35,
        63, 243, 81, 87, 163, 125, 1, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 138, 91, 10, 0, 32, 16, 2, 109, 171, 175, 46, 221,
        209, 247, 229, 130, 130, 140, 200, 92, 0, 11, 157, 228, 35, 127, 212, 200, 29, 61, 116, 76,
        220, 217, 250, 171, 91, 113, 160, 66, 104, 242, 97, 0, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 74, 135, 9, 0, 48, 8, 75, 171, 224, 255, 15, 139,
        27, 196, 64, 200, 100, 0, 15, 133, 80, 57, 89, 219, 127, 39, 173, 126, 235, 236, 247, 151,
        48, 224, 71, 90, 33, 97, 0, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 210, 7, 74, 3, 1, 20, 69, 209, 177, 247, 222, 123,
        239, 189, 119, 141, 93, 99, 220, 133, 251, 95, 130, 152, 103, 78, 32, 3, 195, 33, 4, 66,
        248, 239, 254, 20, 69, 209, 84, 212, 158, 216, 206, 223, 234, 219, 204, 146, 239, 91, 170,
        111, 103, 245, 109, 101, 27, 219, 217, 193, 250, 219, 197, 110, 246, 176, 151, 125, 236,
        231, 0, 7, 57, 196, 97, 142, 112, 148, 99, 28, 231, 4, 39, 57, 197, 105, 206, 112, 150,
        115, 156, 231, 2, 23, 185, 196, 101, 174, 112, 149, 107, 92, 231, 6, 55, 185, 197, 109,
        238, 112, 151, 123, 220, 231, 1, 15, 121, 196, 99, 158, 240, 148, 103, 60, 231, 5, 47, 121,
        197, 107, 222, 240, 150, 119, 188, 231, 3, 75, 124, 228, 83, 195, 142, 121, 158, 125, 126,
        225, 43, 223, 248, 206, 15, 126, 178, 204, 47, 86, 248, 237, 119, 43, 76, 127, 105, 47,
        189, 165, 181, 116, 150, 198, 234, 125, 117, 249, 47, 233, 41, 45, 165, 163, 52, 148, 126,
        210, 78, 186, 73, 51, 233, 37, 173, 164, 147, 52, 146, 62, 210, 70, 186, 72, 19, 233, 33,
        45, 164, 131, 52, 144, 253, 23, 139, 218, 238, 217, 60, 123, 103, 235, 236, 156, 141, 179,
        239, 166, 93, 183, 237, 185, 107, 199, 125, 251, 29, 218, 237, 216, 94, 167, 118, 58, 183,
        207, 165, 93, 174, 237, 113, 107, 135, 123, 247, 47, 185, 251, 147, 59, 191, 184, 239, 155,
        187, 126, 184, 103, 217, 29, 235, 55, 171, 223, 173, 104, 184, 231, 255, 243, 7, 236, 52,
        239, 128, 225, 3, 0, 0,
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
                destination_value_types: vec![HeapValueType::Simple],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(0))],
                input_value_types: vec![HeapValueType::Simple],
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 65, 10, 192, 32, 12, 4, 215, 148, 150, 246,
        212, 175, 216, 31, 244, 51, 61, 244, 226, 65, 196, 247, 171, 24, 33, 136, 122, 209, 129,
        144, 176, 132, 101, 247, 4, 160, 144, 217, 196, 45, 41, 218, 203, 91, 207, 241, 168, 117,
        94, 90, 230, 37, 238, 144, 216, 27, 249, 11, 87, 156, 131, 239, 223, 248, 207, 186, 81,
        235, 150, 67, 173, 221, 189, 95, 18, 34, 97, 64, 0, 116, 135, 40, 214, 136, 1, 0, 0,
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
                input_value_types: vec![
                    HeapValueType::Array { size: 3, value_types: vec![HeapValueType::Simple] },
                    HeapValueType::Simple,
                ],
                destinations: vec![
                    ValueOrArray::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
                    ValueOrArray::MemoryAddress(MemoryAddress::from(35)),
                    ValueOrArray::MemoryAddress(MemoryAddress::from(36)),
                ],
                destination_value_types: vec![
                    HeapValueType::Array { size: 3, value_types: vec![HeapValueType::Simple] },
                    HeapValueType::Simple,
                    HeapValueType::Simple,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 84, 73, 14, 131, 48, 12, 28, 147, 166, 165, 167,
        126, 161, 82, 251, 128, 180, 47, 224, 47, 85, 111, 32, 56, 242, 124, 130, 24, 68, 176, 2,
        23, 130, 4, 35, 89, 206, 50, 137, 71, 182, 147, 28, 128, 96, 128, 241, 150, 113, 44, 156,
        135, 24, 121, 5, 189, 219, 134, 143, 164, 187, 203, 237, 165, 49, 59, 129, 70, 179, 131,
        198, 177, 31, 14, 90, 239, 148, 117, 73, 154, 63, 19, 121, 63, 23, 111, 214, 219, 149, 243,
        27, 125, 206, 117, 208, 63, 85, 222, 161, 248, 32, 167, 72, 162, 245, 235, 44, 166, 94, 20,
        21, 251, 30, 196, 253, 213, 85, 83, 254, 91, 163, 168, 90, 234, 43, 24, 191, 213, 190, 172,
        156, 235, 17, 126, 59, 49, 142, 68, 120, 75, 220, 7, 166, 84, 90, 68, 72, 194, 139, 180,
        136, 25, 58, 46, 103, 45, 188, 25, 5, 0, 0,
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
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 145, 187, 17, 0, 32, 8, 67, 195, 111, 31, 220, 192,
        253, 167, 178, 144, 2, 239, 236, 132, 194, 52, 129, 230, 93, 8, 6, 64, 176, 101, 225, 28,
        78, 49, 43, 238, 154, 225, 254, 166, 209, 205, 165, 98, 174, 212, 177, 188, 187, 92, 255,
        173, 92, 173, 190, 93, 82, 80, 78, 123, 14, 127, 60, 97, 1, 210, 144, 46, 242, 19, 3, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}
