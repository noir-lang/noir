use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::u32;

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

const BRANCH_COVERAGE_SIZE: usize = 2;
pub struct BranchCoverageRange {
    index: usize,
}
pub struct CmpCoverageRange {
    index: usize,
    bits: usize,
}
pub enum BrilligCoverageItemRange {
    Branch(BranchCoverageRange),
    Comparison(CmpCoverageRange),
}

pub type BrilligCoverageRanges = Vec<BrilligCoverageItemRange>;
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
    encountered_loop_maximum: u32,
    raw_index: usize,
}

#[derive(Default, Clone, Copy)]
pub struct AccumulatedCmpCoverage {
    encountered_loop_log2s: u32,
    encountered_loop_maximum: u32,
    closest_bits: u32,
    raw_index: usize,
    bits: usize,
    enabled: bool,
}
pub struct AccumulatedFuzzerCoverage {
    acir_bool_coverage: HashSet<AcirBoolState>,
    brillig_branch_coverage: Vec<AccumulatedSingleBranchCoverage>,
    brillig_cmp_approach_coverage: Vec<AccumulatedCmpCoverage>,
    pub potential_bool_witness_list: Option<PotentialBoolWitnessList>,
}
impl AccumulatedFuzzerCoverage {
    pub fn new(
        brillig_coverage_map_size: usize,
        coverage_items: &BrilligCoverageRanges,
    ) -> AccumulatedFuzzerCoverage {
        let mut single_branch_coverage = Vec::new();
        let mut cmp_coverage = Vec::new();
        for coverage_item in coverage_items.iter() {
            match coverage_item {
                BrilligCoverageItemRange::Branch(branch_coverage_range) => {
                    let BRANCH_COUNT = 2;
                    for i in 0..BRANCH_COUNT {
                        single_branch_coverage.push(AccumulatedSingleBranchCoverage {
                            encountered_loop_log2s: 0,
                            encountered_loop_maximum: 0,
                            raw_index: branch_coverage_range.index + i,
                        });
                    }
                }
                BrilligCoverageItemRange::Comparison(cmp_coverage_range) => {
                    let BRANCH_COUNT = 2;
                    for i in 0..BRANCH_COUNT {
                        single_branch_coverage.push(AccumulatedSingleBranchCoverage {
                            encountered_loop_log2s: 0,
                            encountered_loop_maximum: 0,
                            raw_index: cmp_coverage_range.index + i,
                        });
                    }
                    cmp_coverage.push(AccumulatedCmpCoverage {
                        encountered_loop_log2s: 0,
                        encountered_loop_maximum: 0,
                        closest_bits: u32::MAX,
                        raw_index: cmp_coverage_range.index + 2,
                        bits: cmp_coverage_range.bits,
                        enabled: true,
                    });
                }
            }
        }
        Self {
            acir_bool_coverage: HashSet::new(),
            brillig_branch_coverage: single_branch_coverage,
            brillig_cmp_approach_coverage: cmp_coverage,
            potential_bool_witness_list: None,
        }
    }

    pub fn merge(&mut self, new_coverage: &SingleTestCaseCoverage) -> bool {
        let mut new_coverage_detected = false;

        // Go through all single branch coverage ranges and check
        for branch in self.brillig_branch_coverage.iter_mut() {
            let prev_value = branch.clone();
            let testcase_value = new_coverage.brillig_coverage[branch.raw_index];
            if !testcase_value.is_zero() {
                branch.encountered_loop_log2s |=
                    1u32 << (if testcase_value.is_zero() { 0 } else { testcase_value.ilog2() + 1 });
                new_coverage_detected = new_coverage_detected
                    | (branch.encountered_loop_log2s != prev_value.encountered_loop_log2s);
                if testcase_value > prev_value.encountered_loop_maximum {
                    new_coverage_detected = true;
                    branch.encountered_loop_maximum = testcase_value;
                }
            }
        }
        // Go through comparison coverage
        for cmp_approach in self.brillig_cmp_approach_coverage.iter_mut() {
            if !cmp_approach.enabled {
                // No need to detect closeness any more if we've hit the equality case
                continue;
            }
            let mut least_different_bits = u32::MAX;
            let mut last_value = 0;
            for i in 0..(cmp_approach.bits + 1) {
                if !new_coverage.brillig_coverage[i + cmp_approach.raw_index].is_zero() {
                    least_different_bits = (cmp_approach.bits - i) as u32;
                    last_value = new_coverage.brillig_coverage[i + cmp_approach.raw_index];
                }
            }

            if least_different_bits < cmp_approach.closest_bits {
                cmp_approach.closest_bits = least_different_bits;
                cmp_approach.encountered_loop_maximum = last_value;
                cmp_approach.encountered_loop_log2s |=
                    1u32 << (if last_value.is_zero() { 0 } else { last_value.ilog2() + 1 });

                new_coverage_detected = true;
                if least_different_bits == 0 {
                    cmp_approach.enabled = false;
                    println!("Disabled one comparison tracing;");
                }
            } else if least_different_bits == cmp_approach.closest_bits {
                let prev_value = cmp_approach.clone();
                cmp_approach.encountered_loop_log2s |=
                    1u32 << (if last_value.is_zero() { 0 } else { last_value.ilog2() + 1 });
                new_coverage_detected = new_coverage_detected
                    | (cmp_approach.encountered_loop_log2s != prev_value.encountered_loop_log2s);
                if last_value > prev_value.encountered_loop_maximum {
                    new_coverage_detected = true;
                    cmp_approach.encountered_loop_maximum = last_value;
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
        let mut new_coverage_detected = false;

        // Go through all single branch coverage ranges and check
        for branch in self.brillig_branch_coverage.iter() {
            let testcase_value = new_coverage.brillig_coverage[branch.raw_index];
            if !testcase_value.is_zero() {
                if (branch.encountered_loop_log2s
                    | 1u32
                        << (if testcase_value.is_zero() { 0 } else { testcase_value.ilog2() + 1 }))
                    != branch.encountered_loop_log2s
                {
                    return true;
                }
                if testcase_value > branch.encountered_loop_maximum {
                    return true;
                }
            }
        }
        // Go through comparison coverage
        for cmp_approach in self.brillig_cmp_approach_coverage.iter() {
            if !cmp_approach.enabled {
                // No need to detect closeness any more if we've hit the equality case
                continue;
            }
            let mut least_different_bits = u32::MAX;
            let mut last_value = 0;
            for i in 0..(cmp_approach.bits + 1) {
                if !new_coverage.brillig_coverage[i + cmp_approach.raw_index].is_zero() {
                    least_different_bits = (cmp_approach.bits - i) as u32;
                    last_value = new_coverage.brillig_coverage[i + cmp_approach.raw_index];
                }
            }

            if least_different_bits < cmp_approach.closest_bits {
                return true;
            } else if least_different_bits == cmp_approach.closest_bits {
                if (cmp_approach.encountered_loop_log2s
                    | 1u32 << (if last_value.is_zero() { 0 } else { last_value.ilog2() + 1 }))
                    != cmp_approach.encountered_loop_log2s
                {
                    return true;
                }
                if last_value > cmp_approach.encountered_loop_maximum {
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
pub fn analyze_brillig_program_before_fuzzing(
    program: &ProgramArtifact,
) -> (BranchToFeatureMap, BrilligCoverageRanges) {
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
    let mut coverage_items = BrilligCoverageRanges::new();
    for (opcode_index, opcode) in fuzzed_brillig_function.bytecode.iter().enumerate() {
        match opcode {
            &BrilligOpcode::JumpIf { location, .. }
            | &BrilligOpcode::JumpIfNot { location, .. } => {
                location_to_feature_map.insert((opcode_index, location), total_features);
                location_to_feature_map
                    .insert((opcode_index, opcode_index + 1), total_features + 1);
                coverage_items.push(BrilligCoverageItemRange::Branch(BranchCoverageRange {
                    index: total_features,
                }));
                total_features += 2;
            }
            &BrilligOpcode::ConditionalMov { .. } => {
                location_to_feature_map.insert((opcode_index, usize::MAX - 1), total_features);
                location_to_feature_map.insert((opcode_index, usize::MAX), total_features + 1);
                coverage_items.push(BrilligCoverageItemRange::Branch(BranchCoverageRange {
                    index: total_features,
                }));
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
                    coverage_items.push(BrilligCoverageItemRange::Comparison(CmpCoverageRange {
                        index: total_features,
                        bits: 254,
                    }));
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
                    coverage_items.push(BrilligCoverageItemRange::Comparison(CmpCoverageRange {
                        index: total_features,
                        bits: features_per_comparison - 3,
                    }));
                    total_features += features_per_comparison;
                }
            },
            _ => (),
        }
    }
    (location_to_feature_map, coverage_items)
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
