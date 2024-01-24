use std::collections::HashMap;

use acvm::ExpressionWidth;
use backend_interface::BackendError;
use clap::Args;
use iter_extended::vecmap;
use nargo::{
    artifacts::debug::DebugArtifact, insert_all_files_for_workspace_into_file_manager,
    package::Package, parse_all,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{
    file_manager_with_stdlib, CompileOptions, CompiledContract, CompiledProgram,
    NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_errors::{debug_info::OpCodesCount, Location};
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

    #[clap(long, hide = true)]
    profile_info: bool,

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
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    let expression_width = backend.get_backend_info_or_default();
    let (compiled_programs, compiled_contracts) = compile_workspace(
        &workspace_file_manager,
        &parsed_files,
        &workspace,
        &args.compile_options,
    )?;

    let compiled_programs = vecmap(compiled_programs, |program| {
        nargo::ops::transform_program(program, expression_width)
    });
    let compiled_contracts = vecmap(compiled_contracts, |contract| {
        nargo::ops::transform_contract(contract, expression_width)
    });

    if args.profile_info {
        for compiled_program in &compiled_programs {
            let span_opcodes = compiled_program.debug.count_span_opcodes();
            let debug_artifact = DebugArtifact::from(compiled_program.clone());
            print_span_opcodes(span_opcodes, &debug_artifact);
        }

        for compiled_contract in &compiled_contracts {
            let debug_artifact = DebugArtifact::from(compiled_contract.clone());
            let functions = &compiled_contract.functions;
            for contract_function in functions {
                let span_opcodes = contract_function.debug.count_span_opcodes();
                print_span_opcodes(span_opcodes, &debug_artifact);
            }
        }
    }

    let binary_packages =
        workspace.into_iter().filter(|package| package.is_binary()).zip(compiled_programs);
    let program_info = binary_packages
        .par_bridge()
        .map(|(package, program)| {
            count_opcodes_and_gates_in_program(backend, program, package, expression_width)
        })
        .collect::<Result<_, _>>()?;

    let contract_info = compiled_contracts
        .into_par_iter()
        .map(|contract| count_opcodes_and_gates_in_contract(backend, contract, expression_width))
        .collect::<Result<_, _>>()?;

    let info_report = InfoReport { programs: program_info, contracts: contract_info };

    if args.json {
        // Expose machine-readable JSON data.
        println!("{}", serde_json::to_string(&info_report).unwrap());
    } else {
        // Otherwise print human-readable table.
        if !info_report.programs.is_empty() {
            let mut program_table = table!([Fm->"Package", Fm->"Expression Width", Fm->"ACIR Opcodes", Fm->"Backend Circuit Size"]);

            for program in info_report.programs {
                program_table.add_row(program.into());
            }
            program_table.printstd();
        }
        if !info_report.contracts.is_empty() {
            let mut contract_table = table!([
                Fm->"Contract",
                Fm->"Function",
                Fm->"Expression Width",
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

/// Provides profiling information on
///
/// Number of OpCodes in relation to Noir source file
/// and line number information
fn print_span_opcodes(
    span_opcodes_map: HashMap<Location, OpCodesCount>,
    debug_artifact: &DebugArtifact,
) {
    let mut pairs: Vec<(&Location, &OpCodesCount)> = span_opcodes_map.iter().collect();

    pairs.sort_by(|a, b| {
        a.1.acir_size.cmp(&b.1.acir_size).then_with(|| a.1.brillig_size.cmp(&b.1.brillig_size))
    });

    for (location, opcodes_count) in pairs {
        let debug_file = debug_artifact.file_map.get(&location.file).unwrap();

        let start_byte = byte_index(&debug_file.source, location.span.start() + 1);
        let end_byte = byte_index(&debug_file.source, location.span.end() + 1);
        let range = start_byte..end_byte;
        let span_content = &debug_file.source[range];
        let line = debug_artifact.location_line_index(*location).unwrap() + 1;
        println!(
            "Ln. {}: {} (ACIR:{}, Brillig:{} opcode|s) in file: {}",
            line,
            span_content,
            opcodes_count.acir_size,
            opcodes_count.brillig_size,
            debug_file.path.to_str().unwrap()
        );
    }
}
fn byte_index(string: &str, index: u32) -> usize {
    let mut byte_index = 0;
    let mut char_index = 0;

    #[allow(clippy::explicit_counter_loop)]
    for (byte_offset, _) in string.char_indices() {
        if char_index == index {
            return byte_index;
        }

        byte_index = byte_offset;
        char_index += 1;
    }

    byte_index
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
    expression_width: ExpressionWidth,
    acir_opcodes: usize,
    circuit_size: u32,
}

impl From<ProgramInfo> for Row {
    fn from(program_info: ProgramInfo) -> Self {
        row![
            Fm->format!("{}", program_info.name),
            format!("{:?}", program_info.expression_width),
            Fc->format!("{}", program_info.acir_opcodes),
            Fc->format!("{}", program_info.circuit_size),
        ]
    }
}

#[derive(Debug, Serialize)]
struct ContractInfo {
    name: String,
    #[serde(skip)]
    expression_width: ExpressionWidth,
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
                format!("{:?}", contract_info.expression_width),
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
    expression_width: ExpressionWidth,
) -> Result<ProgramInfo, CliError> {
    Ok(ProgramInfo {
        name: package.name.to_string(),
        expression_width,
        acir_opcodes: compiled_program.circuit.opcodes.len(),
        circuit_size: backend.get_exact_circuit_size(&compiled_program.circuit)?,
    })
}

fn count_opcodes_and_gates_in_contract(
    backend: &Backend,
    contract: CompiledContract,
    expression_width: ExpressionWidth,
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

    Ok(ContractInfo { name: contract.name, expression_width, functions })
}
