use std::path::PathBuf;

use acvm::{
    acir::native_types::{WitnessMap, WitnessStack},
    brillig_vm::BranchToFeatureMap,
    BlackBoxFunctionSolver, FieldElement,
};
use noir_greybox_fuzzer::{
    AcirAndBrilligPrograms, ErrorAndCoverage, FuzzTestResult, FuzzedExecutorFailureConfiguration,
    FuzzedExecutorFolderConfiguration, WitnessAndCoverage,
};
use noirc_abi::{Abi, InputMap};
use noirc_driver::{compile_no_check, CompileOptions};
use noirc_errors::FileDiagnostic;
use noirc_frontend::hir::{def_map::FuzzingHarness, Context};

use crate::{
    errors::try_to_diagnose_runtime_error, ops::execute::execute_program_with_brillig_fuzzing,
};
use crate::{foreign_calls::DefaultForeignCallExecutor, PrintOutput};

use super::execute_program;

/// Folder configuration for fuzzing
pub struct FuzzFolderConfig {
    /// Corpus folder
    pub corpus_folder: Option<String>,
    /// Minimized corpus folder
    pub minimized_corpus_folder: Option<String>,
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
        error_diagnostic: Option<FileDiagnostic>,
    },
    MinimizationFailure {
        message: String,
    },
    ForeignCallFailure {
        message: String,
    },
    CompileError(FileDiagnostic),
}

impl FuzzingRunStatus {
    pub fn failed(&self) -> bool {
        !matches!(
            self,
            FuzzingRunStatus::ExecutionPass | FuzzingRunStatus::MinimizationFailure { .. }
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_fuzzing_harness<B: BlackBoxFunctionSolver<FieldElement> + Default>(
    context: &mut Context,
    fuzzing_harness: &FuzzingHarness,
    show_output: bool,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
    compile_config: &CompileOptions,
    fuzz_folder_config: &FuzzFolderConfig,
    num_threads: usize,
) -> FuzzingRunStatus {
    let fuzzing_harness_has_no_arguments = context
        .def_interner
        .function_meta(&fuzzing_harness.get_id())
        .function_signature()
        .0
        .is_empty();

    if fuzzing_harness_has_no_arguments {
        return FuzzingRunStatus::ExecutionFailure {
            message: ("Fuzzing harness has no arguments".to_owned()),
            counterexample: (None),
            error_diagnostic: (None),
        };
    }
    // Disable forced brillig
    let acir_config = CompileOptions { force_brillig: false, ..compile_config.clone() };
    let brillig_config = CompileOptions { force_brillig: true, ..compile_config.clone() };

    let acir_program =
        compile_no_check(context, &acir_config, fuzzing_harness.get_id(), None, false);
    let acir_program_copy = if let Ok(acir_program_internal) = &acir_program {
        Some(acir_program_internal.clone())
    } else {
        None
    };
    let brillig_program =
        compile_no_check(context, &brillig_config, fuzzing_harness.get_id(), None, false);
    let brillig_program_copy = if let Ok(brillig_progam_internal) = &brillig_program {
        Some(brillig_progam_internal.clone())
    } else {
        None
    };
    match (acir_program, brillig_program) {
        // Good for us, run fuzzer
        (Ok(acir_program), Ok(brillig_program)) => {
            #[cfg(target_arch = "wasm32")]
            {
                // We currently don't support fuzz testing on wasm32 as the u128 strategies do not exist on this platform.
                FuzzingRunStatus::ExecutionFailure {
                    message: "Fuzz tests are not supported on wasm32".to_string(),
                    error_diagnostic: None,
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                use acvm::acir::circuit::Program;
                use noir_greybox_fuzzer::FuzzedExecutor;

                let acir_error_types = acir_program.abi.error_types.clone();
                let acir_executor = |program: &Program<FieldElement>,
                                     initial_witness: WitnessMap<FieldElement>|
                 -> Result<WitnessStack<FieldElement>, String> {
                    execute_program(
                        program,
                        initial_witness,
                        &B::default(),
                        &mut DefaultForeignCallExecutor::new(
                            PrintOutput::None,
                            foreign_call_resolver_url,
                            root_path.clone(),
                            package_name.clone(),
                        ),
                    )
                    .map_err(|err| {
                        err.to_string()
                            + ": "
                            + &err
                                .user_defined_failure_message(&acir_error_types)
                                .unwrap_or("<no message>".to_owned())
                    })
                };

                let brillig_error_types = brillig_program.abi.error_types.clone();
                let brillig_executor =
                    |program: &Program<FieldElement>,
                     initial_witness: WitnessMap<FieldElement>,
                     location_to_feature_map: &BranchToFeatureMap|
                     -> Result<WitnessAndCoverage, ErrorAndCoverage> {
                        execute_program_with_brillig_fuzzing(
                            program,
                            initial_witness,
                            &B::default(),
                            &mut DefaultForeignCallExecutor::new(
                                if show_output { PrintOutput::Stdout } else { PrintOutput::None },
                                foreign_call_resolver_url,
                                root_path.clone(),
                                package_name.clone(),
                            ),
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
                let mut fuzzer = FuzzedExecutor::new(
                    acir_and_brillig_programs,
                    acir_executor,
                    brillig_executor,
                    &package_name.clone().unwrap(),
                    context.def_interner.function_name(&fuzzing_harness.get_id()),
                    num_threads,
                    FuzzedExecutorFailureConfiguration {
                        fail_on_specific_asserts: fuzzing_harness.only_fail_enabled(),
                        failure_reason: fuzzing_harness.failure_reason(),
                    },
                    FuzzedExecutorFolderConfiguration {
                        corpus_folder: fuzz_folder_config.corpus_folder.clone(),
                        minimized_corpus_folder: fuzz_folder_config.minimized_corpus_folder.clone(),
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

                        // Execute the program with the failing witness
                        let execution_failure = execute_program(
                            &unwrapped_acir_program.program,
                            initial_witness,
                            &B::default(),
                            &mut DefaultForeignCallExecutor::new(
                                PrintOutput::None,
                                foreign_call_resolver_url,
                                root_path.clone(),
                                package_name.clone(),
                            ),
                        );
                        let error_diagnostic = match execution_failure {
                            Err(err) => try_to_diagnose_runtime_error(
                                &err,
                                &unwrapped_acir_program.abi,
                                &unwrapped_acir_program.debug,
                            ),
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
                                    &mut DefaultForeignCallExecutor::new(
                                        PrintOutput::None,
                                        foreign_call_resolver_url,
                                        root_path.clone(),
                                        package_name.clone(),
                                    ),
                                );
                                match execution_failure{
                                    Err(err) => try_to_diagnose_runtime_error(
                                &err,
                                &unwrapped_brillig_program.abi,
                                &unwrapped_brillig_program.debug,
                            ),
                                    Ok(..) => panic!("The program being executed or the system is flakey. Found a failing testcase that didn't fail on reexecution")
                                }
                            }
                        };
                        FuzzingRunStatus::ExecutionFailure {
                            message: program_failure_result.failure_reason,
                            counterexample: Some((program_failure_result.counterexample, abi)),
                            error_diagnostic,
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
        }
        (Err(err), ..) | (.., Err(err)) => {
            // For now just return the error
            FuzzingRunStatus::CompileError(err.into())
        }
    }
}
