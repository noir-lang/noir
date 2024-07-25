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

prop_compose! {
    fn bigint_with_modulus()(modulus in select(allowed_bigint_moduli()))
        (input in proptest::collection::vec(any::<(u8, bool)>(), modulus.len()), modulus in Just(modulus))
        -> (Vec<ConstantOrWitness>, Vec<u8>) {
        let input = input.into_iter().map(|(x, use_constant)| {
            (FieldElement::from(x as u128), use_constant)
        }).collect();
        (input, modulus)
    }
}

prop_compose! {
    fn bigint_pair_with_modulus()(input_modulus in bigint_with_modulus())
        (ys in proptest::collection::vec(any::<(u8, bool)>(), input_modulus.1.len()), input_modulus in Just(input_modulus))
        -> (Vec<ConstantOrWitness>, Vec<ConstantOrWitness>, Vec<u8>) {
        let ys = ys.into_iter().map(|(x, use_constant)| {
            (FieldElement::from(x as u128), use_constant)
        }).collect();
        (input_modulus.0, ys, input_modulus.1)
    }
}

prop_compose! {
    fn bigint_triple_with_modulus()(xs_ys_modulus in bigint_pair_with_modulus())
        (zs in proptest::collection::vec(any::<(u8, bool)>(), xs_ys_modulus.2.len()), xs_ys_modulus in Just(xs_ys_modulus))
        -> (Vec<ConstantOrWitness>, Vec<ConstantOrWitness>, Vec<ConstantOrWitness>, Vec<u8>) {
        let zs = zs.into_iter().map(|(x, use_constant)| {
            (FieldElement::from(x as u128), use_constant)
        }).collect();
        (xs_ys_modulus.0, xs_ys_modulus.1, zs, xs_ys_modulus.2)
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
// Output is a zeroed BigInt with the same byte-length and use_constant values
// as the input.
fn bigint_zeroed(input: &[ConstantOrWitness]) -> Vec<ConstantOrWitness> {
    input.iter().map(|(_, use_constant)| (FieldElement::zero(), *use_constant)).collect()
}

// bigint_zeroed, but returns one
fn bigint_to_one(input: &[ConstantOrWitness]) -> Vec<ConstantOrWitness> {
    let mut one = bigint_zeroed(input);
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
    xs: Vec<ConstantOrWitness>,
    ys: Vec<ConstantOrWitness>,
) -> Vec<FieldElement> {
    let initial_witness_vec: Vec<_> =
        xs.iter().chain(ys.iter()).enumerate().map(|(i, (x, _))| (Witness(i as u32), *x)).collect();
    let output_witnesses: Vec<_> = initial_witness_vec
        .iter()
        .take(xs.len())
        .enumerate()
        .map(|(i, _)| Witness((i + 2 * xs.len()) as u32)) // offset past the indices of xs, ys
        .collect();
    let initial_witness = WitnessMap::from(BTreeMap::from_iter(initial_witness_vec));

    let xs = constant_or_witness_to_function_inputs(xs, 0);
    let ys = constant_or_witness_to_function_inputs(ys, xs.len());

    let to_op_input = if middle_op.is_some() { 2 } else { 0 };

    let bigint_from_x_op = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::BigIntFromLeBytes {
        inputs: xs,
        modulus: modulus.clone(),
        output: 0,
    });
    let bigint_from_y_op = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::BigIntFromLeBytes {
        inputs: ys,
        modulus: modulus.clone(),
        output: 1,
    });
    let bigint_to_op = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::BigIntToLeBytes {
        input: to_op_input,
        outputs: output_witnesses.clone(),
    });

    let mut opcodes = vec![bigint_from_x_op, bigint_from_y_op];
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

    output_witnesses
        .iter()
        .map(|witness| *witness_map.get(witness).expect("all witnesses to be set"))
        .collect()
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
    xs: Vec<ConstantOrWitness>,
    ys: Vec<ConstantOrWitness>,
) -> Vec<FieldElement> {
    bigint_solve_binary_op_opt(Some(middle_op), modulus, xs, ys)
}

// Using the given BigInt modulus, solve the following circuit:
// - Convert the input to a BigInt with ID 0
// - Run BigIntToLeBytes on BigInt ID 0
// - Output the resulting Vec of LE bytes
fn bigint_solve_from_to_le_bytes(
    modulus: Vec<u8>,
    input: Vec<ConstantOrWitness>,
) -> Vec<FieldElement> {
    bigint_solve_binary_op_opt(None, modulus, input, vec![])
}

// NOTE: an "average" bigint is large, so consider increasing the number of proptest shrinking
// iterations (from the default 1024) to reach a simplified case, e.g.
// PROPTEST_MAX_SHRINK_ITERS=1024000
proptest! {
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
    #[should_panic(expected = "attempt to add with overflow")]
    fn bigint_from_to_le_bytes_disallowed_modulus(modulus in select(allowed_bigint_moduli()), patch_location: usize, patch_amount: u8, zero_or_ones_constant: bool, use_constant: bool) {
        let patch_location = patch_location % modulus.len();
        let patch_amount = patch_amount.clamp(1, u8::MAX);
        let mut modulus = modulus;
        modulus[patch_location] += patch_amount;

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
        let xs_ys: Vec<_> = op_xs_ys.into_iter().map(|x| (x, false)).collect();
        let op_xs_ys_op_zs = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), xs_ys, zs.clone());

        // f(xs, f(ys, zs))
        let op_ys_zs = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), ys.clone(), zs.clone());
        let ys_zs: Vec<_> = op_ys_zs.into_iter().map(|x| (x, false)).collect();
        let op_xs_op_ys_zs = bigint_solve_binary_op(bigint_add_op(), modulus, xs, ys_zs);

        prop_assert_eq!(op_xs_ys_op_zs, op_xs_op_ys_zs)
    }

    #[test]
    fn bigint_mul_associative((xs, ys, zs, modulus) in bigint_triple_with_modulus()) {
        // f(f(xs, ys), zs) ==
        let op_xs_ys = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs.clone(), ys.clone());
        let xs_ys: Vec<_> = op_xs_ys.into_iter().map(|x| (x, false)).collect();
        let op_xs_ys_op_zs = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs_ys, zs.clone());

        // f(xs, f(ys, zs))
        let op_ys_zs = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), ys.clone(), zs.clone());
        let ys_zs: Vec<_> = op_ys_zs.into_iter().map(|x| (x, false)).collect();
        let op_xs_op_ys_zs = bigint_solve_binary_op(bigint_mul_op(), modulus, xs, ys_zs);

        prop_assert_eq!(op_xs_ys_op_zs, op_xs_op_ys_zs)
    }

    #[test]
    fn bigint_mul_add_distributive((xs, ys, zs, modulus) in bigint_triple_with_modulus()) {
        // xs * (ys + zs) ==
        let add_ys_zs = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), ys.clone(), zs.clone());
        let add_ys_zs: Vec<_> = add_ys_zs.into_iter().map(|x| (x, false)).collect();
        let mul_xs_add_ys_zs = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs.clone(), add_ys_zs);

        // xs * ys + xs * zs
        let mul_xs_ys = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs.clone(), ys);
        let mul_xs_ys: Vec<_> = mul_xs_ys.into_iter().map(|x| (x, false)).collect();
        let mul_xs_zs = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs, zs);
        let mul_xs_zs: Vec<_> = mul_xs_zs.into_iter().map(|x| (x, false)).collect();
        let add_mul_xs_ys_mul_xs_zs = bigint_solve_binary_op(bigint_add_op(), modulus, mul_xs_ys, mul_xs_zs);

        prop_assert_eq!(mul_xs_add_ys_zs, add_mul_xs_ys_mul_xs_zs)
    }


    // TODO(https://github.com/noir-lang/noir/issues/5579): Fails on 49, see bigint_add_zero_l_single_case_49
    #[test]
    #[should_panic(expected = "Test failed: assertion failed: `(left == right)`")]
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

    // TODO(https://github.com/noir-lang/noir/issues/5579): Fails on 49, see bigint_add_zero_l_single_case_49
    #[test]
    #[should_panic(expected = "Test failed: assertion failed: `(left == right)`")]
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

    // TODO(https://github.com/noir-lang/noir/issues/5579): Fails on 49, see bigint_add_zero_l_single_case_49
    #[test]
    #[should_panic(expected = "Test failed: assertion failed: `(left == right)`")]
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
    fn bigint_div_by_zero((xs, modulus) in bigint_with_modulus()) {
        let zero = bigint_zeroed(&xs);
        let expected_results = drop_use_constant(&zero);
        let results = bigint_solve_binary_op(bigint_div_op(), modulus, xs, zero);
        prop_assert_eq!(results, expected_results)
    }

    // TODO(https://github.com/noir-lang/noir/issues/5579): Fails on 49, see bigint_add_zero_l_single_case_49
    #[test]
    #[should_panic(expected = "Test failed: assertion failed: `(left == right)`")]
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

    // TODO(https://github.com/noir-lang/noir/issues/5579): fails on (x=0, y=97)
    #[test]
    #[should_panic(expected = "Test failed: assertion failed: `(left == right)")]
    fn bigint_add_sub((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let expected_results = drop_use_constant(&xs);
        let add_results = bigint_solve_binary_op(bigint_add_op(), modulus.clone(), xs, ys.clone());
        let add_bigint: Vec<_> = add_results.into_iter().map(|x| (x, false)).collect();
        let results = bigint_solve_binary_op(bigint_sub_op(), modulus, add_bigint, ys);

        prop_assert_eq!(results, expected_results)
    }

    // TODO(https://github.com/noir-lang/noir/issues/5579)
    #[test]
    #[should_panic(expected = "Test failed: assertion failed: `(left == right)")]
    fn bigint_sub_add((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let expected_results = drop_use_constant(&xs);
        let sub_results = bigint_solve_binary_op(bigint_sub_op(), modulus.clone(), xs, ys.clone());
        let add_bigint: Vec<_> = sub_results.into_iter().map(|x| (x, false)).collect();
        let results = bigint_solve_binary_op(bigint_add_op(), modulus, add_bigint, ys);

        prop_assert_eq!(results, expected_results)
    }

    // TODO(https://github.com/noir-lang/noir/issues/5579): Fails on 49, see bigint_add_zero_l_single_case_49
    #[test]
    #[should_panic(expected = "Test failed: assertion failed: `(left == right)`")]
    fn bigint_div_mul((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let expected_results = drop_use_constant(&xs);
        let div_results = bigint_solve_binary_op(bigint_div_op(), modulus.clone(), xs, ys.clone());
        let div_bigint: Vec<_> = div_results.into_iter().map(|x| (x, false)).collect();
        let results = bigint_solve_binary_op(bigint_mul_op(), modulus, div_bigint, ys);

        prop_assert_eq!(results, expected_results)
    }

    // TODO(https://github.com/noir-lang/noir/issues/5579): Fails on 49, see bigint_add_zero_l_single_case_49
    #[test]
    #[should_panic(expected = "Test failed: assertion failed: `(left == right)`")]
    fn bigint_mul_div((xs, ys, modulus) in bigint_pair_with_modulus()) {
        let expected_results = drop_use_constant(&xs);
        let mul_results = bigint_solve_binary_op(bigint_mul_op(), modulus.clone(), xs, ys.clone());
        let mul_bigint: Vec<_> = mul_results.into_iter().map(|x| (x, false)).collect();
        let results = bigint_solve_binary_op(bigint_div_op(), modulus, mul_bigint, ys);

        prop_assert_eq!(results, expected_results)
    }
}
