//! This module has been adapted from Foundry's fuzzing implementation for the EVM.
//! https://github.com/foundry-rs/foundry/blob/6a85dbaa62f1c305f31cab37781232913055ae28/crates/evm/evm/src/executors/fuzz/mod.rs#L40
//!
//! Code is used under the MIT license.

use std::{cmp::min, collections::HashSet};

use acvm::{
    acir::{
        circuit::Program,
        native_types::{WitnessMap, WitnessStack},
    },
    FieldElement,
};
use coverage::{analyze_brillig_program_before_fuzzing, BranchToFeatureMap};
use noir_fuzzer::dictionary::{self, build_dictionary_from_program};
use noirc_abi::InputMap;

mod coverage;
mod strategies;
mod types;

use strategies::InputMutator;
use types::{CaseOutcome, CounterExampleOutcome, DiscrepancyOutcome, FuzzOutcome, FuzzTestResult};

use noirc_artifacts::program::ProgramArtifact;
use rand::{distributions::WeightedError, prelude::*};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
type SingleTestCaseCoverage = Vec<u8>;
struct AccumulatedFuzzerCoverage {
    buffer: Vec<u32>,
}
impl AccumulatedFuzzerCoverage {
    pub fn new(size: usize) -> AccumulatedFuzzerCoverage {
        Self { buffer: vec![0; size] }
    }

    pub fn merge(&mut self, new_coverage: &SingleTestCaseCoverage) -> bool {
        assert!(new_coverage.len() == self.buffer.len());
        let mut new_coverage_detected = false;
        for (idx, value) in new_coverage.iter().enumerate() {
            if *value != 0u8 {
                let prev_value = self.buffer[idx];
                self.buffer[idx] |= 1u32 << value;
                new_coverage_detected = new_coverage_detected | (self.buffer[idx] != prev_value);
            }
        }
        new_coverage_detected
    }
}

#[derive(Debug)]
struct Sequence {
    testcase_index: usize,
    executions_left: u64,
}
impl Sequence {
    pub fn new() -> Self {
        Self { testcase_index: 0, executions_left: 0 }
    }
    pub fn is_empty(&self) -> bool {
        self.executions_left == 0
    }
    pub fn decrement(&mut self) {
        self.executions_left -= 1
    }
}
struct Corpus {
    discovered_testcases: Vec<InputMap>,
    executions_per_testcase: Vec<u64>,
    sequence_number: Vec<u32>,
    total_executions: u64,
    current_sequence: Sequence,
}

impl Corpus {
    const MAX_EXECUTIONS_PER_SEQUENCE_LOG: u32 = 8;
    pub fn new(starting_testcase: InputMap) -> Self {
        Self {
            discovered_testcases: vec![starting_testcase],
            executions_per_testcase: vec![1],
            sequence_number: vec![0],
            total_executions: 1,
            current_sequence: Sequence::new(),
        }
    }
    pub fn insert(&mut self, new_testcase: InputMap) {
        self.discovered_testcases.push(new_testcase);
        self.executions_per_testcase.push(0);
        self.sequence_number.push(0);
    }
    pub fn get_next_testcase(&mut self, prng: &mut XorShiftRng) -> &InputMap {
        if !self.current_sequence.is_empty() {
            // Update counts
            self.current_sequence.decrement();
            self.executions_per_testcase[self.current_sequence.testcase_index] += 1;
            self.total_executions += 1;
            return &self.discovered_testcases[self.current_sequence.testcase_index];
        } else {
            // Compute average
            let average = self.total_executions / self.discovered_testcases.len() as u64;
            // Omit those that have been fuzzed more than average
            let weakly_fuzzed_group: Vec<_> = (0..(self.discovered_testcases.len()))
                .filter(|&index| self.executions_per_testcase[index] <= average)
                .collect();
            let chosen_index = weakly_fuzzed_group.choose(prng).unwrap().clone();
            self.sequence_number[chosen_index] += 1;
            self.current_sequence = Sequence {
                testcase_index: chosen_index,
                executions_left: min(
                    1u64 << min(
                        Self::MAX_EXECUTIONS_PER_SEQUENCE_LOG,
                        self.sequence_number[chosen_index],
                    ),
                    average / 2,
                ),
            };
            self.total_executions += 1;
            self.executions_per_testcase[chosen_index] += 1;

            return &self.discovered_testcases[self.current_sequence.testcase_index];
        }
    }
}
/// An executor for Noir programs which which provides fuzzing support
///
/// After instantiation, calling `fuzz` will proceed to hammer the program with
/// inputs, until it finds a counterexample. The provided [`TestRunner`] contains all the
/// configuration which can be overridden via [environment variables](proptest::test_runner::Config)
pub struct FuzzedExecutor<E, F> {
    /// The program to be fuzzed (acir version)
    acir_program: ProgramArtifact,

    /// The program to be fuzzed (brillig version)
    brillig_program: ProgramArtifact,

    /// A function which executes the programs with a given set of inputs
    acir_executor: E,

    /// A function which executes the programs with a given set of inputs
    brillig_executor: F,

    /// Location to feature map
    location_to_feature_map: BranchToFeatureMap,

    /// Mutator
    mutator: InputMutator,
}

impl<
        E: Fn(
            &Program<FieldElement>,
            WitnessMap<FieldElement>,
        ) -> Result<WitnessStack<FieldElement>, String>,
        F: Fn(
            &Program<FieldElement>,
            WitnessMap<FieldElement>,
            &BranchToFeatureMap,
        ) -> Result<(WitnessStack<FieldElement>, Option<Vec<u8>>), String>,
    > FuzzedExecutor<E, F>
{
    /// Instantiates a fuzzed executor given an executor
    pub fn new(
        acir_program: ProgramArtifact,
        brillig_program: ProgramArtifact,
        acir_executor: E,
        brillig_executor: F,
    ) -> Self {
        let location_to_feature_map = analyze_brillig_program_before_fuzzing(&brillig_program);
        let dictionary = build_dictionary_from_program(&acir_program.bytecode);
        let mutator = InputMutator::new(&acir_program.abi, &dictionary);
        Self {
            acir_program,
            brillig_program,
            acir_executor,
            brillig_executor,
            location_to_feature_map,
            mutator,
        }
    }

    /// Fuzzes the provided program.
    pub fn fuzz(&mut self) -> FuzzTestResult {
        // Generate a seed for the campaign

        let seed = thread_rng().gen::<u64>();
        println!("Fuzzing seed for this campaign: {}", seed);

        let mut prng = XorShiftRng::seed_from_u64(seed);
        let first_input = self.mutator.generate_default_input_map();
        let mut corpus = Corpus::new(first_input.clone());

        let mut accumulated_coverage =
            AccumulatedFuzzerCoverage::new(self.location_to_feature_map.len());

        let (mut fuzz_res, mut coverage) = self.single_fuzz(&first_input).unwrap();
        accumulated_coverage.merge(&(coverage.unwrap()));
        let mut current_iteration = 0;
        loop {
            if current_iteration % 10000 == 0 {
                println!("Current iteration {current_iteration}");
            }
            let input_map = self
                .mutator
                .mutate_input_map_multiple(corpus.get_next_testcase(&mut prng), &mut prng);
            (fuzz_res, coverage) = self.single_fuzz(&input_map).unwrap();
            match fuzz_res {
                FuzzOutcome::Case(_) => (),
                _ => {
                    break;
                }
            }
            if accumulated_coverage.merge(&coverage.unwrap()) {
                //println!("Input: {:?}", input_map);
                self.mutator.update_dictionary(&input_map);
                corpus.insert(input_map);
                //println!("Found new feature!");
            }
            current_iteration += 1;
        }
        println!("Total iterations: {current_iteration}");
        match fuzz_res {
            FuzzOutcome::Case(_) => {
                FuzzTestResult { success: true, reason: None, counterexample: None }
            }
            FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                exit_reason: status,
                acir_failed,
                counterexample,
            }) => {
                let reason = match acir_failed {
                    true => format!(
                        "ACIR failed while brillig executed with no issues: {}",
                        status.to_string()
                    ),
                    false => format!(
                        "brillig failed while ACIR executed with no issues: {}",
                        status.to_string()
                    ),
                };
                let reason = if reason.is_empty() { None } else { Some(reason) };

                FuzzTestResult { success: false, reason, counterexample: Some(counterexample) }
            }
            FuzzOutcome::CounterExample(CounterExampleOutcome {
                exit_reason: status,
                counterexample,
            }) => {
                let reason = status.to_string();
                let reason = if reason.is_empty() { None } else { Some(reason) };

                FuzzTestResult { success: false, reason, counterexample: Some(counterexample) }
            }
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz(&self, input_map: &InputMap) -> Result<(FuzzOutcome, Option<Vec<u8>>), ()> {
        let initial_witness = self.acir_program.abi.encode(&input_map, None).unwrap();
        let initial_witness2 = self.acir_program.abi.encode(&input_map, None).unwrap();
        let result_acir = (self.acir_executor)(&self.acir_program.bytecode, initial_witness);
        let result_brillig = (self.brillig_executor)(
            &self.brillig_program.bytecode,
            initial_witness2,
            &self.location_to_feature_map,
        );

        // TODO: Add handling for `vm.assume` equivalent

        match (result_acir, result_brillig) {
            (Ok(_), Ok((_map, coverage))) => {
                Ok((FuzzOutcome::Case(CaseOutcome { case: input_map.clone() }), coverage))
            }
            (Err(err), Ok(_)) => Ok((
                FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                    exit_reason: err,
                    acir_failed: true,
                    counterexample: input_map.clone(),
                }),
                None,
            )),
            (Ok(_), Err(err)) => Ok((
                FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                    exit_reason: err,
                    acir_failed: false,
                    counterexample: input_map.clone(),
                }),
                None,
            )),
            (Err(err), Err(..)) => Ok((
                FuzzOutcome::CounterExample(CounterExampleOutcome {
                    exit_reason: err,
                    counterexample: input_map.clone(),
                }),
                None,
            )),
        }
    }
}
