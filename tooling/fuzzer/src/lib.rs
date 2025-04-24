//! This module has been adapted from Foundry's fuzzing implementation for the EVM.
//! <https://github.com/foundry-rs/foundry/blob/6a85dbaa62f1c305f31cab37781232913055ae28/crates/evm/evm/src/executors/fuzz/mod.rs#L40>
//!
//! Code is used under the MIT license.

use std::cell::RefCell;

use acvm::{
    FieldElement,
    acir::{
        circuit::Program,
        native_types::{WitnessMap, WitnessStack},
    },
};
use bytes_writer::BytesWriter;
use dictionary::build_dictionary_from_program;
use noirc_abi::InputMap;
use proptest::test_runner::{TestCaseError, TestError, TestRunner};

mod bytes_writer;
pub mod dictionary;
pub mod strategies;
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
        BytesWriter,
    ) -> Result<WitnessStack<FieldElement>, String>,
{
    /// Instantiates a fuzzed executor given a [TestRunner].
    pub fn new(program: ProgramArtifact, executor: E, runner: TestRunner) -> Self {
        Self { program, executor, runner }
    }

    /// Fuzzes the provided program.
    pub fn fuzz(&mut self) -> FuzzTestResult {
        let dictionary = build_dictionary_from_program(&self.program.bytecode);
        let strategy = strategies::arb_input_map(&self.program.abi, &dictionary);

        // Each fuzz run produces output from `print` and `println` calls.
        // We don't print the output of all runs. Instead, two things can happen:
        //  1. There is a counter-example, in which case we'd like to show the output for that counter-example.
        //  2. There is no counter-example, in which case we'd like to show just one of the successfull cases.
        // We accomplish this by replacing the contents of `output` with the output of a run,
        // but only if we didn't find a counter-example so far.
        let output = RefCell::new(Vec::<u8>::new());
        let found_counter_example = RefCell::new(false);

        let run_result: Result<(), TestError<InputMap>> =
            self.runner.clone().run(&strategy, |input_map| {
                let fuzz_res = self.single_fuzz(input_map)?;

                match fuzz_res {
                    FuzzOutcome::Case(CaseOutcome { output: run_output, .. }) => {
                        if !*found_counter_example.borrow() {
                            output.replace(run_output);
                        }

                        Ok(())
                    }
                    FuzzOutcome::CounterExample(CounterExampleOutcome {
                        exit_reason: status,
                        output: run_output,
                        ..
                    }) => {
                        if !*found_counter_example.borrow() {
                            found_counter_example.replace(true);
                            output.replace(run_output);
                        }
                        Err(TestCaseError::fail(status))
                    }
                }
            });

        let output: Vec<u8> = output.into_inner();

        match run_result {
            Ok(()) => FuzzTestResult { success: true, reason: None, counterexample: None, output },

            Err(TestError::Abort(reason)) => FuzzTestResult {
                success: false,
                reason: Some(reason.to_string()),
                counterexample: None,
                output,
            },
            Err(TestError::Fail(reason, counterexample)) => {
                let reason = reason.to_string();
                let reason = if reason.is_empty() { None } else { Some(reason) };

                FuzzTestResult {
                    success: false,
                    reason,
                    counterexample: Some(counterexample),
                    output,
                }
            }
        }
    }

    /// Granular and single-step function that runs only one fuzz and returns either a `CaseOutcome`
    /// or a `CounterExampleOutcome`
    pub fn single_fuzz(&self, input_map: InputMap) -> Result<FuzzOutcome, TestCaseError> {
        let initial_witness = self.program.abi.encode(&input_map, None).unwrap();
        let bytes_writer = BytesWriter::default();
        let result = (self.executor)(&self.program.bytecode, initial_witness, bytes_writer.clone());
        let output = bytes_writer.into_bytes();

        // TODO: Add handling for `vm.assume` equivalent

        match result {
            Ok(_) => Ok(FuzzOutcome::Case(CaseOutcome { case: input_map, output })),
            Err(err) => Ok(FuzzOutcome::CounterExample(CounterExampleOutcome {
                exit_reason: err,
                counterexample: input_map,
                output,
            })),
        }
    }
}
