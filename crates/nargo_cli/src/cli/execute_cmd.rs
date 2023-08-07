use acvm::acir::circuit::OpcodeLabel;
use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::Backend;
use clap::Args;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::ops::{execute_function, optimize_program, SolvedFunction};
use nargo::package::Package;
use nargo::NargoError;
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::{Abi, InputMap};
use noirc_driver::{CompileOptions, CompiledProgram};
use noirc_errors::{debug_info::DebugInfo, CustomDiagnostic};
use noirc_frontend::graph::CrateName;
use noirc_frontend::hir::Context;

use super::compile_cmd::compile_package;
use super::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use super::NargoConfig;
use crate::errors::CliError;
use crate::find_package_manifest;
use crate::manifest::resolve_workspace_from_toml;

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Write the execution witness to named file
    witness_name: Option<String>,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the package to execute
    #[clap(long)]
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: ExecuteCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = find_package_manifest(&config.program_dir)?;
    let workspace = resolve_workspace_from_toml(&toml_path, args.package)?;
    let witness_dir = &workspace.target_directory_path();

    for package in &workspace {
        let solved_functions =
            execute_package(backend, package, &args.prover_name, &args.compile_options)?;

        for func in solved_functions {
            println!("[{}] Circuit witness successfully solved", package.name);
            if let Some(return_value) = func.return_value {
                println!("[{}] Circuit output: {return_value:?}", package.name);
            }
            if let Some(witness_name) = &args.witness_name {
                let witness_path = save_witness_to_dir(func.witness, witness_name, witness_dir)?;

                println!("[{}] Witness saved to {}", package.name, witness_path.display());
            }
        }
    }
    Ok(())
}

fn execute_package<B: Backend>(
    backend: &B,
    package: &Package,
    prover_name: &str,
    compile_options: &CompileOptions,
) -> Result<Vec<SolvedFunction>, CliError<B>> {
    let (context, compiled_program) = compile_package(backend, package, compile_options)?;
    let optimized_program = optimize_program(backend, compiled_program)?;

    let mut solved_functions = Vec::new();
    for func in optimized_program.functions {
        // Parse the initial witness values from Prover.toml
        let (inputs_map, _) =
            read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &func.abi)?;

        let solved_func = match execute_function(backend, func, inputs_map) {
            Err(err @ NargoError::UnsatisfiedConstraint(opcode_idx)) => {
                // TODO: This should resolve to a ReportedErrors error type
                report_unsatisfied_constraint_error(opcode_idx, &func.debug, &context);
                return Err(err.into());
            }
            Err(err) => return Err(err.into()),
            Ok(func) => func,
        };

        solved_functions.push(solved_func);
    }

    Ok(solved_functions)
}

fn report_unsatisfied_constraint_error(opcode_idx: usize, debug: &DebugInfo, context: &Context) {
    if let Some(loc) = debug.opcode_location(opcode_idx) {
        noirc_errors::reporter::report(
            &context.file_manager,
            &CustomDiagnostic::simple_error(
                "Unsatisfied constraint".to_string(),
                "Constraint failed".to_string(),
                loc.span,
            ),
            Some(loc.file),
            false,
        );
    }
}
