use std::collections::BTreeMap;

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
use brillig_vm::brillig::HeapValueType;

use proptest::arbitrary::any;
use proptest::prelude::*;
use proptest::result::maybe_ok;
use proptest::sample::select;

// Reenable these test cases once we move the brillig implementation of inversion down into the acvm stdlib.

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

// Solve the given BlackBoxFuncCall with witnesses: 1, 2 as x, y, resp.
#[cfg(test)]
fn solve_blackbox_func_call(
    blackbox_func_call: impl Fn(Option<FieldElement>, Option<FieldElement>) -> BlackBoxFuncCall<FieldElement>,
    x: (FieldElement, bool), // if false, use a Witness
    y: (FieldElement, bool), // if false, use a Witness
) -> FieldElement {
    let (x, x_constant) = x;
    let (y, y_constant) = y;

    let initial_witness = WitnessMap::from(BTreeMap::from_iter([
        (Witness(1), x),
        (Witness(2), y),
    ]));

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


fn allowed_bigint_moduli() -> Vec<Vec<u8>> {
    let bn254_fq: Vec<u8> = vec![0x47, 0xFD, 0x7C, 0xD8, 0x16, 0x8C, 0x20, 0x3C, 0x8d, 0xca, 0x71, 0x68, 0x91, 0x6a, 0x81, 0x97,
                                 0x5d, 0x58, 0x81, 0x81, 0xb6, 0x45, 0x50, 0xb8, 0x29, 0xa0, 0x31, 0xe1, 0x72, 0x4e, 0x64, 0x30];
    let bn254_fr: Vec<u8> = vec![1, 0, 0, 240, 147, 245, 225, 67, 145, 112, 185, 121, 72, 232, 51, 40, 93, 88, 129, 129, 182, 69, 80, 184, 41, 160, 49, 225, 114, 78, 100, 48];
    let secpk1_fr: Vec<u8> = vec![0x41, 0x41, 0x36, 0xD0, 0x8C, 0x5E, 0xD2, 0xBF, 0x3B, 0xA0, 0x48, 0xAF, 0xE6, 0xDC, 0xAE, 0xBA,
                                  0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let secpk1_fq: Vec<u8> = vec![0x2F, 0xFC, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                                  0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let secpr1_fq: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF];
    let secpr1_fr: Vec<u8> = vec![81, 37, 99, 252, 194, 202, 185, 243, 132, 158, 23, 167, 173, 250, 230, 188, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255];

    vec![
        bn254_fq,
        bn254_fr,
        secpk1_fr,
        secpk1_fq,
        secpr1_fq,
        secpr1_fr,
    ]
}


// TODO: cleanup new tests and rename this
#[test]
fn binary_operations() {

    // // special cases
    //         BlackBoxFuncCall::BigIntSub { .. } => BlackBoxFunc::BigIntSub,
    //         BlackBoxFuncCall::BigIntDiv { .. }
    //
    // // BigInt:
    // //
    // // pub struct BigIntSolver {
    // //     bigint_id_to_value: HashMap<u32, BigUint>,
    // //     bigint_id_to_modulus: HashMap<u32, BigUint>,
    // // }
    // //
    //         BlackBoxFuncCall::BigIntAdd { .. }
    //         BlackBoxFuncCall::BigIntMul { .. }
    //
    // // unique calling convention
    //         BlackBoxFuncCall::EmbeddedCurveAdd { .. }

    // TODO:
    // - commutativity
    // - associativity
    // - left/right identity

    // let x = FieldElement::from(1u128);
    // let y = FieldElement::from(3u128);




    let zero_constant = false;
    let modulus = &allowed_bigint_moduli()[0];

    let initial_witness = WitnessMap::from(BTreeMap::from_iter([
        (Witness(1), FieldElement::zero()),
    ]));

    let zero_function_input = if zero_constant {
        FunctionInput::constant(FieldElement::zero(), FieldElement::max_num_bits())
    } else {
        FunctionInput::witness(Witness(1), FieldElement::max_num_bits())

    };
    let zero: Vec<_> = modulus.clone().into_iter().map(|_| zero_function_input).collect();

    let bigint_from_op = BlackBoxFuncCall::BigIntFromLeBytes {
        inputs: zero,
        modulus: modulus.clone(),
        output: 0,
    };

    // BigIntToLeBytes {
    //     input: u32,
    //     outputs: Vec<Witness>,
    // },

    // let add_op = BlackBoxFuncCall::BigIntAdd {
    //     lhs: 0,
    //     rhs: 1,
    //     output: 2,
    // };

    let op = Opcode::BlackBoxFuncCall(bigint_from_op);
    let opcodes = vec![op];
    let unconstrained_functions = vec![];
    let mut acvm =
        ACVM::new(&StubbedBlackBoxSolver, &opcodes, initial_witness, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();
    
    dbg!(witness_map[&Witness(1)]);


}

fn function_input_from_option(witness: Witness, opt_constant: Option<FieldElement>) -> FunctionInput<FieldElement> {
    opt_constant
        .map(|constant| FunctionInput::constant(constant, FieldElement::max_num_bits()))
        .unwrap_or(FunctionInput::witness(witness, FieldElement::max_num_bits()))
}

fn and_op(x: Option<FieldElement>, y: Option<FieldElement>) -> BlackBoxFuncCall<FieldElement> {
    let lhs = function_input_from_option(Witness(1), x);
    let rhs = function_input_from_option(Witness(2), y);
    BlackBoxFuncCall::AND {
        lhs,
        rhs,
        output: Witness(3),
    }
}

fn xor_op(x: Option<FieldElement>, y: Option<FieldElement>) -> BlackBoxFuncCall<FieldElement> {
    let lhs = function_input_from_option(Witness(1), x);
    let rhs = function_input_from_option(Witness(2), y);
    BlackBoxFuncCall::XOR {
        lhs,
        rhs,
        output: Witness(3),
    }
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
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(&op, x, y), solve_blackbox_func_call(&op, y, x))
}

fn prop_assert_identity_l(
    op: impl Fn(Option<FieldElement>, Option<FieldElement>) -> BlackBoxFuncCall<FieldElement>,
    op_identity: (FieldElement, bool),
    x: (FieldElement, bool),
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(op, op_identity, x), x.0)
}

fn prop_assert_identity_r(
    op: impl Fn(Option<FieldElement>, Option<FieldElement>) -> BlackBoxFuncCall<FieldElement>,
    op_identity: (FieldElement, bool),
    x: (FieldElement, bool),
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(op, x, op_identity), x.0)
}

prop_compose! {
    // Use both `u128` and hex proptest strategies
    fn field_element()
        (u128_or_hex in maybe_ok(any::<u128>(), "[0-9a-f]{64}"),
         constant_input in any::<bool>())
        -> (FieldElement, bool)
    {
        match u128_or_hex {
            Ok(number) => (FieldElement::from(number), constant_input),
            Err(hex) => (FieldElement::from_hex(&hex).expect("should accept any 32 byte hex string"), constant_input),
        }
    }
}

fn field_element_ones() -> FieldElement {
    let exponent: FieldElement = (FieldElement::max_num_bits() as u128).into();
    FieldElement::from(2u128).pow(&exponent) - FieldElement::one()
}

fn bigint_solve_from_to_le_bytes(modulus: Vec<u8>, zero_constant: bool) -> WitnessMap<FieldElement> {
    let initial_witness_vec: Vec<_> = (1..2 + modulus.len())
        .map(|i| (Witness(i as u32), FieldElement::zero()))
        .collect();
    let output_witnesses: Vec<_> = initial_witness_vec
        .iter()
        .skip(1)
        .map(|(witness, _)| *witness)
        .collect();

    let initial_witness = WitnessMap::from(BTreeMap::from_iter(initial_witness_vec));
    let zero_function_input = if zero_constant {
        FunctionInput::constant(FieldElement::zero(), FieldElement::max_num_bits())
    } else {
        FunctionInput::witness(Witness(1), FieldElement::max_num_bits())

    };
    let zero: Vec<_> = modulus.iter().map(|_| zero_function_input).collect();

    let bigint_from_op = BlackBoxFuncCall::BigIntFromLeBytes {
        inputs: zero,
        modulus: modulus.clone(),
        output: 0,
    };
    let bigint_to_op = BlackBoxFuncCall::BigIntToLeBytes {
        input: 0,
        outputs: output_witnesses,
    };

    let bigint_from_op = Opcode::BlackBoxFuncCall(bigint_from_op);
    let bigint_to_op = Opcode::BlackBoxFuncCall(bigint_to_op);
    let opcodes = vec![bigint_from_op, bigint_to_op];
    let unconstrained_functions = vec![];
    let mut acvm =
        ACVM::new(&StubbedBlackBoxSolver, &opcodes, initial_witness, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();

    witness_map
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
    fn and_associative(x in field_element(), y in field_element()) {
        let (lhs, rhs) = prop_assert_associative(and_op, x, y);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn xor_associative(x in field_element(), y in field_element()) {
        let (lhs, rhs) = prop_assert_associative(xor_op, x, y);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn and_identity_l(x in field_element(), ones_constant in any::<bool>()) {
        let ones = (field_element_ones(), ones_constant);
        let (lhs, rhs) = prop_assert_identity_l(and_op, ones, x);
        if x <= ones {
            prop_assert_eq!(lhs, rhs);
        } else {
            // TODO
            prop_assert!(lhs != rhs);
        }
    }

    #[test]
    fn xor_identity_l(x in field_element(), zero_constant in any::<bool>()) {
        let zero = (FieldElement::zero(), zero_constant);
        let (lhs, rhs) = prop_assert_identity_l(xor_op, zero, x);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn and_identity_r(x in field_element(), ones_constant in any::<bool>()) {
        let ones = (field_element_ones(), ones_constant);
        let (lhs, rhs) = prop_assert_identity_r(and_op, ones, x);
        if x <= ones {
            prop_assert_eq!(lhs, rhs);
        } else {
            // TODO
            prop_assert!(lhs != rhs);
        }
    }

    #[test]
    fn xor_identity_r(x in field_element(), zero_constant in any::<bool>()) {
        let zero = (FieldElement::zero(), zero_constant);
        let (lhs, rhs) = prop_assert_identity_r(xor_op, zero, x);
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn bigint_from_to_le_bytes(modulus in select(allowed_bigint_moduli()), zero_constant in any::<bool>()) {
        let modulus_len = modulus.len();
        let witness_map = bigint_solve_from_to_le_bytes(modulus.clone(), zero_constant);
        for i in 1..2 + modulus_len {
            prop_assert_eq!(witness_map.get(&Witness(i as u32)).cloned(), Some(FieldElement::zero()));
        }
    }

    #[test]
    // TODO: this test attempts to use a guaranteed-invalid BigInt modulus
    #[should_panic(expected = "attempt to add with overflow")]
    fn bigint_from_to_le_bytes_disallowed_modulus(modulus in select(allowed_bigint_moduli()), patch_location: usize, patch_amount: u8, zero_constant in any::<bool>()) {
        let patch_location = patch_location % modulus.len();
        let patch_amount = patch_amount.clamp(1, u8::MAX);

        let mut modulus = modulus;
        modulus[patch_location] += patch_amount;
        let modulus_len = modulus.len();

        let witness_map = bigint_solve_from_to_le_bytes(modulus, zero_constant);
        for i in 1..2 + modulus_len {
            prop_assert_eq!(witness_map.get(&Witness(i as u32)).cloned(), Some(FieldElement::zero()));
        }
    }

}


