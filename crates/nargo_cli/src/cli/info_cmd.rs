use acvm::Backend;
use clap::Args;
use iter_extended::try_vecmap;
use nargo::{package::Package, prepare_package};
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{compile_contracts, CompileOptions};
use noirc_frontend::graph::CrateName;
use prettytable::{row, Table};

use crate::{cli::compile_cmd::compile_package, errors::CliError};

use super::{
    compile_cmd::{optimize_contract, report_errors},
    NargoConfig,
};

/// Provides detailed information on a circuit
///
/// Current information provided:
/// 1. The number of ACIR opcodes
/// 2. Counts the final number gates in the circuit used by a backend
#[derive(Debug, Clone, Args)]
pub(crate) struct InfoCommand {
    /// The name of the package to detail
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Detail all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: InfoCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = find_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;

    let mut package_table = Table::new();
    package_table.add_row(
        row![Fm->"Package", Fm->"Language", Fm->"ACIR Opcodes", Fm->"Backend Circuit Size"],
    );
    let mut contract_table = Table::new();
    contract_table.add_row(row![
        Fm->"Contract",
        Fm->"Function",
        Fm->"Language",
        Fm->"ACIR Opcodes",
        Fm->"Backend Circuit Size"
    ]);

    for package in &workspace {
        if package.is_contract() {
            count_opcodes_and_gates_in_contracts(
                backend,
                package,
                &args.compile_options,
                &mut contract_table,
            )?;
        } else {
            count_opcodes_and_gates_in_package(
                backend,
                package,
                &args.compile_options,
                &mut package_table,
            )?;
        }
    }

    if package_table.len() > 1 {
        package_table.printstd();
    }
    if contract_table.len() > 1 {
        contract_table.printstd();
    }

    Ok(())
}

fn count_opcodes_and_gates_in_package<B: Backend>(
    backend: &B,
    package: &Package,
    compile_options: &CompileOptions,
    table: &mut Table,
) -> Result<(), CliError<B>> {
    let (_, compiled_program) = compile_package(backend, package, compile_options)?;

    let num_opcodes = compiled_program.circuit.opcodes.len();
    let exact_circuit_size = backend
        .get_exact_circuit_size(&compiled_program.circuit)
        .map_err(CliError::ProofSystemCompilerError)?;

    table.add_row(row![
        Fm->format!("{}", package.name),
        format!("{:?}", backend.np_language()),
        Fc->format!("{}", num_opcodes),
        Fc->format!("{}", exact_circuit_size),
    ]);

    Ok(())
}

fn count_opcodes_and_gates_in_contracts<B: Backend>(
    backend: &B,
    package: &Package,
    compile_options: &CompileOptions,
    table: &mut Table,
) -> Result<(), CliError<B>> {
    let (mut context, crate_id) = prepare_package(package);
    let result = compile_contracts(&mut context, crate_id, compile_options);
    let contracts = report_errors(result, &context, compile_options.deny_warnings)?;
    let optimized_contracts =
        try_vecmap(contracts, |contract| optimize_contract(backend, contract))?;

    for contract in optimized_contracts {
        let function_info: Vec<(String, usize, u32)> = try_vecmap(contract.functions, |function| {
            let num_opcodes = function.bytecode.opcodes.len();
            let exact_circuit_size = backend.get_exact_circuit_size(&function.bytecode)?;

            Ok((function.name, num_opcodes, exact_circuit_size))
        })
        .map_err(CliError::ProofSystemCompilerError)?;

        for info in function_info {
            table.add_row(row![
                Fm->format!("{}", contract.name),
                Fc->format!("{}", info.0),
                format!("{:?}", backend.np_language()),
                Fc->format!("{}", info.1),
                Fc->format!("{}", info.2),
            ]);
        }
    }

    Ok(())
}
