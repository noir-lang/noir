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
use dictionary::build_dictionary_from_program;
use noirc_abi::InputMap;
use proptest::test_runner::{TestCaseError, TestError, TestRunner};

mod dictionary;
mod strategies;
mod types;

use types::{CaseOutcome, CounterExampleOutcome, FuzzOutcome, FuzzTestResult};

use noirc_artifacts::program::ProgramArtifact;

/// An executor for Noir programs which provides fuzzing support using [`proptest`].
///
/// After instantiation, calling `fuzz` will proceed to hammer the program with
/// inputs, until it finds a counterexample. The provided [`TestRunner`] contains all the
/// configuration which can be overridden via [environment variables](proptest::test_runner::Config)
pub struct FuzzedExecutor<E> {
    /// The program to be fuzzed
    program: ProgramArtifact,

    /// A function which executes the programs with a given set of inputs
    executor: E,

    /// The fuzzer
    runner: TestRunner,
}

impl<E> FuzzedExecutor<E>
where
    E: Fn(
        &Program<FieldElement>,
        WitnessMap<FieldElement>,
    ) -> Result<WitnessStack<FieldElement>, String>,
{
    /// Instantiates a fuzzed executor given a [TestRunner].
    pub fn new(program: ProgramArtifact, executor: E, runner: TestRunner) -> Self {
        Self { program, executor, runner }
    }

    /// Fuzzes the provided program.
    pub fn fuzz(&self) -> FuzzTestResult {
        let dictionary = build_dictionary_from_program(&self.program.bytecode);
        let strategy = strategies::arb_input_map(&self.program.abi, &dictionary);

        let run_result: Result<(), TestError<InputMap>> =
            self.runner.clone().run(&strategy, |input_map| {
                let fuzz_res = self.single_fuzz(input_map)?;

                match fuzz_res {
                    FuzzOutcome::Case(_) => Ok(()),
                    FuzzOutcome::CounterExample(CounterExampleOutcome {
                        exit_reason: status,
                        ..
                    }) => Err(TestCaseError::fail(status)),
                }
            });

        match run_result {
            Ok(()) => FuzzTestResult { success: true, reason: None, counterexample: None },

            Err(TestError::Abort(reason)) => FuzzTestResult {
                success: false,
                reason: Some(reason.to_string()),
                counterexample: None,
            },
            Err(TestError::Fail(reason, counterexample)) => {
                let reason = reason.to_string();
                let reason = if reason.is_empty() { None } else { Some(reason) };

                FuzzTestResult { success: false, reason, counterexample: Some(counterexample) }
            }
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz(&self, input_map: InputMap) -> Result<FuzzOutcome, TestCaseError> {
        let initial_witness = self.program.abi.encode(&input_map, None).unwrap();
        let result = (self.executor)(&self.program.bytecode, initial_witness);

        // TODO: Add handling for `vm.assume` equivalent

        match result {
            Ok(_) => Ok(FuzzOutcome::Case(CaseOutcome { case: input_map })),
            Err(err) => Ok(FuzzOutcome::CounterExample(CounterExampleOutcome {
                exit_reason: err,
                counterexample: input_map,
            })),
        }
    }
}
