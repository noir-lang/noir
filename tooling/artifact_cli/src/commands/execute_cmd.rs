use std::path::PathBuf;

use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;

use crate::{
    errors::CliError,
    execution::{self, ExecutionResults},
    Artifact,
};
use nargo::{
    foreign_calls::{layers, logging::TranscriptForeignCallExecutor, DefaultForeignCallBuilder},
    PrintOutput,
};
use noirc_driver::CompiledProgram;

use super::parse_and_normalize_path;

/// Execute a binary program or a circuit artifact.
#[derive(Debug, Clone, Args)]
pub struct ExecuteCommand {
    /// Path to the JSON build artifact (either a program or a contract).
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub artifact_path: PathBuf,

    /// Path to the Prover.toml file which contains the inputs and the
    /// optional return value in ABI format.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub prover_file: PathBuf,

    /// Path to the directory where the output witness should be saved.
    /// If empty then the results are discarded.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub output_dir: Option<PathBuf>,

    /// Write the execution witness to named file
    ///
    /// Defaults to the name of the circuit being executed.
    #[clap(long, short)]
    pub witness_name: Option<String>,

    /// Name of the function to execute, if the artifact is a contract.
    #[clap(long)]
    pub contract_fn: Option<String>,

    /// Path to the oracle transcript that is to be replayed during the
    /// execution in response to foreign calls. The format is expected
    /// to be JSON Lines, with each request/response on a separate line.
    ///
    /// Note that a transcript might be invalid if the inputs change and
    /// the circuit takes a different path during execution.
    #[clap(long, conflicts_with = "oracle_resolver")]
    pub oracle_file: Option<PathBuf>,

    /// JSON RPC url to solve oracle calls.
    ///
    /// This is to facilitate new executions, as opposed to replays.
    #[clap(long, conflicts_with = "oracle_file")]
    pub oracle_resolver: Option<String>,

    /// Root directory for the RPC oracle resolver.
    #[clap(long, value_parser = parse_and_normalize_path)]
    pub oracle_root_dir: Option<PathBuf>,

    /// Package name for the RPC oracle resolver
    #[clap(long)]
    pub oracle_package_name: Option<String>,

    /// Use pedantic ACVM solving, i.e. double-check some black-box function assumptions when solving.
    #[clap(long, default_value_t = false)]
    pub pedantic_solving: bool,
}

pub fn run(args: ExecuteCommand) -> Result<(), CliError> {
    let artifact = Artifact::read_from_file(&args.artifact_path)?;
    let artifact_name = args.artifact_path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();

    let (circuit, circuit_name): (CompiledProgram, String) = match artifact {
        Artifact::Program(program) => (program.into(), artifact_name.to_string()),
        Artifact::Contract(contract) => {
            let names = || contract.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>();

            let Some(ref name) = args.contract_fn else {
                return Err(CliError::MissingContractFn { names: names() });
            };
            let Some(program) = contract.function_as_compiled_program(name) else {
                return Err(CliError::UnknownContractFn { name: name.clone(), names: names() });
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
        Err(e) => {
            if let CliError::CircuitExecutionError(ref err) = e {
                execution::show_diagnostic(&circuit, err);
            }
            // Still returning the error to facilitate command forwarding, to indicate that the command failed.
            return Err(e);
        }
    }
    Ok(())
}

/// Execute a circuit and return the output witnesses.
fn execute(circuit: &CompiledProgram, args: &ExecuteCommand) -> Result<ExecutionResults, CliError> {
    // Build a custom foreign call executor that reads from the Oracle transcript,
    // and use it as a base for the default executor; see `DefaultForeignCallBuilder::build_with_base`
    let transcript_executor = match args.oracle_file {
        Some(ref path) => layers::Either::Left(TranscriptForeignCallExecutor::from_file(path)?),
        None => layers::Either::Right(layers::Empty),
    };

    let mut foreign_call_executor = DefaultForeignCallBuilder {
        output: PrintOutput::Stdout,
        enable_mocks: false,
        resolver_url: args.oracle_resolver.clone(),
        root_path: None,
        package_name: None,
    }
    .build_with_base(transcript_executor);

    let blackbox_solver = Bn254BlackBoxSolver(args.pedantic_solving);

    execution::execute(circuit, &blackbox_solver, &mut foreign_call_executor, &args.prover_file)
}
