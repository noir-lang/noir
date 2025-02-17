use std::path::PathBuf;

use acir::{native_types::WitnessStack, FieldElement};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use color_eyre::eyre::{self, bail};

use nargo::{foreign_calls::DefaultForeignCallBuilder, NargoError, PrintOutput};
use noir_artifact_cli::{
    errors::CliError,
    fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir},
    Artifact,
};
use noirc_abi::input_parser::InputValue;
use noirc_artifacts::debug::DebugArtifact;
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
        Ok(solved) => {
            save_witness(circuit, &args, &circuit_name, solved)?;
        }
        Err(CliError::CircuitExecutionError(err)) => {
            show_diagnostic(circuit, err);
        }
        Err(e) => {
            bail!("failed to execute the circuit: {e}");
        }
    }
    Ok(())
}

struct SolvedWitnesses {
    expected_return: Option<InputValue>,
    actual_return: Option<InputValue>,
    witness_stack: WitnessStack<FieldElement>,
}

/// Execute a circuit and return the output witnesses.
fn execute(circuit: &CompiledProgram, args: &ExecuteCommand) -> Result<SolvedWitnesses, CliError> {
    let (input_map, expected_return) = read_inputs_from_file(&args.prover_file, &circuit.abi)?;

    let initial_witness = circuit.abi.encode(&input_map, None)?;

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

    let witness_stack = nargo::ops::execute_program(
        &circuit.program,
        initial_witness,
        &Bn254BlackBoxSolver(args.pedantic_solving),
        &mut foreign_call_executor,
    )?;

    let main_witness =
        &witness_stack.peek().expect("Should have at least one witness on the stack").witness;

    let (_, actual_return) = circuit.abi.decode(main_witness)?;

    Ok(SolvedWitnesses { expected_return, actual_return, witness_stack })
}

/// Print an error stack trace, if possible.
fn show_diagnostic(circuit: CompiledProgram, err: NargoError<FieldElement>) {
    if let Some(diagnostic) =
        nargo::errors::try_to_diagnose_runtime_error(&err, &circuit.abi, &circuit.debug)
    {
        let debug_artifact =
            DebugArtifact { debug_symbols: circuit.debug, file_map: circuit.file_map };

        diagnostic.report(&debug_artifact, false);
    }
}

/// Print information about the witness and compare to expectations,
/// returning errors if something isn't right.
fn save_witness(
    circuit: CompiledProgram,
    args: &ExecuteCommand,
    circuit_name: &str,
    solved: SolvedWitnesses,
) -> eyre::Result<()> {
    println!("[{}] Circuit witness successfully solved", circuit_name);

    if let Some(ref witness_dir) = args.output_dir {
        let witness_path = save_witness_to_dir(
            solved.witness_stack,
            &args.witness_name.clone().unwrap_or_else(|| circuit_name.to_string()),
            witness_dir,
        )?;
        println!("[{}] Witness saved to {}", circuit_name, witness_path.display());
    }

    // Check that the circuit returned a non-empty result if the ABI expects a return value.
    if let Some(ref expected) = circuit.abi.return_type {
        if solved.actual_return.is_none() {
            bail!("Missing return witness; expected a value of type {expected:?}");
        }
    }

    // Check that if the prover file contained a `return` entry then that's what we got.
    if let Some(expected) = solved.expected_return {
        match solved.actual_return {
            None => {
                bail!("Missing return witness;\nexpected:\n{expected:?}");
            }
            Some(actual) if actual != expected => {
                bail!("Unexpected return witness;\nexpected:\n{expected:?}\ngot:\n{actual:?}");
            }
            _ => {}
        }
    }

    Ok(())
}
