use acvm::acir::native_types::{WitnessMap, WitnessStack};
use acvm::FieldElement;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;

use nargo::artifacts::debug::DebugArtifact;
use nargo::constants::PROVER_INPUT_FILE;
use nargo::package::Package;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::InputMap;
use noirc_driver::{
    file_manager_with_stdlib, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_frontend::graph::CrateName;

use super::debug_cmd::compile_bin_package_for_debugging;
use super::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use crate::errors::CliError;

use super::NargoConfig;

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct TraceCommand {
    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the package to execute
    #[clap(long)]
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// Directory where to store trace.json
    #[clap(long, short)]
    trace_dir: String,
}

pub(crate) fn run(args: TraceCommand, config: NargoConfig) -> Result<(), CliError> {
    let acir_mode = false;
    let skip_instrumentation = false;

    let toml_path = get_package_manifest(&config.program_dir)?;
    let selection = args.package.map_or(PackageSelection::DefaultOrAll, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let Some(package) = workspace.into_iter().find(|p| p.is_binary()) else {
        println!(
            "No matching binary packages found in workspace. Only binary packages can be debugged."
        );
        return Ok(());
    };

    let compiled_program = compile_bin_package_for_debugging(
        &workspace,
        package,
        acir_mode,
        skip_instrumentation,
        args.compile_options.clone(),
    )?;

    let (return_value, solved_witness) =
        trace_program_and_decode(compiled_program, package, &args.prover_name)?;

    Ok(())
}

fn trace_program_and_decode(
    program: CompiledProgram,
    package: &Package,
    prover_name: &str,
) -> Result<(Option<InputValue>, Option<WitnessMap<FieldElement>>), CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &program.abi)?;
    let solved_witness = trace_program(&program, &inputs_map)?;
    let public_abi = program.abi.public_abi();

    match solved_witness {
        Some(witness) => {
            let (_, return_value) = public_abi.decode(&witness)?;
            Ok((return_value, Some(witness)))
        }
        None => Ok((None, None)),
    }
}

pub(crate) fn trace_program(
    compiled_program: &CompiledProgram,
    inputs_map: &InputMap,
) -> Result<Option<WitnessMap<FieldElement>>, CliError> {
    let initial_witness = compiled_program.abi.encode(inputs_map, None)?;

    let debug_artifact = DebugArtifact {
        debug_symbols: compiled_program.debug.clone(),
        file_map: compiled_program.file_map.clone(),
    };

    noir_tracer::trace_circuit(
        &Bn254BlackBoxSolver,
        &compiled_program.program.functions[0],
        &debug_artifact,
        initial_witness,
        &compiled_program.program.unconstrained_functions,
    )
    .map_err(CliError::from)
}
