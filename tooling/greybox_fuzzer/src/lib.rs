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

/// An executor for Noir programs which which provides fuzzing support
///
/// After instantiation, calling `fuzz` will proceed to hammer the program with
/// inputs, until it finds a counterexample. The provided [`TestRunner`] contains all the
/// configuration which can be overridden via [environment variables](proptest::test_runner::Config)
pub struct FuzzedExecutor<E> {
    /// The program to be fuzzed (acir version)
    acir_program: ProgramArtifact,

    /// The program to be fuzzed (brillig version)
    brillig_program: ProgramArtifact,

    /// A function which executes the programs with a given set of inputs
    executor: E,
}

impl<
        E: Fn(
            &Program<FieldElement>,
            WitnessMap<FieldElement>,
        ) -> Result<WitnessStack<FieldElement>, String>,
    > FuzzedExecutor<E>
{
    /// Instantiates a fuzzed executor given an executor
    pub fn new(
        acir_program: ProgramArtifact,
        brillig_program: ProgramArtifact,
        executor: E,
    ) -> Self {
        Self { acir_program, brillig_program, executor }
    }

    /// Fuzzes the provided program.
    pub fn fuzz(&self) -> FuzzTestResult {
        // Generate a seed for the campaign

        let seed = thread_rng().gen::<u64>();
        println!("Fuzzing seed for this campaign: {}", seed);

        let mut prng = XorShiftRng::seed_from_u64(seed);
        let dictionary = build_dictionary_from_program(&self.acir_program.bytecode);
        let input_map = generate_default_input_map(&self.acir_program.abi);

        let mut fuzz_res = self.single_fuzz(&input_map).unwrap();
        for _i in 0..20 {
            let input_map =
                mutate_input_map(&self.acir_program.abi, &input_map, &dictionary, &mut prng);
            fuzz_res = self.single_fuzz(&input_map).unwrap();
            match fuzz_res {
                FuzzOutcome::Case(_) => (),
                _ => {
                    break;
                }
            }
        }
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
    pub fn single_fuzz(&self, input_map: &InputMap) -> Result<FuzzOutcome, ()> {
        let initial_witness = self.acir_program.abi.encode(&input_map, None).unwrap();
        let initial_witness2 = self.acir_program.abi.encode(&input_map, None).unwrap();
        let result_acir = (self.executor)(&self.acir_program.bytecode, initial_witness);
        let result_brillig = (self.executor)(&self.brillig_program.bytecode, initial_witness2);

        // TODO: Add handling for `vm.assume` equivalent

        match (result_acir, result_brillig) {
            (Ok(_), Ok(_)) => Ok(FuzzOutcome::Case(CaseOutcome { case: input_map.clone() })),
            (Err(err), Ok(_)) => Ok(FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                exit_reason: err,
                acir_failed: true,
                counterexample: input_map.clone(),
            })),
            (Ok(_), Err(err)) => Ok(FuzzOutcome::Discrepancy(DiscrepancyOutcome {
                exit_reason: err,
                acir_failed: false,
                counterexample: input_map.clone(),
            })),
            (Err(err), Err(..)) => Ok(FuzzOutcome::CounterExample(CounterExampleOutcome {
                exit_reason: err,
                counterexample: input_map.clone(),
            })),
        }
    }
}
