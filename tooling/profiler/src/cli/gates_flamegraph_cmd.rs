use std::path::{Path, PathBuf};

use acir::circuit::OpcodeLocation;
use clap::Args;
use color_eyre::eyre::{self, Context};

use noir_artifact_cli::fs::artifact::read_program_from_file;
use noirc_artifacts::debug::DebugArtifact;

use crate::flamegraph::{CompilationSample, FlamegraphGenerator, InfernoFlamegraphGenerator};
use crate::gates_provider::{BackendGatesProvider, GatesProvider};
use crate::opcode_formatter::format_acir_opcode;

/// Generates a flamegraph mapping backend opcodes to their associated locations in the source code.
#[derive(Debug, Clone, Args)]
pub(crate) struct GatesFlamegraphCommand {
    /// The path to the artifact JSON file
    #[clap(long, short)]
    artifact_path: PathBuf,

    /// Path to the Noir backend binary
    #[clap(long, short)]
    backend_path: PathBuf,

    /// Command to get a gates report from the backend. Defaults to "gates"
    #[clap(long, short = 'g', default_value = "gates")]
    backend_gates_command: String,

    /// Optional arguments for the backend gates command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    backend_extra_args: Vec<String>,

    /// The output folder for the flamegraph svg files
    #[clap(long, short)]
    output: PathBuf,

    /// The output name for the flamegraph svg files
    #[clap(long, short = 'f')]
    output_filename: Option<String>,
}

pub(crate) fn run(args: GatesFlamegraphCommand) -> eyre::Result<()> {
    run_with_provider(
        &args.artifact_path,
        &BackendGatesProvider {
            backend_path: args.backend_path,
            gates_command: args.backend_gates_command,
            extra_args: args.backend_extra_args,
        },
        &InfernoFlamegraphGenerator { count_name: "gates".to_string() },
        &args.output,
        args.output_filename,
    )
}

fn run_with_provider<Provider: GatesProvider, Generator: FlamegraphGenerator>(
    artifact_path: &Path,
    gates_provider: &Provider,
    flamegraph_generator: &Generator,
    output_path: &Path,
    output_filename: Option<String>,
) -> eyre::Result<()> {
    let mut program =
        read_program_from_file(artifact_path).context("Error reading program from file")?;

    let backend_gates_response =
        gates_provider.get_gates(artifact_path).context("Error querying backend for gates")?;

    let bytecode = std::mem::take(&mut program.bytecode);
    let debug_artifact: DebugArtifact = program.into();

    let num_functions = bytecode.functions.len();
    for (func_idx, (func_gates, circuit)) in
        backend_gates_response.functions.into_iter().zip(bytecode.functions).enumerate()
    {
        // We can have repeated names if there are functions with the same name in different
        // modules or functions that use generics. Thus, add the unique function index as a suffix.
        let function_name = if num_functions > 1 {
            format!("{}_{}", circuit.function_name.as_str(), func_idx)
        } else {
            circuit.function_name.to_owned()
        };

        println!(
            "Opcode count: {}, Total gates by opcodes: {}, Circuit size: {}",
            func_gates.acir_opcodes,
            func_gates.gates_per_opcode.iter().sum::<usize>(),
            func_gates.circuit_size
        );

        let samples = func_gates
            .gates_per_opcode
            .into_iter()
            .zip(circuit.opcodes)
            .enumerate()
            .map(|(index, (gates, opcode))| CompilationSample {
                opcode: Some(format_acir_opcode(&opcode)),
                call_stack: vec![OpcodeLocation::Acir(index)],
                count: gates,
                brillig_function_id: None,
            })
            .collect();

        let output_filename = if let Some(output_filename) = &output_filename {
            format!("{output_filename}_{function_name}_gates.svg")
        } else {
            format!("{function_name}_gates.svg")
        };
        flamegraph_generator.generate_flamegraph(
            samples,
            &debug_artifact.debug_symbols[func_idx],
            &debug_artifact,
            artifact_path.to_str().unwrap(),
            &function_name,
            &Path::new(&output_path).join(Path::new(&output_filename)),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use acir::circuit::{Circuit, Program};
    use color_eyre::eyre;
    use fm::codespan_files::Files;
    use noirc_artifacts::{
        debug::{DebugInfo, ProgramDebugInfo},
        program::ProgramArtifact,
    };
    use std::{
        collections::{BTreeMap, HashMap},
        path::{Path, PathBuf},
    };

    use crate::{
        flamegraph::Sample,
        gates_provider::{BackendGatesReport, BackendGatesResponse, GatesProvider},
    };

    struct TestGateProvider {
        mock_responses: HashMap<PathBuf, BackendGatesResponse>,
    }

    impl GatesProvider for TestGateProvider {
        fn get_gates(&self, artifact_path: &Path) -> eyre::Result<BackendGatesResponse> {
            let response = self
                .mock_responses
                .get(artifact_path)
                .expect("should have a mock response for the artifact path");

            Ok(response.clone())
        }
    }

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
    fn smoke_test() {
        let temp_dir = tempfile::tempdir().unwrap();

        let artifact_path = temp_dir.path().join("test.json");

        let artifact = ProgramArtifact {
            noir_version: "0.0.0".to_string(),
            hash: 27,
            abi: noirc_abi::Abi::default(),
            bytecode: Program {
                functions: vec![Circuit {
                    function_name: "main".to_string(),
                    ..Circuit::default()
                }],
                ..Program::default()
            },
            debug_symbols: ProgramDebugInfo { debug_infos: vec![DebugInfo::default()] },
            file_map: BTreeMap::default(),
            expression_width: acir::circuit::ExpressionWidth::Bounded { width: 4 },
        };

        // Write the artifact to a file
        let artifact_file = std::fs::File::create(&artifact_path).unwrap();
        serde_json::to_writer(artifact_file, &artifact).unwrap();

        let mock_gates_response = BackendGatesResponse {
            functions: vec![
                (BackendGatesReport { acir_opcodes: 0, gates_per_opcode: vec![], circuit_size: 0 }),
            ],
        };

        let provider = TestGateProvider {
            mock_responses: HashMap::from([(artifact_path.clone(), mock_gates_response)]),
        };
        let flamegraph_generator = TestFlamegraphGenerator::default();

        super::run_with_provider(
            &artifact_path,
            &provider,
            &flamegraph_generator,
            temp_dir.path(),
            Some(String::from("test_filename")),
        )
        .expect("should run without errors");

        // Check that the output file was written to
        let output_file = temp_dir.path().join("test_filename_main_gates.svg");
        assert!(output_file.exists());
    }
}
