use acvm::{
    acir::native_types::{WitnessMap, WitnessStack},
    BlackBoxFunctionSolver, FieldElement,
};
use noirc_abi::Abi;
use noirc_driver::{compile_no_check, CompileError, CompileOptions};
use noirc_errors::{debug_info::DebugInfo, FileDiagnostic};
use noirc_frontend::hir::{def_map::TestFunction, Context};

use crate::{errors::try_to_diagnose_runtime_error, NargoError};

use super::{execute_program, DefaultForeignCallExecutor};

pub enum TestStatus {
    Pass,
    Fail { message: String, error_diagnostic: Option<FileDiagnostic> },
    CompileError(FileDiagnostic),
}

impl TestStatus {
    pub fn failed(&self) -> bool {
        !matches!(self, TestStatus::Pass)
    }
}

pub fn run_test<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    context: &mut Context,
    test_function: &TestFunction,
    show_output: bool,
    foreign_call_resolver_url: Option<&str>,
    config: &CompileOptions,
) -> TestStatus {
    let compiled_program = compile_no_check(context, config, test_function.get_id(), None, false);
    match compiled_program {
        Ok(compiled_program) => {
            // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
            // otherwise constraints involving these expressions will not error.
            let circuit_execution = execute_program(
                &compiled_program.program,
                WitnessMap::new(),
                blackbox_solver,
                &mut DefaultForeignCallExecutor::new(show_output, foreign_call_resolver_url),
            );
            test_status_program_compile_pass(
                test_function,
                compiled_program.abi,
                compiled_program.debug,
                circuit_execution,
            )
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
    circuit_execution: Result<WitnessStack<FieldElement>, NargoError>,
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

    let expected_failure_message_matches =
        matches!(&failed_assertion, Some(message) if message == expected_failure_message);
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
