use std::path::PathBuf;

use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use color_eyre::eyre::{self, bail};

use nargo::{foreign_calls::DefaultForeignCallBuilder, PrintOutput};
use noir_artifact_cli::{
    errors::CliError,
    execution::{self, ExecutionResults},
    Artifact,
};
use noirc_driver::CompiledProgram;

use super::parse_and_normalize_path;

/// Execute a binary program or a circuit artifact.
#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Path to the JSON build artifact (either a program or a contract).
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    artifact: PathBuf,

    /// Path to the Prover.toml file which contains the inputs and the
    /// optional return value in ABI format.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    prover_file: PathBuf,

    /// Path to the directory where the output witness should be saved.
    /// If empty then the results are discarded.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    output_dir: Option<PathBuf>,

    /// Write the execution witness to named file
    ///
    /// Defaults to the name of the circuit being executed.
    #[clap(long, short)]
    witness_name: Option<String>,

    /// Name of the function to execute, if the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,

    /// Part to the Oracle.toml file which contains the Oracle transcript,
    /// which is a list of responses captured during an earlier execution,
    /// which can replayed via mocks.
    ///
    /// Note that a transcript might be invalid if the inputs change and
    /// the circuit takes a different path during execution.
    #[clap(long, conflicts_with = "oracle_resolver")]
    oracle_file: Option<String>,

    /// JSON RPC url to solve oracle calls.
    ///
    /// This is to facilitate new executions, as opposed to replays.
    #[clap(long, conflicts_with = "oracle_file")]
    oracle_resolver: Option<String>,

    /// Use pedantic ACVM solving, i.e. double-check some black-box function assumptions when solving.
    #[clap(long, default_value_t = false)]
    pedantic_solving: bool,
}

pub(crate) fn run(args: ExecuteCommand) -> eyre::Result<()> {
    let artifact = Artifact::read_from_file(&args.artifact)?;
    let artifact_name = args.artifact.file_stem().and_then(|s| s.to_str()).unwrap_or_default();

    let (circuit, circuit_name): (CompiledProgram, String) = match artifact {
        Artifact::Program(program) => (program.into(), artifact_name.to_string()),
        Artifact::Contract(contract) => {
            let names =
                contract.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(",");

            let Some(ref name) = args.contract_fn else {
                bail!("--contract-fn missing; options: [{names}]");
            };
            let Some(program) = contract.function_as_compiled_program(name) else {
                bail!("unknown --contract-fn '{name}'; options: [{names}]");
            };

            (program, format!("{artifact_name}::{name}"))
        }
    };

    match execute(&circuit, &args) {
        Ok(results) => {
            execution::save_and_check_witness(
                &circuit,
                results,
                &circuit_name,
                args.output_dir.as_deref(),
                args.witness_name.as_deref(),
            )?;
        }
        Err(CliError::CircuitExecutionError(err)) => {
            execution::show_diagnostic(circuit, err);
        }
        Err(e) => {
            bail!("failed to execute the circuit: {e}");
        }
    }
    Ok(())
}

/// Execute a circuit and return the output witnesses.
fn execute(circuit: &CompiledProgram, args: &ExecuteCommand) -> Result<ExecutionResults, CliError> {
    // TODO: Build a custom foreign call executor that reads from the Oracle transcript,
    // and use it as a base for the default executor; see `DefaultForeignCallBuilder::build_with_base`
    let mut foreign_call_executor = DefaultForeignCallBuilder {
        output: PrintOutput::Stdout,
        enable_mocks: false,
        resolver_url: args.oracle_resolver.clone(),
        root_path: None,
        package_name: None,
    }
    .build();

    let blackbox_solver = Bn254BlackBoxSolver(args.pedantic_solving);

    execution::execute(circuit, &blackbox_solver, &mut foreign_call_executor, &args.prover_file)
}
