use std::{collections::BTreeMap, path::PathBuf};

use acir::{circuit::Program, native_types::WitnessStack, FieldElement};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use color_eyre::eyre::{self, bail};

use nargo::{foreign_calls::DefaultForeignCallBuilder, NargoError, PrintOutput};
use noir_artifact_cli::{errors::CliError, fs::inputs::read_inputs_from_file, Artifact};
use noirc_abi::{input_parser::InputValue, Abi};
use noirc_artifacts::{
    contract::ContractFunctionArtifact, debug::DebugArtifact, program::ProgramArtifact,
};

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

    let circuit = match artifact {
        Artifact::Program(program) => Circuit {
            abi: program.abi,
            bytecode: program.bytecode,
            debug_symbols: program.debug_symbols,
            file_map: program.file_map,
        },
        Artifact::Contract(contract) => {
            let names =
                contract.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(",");

            let Some(ref name) = args.contract_fn else {
                bail!("--contract-fn missing; options: [{names}]");
            };
            let Some(function) = contract.functions.into_iter().find(|f| f.name == *name) else {
                bail!("unknown --contract-fn '{name}'; options: [{names}]");
            };

            Circuit {
                abi: function.abi,
                bytecode: function.bytecode,
                debug_symbols: function.debug_symbols,
                file_map: contract.file_map,
            }
        }
    };

    match execute(&circuit, &args) {
        Ok(solved_witnesses) => {
            todo!("save witness with program name");
            todo!("check that the witness is not empty");
            todo!("check that the witness is what was expected");
        }
        Err(CliError::CircuitExecutionError(err)) => {
            show_diagnostic(circuit, err);
        }
        Err(e) => {
            bail!("error executing circuit: {e}");
        }
    }
    Ok(())
}

/// Parameters necessary to execute a circuit, display execution failures, etc.
struct Circuit {
    abi: Abi,
    bytecode: Program<FieldElement>,
    debug_symbols: noirc_errors::debug_info::ProgramDebugInfo,
    file_map: BTreeMap<fm::FileId, noirc_driver::DebugFile>,
}

struct SolvedWitnesses {
    expected_return: Option<InputValue>,
    actual_return: Option<InputValue>,
    witness_stack: WitnessStack<FieldElement>,
}

/// Execute a circuit and return the output witnesses.
fn execute(circuit: &Circuit, args: &ExecuteCommand) -> Result<SolvedWitnesses, CliError> {
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
        &circuit.bytecode,
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
fn show_diagnostic(circuit: Circuit, err: NargoError<FieldElement>) {
    if let Some(diagnostic) = nargo::errors::try_to_diagnose_runtime_error(
        &err,
        &circuit.abi,
        &circuit.debug_symbols.debug_infos,
    ) {
        let debug_artifact = DebugArtifact {
            debug_symbols: circuit.debug_symbols.debug_infos,
            file_map: circuit.file_map,
        };
        diagnostic.report(&debug_artifact, false);
    }
}
