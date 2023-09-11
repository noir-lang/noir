use acvm::acir::circuit::Opcode;
use acvm::Language;
use acvm_backend_barretenberg::BackendError;
use clap::Args;
use iter_extended::{try_vecmap, vecmap};
use nargo::package::Package;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::CompileOptions;
use noirc_frontend::graph::CrateName;
use prettytable::{row, table, Row};
use serde::Serialize;

use crate::backends::Backend;
use crate::{cli::compile_cmd::compile_package, errors::CliError};

use super::{compile_cmd::compile_contracts, NargoConfig};

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

    let mut info_report = InfoReport::default();

    let (np_language, is_opcode_supported) = backend.get_backend_info()?;
    for package in &workspace {
        if package.is_contract() {
            let contract_info = count_opcodes_and_gates_in_contracts(
                backend,
                package,
                &args.compile_options,
                np_language,
                &is_opcode_supported,
            )?;
            info_report.contracts.extend(contract_info);
        } else {
            let program_info = count_opcodes_and_gates_in_program(
                backend,
                package,
                &args.compile_options,
                np_language,
                &is_opcode_supported,
            )?;
            info_report.programs.push(program_info);
        }
    }

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
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<ProgramInfo, CliError> {
    let (compiled_program, _) =
        compile_package(package, compile_options, np_language, &is_opcode_supported)?;
    let (language, _) = backend.get_backend_info()?;

    Ok(ProgramInfo {
        name: package.name.to_string(),
        language,
        acir_opcodes: compiled_program.circuit.opcodes.len(),
        circuit_size: backend.get_exact_circuit_size(&compiled_program.circuit)?,
    })
}

fn count_opcodes_and_gates_in_contracts(
    backend: &Backend,
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<Vec<ContractInfo>, CliError> {
    let contracts = compile_contracts(package, compile_options, np_language, &is_opcode_supported)?;
    let (language, _) = backend.get_backend_info()?;

    try_vecmap(contracts, |(contract, _)| {
        let functions = try_vecmap(contract.functions, |function| -> Result<_, BackendError> {
            Ok(FunctionInfo {
                name: function.name,
                acir_opcodes: function.bytecode.opcodes.len(),
                circuit_size: backend.get_exact_circuit_size(&function.bytecode)?,
            })
        })?;

        Ok(ContractInfo { name: contract.name, language, functions })
    })
}
