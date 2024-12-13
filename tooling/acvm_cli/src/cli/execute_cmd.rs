use std::io::{self, Write};

use acir::circuit::Program;
use acir::native_types::{WitnessMap, WitnessStack};
use acir::FieldElement;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use nargo::PrintOutput;

use crate::cli::fs::inputs::{read_bytecode_from_file, read_inputs_from_file};
use crate::errors::CliError;
use nargo::{foreign_calls::DefaultForeignCallExecutor, ops::execute_program};

use super::fs::witness::{create_output_witness_string, save_witness_to_dir};

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
    let output_witness = execute_program_from_witness(circuit_inputs, &bytecode)?;
    assert_eq!(output_witness.length(), 1, "ACVM CLI only supports a witness stack of size 1");
    let output_witness_string = create_output_witness_string(
        &output_witness.peek().expect("Should have a witness stack item").witness,
    )?;
    if args.output_witness.is_some() {
        save_witness_to_dir(
            output_witness,
            &args.output_witness.unwrap(),
            &args.working_directory,
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
    inputs_map: WitnessMap<FieldElement>,
    bytecode: &[u8],
) -> Result<WitnessStack<FieldElement>, CliError> {
    let program: Program<FieldElement> = Program::deserialize_program(bytecode)
        .map_err(|_| CliError::CircuitDeserializationError())?;
    execute_program(
        &program,
        inputs_map,
        &Bn254BlackBoxSolver,
        &mut DefaultForeignCallExecutor::new(PrintOutput::Stdout, None, None, None),
    )
    .map_err(CliError::CircuitExecutionError)
}
