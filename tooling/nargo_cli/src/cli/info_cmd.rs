use acvm::Language;
use backend_interface::BackendError;
use clap::Args;
use iter_extended::vecmap;
use nargo::package::Package;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{CompileOptions, CompiledContract, CompiledProgram};
use noirc_frontend::graph::CrateName;
use prettytable::{row, table, Row};
use rayon::prelude::*;
use serde::Serialize;

use crate::backends::Backend;
use crate::errors::CliError;

use super::{compile_cmd::compile_workspace, NargoConfig};

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

    /// Output a JSON formatted report. Changes to this format are not currently considered breaking.
    #[clap(long, hide = true)]
    json: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(
    backend: &Backend,
    args: InfoCommand,
    config: NargoConfig,
) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;

    let (binary_packages, contract_packages): (Vec<_>, Vec<_>) = workspace
        .into_iter()
        .filter(|package| !package.is_library())
        .cloned()
        .partition(|package| package.is_binary());

    let (np_language, opcode_support) = backend.get_backend_info()?;
    let (compiled_programs, compiled_contracts) = compile_workspace(
        &workspace,
        &binary_packages,
        &contract_packages,
        np_language,
        &opcode_support,
        &args.compile_options,
        false,
    )?;

    let program_info = binary_packages
        .into_par_iter()
        .zip(compiled_programs)
        .map(|(package, program)| {
            count_opcodes_and_gates_in_program(backend, program, &package, np_language)
        })
        .collect::<Result<_, _>>()?;

    let contract_info = compiled_contracts
        .into_par_iter()
        .map(|contract| count_opcodes_and_gates_in_contract(backend, contract, np_language))
        .collect::<Result<_, _>>()?;

    let info_report = InfoReport { programs: program_info, contracts: contract_info };

    if args.json {
        // Expose machine-readable JSON data.
        println!("{}", serde_json::to_string(&info_report).unwrap());
    } else {
        // Otherwise print human-readable table.
        if !info_report.programs.is_empty() {
            let mut program_table = table!([Fm->"Package", Fm->"Language", Fm->"ACIR Opcodes", Fm->"Backend Circuit Size"]);

            for program in info_report.programs {
                program_table.add_row(program.into());
            }
            program_table.printstd();
        }
        if !info_report.contracts.is_empty() {
            let mut contract_table = table!([
                Fm->"Contract",
                Fm->"Function",
                Fm->"Language",
                Fm->"ACIR Opcodes",
                Fm->"Backend Circuit Size"
            ]);
            for contract_info in info_report.contracts {
                let contract_rows: Vec<Row> = contract_info.into();
                for row in contract_rows {
                    contract_table.add_row(row);
                }
            }

            contract_table.printstd();
        }
    }

    Ok(())
}

#[derive(Debug, Default, Serialize)]
struct InfoReport {
    programs: Vec<ProgramInfo>,
    contracts: Vec<ContractInfo>,
}

#[derive(Debug, Serialize)]
struct ProgramInfo {
    name: String,
    #[serde(skip)]
    language: Language,
    acir_opcodes: usize,
    circuit_size: u32,
}

impl From<ProgramInfo> for Row {
    fn from(program_info: ProgramInfo) -> Self {
        row![
            Fm->format!("{}", program_info.name),
            format!("{:?}", program_info.language),
            Fc->format!("{}", program_info.acir_opcodes),
            Fc->format!("{}", program_info.circuit_size),
        ]
    }
}

#[derive(Debug, Serialize)]
struct ContractInfo {
    name: String,
    #[serde(skip)]
    language: Language,
    functions: Vec<FunctionInfo>,
}

#[derive(Debug, Serialize)]
struct FunctionInfo {
    name: String,
    acir_opcodes: usize,
    circuit_size: u32,
}

impl From<ContractInfo> for Vec<Row> {
    fn from(contract_info: ContractInfo) -> Self {
        vecmap(contract_info.functions, |function| {
            row![
                Fm->format!("{}", contract_info.name),
                Fc->format!("{}", function.name),
                format!("{:?}", contract_info.language),
                Fc->format!("{}", function.acir_opcodes),
                Fc->format!("{}", function.circuit_size),
            ]
        })
    }
}

fn count_opcodes_and_gates_in_program(
    backend: &Backend,
    compiled_program: CompiledProgram,
    package: &Package,
    language: Language,
) -> Result<ProgramInfo, CliError> {
    Ok(ProgramInfo {
        name: package.name.to_string(),
        language,
        acir_opcodes: compiled_program.circuit.opcodes.len(),
        circuit_size: backend.get_exact_circuit_size(&compiled_program.circuit)?,
    })
}

fn count_opcodes_and_gates_in_contract(
    backend: &Backend,
    contract: CompiledContract,
    language: Language,
) -> Result<ContractInfo, CliError> {
    let functions = contract
        .functions
        .into_par_iter()
        .map(|function| -> Result<_, BackendError> {
            Ok(FunctionInfo {
                name: function.name,
                acir_opcodes: function.bytecode.opcodes.len(),
                circuit_size: backend.get_exact_circuit_size(&function.bytecode)?,
            })
        })
        .collect::<Result<_, _>>()?;

    Ok(ContractInfo { name: contract.name, language, functions })
}
