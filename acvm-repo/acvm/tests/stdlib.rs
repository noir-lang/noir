#![cfg(feature = "testing")]
mod solver;
use crate::solver::StubbedBackend;
use acir::{
    circuit::{
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Circuit, Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};
use acvm::{
    compiler::compile,
    pwg::{ACVMStatus, ACVM},
    Language,
};
use acvm_blackbox_solver::{blake2s, hash_to_field_128_security, keccak256, sha256};
use paste::paste;
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use stdlib::blackbox_fallbacks::{UInt32, UInt64, UInt8};

test_uint!(test_uint8, UInt8, u8, 8);
test_uint!(test_uint32, UInt32, u32, 32);
test_uint!(test_uint64, UInt64, u64, 64);

#[macro_export]
macro_rules! test_uint {
    (
        $name:tt,
        $uint:ident,
        $u:ident,
        $size:expr
    ) => {
        paste! {
            test_uint_inner!(
                [<$name _rol>],
                [<$name _ror>],
                [<$name _euclidean_division>],
                [<$name _add>],
                [<$name _sub>],
                [<$name _left_shift>],
                [<$name _right_shift>],
                [<$name _less_than>],
                $uint,
                $u,
                $size
            );
        }
    };
}

#[macro_export]
macro_rules! test_uint_inner {
    (
        $rol:tt,
        $ror:tt,
        $euclidean_division:tt,
        $add:tt,
        $sub:tt,
        $left_shift:tt,
        $right_shift:tt,
        $less_than:tt,
        $uint: ident,
        $u: ident,
        $size: expr
    ) => {
        proptest! {
            #[test]
            fn $rol(x in 0..$u::MAX, y in 0..32_u32) {
                let fe = FieldElement::from(x as u128);
                let w = Witness(1);
                let result = x.rotate_left(y);
                let uint = $uint::new(w);
                let (w, extra_opcodes, _) = uint.rol(y, 2);
                let witness_assignments = BTreeMap::from([(Witness(1), fe)]).into();
                let mut acvm = ACVM::new(&StubbedBackend, extra_opcodes, witness_assignments);
                let solver_status = acvm.solve();

                prop_assert_eq!(acvm.witness_map().get(&w.get_inner()).unwrap(), &FieldElement::from(result as u128));
                prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
            }

            #[test]
            fn $ror(x in 0..$u::MAX, y in 0..32_u32) {
                let fe = FieldElement::from(x as u128);
                let w = Witness(1);
                let result = x.rotate_right(y);
                let uint = $uint::new(w);
                let (w, extra_opcodes, _) = uint.ror(y, 2);
                let witness_assignments = BTreeMap::from([(Witness(1), fe)]).into();
                let mut acvm = ACVM::new(&StubbedBackend, extra_opcodes, witness_assignments);
                let solver_status = acvm.solve();

                prop_assert_eq!(acvm.witness_map().get(&w.get_inner()).unwrap(), &FieldElement::from(result as u128));
                prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
            }

            #[test]
            fn $euclidean_division(x in 0..$u::MAX, y in 1
                ..$u::MAX) {
                let lhs = FieldElement::from(x as u128);
                let rhs = FieldElement::from(y as u128);
                let w1 = Witness(1);
                let w2 = Witness(2);
                let q = x.div_euclid(y);
                let r = x.rem_euclid(y);
                let u32_1 = $uint::new(w1);
                let u32_2 = $uint::new(w2);
                let (q_w, r_w, extra_opcodes, _) = $uint::euclidean_division(&u32_1, &u32_2, 3);
                let witness_assignments = BTreeMap::from([(Witness(1), lhs),(Witness(2), rhs)]).into();
                let mut acvm = ACVM::new(&StubbedBackend, extra_opcodes, witness_assignments);
                let solver_status = acvm.solve();

                prop_assert_eq!(acvm.witness_map().get(&q_w.get_inner()).unwrap(), &FieldElement::from(q as u128));
                prop_assert_eq!(acvm.witness_map().get(&r_w.get_inner()).unwrap(), &FieldElement::from(r as u128));
                prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
            }

            #[test]
            fn $add(x in 0..$u::MAX, y in 0..$u::MAX, z in 0..$u::MAX) {
                let lhs = FieldElement::from(x as u128);
                let rhs = FieldElement::from(y as u128);
                let rhs_z = FieldElement::from(z as u128);
                let result = FieldElement::from(((x as u128).wrapping_add(y as u128) % (1_u128 << $size)).wrapping_add(z as u128) % (1_u128 << $size));
                let w1 = Witness(1);
                let w2 = Witness(2);
                let w3 = Witness(3);
                let u32_1 = $uint::new(w1);
                let u32_2 = $uint::new(w2);
                let u32_3 = $uint::new(w3);
                let mut opcodes = Vec::new();
                let (w, extra_opcodes, num_witness) = u32_1.add(&u32_2, 4);
                opcodes.extend(extra_opcodes);
                let (w2, extra_opcodes, _) = w.add(&u32_3, num_witness);
                opcodes.extend(extra_opcodes);
                let witness_assignments = BTreeMap::from([(Witness(1), lhs), (Witness(2), rhs), (Witness(3), rhs_z)]).into();
                let mut acvm = ACVM::new(&StubbedBackend, opcodes, witness_assignments);
                let solver_status = acvm.solve();

                prop_assert_eq!(acvm.witness_map().get(&w2.get_inner()).unwrap(), &result);
                prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
            }

            #[test]
            fn $sub(x in 0..$u::MAX, y in 0..$u::MAX, z in 0..$u::MAX) {
                let lhs = FieldElement::from(x as u128);
                let rhs = FieldElement::from(y as u128);
                let rhs_z = FieldElement::from(z as u128);
                let result = FieldElement::from(((x as u128).wrapping_sub(y as u128) % (1_u128 << $size)).wrapping_sub(z as u128) % (1_u128 << $size));
                let w1 = Witness(1);
                let w2 = Witness(2);
                let w3 = Witness(3);
                let u32_1 = $uint::new(w1);
                let u32_2 = $uint::new(w2);
                let u32_3 = $uint::new(w3);
                let mut opcodes = Vec::new();
                let (w, extra_opcodes, num_witness) = u32_1.sub(&u32_2, 4);
                opcodes.extend(extra_opcodes);
                let (w2, extra_opcodes, _) = w.sub(&u32_3, num_witness);
                opcodes.extend(extra_opcodes);
                let witness_assignments = BTreeMap::from([(Witness(1), lhs), (Witness(2), rhs), (Witness(3), rhs_z)]).into();
                let mut acvm = ACVM::new(&StubbedBackend, opcodes, witness_assignments);
                let solver_status = acvm.solve();

                prop_assert_eq!(acvm.witness_map().get(&w2.get_inner()).unwrap(), &result);
                prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
            }

            #[test]
            fn $left_shift(x in 0..$u::MAX, y in 0..32_u32) {
                let lhs = FieldElement::from(x as u128);
                let w1 = Witness(1);
                let result = x.overflowing_shl(y).0;
                let u32_1 = $uint::new(w1);
                let (w, extra_opcodes, _) = u32_1.leftshift(y, 2);
                let witness_assignments = BTreeMap::from([(Witness(1), lhs)]).into();
                let mut acvm = ACVM::new(&StubbedBackend, extra_opcodes, witness_assignments);
                let solver_status = acvm.solve();

                prop_assert_eq!(acvm.witness_map().get(&w.get_inner()).unwrap(), &FieldElement::from(result as u128));
                prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
            }

            #[test]
            fn $right_shift(x in 0..$u::MAX, y in 0..32_u32) {
                let lhs = FieldElement::from(x as u128);
                let w1 = Witness(1);
                let result = x.overflowing_shr(y).0;
                let u32_1 = $uint::new(w1);
                let (w, extra_opcodes, _) = u32_1.rightshift(y, 2);
                let witness_assignments = BTreeMap::from([(Witness(1), lhs)]).into();
                let mut acvm = ACVM::new(&StubbedBackend, extra_opcodes, witness_assignments);
                let solver_status = acvm.solve();

                prop_assert_eq!(acvm.witness_map().get(&w.get_inner()).unwrap(), &FieldElement::from(result as u128));
                prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
            }

            #[test]
            fn $less_than(x in 0..$u::MAX, y in 0..$u::MAX) {
                let lhs = FieldElement::from(x as u128);
                let rhs = FieldElement::from(y as u128);
                let w1 = Witness(1);
                let w2 = Witness(2);
                let result = x < y;
                let u32_1 = $uint::new(w1);
                let u32_2 = $uint::new(w2);
                let (w, extra_opcodes, _) = u32_1.less_than_comparison(&u32_2, 3);
                let witness_assignments = BTreeMap::from([(Witness(1), lhs), (Witness(2), rhs)]).into();
                let mut acvm = ACVM::new(&StubbedBackend, extra_opcodes, witness_assignments);
                let solver_status = acvm.solve();

                prop_assert_eq!(acvm.witness_map().get(&w.get_inner()).unwrap(), &FieldElement::from(result as u128));
                prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
            }
        }
    };
}

test_hashes!(test_sha256, sha256, SHA256, does_not_support_sha256);
test_hashes!(test_blake2s, blake2s, Blake2s, does_not_support_blake2s);
test_hashes!(test_keccak, keccak256, Keccak256, does_not_support_keccak);

fn does_not_support_sha256(opcode: &Opcode) -> bool {
    !matches!(opcode, Opcode::BlackBoxFuncCall(BlackBoxFuncCall::SHA256 { .. }))
}
fn does_not_support_blake2s(opcode: &Opcode) -> bool {
    !matches!(opcode, Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Blake2s { .. }))
}
fn does_not_support_keccak(opcode: &Opcode) -> bool {
    !matches!(opcode, Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Keccak256 { .. }))
}

#[macro_export]
macro_rules! test_hashes {
    (
        $name:ident,
        $hasher:ident,
        $opcode:ident,
        $opcode_support: ident
    ) => {
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(3))]
            #[test]
            fn $name(input_values in proptest::collection::vec(0..u8::MAX, 1..50)) {
                let mut opcodes = Vec::new();
                let mut witness_assignments = BTreeMap::new();
                let mut input_witnesses: Vec<FunctionInput> = Vec::new();
                let mut correct_result_witnesses: Vec<Witness> = Vec::new();
                let mut output_witnesses: Vec<Witness> = Vec::new();

                // prepare test data
                let mut counter = 0;
                let output = $hasher(&input_values).unwrap();
                for inp_v in input_values {
                    counter += 1;
                    let function_input = FunctionInput { witness: Witness(counter), num_bits: 8 };
                    input_witnesses.push(function_input);
                    witness_assignments.insert(Witness(counter), FieldElement::from(inp_v as u128));
                }

                for o_v in output {
                    counter += 1;
                    correct_result_witnesses.push(Witness(counter));
                    witness_assignments.insert(Witness(counter), FieldElement::from(o_v as u128));
                }

                for _ in 0..32 {
                    counter += 1;
                    output_witnesses.push(Witness(counter));
                }
                let blackbox = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::$opcode { inputs: input_witnesses, outputs: output_witnesses.clone() });
                opcodes.push(blackbox);

                // constrain the output to be the same as the hasher
                for i in 0..correct_result_witnesses.len() {
                    let mut output_constraint = Expression::from(correct_result_witnesses[i]);
                    output_constraint.push_addition_term(-FieldElement::one(), output_witnesses[i]);
                    opcodes.push(Opcode::Arithmetic(output_constraint));
                }

                // compile circuit
                let circuit = Circuit {
                    current_witness_index: witness_assignments.len() as u32 + 32,
                    opcodes,
                    private_parameters: BTreeSet::new(), // This is not correct but is unused in this test.
                    ..Circuit::default()
                 };
                let circuit = compile(circuit, Language::PLONKCSat{ width: 3 }, $opcode_support).unwrap().0;

                // solve witnesses
                let mut acvm = ACVM::new(&StubbedBackend, circuit.opcodes, witness_assignments.into());
                let solver_status = acvm.solve();

                prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
            }
        }
    };
}

fn does_not_support_hash_to_field(opcode: &Opcode) -> bool {
    !matches!(opcode, Opcode::BlackBoxFuncCall(BlackBoxFuncCall::HashToField128Security { .. }))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3))]
    #[test]
    fn test_hash_to_field(input_values in proptest::collection::vec(0..u8::MAX, 1..50)) {
        let mut opcodes = Vec::new();
        let mut witness_assignments = BTreeMap::new();
        let mut input_witnesses: Vec<FunctionInput> = Vec::new();

        // prepare test data
        let mut counter = 0;
        let output = hash_to_field_128_security(&input_values).unwrap();
        for inp_v in input_values {
            counter += 1;
            let function_input = FunctionInput { witness: Witness(counter), num_bits: 8 };
            input_witnesses.push(function_input);
            witness_assignments.insert(Witness(counter), FieldElement::from(inp_v as u128));
        }

        counter += 1;
        let correct_result_witnesses: Witness = Witness(counter);
        witness_assignments.insert(Witness(counter), output);

        counter += 1;
        let output_witness: Witness = Witness(counter);

        let blackbox = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::HashToField128Security { inputs: input_witnesses, output: output_witness });
        opcodes.push(blackbox);

        // constrain the output to be the same as the hasher
        let mut output_constraint = Expression::from(correct_result_witnesses);
        output_constraint.push_addition_term(-FieldElement::one(), output_witness);
        opcodes.push(Opcode::Arithmetic(output_constraint));

        // compile circuit
        let circuit = Circuit {
            current_witness_index: witness_assignments.len() as u32 + 1,
            opcodes,
            private_parameters: BTreeSet::new(), // This is not correct but is unused in this test.
            ..Circuit::default()
        };
        let circuit = compile(circuit, Language::PLONKCSat{ width: 3 }, does_not_support_hash_to_field).unwrap().0;

        // solve witnesses
        let mut acvm = ACVM::new(&StubbedBackend, circuit.opcodes, witness_assignments.into());
        let solver_status = acvm.solve();

        prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
    }
}
