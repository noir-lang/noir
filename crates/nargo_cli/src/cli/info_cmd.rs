use acvm::Backend;
use clap::Args;
use nargo::{package::Package, prepare_package};
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml};
use noirc_driver::{compile_contracts, CompileOptions};
use noirc_frontend::graph::CrateName;

use crate::{cli::compile_cmd::compile_package, errors::CliError};

use super::{compile_cmd::report_errors, NargoConfig};

/// Provides detailed information on a circuit
///
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
        if package.is_contract() {
            count_opcodes_and_gates_in_contracts(backend, package, &args.compile_options)?;
        } else {
            count_opcodes_and_gates_in_package(backend, package, &args.compile_options)?;
        }
    }

    Ok(())
}

fn count_opcodes_and_gates_in_package<B: Backend>(
    backend: &B,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let (_, compiled_program) = compile_package(backend, package, compile_options)?;

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

fn count_opcodes_and_gates_in_contracts<B: Backend>(
    backend: &B,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let (mut context, crate_id) = prepare_package(package);
    let result = compile_contracts(&mut context, crate_id, compile_options);
    let contracts = report_errors(result, &context, compile_options.deny_warnings)?;

    for contract in contracts {
        let mut function_info = Vec::new();
        for function in contract.functions {
            let num_opcodes = function.bytecode.opcodes.len();
            let exact_circuit_size = backend
                .get_exact_circuit_size(&function.bytecode)
                .map_err(CliError::ProofSystemCompilerError)?;
            function_info.push((function.name, num_opcodes, exact_circuit_size));
        }

        for info in function_info {
            println!("[{}]({}) Total ACIR opcodes generated: {}", contract.name, info.0, info.1,);
            println!("[{}]({}) Backend circuit size: {}", contract.name, info.0, info.2);
        }
    }

    Ok(())
}
