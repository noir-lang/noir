use acvm::{
    BlackBoxFunctionSolver, FieldElement,
    acir::native_types::{WitnessMap, WitnessStack},
    brillig_vm::BranchToFeatureMap,
};
use noir_greybox_fuzzer::{
    AcirAndBrilligPrograms, ErrorAndCoverage, ErrorAndWitness, FuzzTestResult,
    FuzzedExecutorExecutionConfiguration, FuzzedExecutorFailureConfiguration,
    FuzzedExecutorFolderConfiguration, WitnessAndCoverage,
};
use noirc_abi::{Abi, InputMap};
use noirc_driver::{CompileOptions, compile_no_check};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::{Context, def_map::FuzzingHarness};

use crate::foreign_calls::ForeignCallExecutor;
use crate::{
    errors::try_to_diagnose_runtime_error,
    foreign_calls::layers,
    ops::{
        execute::execute_program_with_acir_fuzzing, execute::execute_program_with_brillig_fuzzing,
        test::TestForeignCallExecutor,
    },
};

use super::execute_program;
/// Configuration for fuzzing loop execution
pub struct FuzzExecutionConfig {
    /// Number of threads to use for fuzzing
    pub num_threads: usize,
    /// Maximum time in seconds to spend fuzzing (default: no timeout)
    pub timeout: u64,
    /// Whether to output progress to stdout or not.
    pub show_progress: bool,
    /// Maximum number of executions of ACIR and Brillig (default: no limit)
    pub max_executions: usize,
}

/// Folder configuration for fuzzing
pub struct FuzzFolderConfig {
    /// Corpus folder
    pub corpus_dir: Option<String>,
    /// Minimized corpus folder
    pub minimized_corpus_dir: Option<String>,
    /// Fuzzing failure folder
    pub fuzzing_failure_dir: Option<String>,
}

pub enum FuzzingRunStatus {
    ExecutionPass,
    MinimizationPass,
    CorpusFailure {
        message: String,
    },
    ExecutionFailure {
        message: String,
        counterexample: Option<(InputMap, Abi)>,
        error_diagnostic: Option<CustomDiagnostic>,
    },
    MinimizationFailure {
        message: String,
    },
    ForeignCallFailure {
        message: String,
    },
    CompileError(CustomDiagnostic),
}

impl FuzzingRunStatus {
    pub fn failed(&self) -> bool {
        !matches!(self, FuzzingRunStatus::ExecutionPass | FuzzingRunStatus::MinimizationPass)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_fuzzing_harness<'a, B, F, E>(
    context: &mut Context,
    fuzzing_harness: &FuzzingHarness,
    show_output: bool,
    package_name: String,
    compile_config: &CompileOptions,
    fuzz_folder_config: &FuzzFolderConfig,
    fuzz_execution_config: &FuzzExecutionConfig,
    build_foreign_call_executor: F,
) -> FuzzingRunStatus
where
    B: BlackBoxFunctionSolver<FieldElement> + Default,
    F: Fn(Box<dyn std::io::Write + 'a>, layers::Unhandled) -> E + Sync,
    E: ForeignCallExecutor<FieldElement>,
{
    let fuzzing_harness_has_no_arguments =
        context.def_interner.function_meta(&fuzzing_harness.id).parameters.is_empty();

    if fuzzing_harness_has_no_arguments {
        return FuzzingRunStatus::ExecutionFailure {
            message: "Fuzzing harness has no arguments".to_owned(),
            counterexample: None,
            error_diagnostic: None,
        };
    }
    // Disable forced brillig
    let acir_config = CompileOptions { force_brillig: false, ..compile_config.clone() };
    let brillig_config = CompileOptions { force_brillig: true, ..compile_config.clone() };

    let acir_program = compile_no_check(context, &acir_config, fuzzing_harness.id, None, false);

    // We need to clone the acir program because it will be moved into the fuzzer
    // and we need to keep the original program for the error message and callstack
    let acir_program_copy = if let Ok(acir_program_internal) = &acir_program {
        Some(acir_program_internal.clone())
    } else {
        None
    };
    let brillig_program =
        compile_no_check(context, &brillig_config, fuzzing_harness.id, None, false);
    let brillig_program_copy = if let Ok(brillig_program_internal) = &brillig_program {
        Some(brillig_program_internal.clone())
    } else {
        None
    };
    match (acir_program, brillig_program) {
        // Good for us, run fuzzer
        (Ok(acir_program), Ok(brillig_program)) => {
            use acvm::acir::circuit::Program;
            use noir_greybox_fuzzer::FuzzedExecutor;

            let acir_error_types = acir_program.abi.error_types.clone();
            let acir_executor =
                |program: &Program<FieldElement>,
                 initial_witness: WitnessMap<FieldElement>|
                 -> Result<WitnessStack<FieldElement>, ErrorAndWitness> {
                    let foreign_call_executor =
                        build_foreign_call_executor(output(show_output), layers::Unhandled);
                    let mut foreign_call_executor =
                        TestForeignCallExecutor::new(foreign_call_executor);
                    execute_program_with_acir_fuzzing(
                        program,
                        initial_witness,
                        &B::default(),
                        &mut foreign_call_executor,
                    )
                    .map_err(|(nargo_err, witness)| {
                        (
                            nargo_err.to_string()
                                + ": "
                                + &nargo_err
                                    .user_defined_failure_message(&acir_error_types)
                                    .unwrap_or("<no message>".to_owned()),
                            witness,
                        )
                    })
                };

            let brillig_error_types = brillig_program.abi.error_types.clone();
            let brillig_executor = |program: &Program<FieldElement>,
                                    initial_witness: WitnessMap<FieldElement>,
                                    location_to_feature_map: &BranchToFeatureMap|
             -> Result<WitnessAndCoverage, ErrorAndCoverage> {
                let foreign_call_executor =
                    build_foreign_call_executor(output(show_output), layers::Unhandled);
                let mut foreign_call_executor = TestForeignCallExecutor::new(foreign_call_executor);
                execute_program_with_brillig_fuzzing(
                    program,
                    initial_witness,
                    &B::default(),
                    &mut foreign_call_executor,
                    Some(location_to_feature_map),
                )
                .map_err(|(nargo_err, brillig_coverage)| {
                    (
                        nargo_err.to_string()
                            + ": "
                            + &nargo_err
                                .user_defined_failure_message(&brillig_error_types)
                                .unwrap_or("<no message>".to_owned()),
                        brillig_coverage,
                    )
                })
            };
            let abi = acir_program.abi.clone();
            let acir_and_brillig_programs = AcirAndBrilligPrograms {
                acir_program: acir_program.into(),
                brillig_program: brillig_program.into(),
            };
            let failure_configuration = match fuzzing_harness.failure_reason() {
                Some(failure_reason) => {
                    if fuzzing_harness.should_fail_enabled() {
                        FuzzedExecutorFailureConfiguration::ShouldFailWith(failure_reason)
                    } else {
                        assert!(fuzzing_harness.only_fail_enabled());
                        FuzzedExecutorFailureConfiguration::OnlyFailWith(failure_reason)
                    }
                }

                None => {
                    if fuzzing_harness.should_fail_enabled() {
                        FuzzedExecutorFailureConfiguration::ShouldFail
                    } else {
                        FuzzedExecutorFailureConfiguration::None
                    }
                }
            };
            let mut fuzzer = FuzzedExecutor::new(
                acir_and_brillig_programs,
                acir_executor,
                brillig_executor,
                &package_name.clone(),
                context.def_interner.function_name(&fuzzing_harness.id),
                FuzzedExecutorExecutionConfiguration {
                    num_threads: fuzz_execution_config.num_threads,
                    timeout: fuzz_execution_config.timeout,
                    show_progress: fuzz_execution_config.show_progress,
                    max_executions: fuzz_execution_config.max_executions,
                },
                failure_configuration,
                FuzzedExecutorFolderConfiguration {
                    corpus_dir: fuzz_folder_config.corpus_dir.clone(),
                    minimized_corpus_dir: fuzz_folder_config.minimized_corpus_dir.clone(),
                },
            );

            let result = fuzzer.fuzz();
            match result {
                FuzzTestResult::Success => FuzzingRunStatus::ExecutionPass,
                FuzzTestResult::ProgramFailure(program_failure_result) => {
                    // Collect failing callstack
                    let unwrapped_acir_program = acir_program_copy.unwrap();
                    let initial_witness = unwrapped_acir_program
                        .abi
                        .encode(&program_failure_result.counterexample.clone(), None)
                        .unwrap();
                    let foreign_call_executor =
                        build_foreign_call_executor(output(show_output), layers::Unhandled);
                    let mut foreign_call_executor =
                        TestForeignCallExecutor::new(foreign_call_executor);
                    // Execute the program with the failing witness
                    // Execute the program with the failing witness
                    let execution_failure = execute_program(
                        &unwrapped_acir_program.program,
                        initial_witness,
                        &B::default(),
                        &mut foreign_call_executor,
                    );
                    match execution_failure {
                        Err(err) => FuzzingRunStatus::ExecutionFailure {
                            message: if fuzzing_harness.should_fail_enabled() {
                                format!(
                                                "Expected failure message \"{}\", but got a different failing assertion",
                                                fuzzing_harness.failure_reason().expect("There should be a failure reason if we detected a different failure reason during fuzzing")
                                            )
                            } else {
                                program_failure_result.failure_reason
                            },
                            counterexample: Some((program_failure_result.counterexample, abi)),
                            error_diagnostic: try_to_diagnose_runtime_error(
                                &err,
                                &unwrapped_acir_program.abi,
                                &unwrapped_acir_program.debug,
                            ),
                        },
                        // Maybe it was the brillig version that failed and we hade a discrepancy?
                        Ok(..) => {
                            // Collect failing callstack from brillig
                            let unwrapped_brillig_program = brillig_program_copy.unwrap();
                            let initial_witness = unwrapped_acir_program
                                .abi
                                .encode(&program_failure_result.counterexample.clone(), None)
                                .unwrap();

                            // Execute the program with the failing witness
                            let execution_failure = execute_program(
                                &unwrapped_brillig_program.program,
                                initial_witness,
                                &B::default(),
                                &mut foreign_call_executor,
                            );
                            match execution_failure {
                                Err(err) => FuzzingRunStatus::ExecutionFailure {
                                    message: if fuzzing_harness.should_fail_enabled() {
                                        format!(
                                                "Expected failure message \"{}\", but got a different failing assertion",
                                                fuzzing_harness.failure_reason().expect("There should be a failure reason if we detected a different failure reason during fuzzing")
                                            )
                                    } else {
                                        program_failure_result.failure_reason
                                    },
                                    counterexample: Some((
                                        program_failure_result.counterexample,
                                        abi,
                                    )),
                                    error_diagnostic: try_to_diagnose_runtime_error(
                                        &err,
                                        &unwrapped_brillig_program.abi,
                                        &unwrapped_brillig_program.debug,
                                    ),
                                },
                                Ok(..) => {
                                    if fuzzing_harness.should_fail_enabled() {
                                        return FuzzingRunStatus::ExecutionFailure {
                                            message:
                                                "Discovered a testcase that should fail but didn't"
                                                    .to_owned(),
                                            counterexample: Some((
                                                program_failure_result.counterexample,
                                                abi,
                                            )),
                                            error_diagnostic: None,
                                        };
                                    }
                                    panic!(
                                        "The program being executed or the system is flakey. Found a failing testcase that didn't fail on re-execution"
                                    )
                                }
                            }
                        }
                    }
                }
                FuzzTestResult::CorpusFailure(error) => {
                    FuzzingRunStatus::CorpusFailure { message: error }
                }
                FuzzTestResult::ForeignCallFailure(error) => {
                    FuzzingRunStatus::ForeignCallFailure { message: error }
                }
                FuzzTestResult::MinimizationFailure(error) => {
                    FuzzingRunStatus::MinimizationFailure { message: error }
                }
                FuzzTestResult::MinimizationSuccess => FuzzingRunStatus::MinimizationPass,
            }
        }
        (Err(err), ..) | (.., Err(err)) => {
            // For now just return the error
            FuzzingRunStatus::CompileError(err.into())
        }
    }
}

fn output(show_output: bool) -> Box<dyn std::io::Write> {
    if show_output { Box::new(std::io::stdout()) } else { Box::new(std::io::empty()) }
}
