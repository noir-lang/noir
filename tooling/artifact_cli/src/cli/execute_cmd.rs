use std::path::PathBuf;

use acir::{circuit::Program, FieldElement};
use clap::Args;
use color_eyre::eyre::{self, bail};

use noir_artifact_cli::{fs, Artifact};
use noirc_abi::Abi;

#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Path to the JSON build artifact (either a program or a contract).
    #[clap(long, short)]
    artifact: PathBuf,

    /// Path to the Prover.toml file which contains the inputs and the
    /// optional return value in ABI format.
    #[clap(long, short)]
    prover_file: PathBuf,

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

    match artifact {
        Artifact::Program(program) => {
            let circuit = Circuit { abi: &program.abi, bytecode: &program.bytecode };
            let _witness = execute(&circuit, &args)?;
            todo!("save witness with program name");
        }
        Artifact::Contract(contract) => {
            let names =
                || contract.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(",");
            let Some(ref name) = args.contract_fn else {
                bail!("--contract-fn missing; options: [{}]", names());
            };
            let Some(function) = contract.functions.iter().find(|f| f.name == *name) else {
                bail!("unknown --contract-fn '{name}'; options: [{}]", names());
            };
            let circuit = Circuit { abi: &function.abi, bytecode: &function.bytecode };
            let _witness = execute(&circuit, &args)?;
            todo!("save witness with function name");
        }
    }
    Ok(())
}

/// Parameters necessary to execute a circuit, display execution failures, etc.
struct Circuit<'a> {
    abi: &'a Abi,
    bytecode: &'a Program<FieldElement>,
}

/// Execute a circuit and return the output witnesses.
///
/// If the execution fails display the stack trace and return an error.
fn execute(circuit: &Circuit, args: &ExecuteCommand) -> eyre::Result<()> {
    let _inputs = fs::inputs::read_inputs_from_file(&args.prover_file, circuit.abi)?;
    todo!()
}
