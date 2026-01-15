use std::path::{Path, PathBuf};

use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use color_eyre::eyre;
use nargo::{foreign_calls::DefaultForeignCallBuilder, ops::execute_program_with_profiling};
use noir_artifact_cli::Artifact;
use noir_artifact_cli::fs::inputs::read_inputs_from_file;
use noirc_artifacts_info::{
    FunctionInfo, InfoReport, ProgramInfo, count_opcodes_and_gates_in_program, show_info_report,
};

#[derive(Debug, Clone, Args)]
pub(crate) struct InfoCommand {
    /// The artifact to inspect
    artifact: PathBuf,

    /// Output a JSON formatted report. Changes to this format are not currently considered breaking.
    #[clap(long, hide = true)]
    json: bool,

    /// Name of the function to print, if the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,

    /// Profile execution to count actual opcodes executed at runtime
    #[clap(long)]
    profile_execution: bool,

    /// Path to the input file (TOML or JSON) containing witness values.
    /// If not provided, looks for <artifact_name>.toml or <artifact_name>.json
    /// in the same directory as the artifact.
    #[clap(long, short = 'i')]
    input_file: Option<PathBuf>,

    /// Use pedantic ACVM solving
    #[clap(long)]
    pedantic_solving: bool,
}

/// Resolves the input file path for profiling.
/// Priority:
/// 1. Explicit --input-file argument
/// 2. Prover.toml in program directory (parent of target/)
/// 3. Prover.json in program directory
fn resolve_input_file(
    artifact_path: &Path,
    explicit_input: Option<&PathBuf>,
) -> eyre::Result<PathBuf> {
    if let Some(input_path) = explicit_input {
        if !input_path.exists() {
            return Err(eyre::eyre!("Input file not found: {}", input_path.display()));
        }
        return Ok(input_path.clone());
    }

    // Artifact is typically at: <program_dir>/target/<name>.json
    // Input files are at: <program_dir>/Prover.toml
    let artifact_dir =
        artifact_path.parent().ok_or_else(|| eyre::eyre!("Cannot determine artifact directory"))?;

    let program_dir = artifact_dir
        .parent()
        .ok_or_else(|| eyre::eyre!("Cannot determine program directory"))?;

    // Try Prover.toml first
    let toml_path = program_dir.join("Prover.toml");
    if toml_path.exists() {
        return Ok(toml_path);
    }

    // Try Prover.json
    let json_path = program_dir.join("Prover.json");
    if json_path.exists() {
        return Ok(json_path);
    }

    Err(eyre::eyre!(
        "No input file found. Expected {} or {}",
        toml_path.display(),
        json_path.display()
    ))
}

/// Profile a single program's execution
fn profile_program_execution(
    program: noirc_artifacts::program::ProgramArtifact,
    package_name: String,
    input_file: &Path,
    pedantic_solving: bool,
) -> eyre::Result<ProgramInfo> {
    // Read inputs from file
    let (inputs_map, _) = read_inputs_from_file(input_file, &program.abi)?;
    let initial_witness = program.abi.encode(&inputs_map, None)?;

    // Execute with profiling
    let (_, profiling_samples) = execute_program_with_profiling(
        &program.bytecode,
        initial_witness,
        &Bn254BlackBoxSolver(pedantic_solving),
        &mut DefaultForeignCallBuilder::default().build(),
    )
    .map_err(|e| eyre::eyre!("Execution failed: {}", e))?;

    // Build profiling report
    Ok(ProgramInfo {
        package_name,
        functions: vec![FunctionInfo { name: "main".to_string(), opcodes: 0 }],
        unconstrained_functions_opcodes: profiling_samples.len(),
        unconstrained_functions: vec![FunctionInfo {
            name: "main".to_string(),
            opcodes: profiling_samples.len(),
        }],
    })
}

pub(crate) fn run(args: InfoCommand) -> eyre::Result<()> {
    let artifact = Artifact::read_from_file(&args.artifact)?;

    let programs = if args.profile_execution {
        let input_file = resolve_input_file(&args.artifact, args.input_file.as_ref())?;

        match artifact {
            Artifact::Program(program) => {
                let package_name = args
                    .artifact
                    .with_extension("")
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "artifact".to_string());

                vec![profile_program_execution(
                    program,
                    package_name,
                    &input_file,
                    args.pedantic_solving,
                )?]
            }
            Artifact::Contract(contract) => {
                // profile each contract function
                contract
                    .functions
                    .into_iter()
                    .filter(|f| args.contract_fn.as_ref().map(|n| *n == f.name).unwrap_or(true))
                    .map(|f| {
                        let package_name = format!("{}::{}", contract.name, f.name);
                        let program = f.into_compiled_program(
                            contract.noir_version.clone(),
                            contract.file_map.clone(),
                        );
                        profile_program_execution(
                            program.into(),
                            package_name,
                            &input_file,
                            args.pedantic_solving,
                        )
                    })
                    .collect::<eyre::Result<Vec<_>>>()?
            }
        }
    } else {
        match artifact {
            Artifact::Program(program) => {
                let package_name = args
                    .artifact
                    .with_extension("")
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "artifact".to_string());

                vec![count_opcodes_and_gates_in_program(program, package_name)]
            }
            Artifact::Contract(contract) => contract
                .functions
                .into_iter()
                .filter(|f| args.contract_fn.as_ref().map(|n| *n == f.name).unwrap_or(true))
                .map(|f| {
                    let package_name = format!("{}::{}", contract.name, f.name);
                    let program = f.into_compiled_program(
                        contract.noir_version.clone(),
                        contract.file_map.clone(),
                    );
                    count_opcodes_and_gates_in_program(program.into(), package_name)
                })
                .collect::<Vec<_>>(),
        }
    };

    let info_report = InfoReport { programs };
    show_info_report(info_report, args.json);

    Ok(())
}
