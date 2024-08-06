use std::collections::BTreeMap;
use std::sync::Arc;

use acir::{
    acir_field::GenericFieldElement,
    brillig::{BinaryFieldOp, HeapArray, MemoryAddress, Opcode as BrilligOpcode, ValueOrArray},
    circuit::{
        brillig::{BrilligBytecode, BrilligInputs, BrilligOutputs},
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

use proptest::arbitrary::any;
use proptest::prelude::*;
use proptest::result::maybe_ok;
use zkhash::poseidon2::poseidon2_params::Poseidon2Params;

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

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: MemoryAddress::from(0),
        rhs: MemoryAddress::from(1),
        destination: MemoryAddress::from(2),
    };

    let opcodes = vec![
        Opcode::BrilligCall {
            id: 0,
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

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress(0),
                size: 2,
                offset: 0,
            },
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(1))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(0))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 3 },
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

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: MemoryAddress::from(0),
        rhs: MemoryAddress::from(1),
        destination: MemoryAddress::from(4),
    };

    let opcodes = vec![
        Opcode::BrilligCall {
            id: 0,
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

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress(0),
                size: 3,
                offset: 0,
            },
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(1))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(0))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(3))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(2))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 5 },
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

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress(0),
                size: 3,
                offset: 0,
            },
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(1))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(0))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(3))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(2))],
                input_value_types: vec![HeapValueType::field()],
            },
            BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 4 },
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
            id: 0,
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
        lhs: MemoryAddress::from(0),
        rhs: MemoryAddress::from(1),
        destination: MemoryAddress::from(2),
    };

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![
            BrilligOpcode::CalldataCopy {
                destination_address: MemoryAddress(0),
                size: 2,
                offset: 0,
            },
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(1))],
                destination_value_types: vec![HeapValueType::field()],
                inputs: vec![ValueOrArray::MemoryAddress(MemoryAddress::from(0))],
                input_value_types: vec![HeapValueType::field()],
            },
        ],
    };

    let opcodes = vec![Opcode::BrilligCall {
        id: 0,
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

    let calldata_copy_opcode =
        BrilligOpcode::CalldataCopy { destination_address: MemoryAddress(0), size: 2, offset: 0 };

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: MemoryAddress::from(0),
        rhs: MemoryAddress::from(1),
        destination: MemoryAddress::from(2),
    };
    // Jump pass the trap if the values are equal, else
    // jump to the trap
    let location_of_stop = 3;

    let jmp_if_opcode =
        BrilligOpcode::JumpIf { condition: MemoryAddress::from(2), location: location_of_stop };

    let trap_opcode = BrilligOpcode::Trap { revert_data: HeapArray::default() };
    let stop_opcode = BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 0 };

    let brillig_bytecode = BrilligBytecode {
        bytecode: vec![calldata_copy_opcode, equal_opcode, jmp_if_opcode, trap_opcode, stop_opcode],
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
            id: 0,
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
            payload: None,
            call_stack: vec![OpcodeLocation::Brillig { acir_index: 0, brillig_index: 3 }]
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

/// Whether to use a FunctionInput::constant or FunctionInput::witness:
///
/// (value, use_constant)
type ConstantOrWitness = (FieldElement, bool);

// For each ConstantOrWitness,
// - If use_constant, then convert to a FunctionInput::constant
// - Otherwise, convert to FunctionInput::witness
//   + With the Witness index as (input_index + offset)
//
// Both use FieldElement::max_num_bits as the number of bits.
fn constant_or_witness_to_function_inputs(
    xs: Vec<ConstantOrWitness>,
    offset: usize,
) -> Vec<FunctionInput<FieldElement>> {
    xs.into_iter()
        .enumerate()
        .map(|(i, (x, use_constant))| {
            if use_constant {
                FunctionInput::constant(x, FieldElement::max_num_bits())
            } else {
                FunctionInput::witness(Witness((i + offset) as u32), FieldElement::max_num_bits())
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

fn solve_array_input_blackbox_call<F>(
    inputs: Vec<ConstantOrWitness>,
    num_outputs: usize,
    f: F,
) -> Vec<FieldElement>
where
    F: FnOnce((Vec<FunctionInput<FieldElement>>, Vec<Witness>)) -> BlackBoxFuncCall<FieldElement>,
{
    let initial_witness_vec: Vec<_> =
        inputs.iter().enumerate().map(|(i, (x, _))| (Witness(i as u32), *x)).collect();
    let outputs: Vec<_> = (0..num_outputs)
        .map(|i| Witness((i + inputs.len()) as u32)) // offset past the indices of inputs
        .collect();
    let initial_witness = WitnessMap::from(BTreeMap::from_iter(initial_witness_vec));

    let inputs = constant_or_witness_to_function_inputs(inputs, 0);
    let op = Opcode::BlackBoxFuncCall(f((inputs.clone(), outputs.clone())));
    let opcodes = vec![op];
    let unconstrained_functions = vec![];
    let mut acvm =
        ACVM::new(&Bn254BlackBoxSolver, &opcodes, initial_witness, &unconstrained_functions, &[]);

    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();

    outputs
        .iter()
        .map(|witness| *witness_map.get(witness).expect("all witnesses to be set"))
        .collect()
}

// Solve the given BlackBoxFuncCall with witnesses: 1, 2 as x, y, resp.
#[cfg(test)]
fn solve_blackbox_func_call(
    blackbox_func_call: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
    ) -> BlackBoxFuncCall<FieldElement>,
    x: (FieldElement, bool), // if false, use a Witness
    y: (FieldElement, bool), // if false, use a Witness
) -> FieldElement {
    let (x, x_constant) = x;
    let (y, y_constant) = y;

    let initial_witness = WitnessMap::from(BTreeMap::from_iter([(Witness(1), x), (Witness(2), y)]));

    let mut lhs = None;
    if x_constant {
        lhs = Some(x);
    }

    let mut rhs = None;
    if y_constant {
        rhs = Some(y);
    }

    let op = Opcode::BlackBoxFuncCall(blackbox_func_call(lhs, rhs));
    let opcodes = vec![op];
    let unconstrained_functions = vec![];
    let mut acvm =
        ACVM::new(&StubbedBlackBoxSolver, &opcodes, initial_witness, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();

    witness_map[&Witness(3)]
}

// N inputs
// 32 outputs
fn sha256_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> BlackBoxFuncCall<FieldElement> {
    let (function_inputs, outputs) = function_inputs_and_outputs;
    BlackBoxFuncCall::SHA256 {
        inputs: function_inputs,
        outputs: outputs.try_into().expect("SHA256 returns 32 outputs"),
    }
}

// N inputs
// 32 outputs
fn blake2s_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> BlackBoxFuncCall<FieldElement> {
    let (function_inputs, outputs) = function_inputs_and_outputs;
    BlackBoxFuncCall::Blake2s {
        inputs: function_inputs,
        outputs: outputs.try_into().expect("Blake2s returns 32 outputs"),
    }
}

// N inputs
// 32 outputs
fn blake3_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> BlackBoxFuncCall<FieldElement> {
    let (function_inputs, outputs) = function_inputs_and_outputs;
    BlackBoxFuncCall::Blake3 {
        inputs: function_inputs,
        outputs: outputs.try_into().expect("Blake3 returns 32 outputs"),
    }
}

// variable inputs
// 32 outputs
fn keccak256_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> BlackBoxFuncCall<FieldElement> {
    let (function_inputs, mut outputs) = function_inputs_and_outputs;
    let function_inputs_len = function_inputs.len();
    outputs.truncate(32);
    BlackBoxFuncCall::Keccak256 {
        inputs: function_inputs,
        var_message_size: FunctionInput::constant(
            function_inputs_len.into(),
            FieldElement::max_num_bits(),
        ),
        outputs: outputs.try_into().expect("Keccak256 returns 32 outputs"),
    }
}

// var_message_size is the number of bytes to take
// from the input. Note: if `var_message_size`
// is more than the number of bytes in the input,
// then an error is returned.
//
// variable inputs
// 32 outputs
fn keccak256_invalid_message_size_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> BlackBoxFuncCall<FieldElement> {
    let (function_inputs, mut outputs) = function_inputs_and_outputs;
    let function_inputs_len = function_inputs.len();
    outputs.truncate(32);
    BlackBoxFuncCall::Keccak256 {
        inputs: function_inputs,
        var_message_size: FunctionInput::constant(
            (function_inputs_len - 1).into(),
            FieldElement::max_num_bits(),
        ),
        outputs: outputs.try_into().expect("Keccak256 returns 32 outputs"),
    }
}

// 25 inputs
// 25 outputs
fn keccakf1600_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> BlackBoxFuncCall<FieldElement> {
    let (function_inputs, mut outputs) = function_inputs_and_outputs;
    outputs.truncate(25);
    assert_eq!(function_inputs.len(), 25);
    assert_eq!(outputs.len(), 25);
    BlackBoxFuncCall::Keccakf1600 {
        inputs: function_inputs.try_into().expect("Keccakf1600 expects 25 inputs"),
        outputs: outputs.try_into().expect("Keccakf1600 returns 25 outputs"),
    }
}

// N inputs
// N outputs
fn poseidon2_permutation_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> BlackBoxFuncCall<FieldElement> {
    let (inputs, outputs) = function_inputs_and_outputs;
    let len = inputs.len() as u32;
    BlackBoxFuncCall::Poseidon2Permutation {
        inputs,
        outputs,
        len,
    }
}

// N inputs
// N outputs
fn poseidon2_permutation_invalid_len_op(
    function_inputs_and_outputs: (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
) -> BlackBoxFuncCall<FieldElement> {
    let (inputs, outputs) = function_inputs_and_outputs;
    let len = (inputs.len() as u32) + 1;
    BlackBoxFuncCall::Poseidon2Permutation {
        inputs,
        outputs,
        len,
    }
}

// TODO: the following hash function has a "twist" on the above pattern
//
//
// /// Applies the SHA-256 compression function to the input message
// ///
// /// # Arguments
// ///
// /// * `inputs` - input message block
// /// * `hash_values` - state from the previous compression
// /// * `outputs` - result of the input compressed into 256 bits
// Sha256Compression {
//     /// 512 bits of the input message, represented by 16 u32s
//     inputs: Box<[FunctionInput<F>; 16]>,
//     /// Vector of 8 u32s used to compress the input
//     hash_values: Box<[FunctionInput<F>; 8]>,
//     /// Output of the compression, represented by 8 u32s
//     outputs: Box<[Witness; 8]>,
// },
//

fn into_repr_vec<T>(xs: T) -> Vec<ark_bn254::Fr>
where
    T: IntoIterator<Item = FieldElement>,
{
    xs.into_iter().map(|x| x.into_repr()).collect()
}

fn into_repr_mat<T, U>(xs: T) -> Vec<Vec<ark_bn254::Fr>>
where
    T: IntoIterator<Item = U>,
    U: IntoIterator<Item = FieldElement>,
{
    xs.into_iter().map(|x| into_repr_vec(x)).collect()
}

fn run_both_poseidon2_permutations(
    inputs: Vec<ConstantOrWitness>,
) -> (Vec<ark_bn254::Fr>, Vec<ark_bn254::Fr>) {
    let result =
        solve_array_input_blackbox_call(inputs.clone(), inputs.len(), poseidon2_permutation_op);

    let poseidon2_t = POSEIDON2_CONFIG.t as usize;
    let poseidon2_d = 5;
    let rounds_f = POSEIDON2_CONFIG.rounds_f as usize;
    let rounds_p = POSEIDON2_CONFIG.rounds_p as usize;
    let mat_internal_diag_m_1 = into_repr_vec(POSEIDON2_CONFIG.internal_matrix_diagonal);
    let mat_internal = vec![];
    let round_constants = into_repr_mat(POSEIDON2_CONFIG.round_constant);

    let external_poseidon2 =
        zkhash::poseidon2::poseidon2::Poseidon2::new(&Arc::new(Poseidon2Params::new(
            poseidon2_t,
            poseidon2_d,
            rounds_f,
            rounds_p,
            &mat_internal_diag_m_1,
            &mat_internal,
            &round_constants,
        )));

    let expected_result =
        external_poseidon2.permutation(&into_repr_vec(drop_use_constant(&inputs)));
    (into_repr_vec(result), expected_result)
}

fn function_input_from_option(
    witness: Witness,
    opt_constant: Option<FieldElement>,
) -> FunctionInput<FieldElement> {
    opt_constant
        .map(|constant| FunctionInput::constant(constant, FieldElement::max_num_bits()))
        .unwrap_or(FunctionInput::witness(witness, FieldElement::max_num_bits()))
}

fn and_op(x: Option<FieldElement>, y: Option<FieldElement>) -> BlackBoxFuncCall<FieldElement> {
    let lhs = function_input_from_option(Witness(1), x);
    let rhs = function_input_from_option(Witness(2), y);
    BlackBoxFuncCall::AND { lhs, rhs, output: Witness(3) }
}

fn xor_op(x: Option<FieldElement>, y: Option<FieldElement>) -> BlackBoxFuncCall<FieldElement> {
    let lhs = function_input_from_option(Witness(1), x);
    let rhs = function_input_from_option(Witness(2), y);
    BlackBoxFuncCall::XOR { lhs, rhs, output: Witness(3) }
}

fn prop_assert_commutative(
    op: impl Fn(Option<FieldElement>, Option<FieldElement>) -> BlackBoxFuncCall<FieldElement>,
    x: (FieldElement, bool),
    y: (FieldElement, bool),
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(&op, x, y), solve_blackbox_func_call(&op, y, x))
}

fn prop_assert_associative(
    op: impl Fn(Option<FieldElement>, Option<FieldElement>) -> BlackBoxFuncCall<FieldElement>,
    x: (FieldElement, bool),
    y: (FieldElement, bool),
    z: (FieldElement, bool),
    use_constant_xy: bool,
    use_constant_yz: bool,
) -> (FieldElement, FieldElement) {
    let f_xy = (solve_blackbox_func_call(&op, x, y), use_constant_xy);
    let f_f_xy_z = solve_blackbox_func_call(&op, f_xy, z);

    let f_yz = (solve_blackbox_func_call(&op, y, z), use_constant_yz);
    let f_x_f_yz = solve_blackbox_func_call(&op, x, f_yz);

    (f_f_xy_z, f_x_f_yz)
}

fn prop_assert_identity_l(
    op: impl Fn(Option<FieldElement>, Option<FieldElement>) -> BlackBoxFuncCall<FieldElement>,
    op_identity: (FieldElement, bool),
    x: (FieldElement, bool),
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(op, op_identity, x), x.0)
}

fn prop_assert_zero_l(
    op: impl Fn(Option<FieldElement>, Option<FieldElement>) -> BlackBoxFuncCall<FieldElement>,
    op_zero: (FieldElement, bool),
    x: (FieldElement, bool),
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(op, op_zero, x), FieldElement::zero())
}

// Test that varying one of the inputs produces a different result
//
// (is the op injective for the given inputs?, failure string)
fn prop_assert_injective<F>(
    inputs: Vec<ConstantOrWitness>,
    distinct_inputs: Vec<ConstantOrWitness>,
    num_outputs: usize,
    op: F,
) -> (bool, String)
where
    F: FnOnce((Vec<FunctionInput<FieldElement>>, Vec<Witness>)) -> BlackBoxFuncCall<FieldElement>
        + Clone,
{
    let equal_inputs = drop_use_constant_eq(&inputs, &distinct_inputs);
    let message = format!("not injective:\n{:?}\n{:?}", &inputs, &distinct_inputs);
    let outputs_not_equal = solve_array_input_blackbox_call(inputs, num_outputs, op.clone())
        != solve_array_input_blackbox_call(distinct_inputs, num_outputs, op);
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
    fn any_distinct_inputs(max_size: usize)
        (size_and_patch in any::<(usize, usize, usize)>()) // NOTE: local ambiguity when using (x: T)
        // (size in any::<usize>(),
        //  patch_location in any::<usize>(),
        //  patch_value in any::<usize>())
        (inputs_distinct_inputs in (proptest::collection::vec(any::<(u128, bool)>(), size_and_patch.0 % max_size), proptest::collection::vec(any::<(u128, bool)>(), size_and_patch.0 % max_size)), size_and_patch in Just(size_and_patch))
        -> (Vec<ConstantOrWitness>, Vec<ConstantOrWitness>) {
        let (_size, patch_location, patch_value) = size_and_patch;
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let inputs: Vec<_> = inputs.into_iter().map(|(x, use_constant)| {
            (FieldElement::from(x), use_constant)
        }).collect();
        let mut distinct_inputs: Vec<_> = distinct_inputs.into_iter().map(|(x, use_constant)| {
            (FieldElement::from(x), use_constant)
        }).collect();

        // if equivalent w/o use_constant, patch with the patch_value
        if drop_use_constant_eq(&inputs, &distinct_inputs) {
            let distinct_inputs_len = distinct_inputs.len();
            let positive_patch_value = std::cmp::max(patch_value, 1);
            if distinct_inputs_len != 0 {
                distinct_inputs[patch_location % distinct_inputs_len].0 += FieldElement::from(positive_patch_value)
            } else {
                distinct_inputs.push((FieldElement::from(0u128), true))
            }
        }

        (inputs, distinct_inputs)
    }
}

#[test]
fn poseidon2_permutation_zeroes() {
    let use_constants: [bool; 4] = [false; 4];
    let inputs: Vec<_> =
        [FieldElement::zero(); 4].into_iter().zip(use_constants).collect();
    let (result, expected_result) = run_both_poseidon2_permutations(inputs);

    let internal_expected_result = vec![
        field_from_hex("18DFB8DC9B82229CFF974EFEFC8DF78B1CE96D9D844236B496785C698BC6732E"),
        field_from_hex("095C230D1D37A246E8D2D5A63B165FE0FADE040D442F61E25F0590E5FB76F839"),
        field_from_hex("0BB9545846E1AFA4FA3C97414A60A20FC4949F537A68CCECA34C5CE71E28AA59"),
        field_from_hex("18A4F34C9C6F99335FF7638B82AEED9018026618358873C982BBDDE265B2ED6D"),
    ];

    assert_eq!(expected_result, into_repr_vec(internal_expected_result));
    assert_eq!(result, expected_result);
}

#[test]
fn sha256_zeros() {
    let results = solve_array_input_blackbox_call(vec![], 32, sha256_op);
    let expected_results: Vec<_> = vec![
        227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39, 174, 65,
        228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85,
    ]
    .into_iter()
    .map(|x: u128| FieldElement::from(x))
    .collect();
    assert_eq!(results, expected_results);
}

#[test]
fn blake2s_zeros() {
    let results = solve_array_input_blackbox_call(vec![], 32, blake2s_op);
    let expected_results: Vec<_> = vec![
        105, 33, 122, 48, 121, 144, 128, 148, 225, 17, 33, 208, 66, 53, 74, 124, 31, 85, 182, 72,
        44, 161, 165, 30, 27, 37, 13, 253, 30, 208, 238, 249,
    ]
    .into_iter()
    .map(|x: u128| FieldElement::from(x))
    .collect();
    assert_eq!(results, expected_results);
}

#[test]
fn blake3_zeros() {
    let results = solve_array_input_blackbox_call(vec![], 32, blake3_op);
    let expected_results: Vec<_> = vec![
        175, 19, 73, 185, 245, 249, 161, 166, 160, 64, 77, 234, 54, 220, 201, 73, 155, 203, 37,
        201, 173, 193, 18, 183, 204, 154, 147, 202, 228, 31, 50, 98,
    ]
    .into_iter()
    .map(|x: u128| FieldElement::from(x))
    .collect();
    assert_eq!(results, expected_results);
}

#[test]
fn keccak256_zeros() {
    let results = solve_array_input_blackbox_call(vec![], 32, keccak256_op);
    let expected_results: Vec<_> = vec![
        197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0, 182, 83,
        202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112,
    ]
    .into_iter()
    .map(|x: u128| FieldElement::from(x))
    .collect();
    assert_eq!(results, expected_results);
}

// TODO: internal error when calling Keccakf1600
#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn keccakf1600_zeros() {
    let _results = solve_array_input_blackbox_call(
        [(FieldElement::zero(), false); 25].into(),
        25,
        keccakf1600_op,
    );
    // TODO: re-enable once the above works
    // assert_eq!(results, vec![]);
}

// TODO: see keccak256_injective for proptest
#[test]
#[should_panic(expected = "not injective")]
fn keccak256_injective_regression() {
    let x = FieldElement::from(15850513564950279608551620425163945216u128);
    let y = FieldElement::from(56407972000892100756873729306750041856u128);
    assert!(x != y);
    let inputs = vec![(
        x, // 2⁸×61916068613087029720904767285796661,
        false,
    )];
    let distinct_inputs = vec![(
        y, // 2⁸×220343640628484768581538005104492351,
        false,
    )];

    let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, keccak256_op);
    assert!(result, "{}", message);
}

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
        prop_assert_eq!(solve_blackbox_func_call(and_op, x, x), x.0);
    }

    // test that XOR(x, x) == 0
    #[test]
    fn xor_self_zero(x in field_element()) {
        prop_assert_eq!(solve_blackbox_func_call(xor_op, x, x), FieldElement::zero());
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
        let (result, expected_result) = run_both_poseidon2_permutations(inputs);
        prop_assert_eq!(result, expected_result)
    }


    #[test]
    fn sha256_injective(inputs_distinct_inputs in any_distinct_inputs(32)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, sha256_op);
        prop_assert!(result, "{}", message);
    }

    #[test]
    fn blake2s_injective(inputs_distinct_inputs in any_distinct_inputs(32)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, blake2s_op);
        prop_assert!(result, "{}", message);
    }

    #[test]
    fn blake3_injective(inputs_distinct_inputs in any_distinct_inputs(32)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, blake3_op);
        prop_assert!(result, "{}", message);
    }

    // TODO: see keccak256_injective_regression for specific case
    #[test]
    #[should_panic(expected = "not injective")]
    fn keccak256_injective(inputs_distinct_inputs in any_distinct_inputs(32)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, keccak256_op);
        prop_assert!(result, "{}", message);
    }

    // TODO: doesn't fail with an error, returns constant output
    #[test]
    #[should_panic(expected = "Test failed: not injective")]
    fn keccak256_invalid_message_size_fails(inputs_distinct_inputs in any_distinct_inputs(32)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, keccak256_invalid_message_size_op);
        prop_assert!(result, "{}", message);
    }

    // TODO: internal error when calling Keccakf1600
    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn keccakf1600_injective(inputs_distinct_inputs in any_distinct_inputs(25)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (_result, _message) = prop_assert_injective(inputs, distinct_inputs, 25, keccakf1600_op);
        // prop_assert!(result, "{}", message);
    }

    // TODO: wrong failure message? The number of inputs appears to be correct
    #[test]
    #[should_panic(expected = "Failure(BlackBoxFunctionFailed(Poseidon2Permutation, \"the number of inputs does not match specified length. 0 != 1\"))")]
    fn poseidon2_permutation_invalid_size_fails(inputs_distinct_inputs in any_distinct_inputs(6)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 1, poseidon2_permutation_invalid_len_op);
        prop_assert!(result, "{}", message);
    }

}
