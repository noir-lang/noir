use std::path::{Path, PathBuf};

use acir::circuit::OpcodeLocation;
use clap::Args;
use color_eyre::eyre::{self, Context};
use nargo::foreign_calls::DefaultForeignCallBuilder;
use nargo::PrintOutput;

use crate::flamegraph::{BrilligExecutionSample, FlamegraphGenerator, InfernoFlamegraphGenerator};
use crate::fs::{read_inputs_from_file, read_program_from_file};
use crate::opcode_formatter::format_brillig_opcode;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use noirc_abi::input_parser::Format;
use noirc_artifacts::debug::DebugArtifact;

#[derive(Debug, Clone, Args)]
pub(crate) struct ExecutionFlamegraphCommand {
    /// The path to the artifact JSON file
    #[clap(long, short)]
    artifact_path: PathBuf,

    /// The path to the Prover.toml file
    #[clap(long, short)]
    prover_toml_path: PathBuf,

    /// The output folder for the flamegraph svg files
    #[clap(long, short)]
    output: PathBuf,

    /// Use pedantic ACVM solving, i.e. double-check some black-box function
    /// assumptions when solving.
    /// This is disabled by default.
    #[clap(long, default_value = "false")]
    pedantic_solving: bool,
}

pub(crate) fn run(args: ExecutionFlamegraphCommand) -> eyre::Result<()> {
    run_with_generator(
        &args.artifact_path,
        &args.prover_toml_path,
        &InfernoFlamegraphGenerator { count_name: "samples".to_string() },
        &args.output,
        args.pedantic_solving,
    )
}

fn run_with_generator(
    artifact_path: &Path,
    prover_toml_path: &Path,
    flamegraph_generator: &impl FlamegraphGenerator,
    output_path: &Path,
    pedantic_solving: bool,
) -> eyre::Result<()> {
    let program =
        read_program_from_file(artifact_path).context("Error reading program from file")?;

    let (inputs_map, _) = read_inputs_from_file(prover_toml_path, Format::Toml, &program.abi)?;

    let initial_witness = program.abi.encode(&inputs_map, None)?;

    println!("Executing");
    let (_, mut profiling_samples) = nargo::ops::execute_program_with_profiling(
        &program.bytecode,
        initial_witness,
        &Bn254BlackBoxSolver(pedantic_solving),
        &mut DefaultForeignCallBuilder::default().with_output(PrintOutput::Stdout).build(),
    )?;
    println!("Executed");

    println!("Collecting {} samples", profiling_samples.len());

    let profiling_samples: Vec<BrilligExecutionSample> = profiling_samples
        .iter_mut()
        .map(|sample| {
            let call_stack = std::mem::take(&mut sample.call_stack);
            let brillig_function_id = std::mem::take(&mut sample.brillig_function_id);
            let last_entry = call_stack.last();
            let opcode = brillig_function_id
                .and_then(|id| program.bytecode.unconstrained_functions.get(id.0 as usize))
                .and_then(|func| {
                    if let Some(OpcodeLocation::Brillig { brillig_index, .. }) = last_entry {
                        func.bytecode.get(*brillig_index)
                    } else {
                        None
                    }
                })
                .map(format_brillig_opcode);
            BrilligExecutionSample { opcode, call_stack, brillig_function_id }
        })
        .collect();

    let debug_artifact: DebugArtifact = program.into();

    println!("Generating flamegraph with {} samples", profiling_samples.len());

    flamegraph_generator.generate_flamegraph(
        profiling_samples,
        &debug_artifact.debug_symbols[0],
        &debug_artifact,
        artifact_path.to_str().unwrap(),
        "main",
        &Path::new(&output_path).join(Path::new(&format!("{}.svg", "main"))),
    )?;

    Ok(())
}
