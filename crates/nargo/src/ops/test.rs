use acvm::{acir::native_types::WitnessMap, Backend};
use noirc_driver::{compile_no_check, CompileOptions};
use noirc_errors::FileDiagnostic;
use noirc_frontend::hir::{def_map::TestFunction, Context};

use super::execute_circuit;

pub enum TestStatus {
    Pass,
    Fail { message: String },
    CompileError(FileDiagnostic),
}

pub fn run_test<B: Backend>(
    backend: &B,
    test_function: TestFunction,
    context: &Context,
    show_output: bool,
    config: &CompileOptions,
) -> TestStatus {
    let program = compile_no_check(context, config, test_function.get_id());
    match program {
        Ok(program) => {
            // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
            // otherwise constraints involving these expressions will not error.
            let circuit_execution =
                execute_circuit(backend, program.circuit, WitnessMap::new(), show_output);

            if test_function.should_fail() {
                match circuit_execution {
                    Ok(_) => TestStatus::Fail {
                        // TODO: Improve color variations on this message
                        message: "error: Test passed when it should have failed".to_string(),
                    },
                    Err(_) => TestStatus::Pass,
                }
            } else {
                match circuit_execution {
                    Ok(_) => TestStatus::Pass,
                    Err(error) => TestStatus::Fail { message: error.to_string() },
                }
            }
        }
        // Test function failed to compile
        //
        // Note: This could be because the compiler was able to deduce
        // that a constraint was never satisfiable.
        // An example of this is the program `assert(false)`
        //  In that case, we check if the test function should fail, and if so, we return Ok.
        Err(diag) => {
            // The test has failed compilation, but it should never fail. Report error.
            if !test_function.should_fail() {
                return TestStatus::CompileError(diag);
            }

            // The test has failed compilation, check if it is because the program is never satisfiable.
            // If it is never satisfiable, then this is the expected behavior.
            let program_is_never_satisfiable =
                diag.diagnostic.message.contains("Failed constraint");
            if program_is_never_satisfiable {
                return TestStatus::Pass;
            }

            // The test has failed compilation, but its a compilation error. Report error
            TestStatus::CompileError(diag)
        }
    }
}
