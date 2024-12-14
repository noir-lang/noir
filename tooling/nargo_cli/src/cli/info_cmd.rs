use acvm::acir::circuit::ExpressionWidth;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use iter_extended::vecmap;
use nargo::{
    constants::PROVER_INPUT_FILE, foreign_calls::DefaultForeignCallExecutor, package::Package,
    PrintOutput,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml};
use noirc_abi::input_parser::Format;
use noirc_artifacts::program::ProgramArtifact;
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};
use prettytable::{row, table, Row};
use rayon::prelude::*;
use serde::Serialize;

use crate::{cli::fs::inputs::read_inputs_from_file, errors::CliError};

use super::{
    compile_cmd::{compile_workspace_full, get_target_width},
    fs::program::read_program_from_file,
    NargoConfig, PackageOptions,
};

/// Provides detailed information on each of a program's function (represented by a single circuit)
///
/// Current information provided per circuit:
/// 1. The number of ACIR opcodes
/// 2. Counts the final number gates in the circuit used by a backend
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "i")]
pub(crate) struct InfoCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    /// Output a JSON formatted report. Changes to this format are not currently considered breaking.
    #[clap(long, hide = true)]
    json: bool,

    #[clap(long)]
    profile_execution: bool,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(mut args: InfoCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let selection = args.package_options.package_selection();
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    if args.profile_execution {
        // Execution profiling is only relevant with the Brillig VM
        // as a constrained circuit should have totally flattened control flow (e.g. loops and if statements).
        args.compile_options.force_brillig = true;
    }
    // Compile the full workspace in order to generate any build artifacts.
    compile_workspace_full(&workspace, &args.compile_options)?;

    let binary_packages: Vec<(Package, ProgramArtifact)> = workspace
        .into_iter()
        .filter(|package| package.is_binary())
        .map(|package| -> Result<(Package, ProgramArtifact), CliError> {
            let program_artifact_path = workspace.package_build_path(package);
            let program = read_program_from_file(program_artifact_path)?;
            Ok((package.clone(), program))
        })
        .collect::<Result<_, _>>()?;

    let program_info = if args.profile_execution {
        assert!(
            args.compile_options.force_brillig,
            "Internal CLI Error: --force-brillig must be active when --profile-execution is active"
        );
        profile_brillig_execution(
            binary_packages,
            &args.prover_name,
            args.compile_options.expression_width,
        )?
    } else {
        binary_packages
            .into_iter()
            .par_bridge()
            .map(|(package, program)| {
                let target_width = get_target_width(
                    package.expression_width,
                    args.compile_options.expression_width,
                );
                count_opcodes_and_gates_in_program(program, &package, target_width)
            })
            .collect()
    };

    let info_report = InfoReport { programs: program_info };

    if args.json {
        // Expose machine-readable JSON data.
        println!("{}", serde_json::to_string(&info_report).unwrap());
    } else {
        // Otherwise print human-readable table.
        if !info_report.programs.is_empty() {
            let mut program_table = table!([Fm->"Package", Fm->"Function", Fm->"Expression Width", Fm->"ACIR Opcodes", Fm->"Brillig Opcodes"]);

            for program_info in info_report.programs {
                let program_rows: Vec<Row> = program_info.into();
                for row in program_rows {
                    program_table.add_row(row);
                }
            }
            program_table.printstd();
        }
    }

    Ok(())
}

#[derive(Debug, Default, Serialize)]
struct InfoReport {
    programs: Vec<ProgramInfo>,
}

#[derive(Debug, Serialize)]
struct ProgramInfo {
    package_name: String,
    #[serde(skip)]
    expression_width: ExpressionWidth,
    functions: Vec<FunctionInfo>,
    #[serde(skip)]
    unconstrained_functions_opcodes: usize,
    unconstrained_functions: Vec<FunctionInfo>,
}

impl From<ProgramInfo> for Vec<Row> {
    fn from(program_info: ProgramInfo) -> Self {
        let mut main = vecmap(program_info.functions, |function| {
            row![
                Fm->format!("{}", program_info.package_name),
                Fc->format!("{}", function.name),
                format!("{:?}", program_info.expression_width),
                Fc->format!("{}", function.opcodes),
                Fc->format!("{}", program_info.unconstrained_functions_opcodes),
            ]
        });
        main.extend(vecmap(program_info.unconstrained_functions, |function| {
            row![
                Fm->format!("{}", program_info.package_name),
                Fc->format!("{}", function.name),
                format!("N/A", ),
                Fc->format!("N/A"),
                Fc->format!("{}", function.opcodes),
            ]
        }));
        main
    }
}

#[derive(Debug, Serialize)]
struct ContractInfo {
    name: String,
    #[serde(skip)]
    expression_width: ExpressionWidth,
    // TODO(https://github.com/noir-lang/noir/issues/4720): Settle on how to display contract functions with non-inlined Acir calls
    functions: Vec<FunctionInfo>,
}

#[derive(Debug, Serialize)]
struct FunctionInfo {
    name: String,
    opcodes: usize,
}

impl From<ContractInfo> for Vec<Row> {
    fn from(contract_info: ContractInfo) -> Self {
        vecmap(contract_info.functions, |function| {
            row![
                Fm->format!("{}", contract_info.name),
                Fc->format!("{}", function.name),
                format!("{:?}", contract_info.expression_width),
                Fc->format!("{}", function.opcodes),
            ]
        })
    }
}

fn count_opcodes_and_gates_in_program(
    compiled_program: ProgramArtifact,
    package: &Package,
    expression_width: ExpressionWidth,
) -> ProgramInfo {
    let functions = compiled_program
        .bytecode
        .functions
        .into_par_iter()
        .enumerate()
        .map(|(i, function)| FunctionInfo {
            name: compiled_program.names[i].clone(),
            opcodes: function.opcodes.len(),
        })
        .collect();

    let opcodes_len: Vec<usize> = compiled_program
        .bytecode
        .unconstrained_functions
        .iter()
        .map(|func| func.bytecode.len())
        .collect();
    let unconstrained_functions_opcodes = compiled_program
        .bytecode
        .unconstrained_functions
        .into_par_iter()
        .map(|function| function.bytecode.len())
        .sum();
    let unconstrained_info: Vec<FunctionInfo> = compiled_program
        .brillig_names
        .clone()
        .iter()
        .zip(opcodes_len)
        .map(|(name, len)| FunctionInfo { name: name.clone(), opcodes: len })
        .collect();

    ProgramInfo {
        package_name: package.name.to_string(),
        expression_width,
        functions,
        unconstrained_functions_opcodes,
        unconstrained_functions: unconstrained_info,
    }
}

fn profile_brillig_execution(
    binary_packages: Vec<(Package, ProgramArtifact)>,
    prover_name: &str,
    expression_width: Option<ExpressionWidth>,
) -> Result<Vec<ProgramInfo>, CliError> {
    let mut program_info = Vec::new();
    for (package, program_artifact) in binary_packages.iter() {
        // Parse the initial witness values from Prover.toml
        let (inputs_map, _) = read_inputs_from_file(
            &package.root_dir,
            prover_name,
            Format::Toml,
            &program_artifact.abi,
        )?;
        let initial_witness = program_artifact.abi.encode(&inputs_map, None)?;

        let (_, profiling_samples) = nargo::ops::execute_program_with_profiling(
            &program_artifact.bytecode,
            initial_witness,
            &Bn254BlackBoxSolver,
            &mut DefaultForeignCallExecutor::new(PrintOutput::None, None, None, None),
        )?;

        let expression_width = get_target_width(package.expression_width, expression_width);

        program_info.push(ProgramInfo {
            package_name: package.name.to_string(),
            expression_width,
            functions: vec![FunctionInfo { name: "main".to_string(), opcodes: 0 }],
            unconstrained_functions_opcodes: profiling_samples.len(),
            unconstrained_functions: vec![FunctionInfo {
                name: "main".to_string(),
                opcodes: profiling_samples.len(),
            }],
        });
    }
    Ok(program_info)
}
