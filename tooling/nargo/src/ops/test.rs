use std::{fs::OpenOptions, path::PathBuf};

use acvm::{
    AcirField, BlackBoxFunctionSolver, FieldElement,
    acir::{
        brillig::ForeignCallResult,
        native_types::{WitnessMap, WitnessStack},
    },
    pwg::ForeignCallWaitInfo,
};
use noirc_abi::{Abi, input_parser::json::serialize_to_json};
use noirc_driver::{
    CompileError, CompileOptions, CompiledProgram, DEFAULT_EXPRESSION_WIDTH, compile_no_check,
};
use noirc_errors::{CustomDiagnostic, debug_info::DebugInfo};
use noirc_frontend::{
    hir::{
        Context,
        def_map::{FuzzingHarness, TestFunction},
    },
    token::{FuzzingScope, TestScope},
};

use crate::{
    NargoError,
    errors::try_to_diagnose_runtime_error,
    foreign_calls::{
        ForeignCallError, ForeignCallExecutor, layers, transcript::LoggingForeignCallExecutor,
    },
};

use super::{
    FuzzExecutionConfig, FuzzFolderConfig, FuzzingRunStatus, execute_program, run_fuzzing_harness,
};

#[derive(Debug)]
pub enum TestStatus {
    Pass,
    Fail { message: String, error_diagnostic: Option<CustomDiagnostic> },
    Skipped,
    CompileError(CustomDiagnostic),
}

impl TestStatus {
    pub fn failed(&self) -> bool {
        !matches!(self, TestStatus::Pass | TestStatus::Skipped)
    }
}

pub struct FuzzConfig {
    pub folder_config: FuzzFolderConfig,
    pub execution_config: FuzzExecutionConfig,
}

/// Runs a test function. This will either run the test or fuzz it, depending on whether the function has arguments.
#[allow(clippy::too_many_arguments)]
pub fn run_or_fuzz_test<'a, W, B, F, E>(
    blackbox_solver: &B,
    context: &mut Context,
    test_function: &TestFunction,
    output: W,
    package_name: String,
    config: &CompileOptions,
    fuzz_config: FuzzConfig,
    build_foreign_call_executor: F,
) -> TestStatus
where
    W: std::io::Write + 'a,
    B: BlackBoxFunctionSolver<FieldElement> + Default,
    F: Fn(Box<dyn std::io::Write + 'a>, layers::Unhandled) -> E + Sync,
    E: ForeignCallExecutor<FieldElement>,
{
    if test_function.has_arguments {
        fuzz_test::<B, F, E>(
            context,
            test_function,
            package_name,
            config,
            fuzz_config,
            build_foreign_call_executor,
        )
    } else {
        run_test::<W, B, F, E>(
            blackbox_solver,
            context,
            test_function,
            output,
            config,
            build_foreign_call_executor,
        )
    }
}

/// Runs a test function. This assumes the function has no arguments.
pub fn run_test<'a, W, B, F, E>(
    blackbox_solver: &B,
    context: &mut Context,
    test_function: &TestFunction,
    output: W,
    config: &CompileOptions,
    build_foreign_call_executor: F,
) -> TestStatus
where
    W: std::io::Write + 'a,
    B: BlackBoxFunctionSolver<FieldElement>,
    F: Fn(Box<dyn std::io::Write + 'a>, layers::Unhandled) -> E,
    E: ForeignCallExecutor<FieldElement>,
{
    match compile_no_check(context, config, test_function.id, None, false) {
        Ok(compiled_program) => run_test_impl(
            blackbox_solver,
            compiled_program,
            test_function,
            output,
            config,
            build_foreign_call_executor,
        ),
        Err(err) => test_status_program_compile_fail(err, test_function),
    }
}

fn run_test_impl<'a, W, B, F, E>(
    blackbox_solver: &B,
    compiled_program: CompiledProgram,
    test_function: &TestFunction,
    output: W,
    config: &CompileOptions,
    build_foreign_call_executor: F,
) -> TestStatus
where
    W: std::io::Write + 'a,
    B: BlackBoxFunctionSolver<FieldElement>,
    F: Fn(Box<dyn std::io::Write + 'a>, layers::Unhandled) -> E,
    E: ForeignCallExecutor<FieldElement>,
{
    // Do the same optimizations as `compile_cmd`.
    let target_width = config.expression_width.unwrap_or(DEFAULT_EXPRESSION_WIDTH);
    let compiled_program = crate::ops::transform_program(compiled_program, target_width);

    let ignore_foreign_call_failures =
        std::env::var("NARGO_IGNORE_TEST_FAILURES_FROM_FOREIGN_CALLS")
            .is_ok_and(|var| &var == "true");

    let writer: Box<dyn std::io::Write> = match std::env::var("NARGO_TEST_FOREIGN_CALL_LOG") {
        Err(_) => Box::new(std::io::empty()),
        Ok(s) if s == "stdout" => Box::new(std::io::stdout()),
        Ok(s) => Box::new(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(PathBuf::from(s))
                .unwrap(),
        ),
    };

    // Run the backend to ensure the PWG evaluates functions like std::hash::pedersen,
    // otherwise constraints involving these expressions will not error.
    // Use a base layer that doesn't handle anything, which we handle in the `execute` below.
    let foreign_call_executor = build_foreign_call_executor(Box::new(output), layers::Unhandled);
    let foreign_call_executor = TestForeignCallExecutor::new(foreign_call_executor);
    let mut foreign_call_executor = LoggingForeignCallExecutor::new(foreign_call_executor, writer);

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

    let foreign_call_executor = foreign_call_executor.executor;

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

/// Runs the fuzzer on a test function. This assumes the function has arguments.
pub fn fuzz_test<'a, B, F, E>(
    context: &mut Context,
    test_function: &TestFunction,
    package_name: String,
    config: &CompileOptions,
    fuzz_config: FuzzConfig,
    build_foreign_call_executor: F,
) -> TestStatus
where
    B: BlackBoxFunctionSolver<FieldElement> + Default,
    F: Fn(Box<dyn std::io::Write + 'a>, layers::Unhandled) -> E + Sync,
    E: ForeignCallExecutor<FieldElement>,
{
    match compile_no_check(context, config, test_function.id, None, false) {
        Ok(_) => fuzz_test_impl::<B, F, E>(
            context,
            test_function,
            package_name,
            config,
            fuzz_config,
            build_foreign_call_executor,
        ),
        Err(err) => test_status_program_compile_fail(err, test_function),
    }
}

fn fuzz_test_impl<'a, B, F, E>(
    context: &mut Context,
    test_function: &TestFunction,
    package_name: String,
    config: &CompileOptions,
    fuzz_config: FuzzConfig,
    build_foreign_call_executor: F,
) -> TestStatus
where
    B: BlackBoxFunctionSolver<FieldElement> + Default,
    F: Fn(Box<dyn std::io::Write + 'a>, layers::Unhandled) -> E + Sync,
    E: ForeignCallExecutor<FieldElement>,
{
    let id = test_function.id;
    let scope = match &test_function.scope {
        TestScope::ShouldFailWith { reason } => {
            FuzzingScope::ShouldFailWith { reason: reason.clone() }
        }
        TestScope::OnlyFailWith { reason } => FuzzingScope::OnlyFailWith { reason: reason.clone() },
        TestScope::None => FuzzingScope::None,
    };
    let location = test_function.location;
    let fuzzing_harness = FuzzingHarness { id, scope, location };

    let mut temporary_dirs_to_delete = Vec::new();

    let mut config_or_temporary_dir = |dir: Option<String>| match dir {
        Some(ref dir) => PathBuf::from(dir),
        None => {
            let corpus_dir = tempfile::tempdir().expect("Couldn't create temporary directory");
            let corpus_dir = corpus_dir.keep();
            temporary_dirs_to_delete.push(corpus_dir.clone());
            corpus_dir
        }
    };

    let corpus_dir = config_or_temporary_dir(fuzz_config.folder_config.corpus_dir);
    let fuzzing_failure_dir =
        config_or_temporary_dir(fuzz_config.folder_config.fuzzing_failure_dir);

    let fuzz_folder_config = FuzzFolderConfig {
        corpus_dir: Some(corpus_dir.to_string_lossy().to_string()),
        fuzzing_failure_dir: Some(fuzzing_failure_dir.to_string_lossy().to_string()),
        minimized_corpus_dir: fuzz_config.folder_config.minimized_corpus_dir,
    };
    let fuzz_execution_config = fuzz_config.execution_config;

    // TODO: show output?
    let show_output = false;
    let fuzz_result = run_fuzzing_harness::<B, _, _>(
        context,
        &fuzzing_harness,
        show_output,
        package_name,
        config,
        &fuzz_folder_config,
        &fuzz_execution_config,
        build_foreign_call_executor,
    );

    for temporary_dir_to_delete in temporary_dirs_to_delete {
        // Not a big deal if we can't delete a temporary directory
        let _ = std::fs::remove_dir_all(temporary_dir_to_delete);
    }

    match fuzz_result {
        FuzzingRunStatus::ExecutionPass | FuzzingRunStatus::MinimizationPass => TestStatus::Pass,
        FuzzingRunStatus::CorpusFailure { message } => {
            let message = format!("Corpus failure: {message}");
            TestStatus::Fail { message, error_diagnostic: None }
        }
        FuzzingRunStatus::ExecutionFailure { message, counterexample, error_diagnostic } => {
            let message = format!("Execution failed: {message}");
            if let Some((input_map, abi)) = &counterexample {
                let input =
                    serialize_to_json(input_map, abi).expect("Couldn't serialize input to JSON");
                let message = format!("{message}\nFailing input: {input}");
                TestStatus::Fail { message, error_diagnostic }
            } else {
                TestStatus::Fail { message, error_diagnostic }
            }
        }
        FuzzingRunStatus::MinimizationFailure { message } => {
            let message = format!("Minimization failed: {message}");
            TestStatus::Fail { message, error_diagnostic: None }
        }
        FuzzingRunStatus::ForeignCallFailure { message } => {
            let message = format!("Foreign call failed: {message}");
            TestStatus::Fail { message, error_diagnostic: None }
        }
        FuzzingRunStatus::CompileError(custom_diagnostic) => {
            TestStatus::CompileError(custom_diagnostic)
        }
    }
}
/// Test function failed to compile
///
/// Note: This could be because the compiler was able to deduce
/// that a constraint was never satisfiable.
/// An example of this is the program `assert(false)`
/// In that case, we check if the test function should fail, and if so, we return `TestStatus::Pass`.
pub fn test_status_program_compile_fail(
    err: CompileError,
    test_function: &TestFunction,
) -> TestStatus {
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
pub fn test_status_program_compile_pass(
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

pub fn check_expected_failure_message(
    test_function: &TestFunction,
    failed_assertion: Option<String>,
    error_diagnostic: Option<CustomDiagnostic>,
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
        .or_else(|| error_diagnostic.as_ref().map(|file_diagnostic| &file_diagnostic.message))
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
pub(crate) struct TestForeignCallExecutor<E> {
    executor: E,
    encountered_unknown_foreign_call: bool,
}

impl<E> TestForeignCallExecutor<E> {
    pub(crate) fn new(executor: E) -> Self {
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
