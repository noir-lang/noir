use std::{collections::BTreeMap, path::PathBuf};

use acir::{FieldElement, circuit::Program, native_types::WitnessStack};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use color_eyre::eyre::{self, bail};

use crate::{
    Artifact,
    errors::CliError,
    fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir},
};
use nargo::{NargoError, PrintOutput, foreign_calls::DefaultForeignCallBuilder};
use noirc_abi::{Abi, input_parser::InputValue};
use noirc_artifacts::debug::DebugArtifact;

use super::parse_and_normalize_path;

/// Execute a binary program or a circuit artifact.
#[derive(Debug, Clone, Args)]
pub struct ExecuteCommand {
    /// Path to the JSON build artifact (either a program or a contract).
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    artifact_path: PathBuf,

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

    /// JSON RPC url to solve oracle calls.
    #[clap(long)]
    oracle_resolver: Option<String>,

    /// Use pedantic ACVM solving, i.e. double-check some black-box function assumptions when solving.
    #[clap(long, default_value_t = false)]
    pedantic_solving: bool,
}

pub fn run(args: ExecuteCommand) -> eyre::Result<()> {
    let artifact = Artifact::read_from_file(&args.artifact_path)?;

    let circuit = match artifact {
        Artifact::Program(program) => Circuit {
            name: None,
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
                name: Some(name.clone()),
                abi: function.abi,
                bytecode: function.bytecode,
                debug_symbols: function.debug_symbols,
                file_map: contract.file_map,
            }
        }
    };

    match execute(&circuit, &args) {
        Ok(solved) => {
            save_witness(circuit, args, solved)?;
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

/// Parameters necessary to execute a circuit, display execution failures, etc.
struct Circuit {
    name: Option<String>,
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

/// Print information about the witness and compare to expectations,
/// returning errors if something isn't right.
fn save_witness(
    circuit: Circuit,
    args: ExecuteCommand,
    solved: SolvedWitnesses,
) -> eyre::Result<()> {
    let artifact = args.artifact_path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let name = circuit
        .name
        .as_ref()
        .map(|name| format!("{artifact}.{name}"))
        .unwrap_or_else(|| artifact.to_string());

    println!("[{}] Circuit witness successfully solved", name);

    if let Some(ref witness_dir) = args.output_dir {
        let witness_path = save_witness_to_dir(
            solved.witness_stack,
            &args.witness_name.unwrap_or_else(|| name.clone()),
            witness_dir,
        )?;
        println!("[{}] Witness saved to {}", name, witness_path.display());
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
