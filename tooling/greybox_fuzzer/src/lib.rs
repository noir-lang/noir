//! This module has been adapted from Foundry's fuzzing implementation for the EVM.
//! https://github.com/foundry-rs/foundry/blob/6a85dbaa62f1c305f31cab37781232913055ae28/crates/evm/evm/src/executors/fuzz/mod.rs#L40
//!
//! Code is used under the MIT license.

use acvm::{
    acir::{
        circuit::Program,
        native_types::{WitnessMap, WitnessStack},
    },
    FieldElement,
};
use noir_fuzzer::dictionary::build_dictionary_from_program;
use noirc_abi::InputMap;

mod strategies;
mod types;

use strategies::{generate_default_input_map, mutate_input_map};
use types::{CaseOutcome, CounterExampleOutcome, DiscrepancyOutcome, FuzzOutcome, FuzzTestResult};

use noirc_artifacts::program::ProgramArtifact;
use rand::prelude::*;
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

type Corpus = Vec<InputMap>;
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
}

impl<
        E: Fn(
            &Program<FieldElement>,
            WitnessMap<FieldElement>,
        ) -> Result<WitnessStack<FieldElement>, String>,
        F: Fn(
            &Program<FieldElement>,
            WitnessMap<FieldElement>,
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
        Self { acir_program, brillig_program, acir_executor, brillig_executor }
    }

    /// Fuzzes the provided program.
    pub fn fuzz(&self) -> FuzzTestResult {
        // Generate a seed for the campaign

        let seed = thread_rng().gen::<u64>();
        println!("Fuzzing seed for this campaign: {}", seed);

        let mut prng = XorShiftRng::seed_from_u64(seed);
        let dictionary = build_dictionary_from_program(&self.acir_program.bytecode);
        let mut corpus = Corpus::new();
        corpus.push(generate_default_input_map(&self.acir_program.abi));

        let mut accumulated_coverage = AccumulatedFuzzerCoverage::new(65536);

        let (mut fuzz_res, mut coverage) = self.single_fuzz(&corpus[0]).unwrap();
        accumulated_coverage.merge(&(coverage.unwrap()));
        let mut last_i = 0;
        for i in 0..200000 {
            let input_map = mutate_input_map(
                &self.acir_program.abi,
                corpus.choose(&mut prng).unwrap(),
                &dictionary,
                &mut prng,
            );
            (fuzz_res, coverage) = self.single_fuzz(&input_map).unwrap();
            match fuzz_res {
                FuzzOutcome::Case(_) => (),
                _ => {
                    break;
                }
            }
            if accumulated_coverage.merge(&coverage.unwrap()) {
                println!("Input: {:?}", input_map);
                corpus.push(input_map);
                println!("Found new feature!");
            }
            last_i = i;
        }
        println!("{last_i}");
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
        let result_brillig =
            (self.brillig_executor)(&self.brillig_program.bytecode, initial_witness2);

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
