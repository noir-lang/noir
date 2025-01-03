use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use acir::brillig::{BitSize, HeapVector, IntegerBitSize};
use acir::{
    acir_field::GenericFieldElement,
    brillig::{BinaryFieldOp, MemoryAddress, Opcode as BrilligOpcode, ValueOrArray},
    circuit::{
        brillig::{BrilligBytecode, BrilligFunctionId, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, BlockId, BlockType, FunctionInput, MemOp},
        Opcode, OpcodeLocation,
    },
    native_types::{Expression, Witness, WitnessMap},
    AcirField, FieldElement,
};

use acvm::pwg::{ACVMStatus, ErrorLocation, ForeignCallWaitInfo, OpcodeResolutionError, ACVM};
use acvm_blackbox_solver::StubbedBlackBoxSolver;
use bn254_blackbox_solver::{field_from_hex, Bn254BlackBoxSolver, POSEIDON2_CONFIG};
use brillig_vm::brillig::HeapValueType;

use num_bigint::BigUint;
use proptest::arbitrary::any;
use proptest::prelude::*;
use proptest::result::maybe_ok;
use proptest::sample::select;

#[test]
fn bls12_381_circuit() {
    type Bls12FieldElement = GenericFieldElement<ark_bls12_381::Fr>;

    let addition = Opcode::AssertZero(Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![
            (Bls12FieldElement::one(), Witness(1)),
            (Bls12FieldElement::one(), Witness(2)),
            (-Bls12FieldElement::one(), Witness(3)),
        ],
        q_c: Bls12FieldElement::zero(),
    });
    let opcodes = [addition];

    let witness_assignments = BTreeMap::from([
        (Witness(1), Bls12FieldElement::from(2u128)),
        (Witness(2), Bls12FieldElement::from(3u128)),
    ])
    .into();

    let mut acvm = ACVM::new(&StubbedBlackBoxSolver, &opcodes, witness_assignments, &[], &[]);
    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    let witness_stack = acvm.finalize();

    assert_eq!(witness_stack.get(&Witness(3)).unwrap(), &Bls12FieldElement::from(5u128));
}

#[test]
fn inversion_brillig_oracle_equivalence() {
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     let z = x + y;
    //     assert( 1/z == Oracle("inverse", x + y) );
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_z = Witness(4);
    let w_z_inverse = Witness(5);
    let w_x_plus_y = Witness(6);
    let w_equal_res = Witness(7);

    let opcodes = vec![
        Opcode::BrilligCall {
            id: BrilligFunctionId(0),
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                    q_c: fe_0,
                }),
                BrilligInputs::Single(Expression::default()), // Input Register 1
            ],
            // This tells the BrilligSolver which witnesses its output values correspond to
            outputs: vec![
                BrilligOutputs::Simple(w_x_plus_y), // Output Register 0 - from input
                BrilligOutputs::Simple(w_oracle),   // Output Register 1
                BrilligOutputs::Simple(w_equal_res), // Output Register 2
            ],
            predicate: None,
        },
        Opcode::AssertZero(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
            q_c: fe_0,
        }),
        // Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
        Opcode::AssertZero(Expression {
            mul_terms: vec![(fe_1, w_z, w_z_inverse)],
            linear_combinations: vec![],
            q_c: -fe_1,
        }),
        Opcode::AssertZero(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
            q_c: fe_0,
        }),
    ];

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: MemoryAddress::direct(0),
        rhs: MemoryAddress::direct(1),
        destination: MemoryAddress::direct(2),
    };

    let zero_usize = MemoryAddress::direct(3);
    let two_usize = MemoryAddress::direct(4);
    let three_usize = MemoryAddress::direct(5);

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            BrilligOpcode::Const {
                destination: zero_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            BrilligOpcode::Const {
                destination: two_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(2u64),
            },
            BrilligOpcode::Const {
                destination: three_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(3u64),
            },
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: two_usize,
                offset_address: zero_usize,
            },
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(1))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(0))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::Stop {
                return_data: HeapVector { pointer: zero_usize, size: three_usize },
            },
        ],
    };

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
    ])
    .into();
    let unconstrained_functions = vec![brillig_bytecode];
    let mut acvm = ACVM::new(
        &StubbedBlackBoxSolver,
        &opcodes,
        witness_assignments,
        &unconstrained_functions,
        &[],
    );
    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve();

    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 0, "brillig should have been removed");

    let foreign_call_wait_info: &ForeignCallWaitInfo<FieldElement> =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    // As caller of VM, need to resolve foreign calls
    let foreign_call_result = foreign_call_wait_info.inputs[0].unwrap_field().inverse();
    // Alter Brillig oracle opcode with foreign call resolution
    acvm.resolve_pending_foreign_call(foreign_call_result.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    acvm.finalize();
}

#[test]
fn double_inversion_brillig_oracle() {
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     let z = x + y;
    //     let ij = i + j;
    //     assert( 1/z == Oracle("inverse", x + y) );
    //     assert( 1/ij == Oracle("inverse", i + j) );
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_z = Witness(4);
    let w_z_inverse = Witness(5);
    let w_x_plus_y = Witness(6);
    let w_equal_res = Witness(7);
    let w_i = Witness(8);
    let w_j = Witness(9);
    let w_ij_oracle = Witness(10);
    let w_i_plus_j = Witness(11);

    let opcodes = vec![
        Opcode::BrilligCall {
            id: BrilligFunctionId(0),
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                    q_c: fe_0,
                }),
                BrilligInputs::Single(Expression::default()), // Input Register 1
                BrilligInputs::Single(Expression {
                    // Input Register 2
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_i), (fe_1, w_j)],
                    q_c: fe_0,
                }),
            ],
            outputs: vec![
                BrilligOutputs::Simple(w_x_plus_y), // Output Register 0 - from input
                BrilligOutputs::Simple(w_oracle),   // Output Register 1
                BrilligOutputs::Simple(w_i_plus_j), // Output Register 2 - from input
                BrilligOutputs::Simple(w_ij_oracle), // Output Register 3
                BrilligOutputs::Simple(w_equal_res), // Output Register 4
            ],
            predicate: None,
        },
        Opcode::AssertZero(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
            q_c: fe_0,
        }),
        // Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
        Opcode::AssertZero(Expression {
            mul_terms: vec![(fe_1, w_z, w_z_inverse)],
            linear_combinations: vec![],
            q_c: -fe_1,
        }),
        Opcode::AssertZero(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
            q_c: fe_0,
        }),
    ];

    let zero_usize = MemoryAddress::direct(5);
    let three_usize = MemoryAddress::direct(6);
    let five_usize = MemoryAddress::direct(7);

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: MemoryAddress::direct(0),
        rhs: MemoryAddress::direct(1),
        destination: MemoryAddress::direct(4),
    };

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            BrilligOpcode::Const {
                destination: zero_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            BrilligOpcode::Const {
                destination: three_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(3u64),
            },
            BrilligOpcode::Const {
                destination: five_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(5u64),
            },
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: three_usize,
                offset_address: zero_usize,
            },
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(1))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(0))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(3))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(2))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::Stop {
                return_data: HeapVector { pointer: zero_usize, size: five_usize },
            },
        ],
    };

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
        (Witness(8), FieldElement::from(5u128)),
        (Witness(9), FieldElement::from(10u128)),
    ])
    .into();
    let unconstrained_functions = vec![brillig_bytecode];
    let mut acvm = ACVM::new(
        &StubbedBlackBoxSolver,
        &opcodes,
        witness_assignments,
        &unconstrained_functions,
        &[],
    );

    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve();
    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 0, "should stall on brillig");

    let foreign_call_wait_info: &ForeignCallWaitInfo<FieldElement> =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    let x_plus_y_inverse = foreign_call_wait_info.inputs[0].unwrap_field().inverse();

    // Resolve Brillig foreign call
    acvm.resolve_pending_foreign_call(x_plus_y_inverse.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 0, "should stall on brillig");

    let foreign_call_wait_info =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    let i_plus_j_inverse = foreign_call_wait_info.inputs[0].unwrap_field().inverse();
    assert_ne!(x_plus_y_inverse, i_plus_j_inverse);

    // Alter Brillig oracle opcode
    acvm.resolve_pending_foreign_call(i_plus_j_inverse.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    acvm.finalize();
}

#[test]
fn oracle_dependent_execution() {
    // This test ensures that we properly track the list of opcodes which still need to be resolved
    // across any brillig foreign calls we may have to perform.
    //
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     assert(x == y);
    //     let x_inv = Oracle("inverse", x);
    //     let y_inv = Oracle("inverse", y);
    //
    //     assert(x_inv == y_inv);
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_x_inv = Witness(3);
    let w_y_inv = Witness(4);

    let zero_usize = MemoryAddress::direct(4);
    let three_usize = MemoryAddress::direct(5);
    let four_usize = MemoryAddress::direct(6);

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            BrilligOpcode::Const {
                destination: zero_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            BrilligOpcode::Const {
                destination: three_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(3u64),
            },
            BrilligOpcode::Const {
                destination: four_usize,
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(4u64),
            },
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: three_usize,
                offset_address: zero_usize,
            }, // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(1))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(0))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(3))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(2))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::Stop {
                return_data: HeapVector { pointer: zero_usize, size: four_usize },
            },
        ],
    };

    // This equality check can be executed immediately before resolving any foreign calls.
    let equality_check = Expression {
        mul_terms: vec![],
        linear_combinations: vec![(-fe_1, w_x), (fe_1, w_y)],
        q_c: fe_0,
    };

    // This equality check relies on the outputs of the Brillig call.
    // It then cannot be solved until the foreign calls are resolved.
    let inverse_equality_check = Expression {
        mul_terms: vec![],
        linear_combinations: vec![(-fe_1, w_x_inv), (fe_1, w_y_inv)],
        q_c: fe_0,
    };

    let opcodes = vec![
        Opcode::AssertZero(equality_check),
        Opcode::BrilligCall {
            id: BrilligFunctionId(0),
            inputs: vec![
                BrilligInputs::Single(w_x.into()),            // Input Register 0
                BrilligInputs::Single(Expression::default()), // Input Register 1
                BrilligInputs::Single(w_y.into()),            // Input Register 2,
            ],
            outputs: vec![
                BrilligOutputs::Simple(w_x),     // Output Register 0 - from input
                BrilligOutputs::Simple(w_y_inv), // Output Register 1
                BrilligOutputs::Simple(w_y),     // Output Register 2 - from input
                BrilligOutputs::Simple(w_y_inv), // Output Register 3
            ],
            predicate: None,
        },
        Opcode::AssertZero(inverse_equality_check),
    ];

    let witness_assignments =
        BTreeMap::from([(w_x, FieldElement::from(2u128)), (w_y, FieldElement::from(2u128))]).into();
    let unconstrained_functions = vec![brillig_bytecode];
    let mut acvm = ACVM::new(
        &StubbedBlackBoxSolver,
        &opcodes,
        witness_assignments,
        &unconstrained_functions,
        &[],
    );

    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve();
    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 1, "should stall on brillig");

    let foreign_call_wait_info: &ForeignCallWaitInfo<FieldElement> =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    // Resolve Brillig foreign call
    let x_inverse = foreign_call_wait_info.inputs[0].unwrap_field().inverse();
    acvm.resolve_pending_foreign_call(x_inverse.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 1, "should stall on brillig");

    let foreign_call_wait_info: &ForeignCallWaitInfo<FieldElement> =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    // Resolve Brillig foreign call
    let y_inverse = foreign_call_wait_info.inputs[0].unwrap_field().inverse();
    acvm.resolve_pending_foreign_call(y_inverse.into());

    // We've resolved all the brillig foreign calls so we should be able to complete execution now.

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    acvm.finalize();
}

#[test]
fn brillig_oracle_predicate() {
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_x_plus_y = Witness(4);
    let w_equal_res = Witness(5);
    let w_lt_res = Witness(6);

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: MemoryAddress::direct(0),
        rhs: MemoryAddress::direct(1),
        destination: MemoryAddress::direct(2),
    };

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(2u64),
            },
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress::direct(0),
                size_address: MemoryAddress::direct(0),
                offset_address: MemoryAddress::direct(1),
            },
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(1))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::direct(0))],
                input_value_types: vec![HeapValueType::field()],
            },
        ],
    };

    let opcodes = vec![Opcode::BrilligCall {
        id: BrilligFunctionId(0),
        inputs: vec![
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                q_c: fe_0,
            }),
            BrilligInputs::Single(Expression::default()),
        ],
        outputs: vec![
            BrilligOutputs::Simple(w_x_plus_y),
            BrilligOutputs::Simple(w_oracle),
            BrilligOutputs::Simple(w_equal_res),
            BrilligOutputs::Simple(w_lt_res),
        ],
        predicate: Some(Expression::default()),
    }];

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
    ])
    .into();
    let unconstrained_functions = vec![brillig_bytecode];
    let mut acvm = ACVM::new(
        &StubbedBlackBoxSolver,
        &opcodes,
        witness_assignments,
        &unconstrained_functions,
        &[],
    );
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    acvm.finalize();
}

#[test]
fn unsatisfied_opcode_resolved() {
    let a = Witness(0);
    let b = Witness(1);
    let c = Witness(2);
    let d = Witness(3);

    // a = b + c + d;
    let opcode_a = Expression {
        mul_terms: vec![],
        linear_combinations: vec![
            (FieldElement::one(), a),
            (-FieldElement::one(), b),
            (-FieldElement::one(), c),
            (-FieldElement::one(), d),
        ],
        q_c: FieldElement::zero(),
    };

    let mut values = WitnessMap::new();
    values.insert(a, FieldElement::from(4_i128));
    values.insert(b, FieldElement::from(2_i128));
    values.insert(c, FieldElement::from(1_i128));
    values.insert(d, FieldElement::from(2_i128));

    let opcodes = vec![Opcode::AssertZero(opcode_a)];
    let unconstrained_functions = vec![];
    let mut acvm =
        ACVM::new(&StubbedBlackBoxSolver, &opcodes, values, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(
        solver_status,
        ACVMStatus::Failure(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Resolved(OpcodeLocation::Acir(0)),
            payload: None
        }),
        "The first opcode is not satisfiable, expected an error indicating this"
    );
}

#[test]
fn unsatisfied_opcode_resolved_brillig() {
    let a = Witness(0);
    let b = Witness(1);
    let c = Witness(2);
    let d = Witness(3);

    let fe_1 = FieldElement::one();
    let fe_0 = FieldElement::zero();
    let w_x = Witness(4);
    let w_y = Witness(5);
    let w_result = Witness(6);

    let calldata_copy_opcode = BrilligOpcode::CalldataCopy {
        destination_address: MemoryAddress::direct(0),
        size_address: MemoryAddress::direct(0),
        offset_address: MemoryAddress::direct(1),
    };

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: MemoryAddress::direct(0),
        rhs: MemoryAddress::direct(1),
        destination: MemoryAddress::direct(2),
    };
    // Jump pass the trap if the values are equal, else
    // jump to the trap
    let location_of_stop = 7;

    let jmp_if_opcode =
        BrilligOpcode::JumpIf { condition: MemoryAddress::direct(2), location: location_of_stop };

    let trap_opcode = BrilligOpcode::Trap {
        revert_data: HeapVector {
            pointer: MemoryAddress::direct(0),
            size: MemoryAddress::direct(3),
        },
    };
    let stop_opcode = BrilligOpcode::Stop {
        return_data: HeapVector {
            pointer: MemoryAddress::direct(0),
            size: MemoryAddress::direct(3),
        },
    };

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(2u64),
            },
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(3),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0u64),
            },
            calldata_copy_opcode,
            equal_opcode,
            jmp_if_opcode,
            trap_opcode,
            stop_opcode,
        ],
    };

    let opcode_a = Expression {
        mul_terms: vec![],
        linear_combinations: vec![
            (FieldElement::one(), a),
            (-FieldElement::one(), b),
            (-FieldElement::one(), c),
            (-FieldElement::one(), d),
        ],
        q_c: FieldElement::zero(),
    };

    let mut values = WitnessMap::new();
    values.insert(a, FieldElement::from(4_i128));
    values.insert(b, FieldElement::from(2_i128));
    values.insert(c, FieldElement::from(1_i128));
    values.insert(d, FieldElement::from(2_i128));
    values.insert(w_x, FieldElement::from(0_i128));
    values.insert(w_y, FieldElement::from(1_i128));
    values.insert(w_result, FieldElement::from(0_i128));

    let opcodes = vec![
        Opcode::BrilligCall {
            id: BrilligFunctionId(0),
            inputs: vec![
                BrilligInputs::Single(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_x)],
                    q_c: fe_0,
                }),
                BrilligInputs::Single(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![(fe_1, w_y)],
                    q_c: fe_0,
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(w_result)],
            predicate: Some(Expression::one()),
        },
        Opcode::AssertZero(opcode_a),
    ];
    let unconstrained_functions = vec![brillig_bytecode];
    let mut acvm =
        ACVM::new(&StubbedBlackBoxSolver, &opcodes, values, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(
        solver_status,
        ACVMStatus::Failure(OpcodeResolutionError::BrilligFunctionFailed {
            function_id: BrilligFunctionId(0),
            payload: None,
            call_stack: vec![OpcodeLocation::Brillig { acir_index: 0, brillig_index: 6 }]
        }),
        "The first opcode is not satisfiable, expected an error indicating this"
    );
}

#[test]
fn memory_operations() {
    let initial_witness = WitnessMap::from(BTreeMap::from_iter([
        (Witness(1), FieldElement::from(1u128)),
        (Witness(2), FieldElement::from(2u128)),
        (Witness(3), FieldElement::from(3u128)),
        (Witness(4), FieldElement::from(4u128)),
        (Witness(5), FieldElement::from(5u128)),
        (Witness(6), FieldElement::from(4u128)),
    ]));

    let block_id = BlockId(0);

    let init = Opcode::MemoryInit {
        block_id,
        init: (1..6).map(Witness).collect(),
        block_type: BlockType::Memory,
    };

    let read_op = Opcode::MemoryOp {
        block_id,
        op: MemOp::read_at_mem_index(Witness(6).into(), Witness(7)),
        predicate: None,
    };

    let expression = Opcode::AssertZero(Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![
            (FieldElement::one(), Witness(7)),
            (-FieldElement::one(), Witness(8)),
        ],
        q_c: FieldElement::one(),
    });

    let opcodes = vec![init, read_op, expression];
    let unconstrained_functions = vec![];
    let mut acvm =
        ACVM::new(&StubbedBlackBoxSolver, &opcodes, initial_witness, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();

    assert_eq!(witness_map[&Witness(8)], FieldElement::from(6u128));
}

fn allowed_bigint_moduli() -> Vec<Vec<u8>> {
    let bn254_fq: Vec<u8> = vec![
        0x47, 0xFD, 0x7C, 0xD8, 0x16, 0x8C, 0x20, 0x3C, 0x8d, 0xca, 0x71, 0x68, 0x91, 0x6a, 0x81,
        0x97, 0x5d, 0x58, 0x81, 0x81, 0xb6, 0x45, 0x50, 0xb8, 0x29, 0xa0, 0x31, 0xe1, 0x72, 0x4e,
        0x64, 0x30,
    ];
    let bn254_fr: Vec<u8> = vec![
        1, 0, 0, 240, 147, 245, 225, 67, 145, 112, 185, 121, 72, 232, 51, 40, 93, 88, 129, 129,
        182, 69, 80, 184, 41, 160, 49, 225, 114, 78, 100, 48,
    ];
    let secpk1_fr: Vec<u8> = vec![
        0x41, 0x41, 0x36, 0xD0, 0x8C, 0x5E, 0xD2, 0xBF, 0x3B, 0xA0, 0x48, 0xAF, 0xE6, 0xDC, 0xAE,
        0xBA, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF,
    ];
    let secpk1_fq: Vec<u8> = vec![
        0x2F, 0xFC, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF,
    ];
    let secpr1_fq: Vec<u8> = vec![
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF,
        0xFF, 0xFF,
    ];
    let secpr1_fr: Vec<u8> = vec![
        81, 37, 99, 252, 194, 202, 185, 243, 132, 158, 23, 167, 173, 250, 230, 188, 255, 255, 255,
        255, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255,
    ];

    vec![bn254_fq, bn254_fr, secpk1_fr, secpk1_fq, secpr1_fq, secpr1_fr]
}

/// Whether to use a FunctionInput::constant or FunctionInput::witness:
///
/// (value, use_constant)
type ConstantOrWitness = (FieldElement, bool);

// For each ConstantOrWitness,
// - If use_constant, then convert to a FunctionInput::constant
// - Otherwise, convert to FunctionInput::witness
//   + With the Witness index as (input_index + offset)
fn constant_or_witness_to_function_inputs(
    xs: Vec<ConstantOrWitness>,
    offset: usize,
    num_bits: Option<u32>,
) -> Result<Vec<FunctionInput<FieldElement>>, OpcodeResolutionError<FieldElement>> {
    let num_bits = num_bits.unwrap_or(FieldElement::max_num_bits());
    xs.into_iter()
        .enumerate()
        .map(|(i, (x, use_constant))| {
            if use_constant {
                FunctionInput::constant(x, num_bits).map_err(From::from)
            } else {
                Ok(FunctionInput::witness(Witness((i + offset) as u32), num_bits))
            }
        })
        .collect()
}

// Convert ConstantOrWitness's back to FieldElement's by dropping the bool's
fn drop_use_constant(input: &[ConstantOrWitness]) -> Vec<FieldElement> {
    input.iter().map(|x| x.0).collect()
}

// equivalent values (ignoring use_constant)
fn drop_use_constant_eq(x: &[ConstantOrWitness], y: &[ConstantOrWitness]) -> bool {
    drop_use_constant(x) == drop_use_constant(y)
}

// Convert FieldElement's to ConstantOrWitness's by making all of them witnesses
fn use_witnesses(inputs: Vec<FieldElement>) -> Vec<ConstantOrWitness> {
    inputs.into_iter().map(|input| (input, false)).collect()
}

fn solve_array_input_blackbox_call<F>(
    inputs: Vec<ConstantOrWitness>,
    num_outputs: usize,
    num_bits: Option<u32>,
    f: F,
) -> Result<Vec<FieldElement>, OpcodeResolutionError<FieldElement>>
where
    F: FnOnce(
        (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
{
    let initial_witness_vec: Vec<_> =
        inputs.iter().enumerate().map(|(i, (x, _))| (Witness(i as u32), *x)).collect();
    let outputs: Vec<_> = (0..num_outputs)
        .map(|i| Witness((i + inputs.len()) as u32)) // offset past the indices of inputs
        .collect();
    let initial_witness = WitnessMap::from(BTreeMap::from_iter(initial_witness_vec));

    let inputs = constant_or_witness_to_function_inputs(inputs, 0, num_bits)?;
    let op = Opcode::BlackBoxFuncCall(f((inputs.clone(), outputs.clone()))?);
    let opcodes = vec![op];
    let unconstrained_functions = vec![];
    let mut acvm =
        ACVM::new(&Bn254BlackBoxSolver, &opcodes, initial_witness, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();

    Ok(outputs
        .iter()
        .map(|witness| *witness_map.get(witness).expect("all witnesses to be set"))
        .collect())
}

prop_compose! {
    fn bigint_with_modulus()(modulus in select(allowed_bigint_moduli()))
        (inputs in proptest::collection::vec(any::<(u8, bool)>(), modulus.len()), modulus in Just(modulus))
        -> (Vec<ConstantOrWitness>, Vec<u8>) {
        let inputs = inputs.into_iter().zip(modulus.iter()).map(|((input, use_constant), modulus_byte)| {
            (FieldElement::from(input.clamp(0, *modulus_byte) as u128), use_constant)
        }).collect();
        (inputs, modulus)
    }
}

prop_compose! {
    fn bigint_pair_with_modulus()(inputs_modulus in bigint_with_modulus())
        (second_inputs in proptest::collection::vec(any::<(u8, bool)>(), inputs_modulus.1.len()), inputs_modulus in Just(inputs_modulus))
        -> (Vec<ConstantOrWitness>, Vec<ConstantOrWitness>, Vec<u8>) {
        let (inputs, modulus) = inputs_modulus;
        let second_inputs = second_inputs.into_iter().zip(modulus.iter()).map(|((input, use_constant), modulus_byte)| {
            (FieldElement::from(input.clamp(0, *modulus_byte) as u128), use_constant)
        }).collect();
        (inputs, second_inputs, modulus)
    }
}

prop_compose! {
    fn bigint_triple_with_modulus()(inputs_pair_modulus in bigint_pair_with_modulus())
        (third_inputs in proptest::collection::vec(any::<(u8, bool)>(), inputs_pair_modulus.2.len()), inputs_pair_modulus in Just(inputs_pair_modulus))
        -> (Vec<ConstantOrWitness>, Vec<ConstantOrWitness>, Vec<ConstantOrWitness>, Vec<u8>) {
        let (inputs, second_inputs, modulus) = inputs_pair_modulus;
        let third_inputs = third_inputs.into_iter().zip(modulus.iter()).map(|((input, use_constant), modulus_byte)| {
            (FieldElement::from(input.clamp(0, *modulus_byte) as u128), use_constant)
        }).collect();
        (inputs, second_inputs, third_inputs, modulus)
    }
}

fn bigint_add_op() -> BlackBoxFuncCall<FieldElement> {
    BlackBoxFuncCall::BigIntAdd { lhs: 0, rhs: 1, output: 2 }
}

fn bigint_mul_op() -> BlackBoxFuncCall<FieldElement> {
    BlackBoxFuncCall::BigIntMul { lhs: 0, rhs: 1, output: 2 }
}

fn bigint_sub_op() -> BlackBoxFuncCall<FieldElement> {
    BlackBoxFuncCall::BigIntSub { lhs: 0, rhs: 1, output: 2 }
}

fn bigint_div_op() -> BlackBoxFuncCall<FieldElement> {
    BlackBoxFuncCall::BigIntDiv { lhs: 0, rhs: 1, output: 2 }
}

// Input is a BigInt, represented as a LE Vec of u8-range FieldElement's along
// with their use_constant values.
//
// Output is a zeroed BigInt that matches the input BigInt's
// - Byte length
// - use_constant values
fn bigint_zeroed(inputs: &[ConstantOrWitness]) -> Vec<ConstantOrWitness> {
    inputs.iter().map(|(_, use_constant)| (FieldElement::zero(), *use_constant)).collect()
}

// bigint_zeroed, but returns one
fn bigint_to_one(inputs: &[ConstantOrWitness]) -> Vec<ConstantOrWitness> {
    let mut one = bigint_zeroed(inputs);
    // little-endian
    one[0] = (FieldElement::one(), one[0].1);
    one
}

// Using the given BigInt modulus, solve the following circuit:
// - Convert xs, ys to BigInt's with ID's 0, 1, resp.
// - If the middle_op is present, run it
//   + Input BigInt ID's: 0, 1
//   + Output BigInt ID: 2
// - If the middle_op is missing, the output BigInt ID is 0
// - Run BigIntToLeBytes on the output BigInt ID
// - Output the resulting Vec of LE bytes
fn bigint_solve_binary_op_opt(
    middle_op: Option<BlackBoxFuncCall<FieldElement>>,
    modulus: Vec<u8>,
    lhs: Vec<ConstantOrWitness>,
    rhs: Vec<ConstantOrWitness>,
) -> Result<Vec<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let initial_witness_vec: Vec<_> = lhs
        .iter()
        .chain(rhs.iter())
        .enumerate()
        .map(|(i, (x, _))| (Witness(i as u32), *x))
        .collect();
    let output_witnesses: Vec<_> = initial_witness_vec
        .iter()
        .take(lhs.len())
        .enumerate()
        .map(|(index, _)| Witness((index + 2 * lhs.len()) as u32)) // offset past the indices of lhs, rhs
        .collect();
    let initial_witness = WitnessMap::from(BTreeMap::from_iter(initial_witness_vec));

    let lhs = constant_or_witness_to_function_inputs(lhs, 0, None)?;
    let rhs = constant_or_witness_to_function_inputs(rhs, lhs.len(), None)?;

    let to_op_input = if middle_op.is_some() { 2 } else { 0 };

    let bigint_from_lhs_op = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::BigIntFromLeBytes {
        inputs: lhs,
        modulus: modulus.clone(),
        output: 0,
    });
    let bigint_from_rhs_op = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::BigIntFromLeBytes {
        inputs: rhs,
        modulus: modulus.clone(),
        output: 1,
    });
    let bigint_to_op = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::BigIntToLeBytes {
        input: to_op_input,
        outputs: output_witnesses.clone(),
    });

    let mut opcodes = vec![bigint_from_lhs_op, bigint_from_rhs_op];
    if let Some(middle_op) = middle_op {
        opcodes.push(Opcode::BlackBoxFuncCall(middle_op));
    }
    opcodes.push(bigint_to_op);

    let unconstrained_functions = vec![];
    let mut acvm =
        ACVM::new(&StubbedBlackBoxSolver, &opcodes, initial_witness, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();
    Ok(output_witnesses
        .iter()
        .map(|witness| *witness_map.get(witness).expect("all witnesses to be set"))
        .collect())
}

// Solve the given BlackBoxFuncCall with witnesses: 1, 2 as x, y, resp.
#[cfg(test)]
fn solve_blackbox_func_call(
    blackbox_func_call: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
    ) -> Result<
        BlackBoxFuncCall<FieldElement>,
        OpcodeResolutionError<FieldElement>,
    >,
    lhs: (FieldElement, bool), // if false, use a Witness
    rhs: (FieldElement, bool), // if false, use a Witness
) -> Result<FieldElement, OpcodeResolutionError<FieldElement>> {
    let (lhs, lhs_constant) = lhs;
    let (rhs, rhs_constant) = rhs;

    let initial_witness =
        WitnessMap::from(BTreeMap::from_iter([(Witness(1), lhs), (Witness(2), rhs)]));

    let mut lhs_opt = None;
    if lhs_constant {
        lhs_opt = Some(lhs);
    }

    let mut rhs_opt = None;
    if rhs_constant {
        rhs_opt = Some(rhs);
    }

    let op = Opcode::BlackBoxFuncCall(blackbox_func_call(lhs_opt, rhs_opt)?);
    let opcodes = vec![op];
    let unconstrained_functions = vec![];
    let mut acvm =
        ACVM::new(&StubbedBlackBoxSolver, &opcodes, initial_witness, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();

    Ok(witness_map[&Witness(3)])
}

// N inputs
// 32 outputs
fn blake2s_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let (function_inputs, outputs) = function_inputs_and_outputs;
    Ok(BlackBoxFuncCall::Blake2s {
        inputs: function_inputs,
        outputs: outputs.try_into().expect("Blake2s returns 32 outputs"),
    })
}

// N inputs
// 32 outputs
fn blake3_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let (function_inputs, outputs) = function_inputs_and_outputs;
    Ok(BlackBoxFuncCall::Blake3 {
        inputs: function_inputs,
        outputs: outputs.try_into().expect("Blake3 returns 32 outputs"),
    })
}

// 25 inputs
// 25 outputs
fn keccakf1600_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let (function_inputs, outputs) = function_inputs_and_outputs;
    Ok(BlackBoxFuncCall::Keccakf1600 {
        inputs: function_inputs.try_into().expect("Keccakf1600 expects 25 inputs"),
        outputs: outputs.try_into().expect("Keccakf1600 returns 25 outputs"),
    })
}

// N inputs
// N outputs
fn poseidon2_permutation_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let (inputs, outputs) = function_inputs_and_outputs;
    let len = inputs.len() as u32;
    Ok(BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs, len })
}

// N inputs
// N outputs
fn poseidon2_permutation_invalid_len_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let (inputs, outputs) = function_inputs_and_outputs;
    let len = (inputs.len() as u32) + 1;
    Ok(BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs, len })
}

// 24 inputs (16 + 8)
// 8 outputs
fn sha256_compression_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let (function_inputs, outputs) = function_inputs_and_outputs;
    let mut function_inputs = function_inputs.into_iter();
    let inputs = core::array::from_fn(|_| function_inputs.next().unwrap());
    let hash_values = core::array::from_fn(|_| function_inputs.next().unwrap());
    Ok(BlackBoxFuncCall::Sha256Compression {
        inputs: Box::new(inputs),
        hash_values: Box::new(hash_values),
        outputs: outputs.try_into().unwrap(),
    })
}

fn into_repr_vec<T>(fields: T) -> Vec<ark_bn254::Fr>
where
    T: IntoIterator<Item = FieldElement>,
{
    fields.into_iter().map(|field| field.into_repr()).collect()
}

// fn into_repr_mat<T, U>(fields: T) -> Vec<Vec<ark_bn254::Fr>>
// where
//     T: IntoIterator<Item = U>,
//     U: IntoIterator<Item = FieldElement>,
// {
//     fields.into_iter().map(|field| into_repr_vec(field)).collect()
// }

fn into_old_ark_field<T, U>(field: T) -> U
where
    T: AcirField,
    U: ark_ff_v04::PrimeField,
{
    U::from_be_bytes_mod_order(&field.to_be_bytes())
}

fn into_new_ark_field<T, U>(field: T) -> U
where
    T: ark_ff_v04::PrimeField,
    U: ark_ff::PrimeField,
{
    use zkhash::ark_ff::BigInteger;

    U::from_be_bytes_mod_order(&field.into_bigint().to_bytes_be())
}

fn run_both_poseidon2_permutations(
    inputs: Vec<ConstantOrWitness>,
) -> Result<(Vec<ark_bn254::Fr>, Vec<ark_bn254::Fr>), OpcodeResolutionError<FieldElement>> {
    let result = solve_array_input_blackbox_call(
        inputs.clone(),
        inputs.len(),
        None,
        poseidon2_permutation_op,
    )?;

    let poseidon2_t = POSEIDON2_CONFIG.t as usize;
    let poseidon2_d = 5;
    let rounds_f = POSEIDON2_CONFIG.rounds_f as usize;
    let rounds_p = POSEIDON2_CONFIG.rounds_p as usize;
    let mat_internal_diag_m_1: Vec<ark_bn254_v04::Fr> =
        POSEIDON2_CONFIG.internal_matrix_diagonal.into_iter().map(into_old_ark_field).collect();
    let mat_internal = vec![];
    let round_constants: Vec<Vec<ark_bn254_v04::Fr>> = POSEIDON2_CONFIG
        .round_constant
        .into_iter()
        .map(|fields| fields.into_iter().map(into_old_ark_field).collect())
        .collect();

    let external_poseidon2 = zkhash::poseidon2::poseidon2::Poseidon2::new(&Arc::new(
        zkhash::poseidon2::poseidon2_params::Poseidon2Params::new(
            poseidon2_t,
            poseidon2_d,
            rounds_f,
            rounds_p,
            &mat_internal_diag_m_1,
            &mat_internal,
            &round_constants,
        ),
    ));

    let expected_result = external_poseidon2.permutation(
        &drop_use_constant(&inputs)
            .into_iter()
            .map(into_old_ark_field)
            .collect::<Vec<ark_bn254_v04::Fr>>(),
    );
    Ok((into_repr_vec(result), expected_result.into_iter().map(into_new_ark_field).collect()))
}

// Using the given BigInt modulus, solve the following circuit:
// - Convert xs, ys to BigInt's with ID's 0, 1, resp.
// - Run the middle_op:
//   + Input BigInt ID's: 0, 1
//   + Output BigInt ID: 2
// - Run BigIntToLeBytes on the output BigInt ID
// - Output the resulting Vec of LE bytes
fn bigint_solve_binary_op(
    middle_op: BlackBoxFuncCall<FieldElement>,
    modulus: Vec<u8>,
    lhs: Vec<ConstantOrWitness>,
    rhs: Vec<ConstantOrWitness>,
) -> Vec<FieldElement> {
    bigint_solve_binary_op_opt(Some(middle_op), modulus, lhs, rhs).unwrap()
}

// Using the given BigInt modulus, solve the following circuit:
// - Convert the input to a BigInt with ID 0
// - Run BigIntToLeBytes on BigInt ID 0
// - Output the resulting Vec of LE bytes
fn bigint_solve_from_to_le_bytes(
    modulus: Vec<u8>,
    inputs: Vec<ConstantOrWitness>,
) -> Vec<FieldElement> {
    bigint_solve_binary_op_opt(None, modulus, inputs, vec![]).unwrap()
}

fn function_input_from_option(
    witness: Witness,
    opt_constant: Option<FieldElement>,
) -> Result<FunctionInput<FieldElement>, OpcodeResolutionError<FieldElement>> {
    opt_constant
        .map(|constant| {
            FunctionInput::constant(constant, FieldElement::max_num_bits()).map_err(From::from)
        })
        .unwrap_or(Ok(FunctionInput::witness(witness, FieldElement::max_num_bits())))
}

fn and_op(
    x: Option<FieldElement>,
    y: Option<FieldElement>,
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let lhs = function_input_from_option(Witness(1), x)?;
    let rhs = function_input_from_option(Witness(2), y)?;
    Ok(BlackBoxFuncCall::AND { lhs, rhs, output: Witness(3) })
}

fn xor_op(
    x: Option<FieldElement>,
    y: Option<FieldElement>,
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let lhs = function_input_from_option(Witness(1), x)?;
    let rhs = function_input_from_option(Witness(2), y)?;
    Ok(BlackBoxFuncCall::XOR { lhs, rhs, output: Witness(3) })
}

fn prop_assert_commutative(
    op: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
    x: (FieldElement, bool),
    y: (FieldElement, bool),
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(&op, x, y).unwrap(), solve_blackbox_func_call(&op, y, x).unwrap())
}

fn prop_assert_associative(
    op: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
    x: (FieldElement, bool),
    y: (FieldElement, bool),
    z: (FieldElement, bool),
    use_constant_xy: bool,
    use_constant_yz: bool,
) -> (FieldElement, FieldElement) {
    let f_xy = (solve_blackbox_func_call(&op, x, y).unwrap(), use_constant_xy);
    let f_f_xy_z = solve_blackbox_func_call(&op, f_xy, z).unwrap();

    let f_yz = (solve_blackbox_func_call(&op, y, z).unwrap(), use_constant_yz);
    let f_x_f_yz = solve_blackbox_func_call(&op, x, f_yz).unwrap();

    (f_f_xy_z, f_x_f_yz)
}

fn prop_assert_identity_l(
    op: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
    op_identity: (FieldElement, bool),
    x: (FieldElement, bool),
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(op, op_identity, x).unwrap(), x.0)
}

fn prop_assert_zero_l(
    op: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
    op_zero: (FieldElement, bool),
    x: (FieldElement, bool),
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(op, op_zero, x).unwrap(), FieldElement::zero())
}

// Test that varying one of the inputs produces a different result
//
// (is the op injective for the given inputs?, failure string)
fn prop_assert_injective<F>(
    inputs: Vec<ConstantOrWitness>,
    distinct_inputs: Vec<ConstantOrWitness>,
    num_outputs: usize,
    num_bits: Option<u32>,
    op: F,
) -> (bool, String)
where
    F: FnOnce(
            (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
        )
            -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>
        + Clone,
{
    let equal_inputs = drop_use_constant_eq(&inputs, &distinct_inputs);
    let message = format!("not injective:\n{:?}\n{:?}", &inputs, &distinct_inputs);
    let outputs_not_equal =
        solve_array_input_blackbox_call(inputs, num_outputs, num_bits, op.clone())
            .expect("injectivity test operations to have valid input")
            != solve_array_input_blackbox_call(distinct_inputs, num_outputs, num_bits, op)
                .expect("injectivity test operations to have valid input");
    (equal_inputs || outputs_not_equal, message)
}

fn field_element_ones() -> FieldElement {
    let exponent: FieldElement = (253_u128).into();
    FieldElement::from(2u128).pow(&exponent) - FieldElement::one()
}

prop_compose! {
    // Use both `u128` and hex proptest strategies
    fn field_element()
        (u128_or_hex in maybe_ok(any::<u128>(), "[0-9a-f]{64}"),
         constant_input: bool)
        -> (FieldElement, bool)
    {
        match u128_or_hex {
            Ok(number) => (FieldElement::from(number), constant_input),
            Err(hex) => (FieldElement::from_hex(&hex).expect("should accept any 32 byte hex string"), constant_input),
        }
    }
}

prop_compose! {
    fn any_distinct_inputs(max_input_bits: Option<usize>, min_size: usize, max_size: usize)
        (size_and_patch in any::<(usize, usize, usize)>()) // NOTE: macro ambiguity when using (x: T)
        (inputs_distinct_inputs in
            (proptest::collection::vec(any::<(u128, bool)>(), std::cmp::max(min_size, size_and_patch.0 % max_size)),
             proptest::collection::vec(any::<(u128, bool)>(), std::cmp::max(min_size, size_and_patch.0 % max_size))),
            size_and_patch in Just(size_and_patch))
        -> (Vec<ConstantOrWitness>, Vec<ConstantOrWitness>) {
        let (_size, patch_location, patch_value) = size_and_patch;
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let modulus = if let Some(max_input_bits) = max_input_bits {
            1u128 << max_input_bits
        } else {
            1
        };
        let to_input = |(x, use_constant)| {
            (FieldElement::from(x % modulus), use_constant)
        };
        let inputs: Vec<_> = inputs.into_iter().map(to_input).collect();
        let mut distinct_inputs: Vec<_> = distinct_inputs.into_iter().map(to_input).collect();

        // if equivalent w/o use_constant, patch with the patch_value
        if drop_use_constant_eq(&inputs, &distinct_inputs) {
            let distinct_inputs_len = distinct_inputs.len();
            let positive_patch_value = std::cmp::max(patch_value, 1);
            if distinct_inputs_len != 0 {
                let previous_input = &mut distinct_inputs[patch_location % distinct_inputs_len].0;
                let patched_input: BigUint = (*previous_input + FieldElement::from(positive_patch_value)).into_repr().into();
                *previous_input = FieldElement::from_be_bytes_reduce(&(patched_input % BigUint::from(modulus)).to_bytes_be());
            } else {
                distinct_inputs.push((FieldElement::zero(), true));
            }
        }

        (inputs, distinct_inputs)
    }
}

#[test]
fn poseidon2_permutation_zeroes() {
    let use_constants: [bool; 4] = [false; 4];
    let inputs: Vec<_> = [FieldElement::zero(); 4].into_iter().zip(use_constants).collect();
    let (results, expected_results) = run_both_poseidon2_permutations(inputs).unwrap();

    let internal_expected_results = vec![
        field_from_hex("18DFB8DC9B82229CFF974EFEFC8DF78B1CE96D9D844236B496785C698BC6732E"),
        field_from_hex("095C230D1D37A246E8D2D5A63B165FE0FADE040D442F61E25F0590E5FB76F839"),
        field_from_hex("0BB9545846E1AFA4FA3C97414A60A20FC4949F537A68CCECA34C5CE71E28AA59"),
        field_from_hex("18A4F34C9C6F99335FF7638B82AEED9018026618358873C982BBDDE265B2ED6D"),
    ];

    assert_eq!(expected_results, into_repr_vec(internal_expected_results));
    assert_eq!(results, expected_results);
}

#[test]
fn sha256_compression_zeros() {
    let results = solve_array_input_blackbox_call(
        [(FieldElement::zero(), false); 24].into(),
        8,
        None,
        sha256_compression_op,
    );
    let expected_results: Vec<_> = vec![
        2091193876, 1113340840, 3461668143, 3254913767, 3068490961, 2551409935, 2927503052,
        3205228454,
    ]
    .into_iter()
    .map(|x: u128| FieldElement::from(x))
    .collect();
    assert_eq!(results, Ok(expected_results));
}

#[test]
fn blake2s_zeros() {
    let results = solve_array_input_blackbox_call(vec![], 32, None, blake2s_op);
    let expected_results: Vec<_> = vec![
        105, 33, 122, 48, 121, 144, 128, 148, 225, 17, 33, 208, 66, 53, 74, 124, 31, 85, 182, 72,
        44, 161, 165, 30, 27, 37, 13, 253, 30, 208, 238, 249,
    ]
    .into_iter()
    .map(|x: u128| FieldElement::from(x))
    .collect();
    assert_eq!(results, Ok(expected_results));
}

#[test]
fn blake3_zeros() {
    let results = solve_array_input_blackbox_call(vec![], 32, None, blake3_op);
    let expected_results: Vec<_> = vec![
        175, 19, 73, 185, 245, 249, 161, 166, 160, 64, 77, 234, 54, 220, 201, 73, 155, 203, 37,
        201, 173, 193, 18, 183, 204, 154, 147, 202, 228, 31, 50, 98,
    ]
    .into_iter()
    .map(|x: u128| FieldElement::from(x))
    .collect();
    assert_eq!(results, Ok(expected_results));
}

#[test]
fn keccakf1600_zeros() {
    let results = solve_array_input_blackbox_call(
        [(FieldElement::zero(), false); 25].into(),
        25,
        Some(64),
        keccakf1600_op,
    );
    let expected_results: Vec<_> = vec![
        17376452488221285863,
        9571781953733019530,
        15391093639620504046,
        13624874521033984333,
        10027350355371872343,
        18417369716475457492,
        10448040663659726788,
        10113917136857017974,
        12479658147685402012,
        3500241080921619556,
        16959053435453822517,
        12224711289652453635,
        9342009439668884831,
        4879704952849025062,
        140226327413610143,
        424854978622500449,
        7259519967065370866,
        7004910057750291985,
        13293599522548616907,
        10105770293752443592,
        10668034807192757780,
        1747952066141424100,
        1654286879329379778,
        8500057116360352059,
        16929593379567477321,
    ]
    .into_iter()
    .map(|x: u128| FieldElement::from(x))
    .collect();

    assert_eq!(results, Ok(expected_results));
}

// NOTE: an "average" bigint is large, so consider increasing the number of proptest shrinking
// iterations (from the default 1024) to reach a simplified case, e.g.
// PROPTEST_MAX_SHRINK_ITERS=1024000
proptest! {

    #[test]
    fn and_commutative(x in field_element(), y in field_element()) {
        let (lhs, rhs) = prop_assert_commutative(and_op, x, y);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn xor_commutative(x in field_element(), y in field_element()) {
        let (lhs, rhs) = prop_assert_commutative(xor_op, x, y);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn and_associative(x in field_element(), y in field_element(), z in field_element(), use_constant_xy: bool, use_constant_yz: bool) {
        let (lhs, rhs) = prop_assert_associative(and_op, x, y, z, use_constant_xy, use_constant_yz);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    // TODO(https://github.com/noir-lang/noir/issues/5638)
    #[should_panic(expected = "assertion failed: `(left == right)`")]
    fn xor_associative(x in field_element(), y in field_element(), z in field_element(), use_constant_xy: bool, use_constant_yz: bool) {
        let (lhs, rhs) = prop_assert_associative(xor_op, x, y, z, use_constant_xy, use_constant_yz);
        prop_assert_eq!(lhs, rhs);
    }

    // test that AND(x, x) == x
    #[test]
    fn and_self_identity(x in field_element()) {
        prop_assert_eq!(solve_blackbox_func_call(and_op, x, x).unwrap(), x.0);
    }

    // test that XOR(x, x) == 0
    #[test]
    fn xor_self_zero(x in field_element()) {
        prop_assert_eq!(solve_blackbox_func_call(xor_op, x, x).unwrap(), FieldElement::zero());
    }

    #[test]
    fn and_identity_l(x in field_element(), ones_constant: bool) {
        let ones = (field_element_ones(), ones_constant);
        let (lhs, rhs) = prop_assert_identity_l(and_op, ones, x);
        if x <= ones {
            prop_assert_eq!(lhs, rhs);
        } else {
            prop_assert!(lhs != rhs);
        }
    }

    #[test]
    fn xor_identity_l(x in field_element(), zero_constant: bool) {
        let zero = (FieldElement::zero(), zero_constant);
        let (lhs, rhs) = prop_assert_identity_l(xor_op, zero, x);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn and_zero_l(x in field_element(), ones_constant: bool) {
        let zero = (FieldElement::zero(), ones_constant);
        let (lhs, rhs) = prop_assert_zero_l(and_op, zero, x);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn poseidon2_permutation_matches_external_impl(inputs in proptest::collection::vec(field_element(), 4)) {
        let (result, expected_result) = run_both_poseidon2_permutations(inputs).unwrap();
        prop_assert_eq!(result, expected_result)
    }


    #[test]
    fn sha256_compression_injective(inputs_distinct_inputs in any_distinct_inputs(None, 24, 24)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        if inputs.len() == 24 && distinct_inputs.len() == 24 {
            let (result, message) = prop_assert_injective(inputs, distinct_inputs, 8, None, sha256_compression_op);
            prop_assert!(result, "{}", message);
        }
    }

    #[test]
    fn blake2s_injective(inputs_distinct_inputs in any_distinct_inputs(None, 0, 32)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, None, blake2s_op);
        prop_assert!(result, "{}", message);
    }

    #[test]
    fn blake3_injective(inputs_distinct_inputs in any_distinct_inputs(None, 0, 32)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, None, blake3_op);
        prop_assert!(result, "{}", message);
    }

    #[test]
    fn keccakf1600_injective(inputs_distinct_inputs in any_distinct_inputs(Some(8), 25, 25)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        assert_eq!(inputs.len(), 25);
        assert_eq!(distinct_inputs.len(), 25);
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 25, Some(64), keccakf1600_op);
        prop_assert!(result, "{}", message);
    }

    // TODO(https://github.com/noir-lang/noir/issues/5699): wrong failure message
    #[test]
    #[should_panic(expected = "Failure(BlackBoxFunctionFailed(Poseidon2Permutation, \"the number of inputs does not match specified length. 6 != 7\"))")]
    fn poseidon2_permutation_invalid_size_fails(inputs_distinct_inputs in any_distinct_inputs(None, 6, 6)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 1, None, poseidon2_permutation_invalid_len_op);
        prop_assert!(result, "{}", message);
    }

    #[test]
    fn bigint_from_to_le_bytes_zero_one(modulus in select(allowed_bigint_moduli()), zero_or_ones_constant: bool, use_constant: bool) {
        let zero_function_input = if zero_or_ones_constant {
            FieldElement::one()
        } else {
            FieldElement::zero()
        };
        let zero_or_ones: Vec<_> = modulus.iter().map(|_| (zero_function_input, use_constant)).collect();
        let expected_results = drop_use_constant(&zero_or_ones);
        let results = bigint_solve_from_to_le_bytes(modulus.clone(), zero_or_ones);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_from_to_le_bytes((input, modulus) in bigint_with_modulus()) {
        let expected_results: Vec<_> = drop_use_constant(&input);
        let results = bigint_solve_from_to_le_bytes(modulus.clone(), input);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    // TODO(https://github.com/noir-lang/noir/issues/5580): desired behavior?
    fn bigint_from_to_le_bytes_extra_input_bytes((input, modulus) in bigint_with_modulus(), extra_bytes_len: u8, extra_bytes in proptest::collection::vec(any::<(u8, bool)>(), u8::MAX as usize)) {
        let mut input = input;
        let mut extra_bytes: Vec<_> = extra_bytes.into_iter().take(extra_bytes_len as usize).map(|(x, use_constant)| (FieldElement::from(x as u128), use_constant)).collect();
        input.append(&mut extra_bytes);
        let expected_results: Vec<_> = drop_use_constant(&input);
        let results = bigint_solve_from_to_le_bytes(modulus.clone(), input);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    // TODO(https://github.com/noir-lang/noir/issues/5580): desired behavior?
    #[should_panic(expected = "Test failed: assertion failed: `(left == right)`")]
    fn bigint_from_to_le_bytes_bigger_than_u8((input, modulus) in bigint_with_modulus(), patch_location: usize, larger_value: u16, use_constant: bool) {
        let mut input = input;
        let patch_location = patch_location % input.len();
        let larger_value = FieldElement::from(std::cmp::max((u8::MAX as u16) + 1, larger_value) as u128);
        input[patch_location] = (larger_value, use_constant);
        let expected_results: Vec<_> = drop_use_constant(&input);
        let results = bigint_solve_from_to_le_bytes(modulus.clone(), input);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    // TODO(https://github.com/noir-lang/noir/issues/5578): this test attempts to use a guaranteed-invalid BigInt modulus
    // #[should_panic(expected = "attempt to add with overflow")]
    fn bigint_from_to_le_bytes_disallowed_modulus(mut modulus in select(allowed_bigint_moduli()), patch_location: usize, patch_amount: u8, zero_or_ones_constant: bool, use_constant: bool) {
        let allowed_moduli: HashSet<Vec<u8>> = allowed_bigint_moduli().into_iter().collect();
        let mut patch_location = patch_location % modulus.len();
        let patch_amount = patch_amount.clamp(1, u8::MAX);
        while allowed_moduli.contains(&modulus) {
            modulus[patch_location] = patch_amount.wrapping_add(modulus[patch_location]);
            patch_location += 1;
            patch_location %= modulus.len();
        }

        let zero_function_input = if zero_or_ones_constant {
            FieldElement::zero()
        } else {
            FieldElement::one()
        };
        let zero: Vec<_> = modulus.iter().map(|_| (zero_function_input, use_constant)).collect();
        let expected_results: Vec<_> = drop_use_constant(&zero);
        let results = bigint_solve_from_to_le_bytes(modulus.clone(), zero);

        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_add_commutative((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let lhs_results = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), xs.clone(), ys.clone());
        let rhs_results = bigint_solve_binary_op(bigint_add_op(), modulus, ys, xs);

        prop_assert_eq!(lhs_results, rhs_results)
    }

    #[test]
    fn bigint_mul_commutative((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let lhs_results = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs.clone(), ys.clone());
        let rhs_results = bigint_solve_binary_op(bigint_mul_op(), modulus, ys, xs);

        prop_assert_eq!(lhs_results, rhs_results)
    }

    #[test]
    fn bigint_add_associative((xs, ys, zs, modulus) in bigint_triple_with_modulus()) {
        // f(f(xs, ys), zs) ==
        let op_xs_ys = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), xs.clone(), ys.clone());
        let xs_ys = use_witnesses(op_xs_ys);
        let op_xs_ys_op_zs = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), xs_ys, zs.clone());

        // f(xs, f(ys, zs))
        let op_ys_zs = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), ys.clone(), zs.clone());
        let ys_zs = use_witnesses(op_ys_zs);
        let op_xs_op_ys_zs = bigint_solve_binary_op(bigint_add_op(), modulus, xs, ys_zs);

        prop_assert_eq!(op_xs_ys_op_zs, op_xs_op_ys_zs)
    }

    #[test]
    fn bigint_mul_associative((xs, ys, zs, modulus) in bigint_triple_with_modulus()) {
        // f(f(xs, ys), zs) ==
        let op_xs_ys = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs.clone(), ys.clone());
        let xs_ys = use_witnesses(op_xs_ys);
        let op_xs_ys_op_zs = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs_ys, zs.clone());

        // f(xs, f(ys, zs))
        let op_ys_zs = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), ys.clone(), zs.clone());
        let ys_zs = use_witnesses(op_ys_zs);
        let op_xs_op_ys_zs = bigint_solve_binary_op(bigint_mul_op(), modulus, xs, ys_zs);

        prop_assert_eq!(op_xs_ys_op_zs, op_xs_op_ys_zs)
    }

    #[test]
    fn bigint_mul_add_distributive((xs, ys, zs, modulus) in bigint_triple_with_modulus()) {
        // xs * (ys + zs) ==
        let add_ys_zs = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), ys.clone(), zs.clone());
        let add_ys_zs = use_witnesses(add_ys_zs);
        let mul_xs_add_ys_zs = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs.clone(), add_ys_zs);

        // xs * ys + xs * zs
        let mul_xs_ys = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs.clone(), ys);
        let mul_xs_ys = use_witnesses(mul_xs_ys);
        let mul_xs_zs = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs, zs);
        let mul_xs_zs = use_witnesses(mul_xs_zs);
        let add_mul_xs_ys_mul_xs_zs = bigint_solve_binary_op(bigint_add_op(), modulus, mul_xs_ys, mul_xs_zs);

        prop_assert_eq!(mul_xs_add_ys_zs, add_mul_xs_ys_mul_xs_zs)
    }


    #[test]
    fn bigint_add_zero_l((xs, modulus) in bigint_with_modulus()) {
        let zero = bigint_zeroed(&xs);
        let expected_results = drop_use_constant(&xs);
        let results = bigint_solve_binary_op(bigint_add_op(), modulus, zero, xs);

        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_mul_zero_l((xs, modulus) in bigint_with_modulus()) {
        let zero = bigint_zeroed(&xs);
        let expected_results = drop_use_constant(&zero);
        let results = bigint_solve_binary_op(bigint_mul_op(), modulus, zero, xs);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_mul_one_l((xs, modulus) in bigint_with_modulus()) {
        let one = bigint_to_one(&xs);
        let expected_results: Vec<_> = drop_use_constant(&xs);
        let results = bigint_solve_binary_op(bigint_mul_op(), modulus, one, xs);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_sub_self((xs, modulus) in bigint_with_modulus()) {
        let expected_results = drop_use_constant(&bigint_zeroed(&xs));
        let results = bigint_solve_binary_op(bigint_sub_op(), modulus, xs.clone(), xs);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_sub_zero((xs, modulus) in bigint_with_modulus()) {
        let zero = bigint_zeroed(&xs);
        let expected_results: Vec<_> = drop_use_constant(&xs);
        let results = bigint_solve_binary_op(bigint_sub_op(), modulus, xs, zero);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_sub_one((xs, modulus) in bigint_with_modulus()) {
        let one = bigint_to_one(&xs);
        let expected_results: Vec<_> = drop_use_constant(&xs);
        let results = bigint_solve_binary_op(bigint_sub_op(), modulus, xs, one);
        prop_assert!(results != expected_results, "{:?} == {:?}", results, expected_results)
    }

    #[test]
    fn bigint_div_self((xs, modulus) in bigint_with_modulus()) {
        let one = drop_use_constant(&bigint_to_one(&xs));
        let results = bigint_solve_binary_op(bigint_div_op(), modulus, xs.clone(), xs);
        prop_assert_eq!(results, one)
    }

    #[test]
    // TODO(https://github.com/noir-lang/noir/issues/5645)
    fn bigint_div_by_zero((xs, modulus) in bigint_with_modulus()) {
        let zero = bigint_zeroed(&xs);
        let expected_results = drop_use_constant(&zero);
        let results = bigint_solve_binary_op(bigint_div_op(), modulus, xs, zero);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_div_one((xs, modulus) in bigint_with_modulus()) {
        let one = bigint_to_one(&xs);
        let expected_results = drop_use_constant(&xs);
        let results = bigint_solve_binary_op(bigint_div_op(), modulus, xs, one);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_div_zero((xs, modulus) in bigint_with_modulus()) {
        let zero = bigint_zeroed(&xs);
        let expected_results = drop_use_constant(&zero);
        let results = bigint_solve_binary_op(bigint_div_op(), modulus, zero, xs);
        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_add_sub((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let expected_results = drop_use_constant(&xs);
        let add_results = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), xs, ys.clone());
        let add_bigint = use_witnesses(add_results);
        let results = bigint_solve_binary_op(bigint_sub_op(), modulus, add_bigint, ys);

        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_sub_add((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let expected_results = drop_use_constant(&xs);
        let sub_results = bigint_solve_binary_op(bigint_sub_op(), modulus.clone(), xs, ys.clone());
        let add_bigint = use_witnesses(sub_results);
        let results = bigint_solve_binary_op(bigint_add_op(), modulus, add_bigint, ys);

        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_div_mul((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let expected_results = drop_use_constant(&xs);
        let div_results = bigint_solve_binary_op(bigint_div_op(), modulus.clone(), xs, ys.clone());
        let div_bigint = use_witnesses(div_results);
        let results = bigint_solve_binary_op(bigint_mul_op(), modulus, div_bigint, ys);

        prop_assert_eq!(results, expected_results)
    }

    #[test]
    fn bigint_mul_div((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let expected_results = drop_use_constant(&xs);
        let mul_results = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs, ys.clone());
        let mul_bigint = use_witnesses(mul_results);
        let results = bigint_solve_binary_op(bigint_div_op(), modulus, mul_bigint, ys);

        prop_assert_eq!(results, expected_results)
    }
}
