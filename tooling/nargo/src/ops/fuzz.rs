use std::path::PathBuf;

use acvm::{
    acir::native_types::{WitnessMap, WitnessStack},
    brillig_vm::BranchToFeatureMap,
    BlackBoxFunctionSolver, FieldElement,
};
use noirc_abi::{input_parser::json::serialize_to_json, InputMap};
use noirc_driver::{compile_no_check, CompileOptions};
use noirc_errors::FileDiagnostic;
use noirc_frontend::hir::{def_map::FuzzingHarness, Context};

use crate::{
    errors::try_to_diagnose_runtime_error, ops::execute::execute_program_with_brillig_fuzzing,
};

use super::{execute_program, DefaultForeignCallExecutor};

pub enum FuzzingRunStatus {
    Pass,
    Fail {
        message: String,
        counterexample: Option<String>,
        error_diagnostic: Option<FileDiagnostic>,
    },
    CompileError(FileDiagnostic),
}

impl FuzzingRunStatus {
    pub fn failed(&self) -> bool {
        !matches!(self, FuzzingRunStatus::Pass)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_fuzzing_harness<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    context: &mut Context,
    fuzzing_harness: &FuzzingHarness,
    show_output: bool,
    foreign_call_resolver_url: Option<&str>,
    root_path: Option<PathBuf>,
    package_name: Option<String>,
    config: &CompileOptions,
) -> FuzzingRunStatus {
    let fuzzing_harness_has_no_arguments = context
        .def_interner
        .function_meta(&fuzzing_harness.get_id())
        .function_signature()
        .0
        .is_empty();

    if fuzzing_harness_has_no_arguments {
        return FuzzingRunStatus::Fail {
            message: ("Fuzzing harness has no arguments".to_owned()),
            counterexample: (None),
            error_diagnostic: (None),
        };
    }
    // Disable forced brillig
    let acir_config = CompileOptions { force_brillig: false, ..config.clone() };
    let brillig_config = CompileOptions { force_brillig: true, ..config.clone() };

    let acir_program =
        compile_no_check(context, &acir_config, fuzzing_harness.get_id(), None, false);
    let acir_program_copy =
        if acir_program.is_ok() { Some(acir_program.as_ref().unwrap().clone()) } else { None };
    let brillig_program =
        compile_no_check(context, &brillig_config, fuzzing_harness.get_id(), None, false);
    match (acir_program, brillig_program) {
        // Good for us, run fuzzer
        (Ok(acir_program), Ok(brillig_program)) => {
            #[cfg(target_arch = "wasm32")]
            {
                // We currently don't support fuzz testing on wasm32 as the u128 strategies do not exist on this platform.
                FuzzingRunStatus::Fail {
                    message: "Fuzz tests are not supported on wasm32".to_string(),
                    error_diagnostic: None,
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                use acvm::acir::circuit::Program;
                use noir_greybox_fuzzer::FuzzedExecutor;

                let acir_executor = |program: &Program<FieldElement>,
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

                let brillig_executor = |program: &Program<FieldElement>,
                                        initial_witness: WitnessMap<FieldElement>,
                                        location_to_feature_map: &BranchToFeatureMap|
                 -> Result<
                    (WitnessStack<FieldElement>, Option<Vec<u32>>),
                    String,
                > {
                    execute_program_with_brillig_fuzzing(
                        program,
                        initial_witness,
                        blackbox_solver,
                        &mut DefaultForeignCallExecutor::<FieldElement>::new(
                            false,
                            foreign_call_resolver_url,
                            root_path.clone(),
                            package_name.clone(),
                        ),
                        Some(location_to_feature_map),
                    )
                    .map_err(|err| err.to_string())
                };
                let abi = acir_program.abi.clone();
                let mut fuzzer = FuzzedExecutor::new(
                    acir_program.into(),
                    brillig_program.into(),
                    acir_executor,
                    brillig_executor,
                );

                let result = fuzzer.fuzz();
                if result.success {
                    FuzzingRunStatus::Pass
                } else {
                    let unwrapped_acir_program = acir_program_copy.unwrap();
                    let initial_witness = unwrapped_acir_program
                        .abi
                        .encode(
                            &result
                                .counterexample
                                .clone()
                                .expect("There should be a failing witness"),
                            None,
                        )
                        .unwrap();
                    let execution_failure = execute_program(
                        &unwrapped_acir_program.program,
                        initial_witness,
                        blackbox_solver,
                        &mut DefaultForeignCallExecutor::<FieldElement>::new(
                            false,
                            foreign_call_resolver_url,
                            root_path.clone(),
                            package_name.clone(),
                        ),
                    );
                    let execution_error = match execution_failure {
                        Err(err) => err,
                        Ok(..) => panic!("Program is flakey"),
                    };
                    FuzzingRunStatus::Fail {
                        message: result.reason.unwrap_or_default(),
                        counterexample: Some(
                            serialize_to_json(&result.counterexample.expect("huh"), &abi)
                                .expect("Huh"),
                        ),
                        error_diagnostic: try_to_diagnose_runtime_error(
                            &execution_error,
                            &unwrapped_acir_program.abi,
                            &unwrapped_acir_program.debug,
                        ),
                    }
                }
            }
        }
        (Err(err), ..) | (.., Err(err)) => {
            // For now just return the error
            FuzzingRunStatus::CompileError(err.into())
        }
    }
}
