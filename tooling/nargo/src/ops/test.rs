use acvm::{
    acir::{
        brillig::ForeignCallResult,
        native_types::{WitnessMap, WitnessStack},
    },
    pwg::ForeignCallWaitInfo,
    AcirField, BlackBoxFunctionSolver, FieldElement,
};
use noirc_abi::Abi;
use noirc_driver::{compile_no_check, CompileError, CompileOptions, DEFAULT_EXPRESSION_WIDTH};
use noirc_errors::{debug_info::DebugInfo, FileDiagnostic};
use noirc_frontend::hir::{def_map::TestFunction, Context};

use crate::{
    errors::try_to_diagnose_runtime_error,
    foreign_calls::{layers, print::PrintOutput, ForeignCallError, ForeignCallExecutor},
    NargoError,
};

use super::execute_program;

#[derive(Debug)]
pub enum TestStatus {
    Pass,
    Fail { message: String, error_diagnostic: Option<FileDiagnostic> },
    Skipped,
    CompileError(FileDiagnostic),
}

impl TestStatus {
    pub fn failed(&self) -> bool {
        !matches!(self, TestStatus::Pass | TestStatus::Skipped)
    }
}

pub fn run_test<'a, B, F, E>(
    blackbox_solver: &B,
    context: &mut Context,
    test_function: &TestFunction,
    output: PrintOutput<'a>,
    config: &CompileOptions,
    build_foreign_call_executor: F,
) -> TestStatus
where
    B: BlackBoxFunctionSolver<FieldElement>,
    F: Fn(PrintOutput<'a>, layers::Unhandled) -> E + 'a,
    E: ForeignCallExecutor<FieldElement>,
{
    let test_function_has_no_arguments = context
        .def_interner
        .function_meta(&test_function.get_id())
        .function_signature()
        .0
        .is_empty();

    match compile_no_check(context, config, test_function.get_id(), None, false) {
        Ok(compiled_program) => {
            // Do the same optimizations as `compile_cmd`.
            let target_width = config.expression_width.unwrap_or(DEFAULT_EXPRESSION_WIDTH);
            let compiled_program = crate::ops::transform_program(compiled_program, target_width);

            if test_function_has_no_arguments {
                // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
                // otherwise constraints involving these expressions will not error.
                // Use a base layer that doesn't handle anything, which we handle in the `execute` below.
                let inner_executor = build_foreign_call_executor(output, layers::Unhandled);
                let mut foreign_call_executor = TestForeignCallExecutor::new(inner_executor);

                let circuit_execution = execute_program(
                    &compiled_program.program,
                    WitnessMap::new(),
                    blackbox_solver,
                    &mut foreign_call_executor,
                );

                let status = test_status_program_compile_pass(
                    test_function,
                    &compiled_program.abi,
                    &compiled_program.debug,
                    &circuit_execution,
                );

                let ignore_foreign_call_failures =
                    std::env::var("NARGO_IGNORE_TEST_FAILURES_FROM_FOREIGN_CALLS")
                        .is_ok_and(|var| &var == "true");

                if let TestStatus::Fail { .. } = status {
                    if ignore_foreign_call_failures
                        && foreign_call_executor.encountered_unknown_foreign_call
                    {
                        TestStatus::Skipped
                    } else {
                        status
                    }
                } else {
                    status
                }
            } else {
                use acvm::acir::circuit::Program;
                use noir_fuzzer::FuzzedExecutor;
                use proptest::test_runner::Config;
                use proptest::test_runner::TestRunner;

                let runner =
                    TestRunner::new(Config { failure_persistence: None, ..Config::default() });

                let abi = compiled_program.abi.clone();
                let debug = compiled_program.debug.clone();

                let executor = |program: &Program<FieldElement>,
                                initial_witness: WitnessMap<FieldElement>|
                 -> Result<WitnessStack<FieldElement>, String> {
                    // Use a base layer that doesn't handle anything, which we handle in the `execute` below.
                    let inner_executor =
                        build_foreign_call_executor(PrintOutput::None, layers::Unhandled);

                    let mut foreign_call_executor = TestForeignCallExecutor::new(inner_executor);

                    let circuit_execution = execute_program(
                        program,
                        initial_witness,
                        blackbox_solver,
                        &mut foreign_call_executor,
                    );

                    // Check if a failure was actually expected.
                    let status = test_status_program_compile_pass(
                        test_function,
                        &abi,
                        &debug,
                        &circuit_execution,
                    );

                    if let TestStatus::Fail { message, error_diagnostic: _ } = status {
                        Err(message)
                    } else {
                        // The fuzzer doesn't care about the actual result.
                        Ok(WitnessStack::default())
                    }
                };

                let fuzzer = FuzzedExecutor::new(compiled_program.into(), executor, runner);

                let result = fuzzer.fuzz();
                if result.success {
                    TestStatus::Pass
                } else {
                    TestStatus::Fail {
                        message: result.reason.unwrap_or_default(),
                        error_diagnostic: None,
                    }
                }
            }
        }
        Err(err) => test_status_program_compile_fail(err, test_function),
    }
}

/// Test function failed to compile
///
/// Note: This could be because the compiler was able to deduce
/// that a constraint was never satisfiable.
/// An example of this is the program `assert(false)`
/// In that case, we check if the test function should fail, and if so, we return `TestStatus::Pass`.
fn test_status_program_compile_fail(err: CompileError, test_function: &TestFunction) -> TestStatus {
    // The test has failed compilation, but it should never fail. Report error.
    if !test_function.should_fail() {
        return TestStatus::CompileError(err.into());
    }

    check_expected_failure_message(test_function, None, Some(err.into()))
}

/// The test function compiled successfully.
///
/// We now check whether execution passed/failed and whether it should have
/// passed/failed to determine the test status.
fn test_status_program_compile_pass(
    test_function: &TestFunction,
    abi: &Abi,
    debug: &[DebugInfo],
    circuit_execution: &Result<WitnessStack<FieldElement>, NargoError<FieldElement>>,
) -> TestStatus {
    let circuit_execution_err = match circuit_execution {
        // Circuit execution was successful; ie no errors or unsatisfied constraints
        // were encountered.
        Ok(_) => {
            if test_function.should_fail() {
                return TestStatus::Fail {
                    message: "error: Test passed when it should have failed".to_string(),
                    error_diagnostic: None,
                };
            }
            return TestStatus::Pass;
        }
        Err(err) => err,
    };

    // If we reach here, then the circuit execution failed.
    //
    // Check if the function should have passed
    let diagnostic = try_to_diagnose_runtime_error(circuit_execution_err, abi, debug);
    let test_should_have_passed = !test_function.should_fail();
    if test_should_have_passed {
        return TestStatus::Fail {
            message: circuit_execution_err.to_string(),
            error_diagnostic: diagnostic,
        };
    }

    check_expected_failure_message(
        test_function,
        circuit_execution_err.user_defined_failure_message(&abi.error_types),
        diagnostic,
    )
}

fn check_expected_failure_message(
    test_function: &TestFunction,
    failed_assertion: Option<String>,
    error_diagnostic: Option<FileDiagnostic>,
) -> TestStatus {
    // Extract the expected failure message, if there was one
    //
    // #[test(should_fail)] will not produce any message
    // #[test(should_fail_with = "reason")] will produce a message
    //
    let expected_failure_message = match test_function.failure_reason() {
        Some(reason) => reason,
        None => return TestStatus::Pass,
    };

    // Match the failure message that the user will see, i.e. the failed_assertion
    // if present or else the error_diagnostic's message, against the
    // expected_failure_message
    let expected_failure_message_matches = failed_assertion
        .as_ref()
        .or_else(|| {
            error_diagnostic.as_ref().map(|file_diagnostic| &file_diagnostic.diagnostic.message)
        })
        .map(|message| message.contains(expected_failure_message))
        .unwrap_or(false);
    if expected_failure_message_matches {
        return TestStatus::Pass;
    }

    // The expected failure message does not match the actual failure message
    TestStatus::Fail {
        message: format!(
            "\nerror: Test failed with the wrong message. \nExpected: {} \nGot: {}",
            test_function.failure_reason().unwrap_or_default(),
            failed_assertion.unwrap_or_default().trim_matches('\'')
        ),
        error_diagnostic,
    }
}

/// A specialized foreign call executor which tracks whether it has encountered any unknown foreign calls
struct TestForeignCallExecutor<E> {
    executor: E,
    encountered_unknown_foreign_call: bool,
}

impl<E> TestForeignCallExecutor<E> {
    fn new(executor: E) -> Self {
        Self { executor, encountered_unknown_foreign_call: false }
    }
}

impl<E, F> ForeignCallExecutor<F> for TestForeignCallExecutor<E>
where
    F: AcirField,
    E: ForeignCallExecutor<F>,
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        // If the circuit has reached a new foreign call opcode then it can't have failed from any previous unknown foreign calls.
        self.encountered_unknown_foreign_call = false;
        match self.executor.execute(foreign_call) {
            Err(ForeignCallError::NoHandler(_)) => {
                self.encountered_unknown_foreign_call = true;
                // If the inner executor cannot handle this foreign call, then it's very likely that this is a custom
                // foreign call. We then return an empty response in case the foreign call doesn't need return values.
                layers::Empty.execute(foreign_call)
            }
            other => other,
        }
    }
}
