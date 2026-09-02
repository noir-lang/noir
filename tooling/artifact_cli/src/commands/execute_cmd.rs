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
    #[clap(long, short, conflicts_with = "witness_file", value_parser = parse_and_normalize_path)]
    pub prover_file: Option<PathBuf>,

    /// Path to a witness file (.gz) to use as the initial witness instead of a
    /// Prover.toml. This is the same gzipped format that `noir-execute` outputs.
    #[clap(long, conflicts_with = "prover_file", value_parser = parse_and_normalize_path)]
    pub witness_file: Option<PathBuf>,

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
    let input = match (&args.prover_file, &args.witness_file) {
        (Some(path), None) => InputSource::ProverFile(path.as_path()),
        (None, Some(path)) => InputSource::WitnessFile(path.as_path()),
        _ => {
            return Err(CliError::Generic(
                "exactly one of --prover-file or --witness-file must be provided".to_string(),
            ));
        }
    };

    if args.overwrite_return && args.witness_file.is_some() {
        return Err(CliError::Generic(
            "--overwrite-return cannot be used with --witness-file".to_string(),
        ));
    }

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
        &ExecuteProgramArgs {
            input,
            output_dir: args.output_dir.as_deref(),
            witness_name: args.witness_name.as_deref(),
            overwrite_return: args.overwrite_return,
            oracle_file: args.oracle_file.as_deref(),
            oracle_resolver: args.oracle_resolver.as_ref(),
        },
    )
}

/// Where the initial witness comes from.
pub enum InputSource<'a> {
    /// ABI-encoded inputs in TOML or JSON.
    ProverFile(&'a Path),
    /// Pre-computed initial witness (gzipped WitnessStack).
    WitnessFile(&'a Path),
}

/// Inputs and output options for [`execute_program`], borrowed from the caller's own
/// arguments (whether those come from an `ExecuteCommand` or from an in-memory compilation).
pub struct ExecuteProgramArgs<'a> {
    /// Source of the initial witness.
    pub input: InputSource<'a>,
    /// Directory to save the output witness in; results are discarded if `None`.
    pub output_dir: Option<&'a Path>,
    /// Name for the saved witness file; defaults to the circuit name if `None`.
    pub witness_name: Option<&'a str>,
    /// Overwrite the `return` entry in the prover file with the executed return value.
    pub overwrite_return: bool,
    /// Oracle transcript to replay in response to foreign calls.
    pub oracle_file: Option<&'a Path>,
    /// JSON RPC url to solve oracle calls.
    pub oracle_resolver: Option<&'a OracleResolverUrl>,
}

/// Execute an already-loaded compiled program: run it, save and show the witness, and
/// check the return value. On a circuit execution error, print a diagnostic before returning it.
///
/// This is the shared entry point used both by `run` (after reading an artifact from disk) and
/// by callers that hold a `CompiledProgram` in memory.
pub fn execute_program(
    circuit: &CompiledProgram,
    circuit_name: &str,
    args: &ExecuteProgramArgs,
) -> Result<(), CliError> {
    match execute(circuit, args) {
        Ok(results) => {
            execution::save_and_show_witness(
                circuit,
                &results,
                circuit_name,
                args.output_dir,
                args.witness_name,
            )?;
            let overwrite_prover_file = if args.overwrite_return {
                match &args.input {
                    InputSource::ProverFile(path) => Some(*path),
                    InputSource::WitnessFile(_) => None,
                }
            } else {
                None
            };
            execution::check_return(circuit, results.return_values, overwrite_prover_file)?;
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
    args: &ExecuteProgramArgs,
) -> Result<ExecutionResults, CliError> {
    // Build a custom foreign call executor that replays the Oracle transcript,
    // and use it as a base for the default executor. Using it as the innermost rather
    // than top layer so that any extra `print` added for debugging is handled by the
    // default, rather than trying to match it to the transcript.
    let transcript_executor = match args.oracle_file {
        Some(path) => layers::Either::Left(ReplayForeignCallExecutor::from_file(path)?),
        None => layers::Either::Right(layers::Unhandled),
    };

    let mut foreign_call_executor = DefaultForeignCallBuilder {
        output: std::io::stdout(),
        enable_mocks: false,
        resolver_url: args.oracle_resolver.map(|url| url.to_string()),
        root_path: None,
        package_name: None,
    }
    .build_with_base(transcript_executor);

    let blackbox_solver = Bn254BlackBoxSolver;

    execution::execute(circuit, &blackbox_solver, &mut foreign_call_executor, &args.input)
}
