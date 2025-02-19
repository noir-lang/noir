use acvm::acir::circuit::ExpressionWidth;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use iter_extended::vecmap;
use nargo::{
    constants::PROVER_INPUT_FILE, foreign_calls::DefaultForeignCallBuilder, package::Package,
    workspace::Workspace,
};
use nargo_toml::PackageSelection;
use noirc_abi::input_parser::Format;
use noirc_artifacts::program::ProgramArtifact;
use noirc_artifacts_info::{
    count_opcodes_and_gates_in_program, show_info_report, FunctionInfo, InfoReport, ProgramInfo,
};
use noirc_driver::CompileOptions;
use prettytable::{row, Row};
use rayon::prelude::*;
use serde::Serialize;

use crate::errors::CliError;

use super::{
    compile_cmd::{compile_workspace_full, get_target_width},
    fs::{inputs::read_inputs_from_file, program::read_program_from_file},
    LockType, PackageOptions, WorkspaceCommand,
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

impl WorkspaceCommand for InfoCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        LockType::Exclusive
    }
}

pub(crate) fn run(mut args: InfoCommand, workspace: Workspace) -> Result<(), CliError> {
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
            args.compile_options.pedantic_solving,
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
                let package_name = package.name.to_string();
                count_opcodes_and_gates_in_program(program, package_name, Some(target_width))
            })
            .collect()
    };

    let info_report = InfoReport { programs: program_info };
    show_info_report(info_report, args.json);

    Ok(())
}

#[derive(Debug, Serialize)]
struct ContractInfo {
    name: String,
    #[serde(skip)]
    expression_width: ExpressionWidth,
    // TODO(https://github.com/noir-lang/noir/issues/4720): Settle on how to display contract functions with non-inlined Acir calls
    functions: Vec<FunctionInfo>,
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

fn profile_brillig_execution(
    binary_packages: Vec<(Package, ProgramArtifact)>,
    prover_name: &str,
    expression_width: Option<ExpressionWidth>,
    pedantic_solving: bool,
) -> Result<Vec<ProgramInfo>, CliError> {
    let mut program_info = Vec::new();
    for (package, program_artifact) in binary_packages.iter() {
        // Parse the initial witness values from Prover.toml or Prover.json
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
            &Bn254BlackBoxSolver(pedantic_solving),
            &mut DefaultForeignCallBuilder::default().build(),
        )
        .map_err(|e| {
            CliError::Generic(format!(
                "failed to execute '{}': {}",
                package.root_dir.to_string_lossy(),
                e
            ))
        })?;

        let expression_width = get_target_width(package.expression_width, expression_width);

        program_info.push(ProgramInfo {
            package_name: package.name.to_string(),
            expression_width: Some(expression_width),
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
