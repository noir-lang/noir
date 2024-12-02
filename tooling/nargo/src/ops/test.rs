use std::path::PathBuf;

use acvm::{
    acir::native_types::{WitnessMap, WitnessStack},
    BlackBoxFunctionSolver, FieldElement,
};
use noirc_abi::Abi;
use noirc_driver::{compile_no_check, CompileError, CompileOptions};
use noirc_errors::{debug_info::DebugInfo, FileDiagnostic};
use noirc_frontend::hir::{def_map::TestFunction, Context};

use crate::{errors::try_to_diagnose_runtime_error, NargoError};

use super::{execute_program, DefaultForeignCallExecutor, ForeignCall};

pub enum TestStatus {
    Pass,
    Fail { message: String, error_diagnostic: Option<FileDiagnostic> },
    Skipped,
    CompileError(FileDiagnostic),
}

impl TestStatus {
    pub fn failed(&self) -> bool {
        matches!(self, TestStatus::Fail { .. } | TestStatus::CompileError(_))
    }
    pub fn pass(&self) -> bool {
        matches!(self, TestStatus::Pass)
    }
    pub fn skipped(&self) -> bool {
        matches!(self, TestStatus::Skipped)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_test<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    context: &mut Context,
    test_function: &TestFunction,
    show_output: bool,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
    config: &CompileOptions,
) -> TestStatus {
    let test_function_has_no_arguments = context
        .def_interner
        .function_meta(&test_function.get_id())
        .function_signature()
        .0
        .is_empty();

    match compile_no_check(context, config, test_function.get_id(), None, false) {
        Ok(compiled_program) => {
            if config.skip_oracle {
                let has_oracle = compiled_program
                    .program
                    .unconstrained_functions
                    .iter()
                    .any(|func| func.has_oracle(ForeignCall::invalid_name));
                if has_oracle {
                    return TestStatus::Skipped;
                }
            }

            if test_function_has_no_arguments {
                // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
                // otherwise constraints involving these expressions will not error.
                let circuit_execution = execute_program(
                    &compiled_program.program,
                    WitnessMap::new(),
                    blackbox_solver,
                    &mut DefaultForeignCallExecutor::new(
                        show_output,
                        foreign_call_resolver_url,
                        root_path,
                        package_name,
                    ),
                );
                test_status_program_compile_pass(
                    test_function,
                    compiled_program.abi,
                    compiled_program.debug,
                    circuit_execution,
                )
            } else {
                #[cfg(target_arch = "wasm32")]
                {
                    // We currently don't support fuzz testing on wasm32 as the u128 strategies do not exist on this platform.
                    TestStatus::Fail {
                        message: "Fuzz tests are not supported on wasm32".to_string(),
                        error_diagnostic: None,
                    }
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    use acvm::acir::circuit::Program;
                    use noir_fuzzer::FuzzedExecutor;
                    use proptest::test_runner::TestRunner;
                    let runner = TestRunner::default();

                    let executor =
                        |program: &Program<FieldElement>,
                         initial_witness: WitnessMap<FieldElement>|
                         -> Result<WitnessStack<FieldElement>, String> {
                            execute_program(
                                program,
                                initial_witness,
                                blackbox_solver,
                                &mut DefaultForeignCallExecutor::<FieldElement>::new(
                                    false,
                                    foreign_call_resolver_url,
                                    root_path.clone(),
                                    package_name.clone(),
                                ),
                            )
                            .map_err(|err| err.to_string())
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
    abi: Abi,
    debug: Vec<DebugInfo>,
    circuit_execution: Result<WitnessStack<FieldElement>, NargoError<FieldElement>>,
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
    let diagnostic = try_to_diagnose_runtime_error(&circuit_execution_err, &abi, &debug);
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
