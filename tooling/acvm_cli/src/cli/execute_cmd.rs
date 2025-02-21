use std::io::{self, Write};
use std::path::PathBuf;

use acir::circuit::Program;
use acir::native_types::{WitnessMap, WitnessStack};
use acir::FieldElement;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use clap::Args;
use nargo::PrintOutput;

use nargo::{foreign_calls::DefaultForeignCallBuilder, ops::execute_program};
use noir_artifact_cli::errors::CliError;
use noir_artifact_cli::fs::artifact::read_bytecode_from_file;
use noir_artifact_cli::fs::witness::save_witness_to_dir;

use crate::fs::witness::{create_output_witness_string, read_witness_from_file};

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
    working_directory: PathBuf,

    /// Set to print output witness to stdout
    #[clap(long, short, action)]
    print: bool,

    /// Use pedantic ACVM solving, i.e. double-check some black-box function
    /// assumptions when solving.
    /// This is disabled by default.
    #[clap(long, default_value = "false")]
    pedantic_solving: bool,
}

fn run_command(args: ExecuteCommand) -> Result<String, CliError> {
    let bytecode = read_bytecode_from_file(&args.working_directory, &args.bytecode)?;
    let input_witness = read_witness_from_file(&args.working_directory.join(&args.input_witness))?;
    let output_witness =
        execute_program_from_witness(input_witness, &bytecode, args.pedantic_solving)?;
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
    pedantic_solving: bool,
) -> Result<WitnessStack<FieldElement>, CliError> {
    let program: Program<FieldElement> =
        Program::deserialize_program(bytecode).map_err(CliError::CircuitDeserializationError)?;
    execute_program(
        &program,
        inputs_map,
        &Bn254BlackBoxSolver(pedantic_solving),
        &mut DefaultForeignCallBuilder {
            output: PrintOutput::Stdout,
            enable_mocks: false,
            ..Default::default()
        }
        .build(),
    )
    .map_err(CliError::CircuitExecutionError)
}
