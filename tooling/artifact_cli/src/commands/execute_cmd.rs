use std::path::{Path, PathBuf};

use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use noirc_artifacts::program::CompiledProgram;

use crate::{
    Artifact,
    errors::CliError,
    execution::{self, ExecutionResults},
};
use nargo::foreign_calls::{
    DefaultForeignCallBuilder, OracleResolverUrl, layers, transcript::ReplayForeignCallExecutor,
};

use super::parse_and_normalize_path;

/// Execute a binary program or a circuit artifact.
#[derive(Debug, Clone, Args)]
pub struct ExecuteCommand {
    /// Path to the JSON build artifact (either a program or a contract).
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub artifact_path: PathBuf,

    /// Path to the Prover.toml (or .json) file which contains the inputs and the
    /// optional return value in ABI format.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub prover_file: PathBuf,

    /// Optionally overwrite the `return` entry in the Prover.toml file.
    #[clap(long, default_value_t = false)]
    pub overwrite_return: bool,

    /// Path to the directory where the output witness should be saved.
    /// If empty then the results are discarded.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub output_dir: Option<PathBuf>,

    /// Write the execution witness to named file.
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
    #[clap(long, conflicts_with = "oracle_file")]
    pub oracle_resolver: Option<OracleResolverUrl>,

    /// Root directory for the RPC oracle resolver.
    #[clap(long, value_parser = parse_and_normalize_path)]
    pub oracle_root_dir: Option<PathBuf>,

    /// Package name for the RPC oracle resolver.
    #[clap(long)]
    pub oracle_package_name: Option<String>,
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

    execute_program(
        &circuit,
        &circuit_name,
        &args.prover_file,
        args.output_dir.as_deref(),
        args.witness_name.as_deref(),
        args.overwrite_return,
        args.oracle_file.as_deref(),
        args.oracle_resolver.as_ref(),
    )
}

/// Execute an already-loaded compiled program: run it, save and show the witness, and
/// check the return value. On a circuit execution error, print a diagnostic before returning it.
///
/// This is the shared entry point used both by `run` (after reading an artifact from disk) and
/// by callers that hold a `CompiledProgram` in memory.
#[allow(clippy::too_many_arguments)]
pub fn execute_program(
    circuit: &CompiledProgram,
    circuit_name: &str,
    prover_file: &Path,
    output_dir: Option<&Path>,
    witness_name: Option<&str>,
    overwrite_return: bool,
    oracle_file: Option<&Path>,
    oracle_resolver: Option<&OracleResolverUrl>,
) -> Result<(), CliError> {
    match execute(circuit, prover_file, oracle_file, oracle_resolver) {
        Ok(results) => {
            execution::save_and_show_witness(
                circuit,
                &results,
                circuit_name,
                output_dir,
                witness_name,
            )?;
            execution::check_return(
                circuit,
                results.return_values,
                overwrite_return.then_some(prover_file),
            )?;
            Ok(())
        }
        Err(e) => {
            if let CliError::CircuitExecutionError(ref err) = e {
                execution::show_diagnostic(circuit, err);
            }
            // Still returning the error to facilitate command forwarding, to indicate that the command failed.
            Err(e)
        }
    }
}

/// Execute a circuit and return the output witnesses.
fn execute(
    circuit: &CompiledProgram,
    prover_file: &Path,
    oracle_file: Option<&Path>,
    oracle_resolver: Option<&OracleResolverUrl>,
) -> Result<ExecutionResults, CliError> {
    // Build a custom foreign call executor that replays the Oracle transcript,
    // and use it as a base for the default executor. Using it as the innermost rather
    // than top layer so that any extra `print` added for debugging is handled by the
    // default, rather than trying to match it to the transcript.
    let transcript_executor = match oracle_file {
        Some(path) => layers::Either::Left(ReplayForeignCallExecutor::from_file(path)?),
        None => layers::Either::Right(layers::Unhandled),
    };

    let mut foreign_call_executor = DefaultForeignCallBuilder {
        output: std::io::stdout(),
        enable_mocks: false,
        resolver_url: oracle_resolver.map(|url| url.to_string()),
        root_path: None,
        package_name: None,
    }
    .build_with_base(transcript_executor);

    let blackbox_solver = Bn254BlackBoxSolver;

    execution::execute(circuit, &blackbox_solver, &mut foreign_call_executor, prover_file)
}
