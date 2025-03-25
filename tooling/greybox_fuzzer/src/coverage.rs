//! This file implements the mechanisms for coverage - detection of changes is the execution of the target program
//! It assists in exploration of the program through testcase mutation by telling the fuzzer whether a new testcase represents previously unexplored
//! functionality. This in turn allows the fuzzer to add them to the corpus as footholds for further exploration
//!
//! There are several mechanisms for coverage being used:
//! 1. Standard branch coverage taken from brillig, the same as with standard, non-zk programs (detects which branch has been taken in an if)
//! 2. Conditional move coverage from brillig
//! 3. Novel boolean witness coverage. If ACIR execution was successful, we scan the witness for potential boolean values and detect interesting testcases, if we discover a boolean witness with a state that hasn't been previously encountered.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use acvm::AcirField;
use acvm::FieldElement;
use acvm::acir::circuit::Opcode;
use acvm::acir::native_types::{Witness, WitnessStack};
use acvm::brillig_vm::brillig::Opcode as BrilligOpcode;
use noirc_artifacts::program::ProgramArtifact;
use num_traits::Zero;

use crate::corpus::TestCaseId;

/// A state that represents a true comparison as part of a feature
const FUZZING_COMPARISON_TRUE_STATE: usize = usize::MAX - 1;
/// A state that represents a false comparison as part of a feature
const FUZZING_COMPARISON_FALSE_STATE: usize = usize::MAX;

/// The start of the range of the states that represent logarithm of the difference between the comparison arguments as part of a feature
const FUZZING_COMPARISON_LOG_RANGE_START_STATE: usize = 0;

/// The number of states in the range of the states that represent logarithm of the difference between the comparison arguments as part of a feature
const FUZZING_COMPARISON_LOG_RANGE_NUMBER_OF_STATES: usize = 256;

/// The position of an opcode that is currently being executed in the bytecode
pub type OpcodePosition = usize;

/// The position of the next opcode that will be executed in the bytecode or an id of a specific state produced by the opcode
pub type NextOpcodePositionOrState = usize;

/// A tuple of the current opcode position and the next opcode position or state
pub type Feature = (OpcodePosition, NextOpcodePositionOrState);

/// The index of a unique feature in the fuzzing trace
pub type UniqueFeatureIndex = usize;

/// A map from a particular branch or comparison to its unique index in the raw vector used inside brillig vm
pub type FeatureToIndexMap = HashMap<Feature, UniqueFeatureIndex>;

/// Mechanism for automated detection of boolean witnesses in the ACIR witness map
#[derive(Default)]
pub struct PotentialBoolWitnessList {
    // Set of witnesses that could be boolean values
    witness: HashSet<Witness>,
}

impl From<&WitnessStack<FieldElement>> for PotentialBoolWitnessList {
    /// Generate a bool witness list by parsing the witnesses in the program
    fn from(witness_stack: &WitnessStack<FieldElement>) -> Self {
        let mut witness_set = HashSet::new();
        // Should be only one function
        assert!(witness_stack.length() == 1);
        let first_func_witnesses = witness_stack.peek().unwrap();

        // Look for witnesses that are either 0 or 1
        for (witness_index, value) in first_func_witnesses.witness.clone().into_iter() {
            if value.is_one() || value.is_zero() {
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

    /// Given witnesses from a program, remove non-boolean witnesses from the list
    pub fn update(&mut self, witness_stack: &WitnessStack<FieldElement>) {
        assert!(witness_stack.length() == 1);
        let first_func_witnesses = witness_stack.peek().unwrap();
        let mut witnesses_for_removal = Vec::new();

        // Go through the list of perceived boolean witnesses
        for witness_index in self.witness.iter().copied() {
            let value = first_func_witnesses
                .witness
                .get(&witness_index)
                .expect("There should be a witness in the witness map");

            // Check that the values are zero or one
            if !value.is_one() && !value.is_zero() {
                witnesses_for_removal.push(witness_index);
            }
        }

        // Remove values that are not boolean
        for witness_index in witnesses_for_removal.into_iter() {
            self.witness.remove(&witness_index);
        }
    }

    /// Create a new list by filtering the witness indices in this list using the witnesses produced from ACIR execution
    pub fn merge_new(&self, witness_stack: &WitnessStack<FieldElement>) -> Self {
        assert!(witness_stack.length() == 1);
        let first_func_witnesses = witness_stack.peek().unwrap();
        let mut new_set = HashSet::new();

        // Keep only witnesses that are 0 or 1
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

// Constants for branch and equality state counts
const IF_BRANCH_COUNT: usize = 2;
const EQ_STATE_COUNT: usize = 2;

/// Structure containing information that at a particular index in the brillig program there is a branch
pub struct BranchCoverageRange {
    index: usize,
}

/// Structure containing information that at a particular index in the brillig program there is a comparison between elements of the following bit size
pub struct CmpCoverageRange {
    index: usize,
}

/// Structure containing information about positions of coverage-related opcodes in the brillig program
pub enum BrilligCoverageItemRange {
    Branch(BranchCoverageRange),
    Comparison(CmpCoverageRange),
}

pub type BrilligCoverageRanges = Vec<BrilligCoverageItemRange>;

/// Raw brillig coverage is just a buffer of uints that contain counters
pub type RawBrilligCoverage = Vec<u32>;

/// Information about the coverage of a single testcase execution
pub struct SingleTestCaseCoverage {
    /// The id of the testcase
    testcase_id: TestCaseId,
    /// A list of all boolean witness states
    acir_bool_coverage: Vec<AcirBoolState>,
    /// The raw coverage from brillig execution
    pub brillig_coverage: RawBrilligCoverage,
}

impl SingleTestCaseCoverage {
    pub fn new(
        testcase_id: TestCaseId,
        acir_witness_stack: &Option<WitnessStack<FieldElement>>,
        brillig_coverage: RawBrilligCoverage,
        potential_bool_witness_list: &PotentialBoolWitnessList,
    ) -> Self {
        // Process all booleans
        let mut acir_bool_coverage = Vec::new();

        // If the witness stack was not empty
        if let Some(acir_witnesses) = acir_witness_stack {
            let witness_map = &acir_witnesses.peek().unwrap().witness;

            // Collect states of all boolean witnesses
            for potential_bool_witness_index in potential_bool_witness_list.witness.iter() {
                let value =
                    witness_map.get(potential_bool_witness_index).expect("Witness should be there");
                assert!(value.is_zero() || value.is_one());

                acir_bool_coverage.push(AcirBoolState {
                    witness_id: potential_bool_witness_index.witness_index(),
                    state: value.is_one(),
                });
            }
        }

        Self { testcase_id, acir_bool_coverage, brillig_coverage }
    }
}

/// Metrics of a particular branch
#[derive(Default, Clone, Copy)]
pub struct AccumulatedSingleBranchCoverage {
    /// A bitmask of encountered powers of 2 of repetitions of this branch
    encountered_loop_log2s: u32,
    /// Which testcases showed log2 behavior
    testcases_involved: [Option<TestCaseId>; 32],
    /// The maximum number of iterations of this branch encountered in a single execution
    encountered_loop_maximum: u32,
    /// Testcase that produced the maximum iterations count
    maximum_testcase: Option<TestCaseId>,
    /// Index of the counter in the raw vector of coverage
    raw_index: usize,
}

/// Metrics of the closeness of a particular comparison
#[derive(Default, Clone, Copy)]
pub struct AccumulatedCmpCoverage {
    /// How many time during a single execution this comparison had the difference between arguments be this power of 2
    encountered_loop_log2s: u32,
    /// Which testcases exhibited this behavior
    testcases_involved: [Option<TestCaseId>; 32],
    /// The maximum number of iterations of this comparison with this difference log encountered in a single execution
    encountered_loop_maximum: u32,
    /// Which testcase exhibited this behavior
    maximum_testcase: Option<TestCaseId>,
    /// How close did the values get to each other
    closest_bits: u32,
    /// Testcase with the closest arguments
    closest_bits_testcase: Option<TestCaseId>,
    /// The starting index of the region in the raw vector of coverage in brillig
    raw_index: usize,
    /// If tracking this comparison is enabled (we disable it if we've reached equality)
    enabled: bool,
}

/// Total coverage presented by all testcases in the corpus
pub struct AccumulatedFuzzerCoverage {
    /// All observed states of boolean witnesses
    acir_bool_coverage: HashSet<AcirBoolState>,
    /// Testcases in which the boolean states have been observed
    bool_state_to_testcase_id: HashMap<AcirBoolState, TestCaseId>,
    /// Branch coverage in brillig that has been observed
    brillig_branch_coverage: Vec<AccumulatedSingleBranchCoverage>,
    /// Comparison coverage in brillig that has been observed
    brillig_cmp_approach_coverage: Vec<AccumulatedCmpCoverage>,
    /// The list of indices of all witnesses that are inferred to be boolean
    pub potential_bool_witness_list: Option<PotentialBoolWitnessList>,
}

type UnusedTestcaseIdSet = HashSet<TestCaseId>;

impl AccumulatedFuzzerCoverage {
    /// Create an initial AccumulatedFuzzerCoverage object from brillig coverage ranges
    pub fn new(coverage_items: &BrilligCoverageRanges) -> AccumulatedFuzzerCoverage {
        let mut single_branch_coverage = Vec::new();
        let mut cmp_coverage = Vec::new();

        // Process each coverage item
        for coverage_item in coverage_items.iter() {
            match coverage_item {
                // Handle branch coverage
                BrilligCoverageItemRange::Branch(branch_coverage_range) => {
                    for i in 0..IF_BRANCH_COUNT {
                        single_branch_coverage.push(AccumulatedSingleBranchCoverage {
                            encountered_loop_log2s: 0,
                            testcases_involved: [None; 32],
                            encountered_loop_maximum: 0,
                            maximum_testcase: None,
                            raw_index: branch_coverage_range.index + i,
                        });
                    }
                }
                // Handle comparison coverage
                BrilligCoverageItemRange::Comparison(cmp_coverage_range) => {
                    for i in 0..EQ_STATE_COUNT {
                        single_branch_coverage.push(AccumulatedSingleBranchCoverage {
                            encountered_loop_log2s: 0,
                            testcases_involved: [None; 32],
                            encountered_loop_maximum: 0,
                            maximum_testcase: None,
                            raw_index: cmp_coverage_range.index + i,
                        });
                    }
                    cmp_coverage.push(AccumulatedCmpCoverage {
                        encountered_loop_log2s: 0,
                        testcases_involved: [None; 32],
                        encountered_loop_maximum: 0,
                        maximum_testcase: None,
                        closest_bits: u32::MAX,
                        closest_bits_testcase: None,
                        raw_index: cmp_coverage_range.index + 2,
                        enabled: true,
                    });
                }
            }
        }

        Self {
            acir_bool_coverage: HashSet::new(),
            bool_state_to_testcase_id: HashMap::new(),
            brillig_branch_coverage: single_branch_coverage,
            brillig_cmp_approach_coverage: cmp_coverage,
            potential_bool_witness_list: None,
        }
    }

    /// Check if particular testcases are no longer needed as example of a particular behavior (comparison, branch)
    fn check_if_unused(&self, potentials: &UnusedTestcaseIdSet) -> UnusedTestcaseIdSet {
        let mut unused_testcases = potentials.clone();
        if unused_testcases.is_empty() {
            return unused_testcases;
        }

        // Helper closure to remove used testcases from the set
        let mut remove_if_used = |&x| match x {
            Some(testcase_id) => {
                unused_testcases.remove(&testcase_id);
                unused_testcases.is_empty()
            }
            None => false,
        };

        // Go through branch coverage and remove testcase id from the set of unused if we encounter it
        for branch in self.brillig_branch_coverage.iter() {
            if branch.encountered_loop_log2s.is_zero() {
                continue;
            }
            for element in branch.testcases_involved.iter() {
                if remove_if_used(element) {
                    return unused_testcases;
                }
            }
            if remove_if_used(&branch.maximum_testcase) {
                return unused_testcases;
            }
        }

        // Go through comparison coverage and remove testcase id from the set of unused if we encounter it
        for cmp_approach in self.brillig_cmp_approach_coverage.iter() {
            if !cmp_approach.enabled {
                continue;
            }
            if remove_if_used(&cmp_approach.maximum_testcase)
                || remove_if_used(&cmp_approach.closest_bits_testcase)
            {
                return unused_testcases;
            }
            for element in cmp_approach.testcases_involved.iter() {
                if remove_if_used(element) {
                    return unused_testcases;
                }
            }
        }

        // Go through acir boolean state and remove testcase id from the set of unused if we encounter it as an example of existing state
        for testcase_id in self.bool_state_to_testcase_id.values() {
            unused_testcases.remove(testcase_id);
            if unused_testcases.is_empty() {
                break;
            }
        }

        unused_testcases
    }

    /// Merge the coverage of a testcase into accumulated coverage
    /// Returns (false, empty set) if there is no new coverage (true, set of no longer needed testcases' ids) if there is
    pub fn merge(&mut self, new_coverage: &SingleTestCaseCoverage) -> (bool, UnusedTestcaseIdSet) {
        // Use quick detect first to see if we need to try and merge anything
        if !self.detect_new_coverage(new_coverage) {
            return (false, UnusedTestcaseIdSet::new());
        }
        let mut potential_leavers: UnusedTestcaseIdSet = UnusedTestcaseIdSet::new();

        // Helper closure to add testcase IDs to the set of potential leavers
        let mut add_to_leavers = |x| {
            if let Some(leaver_id) = x {
                potential_leavers.insert(leaver_id);
            };
        };

        self.merge_branch_coverage(new_coverage, &mut add_to_leavers);
        self.merge_comparison_coverage(new_coverage, &mut add_to_leavers);
        self.merge_acir_coverage(new_coverage, &mut add_to_leavers);
        self.remove_boolean_witness_false_positives(new_coverage, &mut add_to_leavers);

        // Filter testcase ids of testcases, whose feature ownership has been revoked by this new testcase
        (true, self.check_if_unused(&potential_leavers))
    }

    fn merge_branch_coverage(
        &mut self,
        new_coverage: &SingleTestCaseCoverage,
        add_to_leavers: &mut impl FnMut(Option<TestCaseId>),
    ) {
        // Go through all single branch coverage ranges and merge branch coverage in
        for branch in self.brillig_branch_coverage.iter_mut() {
            let prev_value = *branch;
            let testcase_value = new_coverage.brillig_coverage[branch.raw_index];
            // If the branch was taken at least once
            if !testcase_value.is_zero() {
                // Calculate iteration log
                let shift_index =
                    if testcase_value.is_zero() { 0 } else { testcase_value.ilog2() + 1 };

                // Assign current testcase id to this feature and pick the previous one as a potential leaver
                add_to_leavers(branch.testcases_involved[shift_index as usize]);
                branch.testcases_involved[shift_index as usize] = Some(new_coverage.testcase_id);
                // Register observed log
                branch.encountered_loop_log2s |= 1u32 << shift_index;
                // If this is the maximum loop iteration, save information about it
                if testcase_value > prev_value.encountered_loop_maximum {
                    add_to_leavers(branch.maximum_testcase);
                    branch.maximum_testcase = Some(new_coverage.testcase_id);
                    branch.encountered_loop_maximum = testcase_value;
                }
            }
        }
    }

    fn merge_comparison_coverage(
        &mut self,
        new_coverage: &SingleTestCaseCoverage,
        add_to_leavers: &mut impl FnMut(Option<TestCaseId>),
    ) {
        // Go through comparison coverage
        for cmp_approach in self.brillig_cmp_approach_coverage.iter_mut() {
            if !cmp_approach.enabled {
                // No need to detect closeness any more if we've hit the equality case
                continue;
            }

            let (least_different_bits, last_value) =
                Self::find_closest_comparison(new_coverage, cmp_approach);

            match least_different_bits.cmp(&cmp_approach.closest_bits) {
                std::cmp::Ordering::Less => {
                    Self::handle_closer_comparison(
                        cmp_approach,
                        new_coverage,
                        least_different_bits,
                        last_value,
                        add_to_leavers,
                    );
                }
                std::cmp::Ordering::Equal => {
                    Self::handle_equal_comparison(
                        cmp_approach,
                        new_coverage,
                        last_value,
                        add_to_leavers,
                    );
                }
                std::cmp::Ordering::Greater => {}
            }
        }
    }

    fn find_closest_comparison(
        new_coverage: &SingleTestCaseCoverage,
        cmp_approach: &AccumulatedCmpCoverage,
    ) -> (u32, u32) {
        let mut least_different_bits = u32::MAX;
        let mut last_value = 0;

        // Each log of difference has a separate spot in the raw coverage
        for i in 0..FUZZING_COMPARISON_LOG_RANGE_NUMBER_OF_STATES {
            if !new_coverage.brillig_coverage[i + cmp_approach.raw_index].is_zero() {
                least_different_bits = i as u32;
                last_value = new_coverage.brillig_coverage[i + cmp_approach.raw_index];
            }
        }

        (least_different_bits, last_value)
    }

    fn handle_closer_comparison(
        cmp_approach: &mut AccumulatedCmpCoverage,
        new_coverage: &SingleTestCaseCoverage,
        least_different_bits: u32,
        last_value: u32,
        add_to_leavers: &mut impl FnMut(Option<TestCaseId>),
    ) {
        // Remove testcases used in approach at previous difference
        add_to_leavers(cmp_approach.maximum_testcase);
        add_to_leavers(cmp_approach.closest_bits_testcase);

        // Remove testcases used in approach
        for i in 0..32 {
            add_to_leavers(cmp_approach.testcases_involved[i]);
        }

        // Register new metrics that have been reached
        cmp_approach.closest_bits = least_different_bits;
        cmp_approach.encountered_loop_maximum = last_value;
        let loop_log_shift = if last_value.is_zero() { 0 } else { last_value.ilog2() + 1 };
        cmp_approach.encountered_loop_log2s = 1u32 << loop_log_shift;

        // Memorize the testcase that showed this feature
        cmp_approach.closest_bits_testcase = Some(new_coverage.testcase_id);
        cmp_approach.maximum_testcase = Some(new_coverage.testcase_id);
        cmp_approach.testcases_involved = [None; 32];
        cmp_approach.testcases_involved[loop_log_shift as usize] = Some(new_coverage.testcase_id);

        // If we've hit the equality case, tracking comparisons makes no sense
        if least_different_bits == 0 {
            cmp_approach.enabled = false;
        }
    }

    fn handle_equal_comparison(
        cmp_approach: &mut AccumulatedCmpCoverage,
        new_coverage: &SingleTestCaseCoverage,
        last_value: u32,
        add_to_leavers: &mut impl FnMut(Option<TestCaseId>),
    ) {
        // In case the difference stays the same, observe if there are more repetitions
        let prev_value = *cmp_approach;
        let loop_log_shift = if last_value.is_zero() { 0 } else { last_value.ilog2() + 1 };
        add_to_leavers(cmp_approach.testcases_involved[loop_log_shift as usize]);

        cmp_approach.encountered_loop_log2s |= 1u32 << loop_log_shift;
        cmp_approach.testcases_involved[loop_log_shift as usize] = Some(new_coverage.testcase_id);
        if last_value > prev_value.encountered_loop_maximum {
            cmp_approach.encountered_loop_maximum = last_value;
            add_to_leavers(cmp_approach.maximum_testcase);
            cmp_approach.maximum_testcase = Some(new_coverage.testcase_id);
        }
    }

    fn merge_acir_coverage(
        &mut self,
        new_coverage: &SingleTestCaseCoverage,
        add_to_leavers: &mut impl FnMut(Option<TestCaseId>),
    ) {
        // Insert all ACIR states and replace testcase association
        for acir_bool_state in new_coverage.acir_bool_coverage.iter() {
            add_to_leavers(
                self.bool_state_to_testcase_id.insert(*acir_bool_state, new_coverage.testcase_id),
            );
            if !self.acir_bool_coverage.contains(acir_bool_state) {
                self.acir_bool_coverage.insert(*acir_bool_state);
            }
        }
    }

    fn remove_boolean_witness_false_positives(
        &mut self,
        new_coverage: &SingleTestCaseCoverage,
        add_to_leavers: &mut impl FnMut(Option<TestCaseId>),
    ) {
        // Get all boolean witnesses in the state
        let all_witnesses_in_bool_coverage: HashSet<_> = new_coverage
            .acir_bool_coverage
            .iter()
            .map(|acir_bool_state| acir_bool_state.witness_id)
            .collect();

        let mut states_to_remove = Vec::new();
        // Check that all boolean state witnesses observed in accumulated coverage are booleans here, too
        for state in self.acir_bool_coverage.iter() {
            if !all_witnesses_in_bool_coverage.contains(&state.witness_id) {
                states_to_remove.push(*state);
            }
        }

        // Remove states that are not booleans
        for state in states_to_remove {
            self.acir_bool_coverage.remove(&state);
            add_to_leavers(Some(self.bool_state_to_testcase_id[&state]));
        }
    }

    /// Returns true if there is new coverage in the presented testcase
    pub fn detect_new_coverage(&self, new_coverage: &SingleTestCaseCoverage) -> bool {
        // Go through all single branch coverage ranges and check that either:
        // 1. A new branch is taken
        // 2. A branch is taken more times than ever encountered before in a single execution
        // 3. A branch is taken pow2 times that hasn't been previously observed
        for branch in self.brillig_branch_coverage.iter() {
            let testcase_value = new_coverage.brillig_coverage[branch.raw_index];
            if !testcase_value.is_zero() {
                if (branch.encountered_loop_log2s
                    | (1u32
                        << (if testcase_value.is_zero() { 0 } else { testcase_value.ilog2() + 1 })))
                    != branch.encountered_loop_log2s
                {
                    return true;
                }
                if testcase_value > branch.encountered_loop_maximum {
                    return true;
                }
            }
        }

        // Go through comparison coverage and detect:
        // 1. If a particular comparison has achieved a difference between arguments whose log2 is smaller than previously observed
        // 2. If the smallest log2 previously observed has been detected more times in the same execution
        // 3. If the number of executions of the log2 is a new log2 that hasn't been observed
        for cmp_approach in self.brillig_cmp_approach_coverage.iter() {
            if !cmp_approach.enabled {
                // No need to detect closeness any more if we've hit the equality case
                continue;
            }
            let (least_different_bits, last_value) =
                Self::find_closest_comparison(new_coverage, cmp_approach);

            match least_different_bits.cmp(&cmp_approach.closest_bits) {
                std::cmp::Ordering::Less => return true,
                std::cmp::Ordering::Equal => {
                    if (cmp_approach.encountered_loop_log2s
                        | (1u32 << (if last_value.is_zero() { 0 } else { last_value.ilog2() + 1 })))
                        != cmp_approach.encountered_loop_log2s
                    {
                        return true;
                    }
                    if last_value > cmp_approach.encountered_loop_maximum {
                        return true;
                    }
                }
                std::cmp::Ordering::Greater => {}
            }
        }
        // Check if a boolean state has been observed before
        for acir_bool_state in new_coverage.acir_bool_coverage.iter() {
            if !self.acir_bool_coverage.contains(acir_bool_state) {
                return true;
            }
        }
        false
    }
}

/// Analyze the brillig program to detect:
/// 1. How many branches and conditional moves there are (needed fro branch coverage)
/// 2. How many comparisons there are (needed for comparison coverage)
///
/// Provide a feature to raw index map for brillig
pub fn analyze_brillig_program_before_fuzzing(
    program: &ProgramArtifact,
) -> (FeatureToIndexMap, BrilligCoverageRanges) {
    // Brillig program is an ACIR program where the first opcode is brillig call
    let program_bytecode = &program.bytecode;
    let main_function = &program_bytecode.functions[0];
    let starting_opcode = &main_function.opcodes[0];

    let fuzzed_brillig_function_id = match starting_opcode {
        Opcode::BrilligCall { id, .. } => id,
        _ => panic!(
            "If a method is compiled to brillig, the first opcode in ACIR has to be brillig call"
        ),
    };
    // Get the brillig code
    let fuzzed_brillig_function =
        &program_bytecode.unconstrained_functions[fuzzed_brillig_function_id.as_usize()];

    let mut feature_to_index_map = HashMap::new();
    let mut total_features = 0usize;
    let mut coverage_items = BrilligCoverageRanges::new();
    // Go through each opcode, detect branching and comparison opcodes and then store information about them
    for (opcode_index, opcode) in fuzzed_brillig_function.bytecode.iter().enumerate() {
        match opcode {
            // Conditional branching
            &BrilligOpcode::JumpIf { location, .. }
            | &BrilligOpcode::JumpIfNot { location, .. } => {
                feature_to_index_map.insert((opcode_index, location), total_features);
                feature_to_index_map.insert((opcode_index, opcode_index + 1), total_features + 1);
                coverage_items.push(BrilligCoverageItemRange::Branch(BranchCoverageRange {
                    index: total_features,
                }));
                total_features += 2;
            }
            // Conditional mov
            &BrilligOpcode::ConditionalMov { .. } => {
                feature_to_index_map.insert((opcode_index, FUZZING_COMPARISON_TRUE_STATE), total_features);
                feature_to_index_map.insert((opcode_index, FUZZING_COMPARISON_FALSE_STATE), total_features + 1);
                coverage_items.push(BrilligCoverageItemRange::Branch(BranchCoverageRange {
                    index: total_features,
                }));
                total_features += 2;
            }
            // Binary operations (we need comparisons)
            &BrilligOpcode::BinaryFieldOp { destination: _, op, .. } => match op {
                acvm::acir::brillig::BinaryFieldOp::Add
                | acvm::acir::brillig::BinaryFieldOp::Sub
                | acvm::acir::brillig::BinaryFieldOp::Mul
                | acvm::acir::brillig::BinaryFieldOp::Div
                | acvm::acir::brillig::BinaryFieldOp::IntegerDiv => {}
                // Equality and LessThan(Equals) are comparisons that interest us
                acvm::acir::brillig::BinaryFieldOp::Equals
                | acvm::acir::brillig::BinaryFieldOp::LessThan
                | acvm::acir::brillig::BinaryFieldOp::LessThanEquals => {
                    coverage_items.push(BrilligCoverageItemRange::Comparison(CmpCoverageRange {
                        index: total_features,
                    }));
                    // Insert features for boolean states
                    feature_to_index_map.insert((opcode_index, FUZZING_COMPARISON_TRUE_STATE), total_features);
                    feature_to_index_map.insert((opcode_index, FUZZING_COMPARISON_FALSE_STATE), total_features + 1);
                    total_features += 2;

                    // Insert features for each potential difference log
                    for i in 0..FUZZING_COMPARISON_LOG_RANGE_NUMBER_OF_STATES {
                        feature_to_index_map
                            .insert((opcode_index, FUZZING_COMPARISON_LOG_RANGE_START_STATE + i), total_features + i);
                    }
                    total_features += FUZZING_COMPARISON_LOG_RANGE_NUMBER_OF_STATES;
                }
            },
            // Binary operations (we need comparisons)
            &BrilligOpcode::BinaryIntOp { destination: _, op,  .. } => match op {
                acvm::acir::brillig::BinaryIntOp::Add
                | acvm::acir::brillig::BinaryIntOp::Sub
                | acvm::acir::brillig::BinaryIntOp::Mul
                | acvm::acir::brillig::BinaryIntOp::Div
                | acvm::acir::brillig::BinaryIntOp::And
                | acvm::acir::brillig::BinaryIntOp::Or
                | acvm::acir::brillig::BinaryIntOp::Xor
                | acvm::acir::brillig::BinaryIntOp::Shl
                | acvm::acir::brillig::BinaryIntOp::Shr => {}
                // Equality and LessThan(Equals) are comparisons that interest us
                acvm::acir::brillig::BinaryIntOp::Equals
                | acvm::acir::brillig::BinaryIntOp::LessThan
                | acvm::acir::brillig::BinaryIntOp::LessThanEquals => {
                    coverage_items.push(BrilligCoverageItemRange::Comparison(CmpCoverageRange {
                        index: total_features,
                    }));
                    // Insert features for boolean states
                    feature_to_index_map.insert((opcode_index, FUZZING_COMPARISON_TRUE_STATE), total_features);
                    feature_to_index_map.insert((opcode_index, FUZZING_COMPARISON_FALSE_STATE), total_features + 1);
                    total_features += 2;

                    // Insert features for each potential difference log
                    for i in 0..FUZZING_COMPARISON_LOG_RANGE_NUMBER_OF_STATES {
                        feature_to_index_map
                            .insert((opcode_index, FUZZING_COMPARISON_LOG_RANGE_START_STATE + i), total_features + i);
                    }
                    total_features += FUZZING_COMPARISON_LOG_RANGE_NUMBER_OF_STATES;

                }
            },
            BrilligOpcode::Not { .. }
            | BrilligOpcode::Cast { .. }
            | BrilligOpcode::Jump { .. }
            | BrilligOpcode::CalldataCopy { .. }
            | BrilligOpcode::Call { .. }
            // TODO(Parse constants in brillig and add to the dictionary)
            | BrilligOpcode::Const { .. }
            | BrilligOpcode::IndirectConst { .. }
            | BrilligOpcode::Return{..} |
            BrilligOpcode::ForeignCall { .. }
            | BrilligOpcode::Mov { .. }
            | BrilligOpcode::Load { .. }
            | BrilligOpcode::Store { .. }
            | BrilligOpcode::BlackBox(..)
            | BrilligOpcode::Trap { .. }
            | BrilligOpcode::Stop { .. } => (),
        }
    }
    (feature_to_index_map, coverage_items)
}
