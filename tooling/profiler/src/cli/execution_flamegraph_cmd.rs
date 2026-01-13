use std::path::{Path, PathBuf};

use acir::circuit::Opcode;
use acir::circuit::OpcodeLocation;
use clap::Args;
use color_eyre::eyre::{self, Context};
use nargo::errors::try_to_diagnose_runtime_error;
use nargo::foreign_calls::DefaultForeignCallBuilder;
use noir_artifact_cli::fs::artifact::read_program_from_file;
use noir_artifact_cli::fs::inputs::read_inputs_from_file;
use noirc_artifacts::program::ProgramArtifact;

use crate::errors::{CliError, report_error};
use crate::flamegraph::{BrilligExecutionSample, FlamegraphGenerator, InfernoFlamegraphGenerator};
use crate::opcode_formatter::format_brillig_opcode;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use noirc_artifacts::debug::DebugArtifact;

/// Generates a flamegraph mapping unconstrained Noir execution to source code.
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
    output: Option<PathBuf>,

    /// Use pedantic ACVM solving, i.e. double-check some black-box function
    /// assumptions when solving.
    /// This is disabled by default.
    #[clap(long, default_value = "false")]
    pedantic_solving: bool,

    /// A single number representing the total opcodes executed.
    /// Outputs to stdout and skips generating a flamegraph.
    #[clap(long, default_value = "false")]
    sample_count: bool,

    /// Enables additional logging
    #[clap(long, default_value = "false")]
    verbose: bool,
}

pub(crate) fn run(args: ExecutionFlamegraphCommand) -> eyre::Result<()> {
    run_with_generator(
        &args.artifact_path,
        &args.prover_toml_path,
        &InfernoFlamegraphGenerator { count_name: "samples".to_string() },
        &args.output,
        args.pedantic_solving,
        args.sample_count,
        args.verbose,
    )
}

fn run_with_generator(
    artifact_path: &Path,
    prover_toml_path: &Path,
    flamegraph_generator: &impl FlamegraphGenerator,
    output_path: &Option<PathBuf>,
    pedantic_solving: bool,
    print_sample_count: bool,
    verbose: bool,
) -> eyre::Result<()> {
    let program =
        read_program_from_file(artifact_path).context("Error reading program from file")?;

    ensure_brillig_entry_point(&program)?;

    if !print_sample_count && output_path.is_none() {
        return report_error("Missing --output <OUTPUT> argument for when building a flamegraph")
            .map_err(Into::into);
    }

    let (inputs_map, _) =
        read_inputs_from_file(&prover_toml_path.with_extension("toml"), &program.abi)?;

    let initial_witness = program.abi.encode(&inputs_map, None)?;

    if verbose {
        println!("Executing...");
    }

    let solved_witness_stack_err = nargo::ops::execute_program_with_profiling(
        &program.bytecode,
        initial_witness,
        &Bn254BlackBoxSolver(pedantic_solving),
        &mut DefaultForeignCallBuilder::default().with_output(std::io::stdout()).build(),
    );
    let mut profiling_samples = match solved_witness_stack_err {
        Ok((_, profiling_samples)) => profiling_samples,
        Err(err) => {
            let debug_artifact = DebugArtifact {
                debug_symbols: program.debug_symbols.debug_infos.clone(),
                file_map: program.file_map.clone(),
            };

            if let Some(diagnostic) = try_to_diagnose_runtime_error(
                &err,
                &program.abi,
                &program.debug_symbols.debug_infos,
            ) {
                diagnostic.report(&debug_artifact, false);
            }

            return Err(CliError::Generic.into());
        }
    };

    if verbose {
        println!("Executed");
    }

    if print_sample_count {
        println!("{}", profiling_samples.len());
        return Ok(());
    }

    // We place this logging output before the transforming and collection of the samples.
    // This is done because large traces can take some time, and can make it look
    // as if the profiler has stalled.
    if verbose {
        println!("Generating flamegraph for {} samples...", profiling_samples.len());
    }

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

    let output_path =
        output_path.as_ref().expect("Should have already checked for the output path");

    let debug_artifact: DebugArtifact = program.into();
    flamegraph_generator.generate_flamegraph(
        profiling_samples,
        &debug_artifact.debug_symbols[0],
        &debug_artifact,
        artifact_path.to_str().unwrap(),
        "main",
        &Path::new(output_path).join(Path::new(&format!("{}_brillig_trace.svg", "main"))),
    )?;

    if verbose {
        println!("Generated flamegraph");
    }

    Ok(())
}

fn ensure_brillig_entry_point(artifact: &ProgramArtifact) -> Result<(), CliError> {
    let err_msg = "Command only supports fully unconstrained Noir programs e.g. `unconstrained fn main() { .. }";
    let program = &artifact.bytecode;
    if program.functions.len() != 1 || program.unconstrained_functions.len() != 1 {
        return report_error(err_msg);
    }

    let main_function = &program.functions[0];
    let Opcode::BrilligCall { id, .. } = main_function.opcodes[0] else {
        return report_error(err_msg);
    };

    if id.as_usize() != 0 {
        return report_error(err_msg);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use acir::circuit::{Circuit, Program, brillig::BrilligBytecode};
    use color_eyre::eyre;
    use fm::codespan_files::Files;
    use noirc_artifacts::{
        debug::{DebugInfo, ProgramDebugInfo},
        program::ProgramArtifact,
    };
    use noirc_driver::CrateName;
    use std::{collections::BTreeMap, path::Path, str::FromStr};

    use crate::flamegraph::Sample;

    #[derive(Default)]
    struct TestFlamegraphGenerator {}

    impl super::FlamegraphGenerator for TestFlamegraphGenerator {
        fn generate_flamegraph<'files, S: Sample>(
            &self,
            _samples: Vec<S>,
            _debug_symbols: &DebugInfo,
            _files: &'files impl Files<'files, FileId = fm::FileId>,
            _artifact_name: &str,
            _function_name: &str,
            output_path: &Path,
        ) -> eyre::Result<()> {
            let output_file = std::fs::File::create(output_path).unwrap();
            std::io::Write::write_all(&mut std::io::BufWriter::new(output_file), b"success")
                .unwrap();

            Ok(())
        }
    }

    #[test]
    fn error_reporter_smoke_test() {
        // This test purposefully uses an artifact that does not represent a Brillig entry point.
        // The goal is to see that our program fails gracefully and does not panic.
        let temp_dir = tempfile::tempdir().unwrap();

        let prover_toml_path = temp_dir.path().join("Prover.toml");

        let artifact = ProgramArtifact {
            noir_version: "0.0.0".to_string(),
            hash: 27,
            abi: noirc_abi::Abi::default(),
            bytecode: Program {
                functions: vec![Circuit {
                    function_name: "main".to_string(),
                    ..Circuit::default()
                }],
                unconstrained_functions: vec![
                    BrilligBytecode::default(),
                    BrilligBytecode::default(),
                ],
            },
            debug_symbols: ProgramDebugInfo { debug_infos: vec![DebugInfo::default()] },
            file_map: BTreeMap::default(),
            expression_width: acir::circuit::ExpressionWidth::Bounded { width: 4 },
        };

        // Write the artifact to a file
        let artifact_path = noir_artifact_cli::fs::artifact::save_program_to_file(
            &artifact,
            &CrateName::from_str("test").unwrap(),
            temp_dir.path(),
        )
        .unwrap();

        let flamegraph_generator = TestFlamegraphGenerator::default();

        assert!(
            super::run_with_generator(
                &artifact_path,
                &prover_toml_path,
                &flamegraph_generator,
                &Some(temp_dir.keep()),
                false,
                false,
                false,
            )
            .is_err()
        );
    }
}
