use acvm::Backend;
use clap::Args;
use nargo::package::Package;
use noirc_driver::CompileOptions;
use noirc_frontend::graph::CrateName;

use crate::{
    cli::compile_cmd::compile_circuit, errors::CliError, find_package_manifest,
    manifest::resolve_workspace_from_toml, prepare_package,
};

use super::NargoConfig;

/// Provides detailed information on a circuit
/// Current information provided:
/// 1. The number of ACIR opcodes
/// 2. Counts the final number gates in the circuit used by a backend
#[derive(Debug, Clone, Args)]
pub(crate) struct InfoCommand {
    /// The name of the package to detail
    #[clap(long)]
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: InfoCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = find_package_manifest(&config.program_dir)?;
    let workspace = resolve_workspace_from_toml(&toml_path, args.package)?;

    for package in &workspace {
        count_opcodes_and_gates_in_package(backend, package, &args.compile_options)?;
    }

    Ok(())
}

fn count_opcodes_and_gates_in_package<B: Backend>(
    backend: &B,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let (mut context, crate_id) = prepare_package(package);
    let compiled_program = compile_circuit(backend, &mut context, crate_id, compile_options)?;

    let num_opcodes = compiled_program.circuit.opcodes.len();

    println!(
        "[{}] Total ACIR opcodes generated for language {:?}: {}",
        package.name,
        backend.np_language(),
        num_opcodes
    );

    let exact_circuit_size = backend
        .get_exact_circuit_size(&compiled_program.circuit)
        .map_err(CliError::ProofSystemCompilerError)?;
    println!("[{}] Backend circuit size: {exact_circuit_size}", package.name);

    Ok(())
}
