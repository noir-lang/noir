use std::path::PathBuf;

use acvm::{
    acir::{
        brillig::ForeignCallResult,
        native_types::{WitnessMap, WitnessStack},
    },
    pwg::ForeignCallWaitInfo,
    AcirField, BlackBoxFunctionSolver, FieldElement,
};
use noirc_abi::Abi;
use noirc_driver::CompiledProgram;
use noirc_errors::{debug_info::DebugInfo, FileDiagnostic};
use noirc_frontend::hir::def_map::TestFunction;
use noirc_printable_type::ForeignCallError;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    errors::try_to_diagnose_runtime_error,
    foreign_calls::{
        mocker::MockForeignCallExecutor, print::PrintForeignCallExecutor,
        rpc::RPCForeignCallExecutor, ForeignCall, ForeignCallExecutor,
    },
    NargoError,
};

use super::execute_program;

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

pub fn run_test<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    compiled_program: CompiledProgram,
    test_function: &TestFunction,
    show_output: bool,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
) -> TestStatus {
    let test_function_has_no_arguments = compiled_program.abi.parameters.is_empty();
    if test_function_has_no_arguments {
        run_regular_test(
            blackbox_solver,
            test_function,
            compiled_program,
            show_output,
            foreign_call_resolver_url,
            root_path,
            package_name,
        )
    } else {
        run_fuzz_test(
            blackbox_solver,
            compiled_program,
            foreign_call_resolver_url,
            root_path,
            package_name,
        )
    }
}

fn run_regular_test<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    test_function: &TestFunction,
    compiled_program: CompiledProgram,
    show_output: bool,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
) -> TestStatus {
    let mut foreign_call_executor = TestForeignCallExecutor::new(
        show_output,
        foreign_call_resolver_url,
        root_path,
        package_name,
    );

    let circuit_execution = execute_program(
        &compiled_program.program,
        WitnessMap::new(),
        blackbox_solver,
        &mut foreign_call_executor,
    );

    let status = check_test_status(
        test_function,
        compiled_program.abi,
        compiled_program.debug,
        circuit_execution,
    );

    let ignore_foreign_call_failures =
        std::env::var("NARGO_IGNORE_TEST_FAILURES_FROM_FOREIGN_CALLS")
            .is_ok_and(|var| &var == "true");

    if let TestStatus::Fail { .. } = status {
        if ignore_foreign_call_failures && foreign_call_executor.encountered_unknown_foreign_call {
            TestStatus::Skipped
        } else {
            status
        }
    } else {
        status
    }
}

fn run_fuzz_test<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    compiled_program: CompiledProgram,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
) -> TestStatus {
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

        let executor = |program: &Program<FieldElement>,
                        initial_witness: WitnessMap<FieldElement>|
         -> Result<WitnessStack<FieldElement>, String> {
            execute_program(
                program,
                initial_witness,
                blackbox_solver,
                &mut TestForeignCallExecutor::new(
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
            TestStatus::Fail { message: result.reason.unwrap_or_default(), error_diagnostic: None }
        }
    }
}

/// The test function compiled successfully.
///
/// We now check whether execution passed/failed and whether it should have
/// passed/failed to determine the test status.
fn check_test_status(
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

/// A specialized foreign call executor which tracks whether it has encountered any unknown foreign calls
struct TestForeignCallExecutor<F> {
    /// The executor for any [`ForeignCall::Print`] calls.
    printer: Option<PrintForeignCallExecutor>,
    mocker: MockForeignCallExecutor<F>,
    external: Option<RPCForeignCallExecutor>,

    encountered_unknown_foreign_call: bool,
}

impl<F: Default> TestForeignCallExecutor<F> {
    fn new(
        show_output: bool,
        resolver_url: Option<&str>,
        root_path: Option<PathBuf>,
        package_name: Option<String>,
    ) -> Self {
        let id = rand::thread_rng().gen();
        let printer = if show_output { Some(PrintForeignCallExecutor) } else { None };
        let external_resolver = resolver_url.map(|resolver_url| {
            RPCForeignCallExecutor::new(resolver_url, id, root_path, package_name)
        });
        TestForeignCallExecutor {
            printer,
            mocker: MockForeignCallExecutor::default(),
            external: external_resolver,
            encountered_unknown_foreign_call: false,
        }
    }
}

impl<F: AcirField + Serialize + for<'a> Deserialize<'a>> ForeignCallExecutor<F>
    for TestForeignCallExecutor<F>
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        // If the circuit has reached a new foreign call opcode then it can't have failed from any previous unknown foreign calls.
        self.encountered_unknown_foreign_call = false;

        let foreign_call_name = foreign_call.function.as_str();
        match ForeignCall::lookup(foreign_call_name) {
            Some(ForeignCall::Print) => {
                if let Some(printer) = &mut self.printer {
                    printer.execute(foreign_call)
                } else {
                    Ok(ForeignCallResult::default())
                }
            }

            Some(
                ForeignCall::CreateMock
                | ForeignCall::SetMockParams
                | ForeignCall::GetMockLastParams
                | ForeignCall::SetMockReturns
                | ForeignCall::SetMockTimes
                | ForeignCall::ClearMock,
            ) => self.mocker.execute(foreign_call),

            None => {
                // First check if there's any defined mock responses for this foreign call.
                match self.mocker.execute(foreign_call) {
                    Err(ForeignCallError::NoHandler(_)) => (),
                    response_or_error => return response_or_error,
                };

                if let Some(external_resolver) = &mut self.external {
                    // If the user has registered an external resolver then we forward any remaining oracle calls there.
                    match external_resolver.execute(foreign_call) {
                        Err(ForeignCallError::NoHandler(_)) => (),
                        response_or_error => return response_or_error,
                    };
                }

                self.encountered_unknown_foreign_call = true;

                // If all executors have no handler for the given foreign call then we cannot
                // return a correct response to the ACVM. The best we can do is to return an empty response,
                // this allows us to ignore any foreign calls which exist solely to pass information from inside
                // the circuit to the environment (e.g. custom logging) as the execution will still be able to progress.
                //
                // We optimistically return an empty response for all oracle calls as the ACVM will error
                // should a response have been required.
                Ok(ForeignCallResult::default())
            }
        }
    }
}
