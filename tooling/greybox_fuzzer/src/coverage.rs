use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use acvm::acir::circuit::opcodes::{BlackBoxFuncCall, ConstantOrWitnessEnum};
use acvm::acir::circuit::Opcode;
use acvm::acir::native_types::{Witness, WitnessStack};
use acvm::brillig_vm::brillig::Opcode as BrilligOpcode;
use acvm::AcirField;
use acvm::FieldElement;
use noirc_artifacts::program::ProgramArtifact;
use num_traits::Zero;
pub type Branch = (usize, usize);
pub type BranchToFeatureMap = HashMap<Branch, usize>;

#[derive(Default)]
pub struct PotentialBoolWitnessList {
    witness: HashSet<Witness>,
}

impl From<&WitnessStack<FieldElement>> for PotentialBoolWitnessList {
    fn from(witness_stack: &WitnessStack<FieldElement>) -> Self {
        let mut witness_set = HashSet::new();
        // Should be only one function
        assert!(witness_stack.length() == 1);
        let first_func_witnesses = witness_stack.peek().unwrap();
        for (witness_index, value) in first_func_witnesses.witness.clone().into_iter() {
            if value == FieldElement::one() || value == FieldElement::zero() {
                witness_set.insert(witness_index);
            }
        }
        Self { witness: witness_set }
    }
}

impl PotentialBoolWitnessList {
    pub fn new(given_set: HashSet<Witness>) -> Self {
        Self { witness: given_set }
    }
    pub fn update(&mut self, witness_stack: &WitnessStack<FieldElement>) {
        assert!(witness_stack.length() == 1);
        let first_func_witnesses = witness_stack.peek().unwrap();
        let mut witnesses_for_removal = Vec::new();
        for witness_index in self.witness.iter().copied() {
            let value = first_func_witnesses
                .witness
                .get(&witness_index)
                .expect("There should be a witness in the witness map");
            if *value != FieldElement::zero() && *value != FieldElement::one() {
                witnesses_for_removal.push(witness_index);
            }
        }
        for witness_index in witnesses_for_removal.into_iter() {
            self.witness.remove(&witness_index);
        }
    }
    pub fn merge_new(&self, witness_stack: &WitnessStack<FieldElement>) -> Self {
        assert!(witness_stack.length() == 1);
        let first_func_witnesses = witness_stack.peek().unwrap();
        let mut new_set = HashSet::new();
        for witness_index in self.witness.iter().copied() {
            let value = first_func_witnesses
                .witness
                .get(&witness_index)
                .expect("There should be a witness in the witness map");
            if value.is_zero() || value.is_one() {
                new_set.insert(witness_index);
            }
        }
        Self::new(new_set)
    }
}
/// Represents a single encountered state of a boolean witness in the Acir program
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct AcirBoolState {
    witness_id: u32,
    state: bool,
}

pub struct SingleTestCaseCoverage {
    acir_bool_coverage: Vec<AcirBoolState>,
    pub brillig_coverage: Vec<u32>,
}

impl SingleTestCaseCoverage {
    pub fn new(
        acir_witnesses: &WitnessStack<FieldElement>,
        brillig_coverage: Vec<u32>,
        potential_bool_witness_list: &PotentialBoolWitnessList,
    ) -> Self {
        // Process all booleans
        let mut acir_bool_coverage = Vec::new();
        // If the witness stack was not empty
        if acir_witnesses.length() == 1 {
            let witness_map = &acir_witnesses.peek().unwrap().witness;

            for potential_bool_witness_index in potential_bool_witness_list.witness.iter() {
                let value = witness_map
                    .get(&potential_bool_witness_index)
                    .expect("Witness should be there");
                assert!(value.is_zero() || value.is_one());

                acir_bool_coverage.push(AcirBoolState {
                    witness_id: potential_bool_witness_index.witness_index(),
                    state: value.is_one(),
                });
            }
        }
        Self { acir_bool_coverage, brillig_coverage }
    }
}

#[derive(Default, Clone, Copy)]
pub struct AccumulatedSingleBranchCoverage {
    encountered_loop_log2s: u32,
    encountered_maximum: u32,
}
pub struct AccumulatedFuzzerCoverage {
    acir_bool_coverage: HashSet<AcirBoolState>,
    brillig_branch_coverage: Vec<AccumulatedSingleBranchCoverage>,
    pub potential_bool_witness_list: Option<PotentialBoolWitnessList>,
}
impl AccumulatedFuzzerCoverage {
    pub fn new(brillig_coverage_map_size: usize) -> AccumulatedFuzzerCoverage {
        Self {
            acir_bool_coverage: HashSet::new(),
            brillig_branch_coverage: vec![
                AccumulatedSingleBranchCoverage::default();
                brillig_coverage_map_size
            ],
            potential_bool_witness_list: None,
        }
    }

    pub fn merge(&mut self, new_coverage: &SingleTestCaseCoverage) -> bool {
        assert!(new_coverage.brillig_coverage.len() == self.brillig_branch_coverage.len());
        let mut new_coverage_detected = false;
        for (idx, value) in new_coverage.brillig_coverage.iter().enumerate() {
            if !value.is_zero() {
                let prev_value = self.brillig_branch_coverage[idx];
                self.brillig_branch_coverage[idx].encountered_loop_log2s |=
                    1u32 << (if value.is_zero() { 0 } else { value.ilog2() + 1 });
                new_coverage_detected = new_coverage_detected
                    | (self.brillig_branch_coverage[idx].encountered_loop_log2s
                        != prev_value.encountered_loop_log2s);
                if value > &prev_value.encountered_maximum {
                    new_coverage_detected = true;
                    self.brillig_branch_coverage[idx].encountered_maximum = *value;
                }
            }
        }
        for acir_bool_state in new_coverage.acir_bool_coverage.iter() {
            if !self.acir_bool_coverage.contains(acir_bool_state) {
                new_coverage_detected = true;
                self.acir_bool_coverage.insert(*acir_bool_state);
            }
        }
        new_coverage_detected
    }
    pub fn detect_new_coverage(&self, new_coverage: &SingleTestCaseCoverage) -> bool {
        assert!(new_coverage.brillig_coverage.len() == self.brillig_branch_coverage.len());
        for (idx, value) in new_coverage.brillig_coverage.iter().enumerate() {
            if !value.is_zero() {
                let prev_value = self.brillig_branch_coverage[idx];
                // New power of two detected
                if self.brillig_branch_coverage[idx].encountered_loop_log2s
                    | 1u32 << (if value.is_zero() { 0 } else { value.ilog2() + 1 })
                    != prev_value.encountered_loop_log2s
                {
                    return true;
                }
                if value > &prev_value.encountered_maximum {
                    return true;
                }
            }
        }
        for acir_bool_state in new_coverage.acir_bool_coverage.iter() {
            if !self.acir_bool_coverage.contains(acir_bool_state) {
                return true;
            }
        }
        false
    }
}
pub fn analyze_brillig_program_before_fuzzing(program: &ProgramArtifact) -> BranchToFeatureMap {
    let program_bytecode = &program.bytecode;
    let main_function = &program_bytecode.functions[0];
    let starting_opcode = &main_function.opcodes[0];
    let fuzzed_brillig_function_id = match starting_opcode {
        Opcode::BrilligCall { id, .. } => id,
        _ => panic!(
            "If a method is compiled to brillig, the first opcode in ACIR has to be brillig call"
        ),
    };
    let fuzzed_brillig_function =
        &program_bytecode.unconstrained_functions[fuzzed_brillig_function_id.as_usize()];
    let mut location_to_feature_map = HashMap::new();
    let mut total_features = 0usize;
    for (opcode_index, opcode) in fuzzed_brillig_function.bytecode.iter().enumerate() {
        match opcode {
            &BrilligOpcode::JumpIf { location, .. }
            | &BrilligOpcode::JumpIfNot { location, .. } => {
                location_to_feature_map.insert((opcode_index, location), total_features);
                location_to_feature_map
                    .insert((opcode_index, opcode_index + 1), total_features + 1);
                total_features += 2;
            }
            &BrilligOpcode::ConditionalMov { .. } => {
                location_to_feature_map.insert((opcode_index, usize::MAX - 1), total_features);
                location_to_feature_map.insert((opcode_index, usize::MAX), total_features + 1);
                total_features += 2;
            }
            &BrilligOpcode::BinaryFieldOp { destination: _, op, .. } => match op {
                acvm::acir::brillig::BinaryFieldOp::Add
                | acvm::acir::brillig::BinaryFieldOp::Sub
                | acvm::acir::brillig::BinaryFieldOp::Mul
                | acvm::acir::brillig::BinaryFieldOp::Div
                | acvm::acir::brillig::BinaryFieldOp::IntegerDiv => {}
                acvm::acir::brillig::BinaryFieldOp::Equals
                | acvm::acir::brillig::BinaryFieldOp::LessThan
                | acvm::acir::brillig::BinaryFieldOp::LessThanEquals => {
                    let features_per_comparison= 1 /*true */+1/*false */+255 /*possible bits() results*/;
                    for i in 0..features_per_comparison {
                        location_to_feature_map
                            .insert((opcode_index, usize::MAX - i), total_features + i);
                    }
                    total_features += features_per_comparison;
                }
            },
            &BrilligOpcode::BinaryIntOp { destination: _, op, bit_size, .. } => match op {
                acvm::acir::brillig::BinaryIntOp::Add
                | acvm::acir::brillig::BinaryIntOp::Sub
                | acvm::acir::brillig::BinaryIntOp::Mul
                | acvm::acir::brillig::BinaryIntOp::Div
                | acvm::acir::brillig::BinaryIntOp::And
                | acvm::acir::brillig::BinaryIntOp::Or
                | acvm::acir::brillig::BinaryIntOp::Xor
                | acvm::acir::brillig::BinaryIntOp::Shl
                | acvm::acir::brillig::BinaryIntOp::Shr => {}
                acvm::acir::brillig::BinaryIntOp::Equals
                | acvm::acir::brillig::BinaryIntOp::LessThan
                | acvm::acir::brillig::BinaryIntOp::LessThanEquals => {
                    let features_per_comparison = 1 /*true */+1/*false */+1/*when ilog is zero*/+ match bit_size{
                    acvm::acir::brillig::IntegerBitSize::U1 => 1,
                    acvm::acir::brillig::IntegerBitSize::U8 => 8,
                    acvm::acir::brillig::IntegerBitSize::U16 => 16,
                    acvm::acir::brillig::IntegerBitSize::U32 => 32,
                    acvm::acir::brillig::IntegerBitSize::U64 => 64,
                    acvm::acir::brillig::IntegerBitSize::U128 => 128,
                };
                    for i in 0..features_per_comparison {
                        location_to_feature_map
                            .insert((opcode_index, usize::MAX - i), total_features + i);
                    }
                    total_features += features_per_comparison;
                }
            },
            _ => (),
        }
    }
    location_to_feature_map
}

pub fn analyze_acir_program_before_fuzzing(program: &ProgramArtifact) -> HashSet<Witness> {
    let mut boolean_witness_set = HashSet::new();
    let program_bytecode = &program.bytecode;
    let main_function = &program_bytecode.functions[0];
    for opcode in main_function.opcodes.iter() {
        let range_input = match opcode {
            Opcode::BlackBoxFuncCall(black_box_func_call) => match black_box_func_call {
                BlackBoxFuncCall::RANGE { input } => Some(input),
                _ => None,
            },
            _ => None,
        };
        if range_input.is_none() {
            continue;
        }
        let range_input_unwrapped = range_input.unwrap();
        if range_input_unwrapped.num_bits() == 1 {
            match range_input_unwrapped.input_ref() {
                &ConstantOrWitnessEnum::Constant(..) => {
                    continue;
                }
                &ConstantOrWitnessEnum::Witness(witness) => {
                    boolean_witness_set.insert(witness);
                }
            }
        }
    }
    boolean_witness_set
}
