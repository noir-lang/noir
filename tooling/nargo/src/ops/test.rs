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
    let program = compile_no_check(context, config, test_function.get_id(), None);
    match program {
        Ok(program) => {
            // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
            // otherwise constraints involving these expressions will not error.
            let circuit_execution =
                execute_circuit(blackbox_solver, program.circuit, WitnessMap::new(), show_output);
            test_status_program_compile_pass(test_function, circuit_execution)
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
    if !program_is_never_satisfiable {
        // The test has failed compilation, but its a compilation error. Report error
        return TestStatus::CompileError(diag);
    }

    check_expected_failure_message(test_function, &diag.diagnostic.message)
}

/// The test function compiled successfully.
///
/// We now check whether execution passed/failed and whether it should have
/// passed/failed to determine the test status.
fn test_status_program_compile_pass(
    test_function: TestFunction,
    circuit_execution: Result<WitnessMap, NargoError>,
) -> TestStatus {
    let circuit_execution_err = match circuit_execution {
        // Circuit execution was successful; ie no errors or unsatisfied constraints
        // were encountered.
        Ok(_) => {
            if test_function.should_fail() {
                return TestStatus::Fail {
                    message: "error: Test passed when it should have failed".to_string(),
                };
            }
            return TestStatus::Pass;
        }
        Err(err) => err,
    };

    // If we reach here, then the circuit execution failed.
    //
    // Check if the function should have passed
    let test_should_have_passed = !test_function.should_fail();
    if test_should_have_passed {
        return TestStatus::Fail { message: circuit_execution_err.to_string() };
    }

    check_expected_failure_message(
        test_function,
        circuit_execution_err.user_defined_failure_message().unwrap_or_default(),
    )
}

fn check_expected_failure_message(test_function: TestFunction, got_error: &str) -> TestStatus {
    // Extract the expected failure message, if there was one
    //
    // #[test(should_fail)] will not produce any message
    // #[test(should_fail_with = "reason")] will produce a message
    //
    let expected_failure_message = match test_function.failure_reason() {
        Some(reason) => reason,
        None => return TestStatus::Pass,
    };

    let expected_failure_message_matches = got_error == expected_failure_message;
    if expected_failure_message_matches {
        return TestStatus::Pass;
    }

    // The expected failure message does not match the actual failure message
    TestStatus::Fail {
        message: format!(
            "\nerror: Test failed with the wrong message. \nExpected: {} \nGot: {}",
            test_function.failure_reason().unwrap_or_default(),
            got_error.trim_matches('\'')
        ),
    }
}
