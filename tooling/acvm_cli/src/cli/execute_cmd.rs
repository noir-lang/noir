use std::io::{self, Write};

use acir::circuit::Circuit;
use acir::native_types::WitnessMap;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;

use crate::cli::fs::inputs::{read_bytecode_from_file, read_inputs_from_file};
use crate::cli::fs::witness::save_witness_to_dir;
use crate::errors::CliError;
use nargo::ops::{execute_circuit, DefaultForeignCallExecutor};

use super::fs::witness::create_output_witness_string;

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
pub(crate) struct ExecuteCommand {
    /// Write the execution witness to named file
    #[clap(long, short)]
    output_witness: Option<String>,

    /// The name of the toml file which contains the input witness map
    #[clap(long, short)]
    input_witness: String,

    /// The name of the binary file containing circuit bytecode
    #[clap(long, short)]
    bytecode: String,

    /// The working directory
    #[clap(long, short)]
    working_directory: String,

    /// Set to print output witness to stdout
    #[clap(long, short, action)]
    print: bool,
}

fn run_command(args: ExecuteCommand) -> Result<String, CliError> {
    let bytecode = read_bytecode_from_file(&args.working_directory, &args.bytecode)?;
    let circuit_inputs = read_inputs_from_file(&args.working_directory, &args.input_witness)?;
    let output_witness = execute_program_from_witness(&circuit_inputs, &bytecode, None)?;
    let output_witness_string = create_output_witness_string(&output_witness)?;
    if args.output_witness.is_some() {
        save_witness_to_dir(
            &output_witness_string,
            &args.working_directory,
            &args.output_witness.unwrap(),
        )?;
    }
    Ok(output_witness_string)
}

pub(crate) fn run(args: ExecuteCommand) -> Result<String, CliError> {
    let print = args.print;
    let output_witness_string = run_command(args)?;
    if print {
        io::stdout().write_all(output_witness_string.as_bytes()).unwrap();
    }
    Ok(output_witness_string)
}

pub(crate) fn execute_program_from_witness(
    inputs_map: &WitnessMap,
    bytecode: &[u8],
    foreign_call_resolver_url: Option<&str>,
) -> Result<WitnessMap, CliError> {
    let blackbox_solver = Bn254BlackBoxSolver::new();
    let circuit: Circuit = Circuit::deserialize_circuit(bytecode)
        .map_err(|_| CliError::CircuitDeserializationError())?;
    execute_circuit(
        &circuit,
        inputs_map.clone(),
        &blackbox_solver,
        &mut DefaultForeignCallExecutor::new(true, foreign_call_resolver_url),
    )
    .map_err(CliError::CircuitExecutionError)
}
