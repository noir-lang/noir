use std::collections::BTreeMap;

use acir::acir_field::GenericFieldElement;
use acir::brillig::{BitSize, HeapVector, IntegerBitSize};
use acir::{
    AcirField, FieldElement,
    brillig::{BinaryFieldOp, MemoryAddress, Opcode as BrilligOpcode, ValueOrArray},
    circuit::{
        Opcode, OpcodeLocation,
        brillig::{BrilligBytecode, BrilligFunctionId},
        opcodes::{BlackBoxFuncCall, FunctionInput},
    },
    native_types::{Expression, Witness, WitnessMap},
};
use acir::{InvalidInputBitSize, parse_opcodes};

use acvm::pwg::{ACVM, ACVMStatus, ErrorLocation, ForeignCallWaitInfo, OpcodeResolutionError};
use acvm_blackbox_solver::StubbedBlackBoxSolver;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use brillig_vm::brillig::HeapValueType;

use num_bigint::BigUint;
use proptest::arbitrary::any;
use proptest::prelude::*;
use proptest::result::maybe_ok;

#[test]
fn bls12_381_circuit() {
    let solver = StubbedBlackBoxSolver::default();
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

    let mut acvm = ACVM::new(&solver, &opcodes, witness_assignments, &[], &[]);
    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    let witness_stack = acvm.finalize();

    assert_eq!(witness_stack.get(&Witness(3)).unwrap(), &Bls12FieldElement::from(5u128));
}

#[test]
fn inversion_brillig_oracle_equivalence() {
    let solver = StubbedBlackBoxSolver::default();
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     let z = x + y;
    //     assert( 1/z == Oracle("inverse", x + y) );
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_z = Witness(4);
    let w_z_inverse = Witness(5);
    let w_x_plus_y = Witness(6);
    let w_equal_res = Witness(7);

    let src = format!(
        "
    BRILLIG CALL func: 0, inputs: [{w_x} + {w_y}, 0], outputs: [{w_x_plus_y}, {w_oracle}, {w_equal_res}]
    ASSERT {w_z} = {w_x} + {w_y}
    ASSERT 0 = {w_z}*{w_z_inverse} - 1
    ASSERT {w_z_inverse} = {w_oracle}
    "
    );
    let opcodes = parse_opcodes(&src).unwrap();

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
        function_name: "invert".to_string(),
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
    let mut acvm = ACVM::new(&solver, &opcodes, witness_assignments, &unconstrained_functions, &[]);
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
    let solver = StubbedBlackBoxSolver::default();
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     let z = x + y;
    //     let ij = i + j;
    //     assert( 1/z == Oracle("inverse", x + y) );
    //     assert( 1/ij == Oracle("inverse", i + j) );
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
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

    let src = format!(
        "
    BRILLIG CALL func: 0, inputs: [{w_x} + {w_y}, 0, {w_i} + {w_j}], outputs: [{w_x_plus_y}, {w_oracle}, {w_i_plus_j}, {w_ij_oracle}, {w_equal_res}]
    ASSERT {w_z} = {w_x} + {w_y}
    ASSERT 0 = {w_z}*{w_z_inverse} - 1
    ASSERT {w_z_inverse} = {w_oracle}
    "
    );
    let opcodes = parse_opcodes(&src).unwrap();

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
        function_name: "double_inversion".to_string(),
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
    let mut acvm = ACVM::new(&solver, &opcodes, witness_assignments, &unconstrained_functions, &[]);

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
    let solver = StubbedBlackBoxSolver::default();
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
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_x_inv = Witness(3);
    let w_y_inv = Witness(4);

    let zero_usize = MemoryAddress::direct(4);
    let three_usize = MemoryAddress::direct(5);
    let four_usize = MemoryAddress::direct(6);

    let brillig_bytecode = BrilligBytecode {
        function_name: "double_inverse".to_string(),
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

    let src = format!(
        "
    // This equality check can be executed immediately before resolving any foreign calls.
    ASSERT {w_y} = {w_x}
    BRILLIG CALL func: 0, inputs: [{w_x}, 0, {w_y}], outputs: [{w_x}, {w_y_inv}, {w_y}, {w_y_inv}]
    // This equality check relies on the outputs of the Brillig call.
    // It then cannot be solved until the foreign calls are resolved.
    ASSERT {w_y_inv} = {w_x_inv}
    "
    );
    let opcodes = parse_opcodes(&src).unwrap();

    let witness_assignments =
        BTreeMap::from([(w_x, FieldElement::from(2u128)), (w_y, FieldElement::from(2u128))]).into();
    let unconstrained_functions = vec![brillig_bytecode];
    let mut acvm = ACVM::new(&solver, &opcodes, witness_assignments, &unconstrained_functions, &[]);

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
    let solver = StubbedBlackBoxSolver::default();
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
        function_name: "inverse".to_string(),
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

    let src = format!(
        "
    BRILLIG CALL func: 0, predicate: 0, inputs: [{w_x} + {w_y}, 0], outputs: [{w_x_plus_y}, {w_oracle}, {w_equal_res}, {w_lt_res}]
    "
    );
    let opcodes = parse_opcodes(&src).unwrap();

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
    ])
    .into();
    let unconstrained_functions = vec![brillig_bytecode];
    let mut acvm = ACVM::new(&solver, &opcodes, witness_assignments, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    acvm.finalize();
}

#[test]
fn unsatisfied_opcode_resolved() {
    let solver = StubbedBlackBoxSolver::default();
    let a = Witness(0);
    let b = Witness(1);
    let c = Witness(2);
    let d = Witness(3);

    let mut values = WitnessMap::new();
    values.insert(a, FieldElement::from(4_i128));
    values.insert(b, FieldElement::from(2_i128));
    values.insert(c, FieldElement::from(1_i128));
    values.insert(d, FieldElement::from(2_i128));

    let src = format!("ASSERT {a} = {b} + {c} + {d}");
    let opcodes = parse_opcodes(&src).unwrap();

    let unconstrained_functions = vec![];
    let mut acvm = ACVM::new(&solver, &opcodes, values, &unconstrained_functions, &[]);
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
    let solver = StubbedBlackBoxSolver::default();
    let a = Witness(0);
    let b = Witness(1);
    let c = Witness(2);
    let d = Witness(3);

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
        function_name: "equality_check".to_string(),
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

    let mut values = WitnessMap::new();
    values.insert(a, FieldElement::from(4_i128));
    values.insert(b, FieldElement::from(2_i128));
    values.insert(c, FieldElement::from(1_i128));
    values.insert(d, FieldElement::from(2_i128));
    values.insert(w_x, FieldElement::from(0_i128));
    values.insert(w_y, FieldElement::from(1_i128));
    values.insert(w_result, FieldElement::from(0_i128));

    let src = format!(
        "
    BRILLIG CALL func: 0, predicate: 1, inputs: [{w_x}, {w_y}], outputs: [{w_result}]
    ASSERT {a} = {b} + {c} + {d}
    "
    );
    let opcodes = parse_opcodes(&src).unwrap();

    let unconstrained_functions = vec![brillig_bytecode];
    let mut acvm = ACVM::new(&solver, &opcodes, values, &unconstrained_functions, &[]);
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
    let solver = StubbedBlackBoxSolver::default();

    let initial_witness = WitnessMap::from(BTreeMap::from_iter([
        (Witness(1), FieldElement::from(1u128)),
        (Witness(2), FieldElement::from(2u128)),
        (Witness(3), FieldElement::from(3u128)),
        (Witness(4), FieldElement::from(4u128)),
        (Witness(5), FieldElement::from(5u128)),
        (Witness(6), FieldElement::from(4u128)),
    ]));

    let src = "
    INIT b0 = [w1, w2, w3, w4, w5]
    READ w7 = b0[w6]
    ASSERT w8 = w7 + 1
    ";
    let opcodes = parse_opcodes(src).unwrap();

    let unconstrained_functions = vec![];
    let mut acvm = ACVM::new(&solver, &opcodes, initial_witness, &unconstrained_functions, &[]);
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
// - If use_constant, then convert to a FunctionInput::Constant
// - Otherwise, convert to FunctionInput::Witness
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
                if x.num_bits() > num_bits {
                    return Err(OpcodeResolutionError::InvalidInputBitSize {
                        opcode_location: ErrorLocation::Unresolved,
                        invalid_input_bit_size: InvalidInputBitSize {
                            value: x.to_string(),
                            value_num_bits: x.num_bits(),
                            max_bits: num_bits,
                        },
                    });
                }
                Ok(FunctionInput::Constant(x))
            } else {
                Ok(FunctionInput::Witness(Witness((i + offset) as u32)))
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
    num_bits: Option<u32>,
    pedantic_solving: bool,
    f: F,
) -> Result<Vec<FieldElement>, OpcodeResolutionError<FieldElement>>
where
    F: FnOnce(
        (Vec<FunctionInput<FieldElement>>, Vec<Witness>),
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
{
    let solver = Bn254BlackBoxSolver(pedantic_solving);
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
    let mut acvm = ACVM::new(&solver, &opcodes, initial_witness, &unconstrained_functions, &[]);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();

    Ok(outputs
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
        Option<u32>,
    ) -> Result<
        BlackBoxFuncCall<FieldElement>,
        OpcodeResolutionError<FieldElement>,
    >,
    lhs: (FieldElement, bool), // if false, use a Witness
    rhs: (FieldElement, bool), // if false, use a Witness
    num_bits: Option<u32>,
) -> Result<FieldElement, OpcodeResolutionError<FieldElement>> {
    let solver = StubbedBlackBoxSolver::default();
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

    let op = Opcode::BlackBoxFuncCall(blackbox_func_call(lhs_opt, rhs_opt, num_bits)?);
    let opcodes = vec![op];
    let unconstrained_functions = vec![];
    let mut acvm = ACVM::new(&solver, &opcodes, initial_witness, &unconstrained_functions, &[]);
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
    Ok(BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs })
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

// fn into_repr_mat<T, U>(fields: T) -> Vec<Vec<ark_bn254::Fr>>
// where
//     T: IntoIterator<Item = U>,
//     U: IntoIterator<Item = FieldElement>,
// {
//     fields.into_iter().map(|field| into_repr_vec(field)).collect()
// }

fn function_input_from_option(
    witness: Witness,
    opt_constant: Option<FieldElement>,
) -> Result<FunctionInput<FieldElement>, OpcodeResolutionError<FieldElement>> {
    opt_constant
        .map(|constant| Ok(FunctionInput::Constant(constant)))
        .unwrap_or(Ok(FunctionInput::Witness(witness)))
}

fn and_op(
    x: Option<FieldElement>,
    y: Option<FieldElement>,
    num_bits: Option<u32>,
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let lhs = function_input_from_option(Witness(1), x)?;
    let rhs = function_input_from_option(Witness(2), y)?;
    Ok(BlackBoxFuncCall::AND { lhs, rhs, num_bits: num_bits.unwrap(), output: Witness(3) })
}

fn xor_op(
    x: Option<FieldElement>,
    y: Option<FieldElement>,
    num_bits: Option<u32>,
) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>> {
    let lhs = function_input_from_option(Witness(1), x)?;
    let rhs = function_input_from_option(Witness(2), y)?;
    Ok(BlackBoxFuncCall::XOR { lhs, rhs, num_bits: num_bits.unwrap(), output: Witness(3) })
}

fn prop_assert_commutative(
    op: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
        Option<u32>,
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
    x: (FieldElement, bool),
    y: (FieldElement, bool),
    num_bits: Option<u32>,
) -> (FieldElement, FieldElement) {
    (
        solve_blackbox_func_call(&op, x, y, num_bits).unwrap(),
        solve_blackbox_func_call(&op, y, x, num_bits).unwrap(),
    )
}

fn prop_assert_associative(
    op: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
        Option<u32>,
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
    x: (FieldElement, bool),
    y: (FieldElement, bool),
    z: (FieldElement, bool),
    use_constant_xy: bool,
    use_constant_yz: bool,
    num_bits: Option<u32>,
) -> (FieldElement, FieldElement) {
    let f_xy = (solve_blackbox_func_call(&op, x, y, num_bits).unwrap(), use_constant_xy);
    let f_f_xy_z = solve_blackbox_func_call(&op, f_xy, z, num_bits).unwrap();

    let f_yz = (solve_blackbox_func_call(&op, y, z, num_bits).unwrap(), use_constant_yz);
    let f_x_f_yz = solve_blackbox_func_call(&op, x, f_yz, num_bits).unwrap();

    (f_f_xy_z, f_x_f_yz)
}

fn prop_assert_identity_l(
    op: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
        Option<u32>,
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
    op_identity: (FieldElement, bool),
    x: (FieldElement, bool),
    num_bits: Option<u32>,
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(op, op_identity, x, num_bits).unwrap(), x.0)
}

fn prop_assert_zero_l(
    op: impl Fn(
        Option<FieldElement>,
        Option<FieldElement>,
        Option<u32>,
    ) -> Result<BlackBoxFuncCall<FieldElement>, OpcodeResolutionError<FieldElement>>,
    op_zero: (FieldElement, bool),
    x: (FieldElement, bool),
    num_bits: Option<u32>,
) -> (FieldElement, FieldElement) {
    (solve_blackbox_func_call(op, op_zero, x, num_bits).unwrap(), FieldElement::zero())
}

// Test that varying one of the inputs produces a different result
//
// (is the op injective for the given inputs?, failure string)
fn prop_assert_injective<F>(
    inputs: Vec<ConstantOrWitness>,
    distinct_inputs: Vec<ConstantOrWitness>,
    num_outputs: usize,
    num_bits: Option<u32>,
    pedantic_solving: bool,
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
    let outputs_not_equal = solve_array_input_blackbox_call(
        inputs,
        num_outputs,
        num_bits,
        pedantic_solving,
        op.clone(),
    )
    .expect("injectivity test operations to have valid input")
        != solve_array_input_blackbox_call(
            distinct_inputs,
            num_outputs,
            num_bits,
            pedantic_solving,
            op,
        )
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
fn sha256_compression_zeros() {
    let pedantic_solving = true;
    let results = solve_array_input_blackbox_call(
        [(FieldElement::zero(), false); 24].into(),
        8,
        None,
        pedantic_solving,
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
    let pedantic_solving = true;
    let results = solve_array_input_blackbox_call(vec![], 32, None, pedantic_solving, blake2s_op);
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
    let pedantic_solving = true;
    let results = solve_array_input_blackbox_call(vec![], 32, None, pedantic_solving, blake3_op);
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
    let pedantic_solving = true;
    let results = solve_array_input_blackbox_call(
        [(FieldElement::zero(), false); 25].into(),
        25,
        Some(64),
        pedantic_solving,
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
        let  max_num_bits = if y.0.num_bits() > x.0.num_bits() {
            y.0.num_bits()
        } else {
            x.0.num_bits()
        };
        let (lhs, rhs) = prop_assert_commutative(and_op, x, y, Some(max_num_bits));
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn xor_commutative(x in field_element(), y in field_element()) {
        let  max_num_bits = if y.0.num_bits() > x.0.num_bits() {
            y.0.num_bits()
        } else {
            x.0.num_bits()
        };
        let (lhs, rhs) = prop_assert_commutative(xor_op, x, y,Some(max_num_bits));
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn and_associative(x in field_element(), y in field_element(), z in field_element(), use_constant_xy: bool, use_constant_yz: bool) {
        let mut num_bits = if y.0.num_bits() > x.0.num_bits() {
            y.0.num_bits()
        } else {
            x.0.num_bits()
        };
        if num_bits < z.0.num_bits() {
            num_bits = z.0.num_bits();
        }
        let (lhs, rhs) = prop_assert_associative(and_op, x, y, z, use_constant_xy, use_constant_yz, Some(num_bits));
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    // TODO(https://github.com/noir-lang/noir/issues/5638)
    #[should_panic(expected = "assertion `left == right` failed")]
    fn xor_associative(x in field_element(), y in field_element(), z in field_element(), use_constant_xy: bool, use_constant_yz: bool) {
        let max_num_bits = if y.0.num_bits() > x.0.num_bits() {
            y.0.num_bits()
        } else {
            x.0.num_bits()
        };
        let (lhs, rhs) = prop_assert_associative(xor_op, x, y, z, use_constant_xy, use_constant_yz, Some(max_num_bits));
        prop_assert_eq!(lhs, rhs);
    }

    // test that AND(x, x) == x
    #[test]
    fn and_self_identity(x in field_element()) {
        prop_assert_eq!(solve_blackbox_func_call(and_op, x, x, Some(x.0.num_bits())).unwrap(), x.0);
    }

    // test that XOR(x, x) == 0
    #[test]
    fn xor_self_zero(x in field_element()) {
        prop_assert_eq!(solve_blackbox_func_call(xor_op, x, x, Some(x.0.num_bits())).unwrap(), FieldElement::zero());
    }

    #[test]
    fn and_identity_l(x in field_element(), ones_constant: bool) {
        let ones = (field_element_ones(), ones_constant);
        let max_num_bits = if x.0.num_bits() > ones.0.num_bits() {
            x.0.num_bits()
        } else {
            ones.0.num_bits()
        };
        let (lhs, rhs) = prop_assert_identity_l(and_op, ones, x, Some(max_num_bits));
        if x <= ones {
            prop_assert_eq!(lhs, rhs);
        } else {
            prop_assert!(lhs != rhs);
        }
    }

    #[test]
    fn xor_identity_l(x in field_element(), zero_constant: bool) {
        let zero = (FieldElement::zero(), zero_constant);
        let (lhs, rhs) = prop_assert_identity_l(xor_op, zero, x, Some(x.0.num_bits()));
        prop_assert_eq!(lhs, rhs);
    }

    #[test]
    fn and_zero_l(x in field_element(), ones_constant: bool) {
        let zero = (FieldElement::zero(), ones_constant);
        let (lhs, rhs) = prop_assert_zero_l(and_op, zero, x, Some(x.0.num_bits()));
        prop_assert_eq!(lhs, rhs);
    }




    #[test]
    fn sha256_compression_injective(inputs_distinct_inputs in any_distinct_inputs(None, 24, 24)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        if inputs.len() == 24 && distinct_inputs.len() == 24 {
            let pedantic_solving = true;
            let (result, message) = prop_assert_injective(inputs, distinct_inputs, 8, None, pedantic_solving, sha256_compression_op);
            prop_assert!(result, "{}", message);
        }
    }

    #[test]
    fn blake2s_injective(inputs_distinct_inputs in any_distinct_inputs(None, 0, 32)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let pedantic_solving = true;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, None, pedantic_solving, blake2s_op);
        prop_assert!(result, "{}", message);
    }

    #[test]
    fn blake3_injective(inputs_distinct_inputs in any_distinct_inputs(None, 0, 32)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let pedantic_solving = true;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 32, None, pedantic_solving, blake3_op);
        prop_assert!(result, "{}", message);
    }

    #[test]
    fn keccakf1600_injective(inputs_distinct_inputs in any_distinct_inputs(Some(8), 25, 25)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        assert_eq!(inputs.len(), 25);
        assert_eq!(distinct_inputs.len(), 25);
        let pedantic_solving = true;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 25, Some(64), pedantic_solving, keccakf1600_op);
        prop_assert!(result, "{}", message);
    }

    // TODO(https://github.com/noir-lang/noir/issues/5699): wrong failure message
    #[test]
    #[should_panic(expected = "Failure(BlackBoxFunctionFailed(Poseidon2Permutation, \"the input and output sizes are not consistent. 6 != 1\"))")]
    fn poseidon2_permutation_invalid_size_fails(inputs_distinct_inputs in any_distinct_inputs(None, 6, 6)) {
        let (inputs, distinct_inputs) = inputs_distinct_inputs;
        let pedantic_solving = true;
        let (result, message) = prop_assert_injective(inputs, distinct_inputs, 1, None, pedantic_solving, poseidon2_permutation_op);
        prop_assert!(result, "{}", message);
    }

}
