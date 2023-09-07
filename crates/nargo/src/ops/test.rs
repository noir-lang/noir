use acvm::{acir::native_types::WitnessMap, BlackBoxFunctionSolver};
use noirc_driver::{compile_no_check, CompileOptions};
use noirc_errors::FileDiagnostic;
use noirc_frontend::hir::{def_map::TestFunction, Context};

use crate::NargoError;

use super::execute_circuit;

pub enum TestStatus {
    Pass,
    Fail { message: String },
    CompileError(FileDiagnostic),
}

pub fn run_test<B: BlackBoxFunctionSolver>(
    blackbox_solver: &B,
    context: &Context,
    test_function: TestFunction,
    show_output: bool,
    config: &CompileOptions,
) -> TestStatus {
    let program = compile_no_check(context, config, test_function.get_id());
    match program {
        Ok(program) => {
            // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
            // otherwise constraints involving these expressions will not error.
            let circuit_execution =
                execute_circuit(blackbox_solver, program.circuit, WitnessMap::new(), show_output);
            test_status_program_compiled(test_function, circuit_execution)
        }
        Err(diag) => test_status_program_compile_fail(diag, test_function),
    }
}

/// Test function failed to compile
///
/// Note: This could be because the compiler was able to deduce
/// that a constraint was never satisfiable.
/// An example of this is the program `assert(false)`
/// In that case, we check if the test function should fail, and if so, we return `TestStatus::Pass`.
fn test_status_program_compile_fail(
    diag: FileDiagnostic,
    test_function: TestFunction,
) -> TestStatus {
    // The test has failed compilation, but it should never fail. Report error.
    if !test_function.should_fail() {
        return TestStatus::CompileError(diag);
    }

    // The test has failed compilation, check if it is because the program is never satisfiable.
    // If it is never satisfiable, then this is the expected behavior.
    let program_is_never_satisfiable = diag.diagnostic.message.contains("Failed constraint");
    if program_is_never_satisfiable {
        let expected_failure_message = test_function.failure_reason().unwrap_or_default();
        // Now check to see if it contains the expected failure message
        if diag.diagnostic.message.contains(expected_failure_message) {
            return TestStatus::Pass;
        } else {
            return TestStatus::Fail {
                message: format!(
                    "\nerror: Test failed with the wrong message. \nExpected: {} \nGot: {}",
                    expected_failure_message,
                    diag.diagnostic
                        .message
                        .trim_start_matches("Failed constraint: ")
                        .trim_matches('\'')
                ),
            };
        }
    }

    // The test has failed compilation, but its a compilation error. Report error
    TestStatus::CompileError(diag)
}

/// The test function compiled successfully.
///
/// We now check whether execution passed/failed and whether it should have
/// passed/failed to determine the test status.
fn test_status_program_compiled(
    test_function: TestFunction,
    circuit_execution: Result<WitnessMap, NargoError>,
) -> TestStatus {
    if test_function.should_fail() {
        match circuit_execution {
            Ok(_) => TestStatus::Fail {
                // TODO: Improve color variations on this message
                message: "error: Test passed when it should have failed".to_string(),
            },
            Err(err) => {
                let expected_failure_message = test_function.failure_reason().unwrap_or_default();
                // Check for the failure message
                if err.to_string().contains(expected_failure_message) {
                    TestStatus::Pass
                } else {
                    TestStatus::Fail {
                        message: format!(
                            "\nerror: Test failed with the wrong message. \nExpected: {} \nGot: {}",
                            expected_failure_message,
                            err.to_string().trim_matches('\'')
                        ),
                    }
                }
            }
        }
    } else {
        match circuit_execution {
            Ok(_) => TestStatus::Pass,
            Err(error) => TestStatus::Fail { message: error.to_string() },
        }
    }
}
