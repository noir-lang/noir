use std::path::{Path, PathBuf};

use acir::circuit::OpcodeLocation;
use clap::Args;
use color_eyre::eyre::{self, Context};

use noirc_artifacts::debug::DebugArtifact;

use crate::flamegraph::{CompilationSample, FlamegraphGenerator, InfernoFlamegraphGenerator};
use crate::fs::read_program_from_file;
use crate::gates_provider::{BackendGatesProvider, GatesProvider};
use crate::opcode_formatter::format_acir_opcode;

#[derive(Debug, Clone, Args)]
pub(crate) struct GatesFlamegraphCommand {
    /// The path to the artifact JSON file
    #[clap(long, short)]
    artifact_path: String,

    /// Path to the noir backend binary
    #[clap(long, short)]
    backend_path: String,

    /// Command to get a gates report from the backend. Defaults to "gates"
    #[clap(long, short = 'g', default_value = "gates")]
    backend_gates_command: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    backend_extra_args: Vec<String>,

    /// The output folder for the flamegraph svg files
    #[clap(long, short)]
    output: String,

    /// The output name for the flamegraph svg files
    #[clap(long, short = 'f')]
    output_filename: Option<String>,
}

pub(crate) fn run(args: GatesFlamegraphCommand) -> eyre::Result<()> {
    run_with_provider(
        &PathBuf::from(args.artifact_path),
        &BackendGatesProvider {
            backend_path: PathBuf::from(args.backend_path),
            gates_command: args.backend_gates_command,
            extra_args: args.backend_extra_args,
        },
        &InfernoFlamegraphGenerator { count_name: "gates".to_string() },
        &PathBuf::from(args.output),
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

    let function_names = program.names.clone();

    let bytecode = std::mem::take(&mut program.bytecode);

    let debug_artifact: DebugArtifact = program.into();

    for (func_idx, ((func_gates, func_name), bytecode)) in backend_gates_response
        .functions
        .into_iter()
        .zip(function_names)
        .zip(bytecode.functions)
        .enumerate()
    {
        println!(
            "Opcode count: {}, Total gates by opcodes: {}, Circuit size: {}",
            func_gates.acir_opcodes,
            func_gates.gates_per_opcode.iter().sum::<usize>(),
            func_gates.circuit_size
        );

        let samples = func_gates
            .gates_per_opcode
            .into_iter()
            .zip(bytecode.opcodes)
            .enumerate()
            .map(|(index, (gates, opcode))| CompilationSample {
                opcode: Some(format_acir_opcode(&opcode)),
                call_stack: vec![OpcodeLocation::Acir(index)],
                count: gates,
                brillig_function_id: None,
            })
            .collect();

        let output_filename = if let Some(output_filename) = &output_filename {
            format!("{}::{}::gates.svg", output_filename, func_name)
        } else {
            format!("{}::gates.svg", func_name)
        };
        flamegraph_generator.generate_flamegraph(
            samples,
            &debug_artifact.debug_symbols[func_idx],
            &debug_artifact,
            artifact_path.to_str().unwrap(),
            &func_name,
            &Path::new(&output_path).join(Path::new(&output_filename)),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use acir::circuit::{Circuit, Program};
    use color_eyre::eyre::{self};
    use fm::codespan_files::Files;
    use noirc_artifacts::program::ProgramArtifact;
    use noirc_errors::debug_info::{DebugInfo, ProgramDebugInfo};
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
            bytecode: Program { functions: vec![Circuit::default()], ..Program::default() },
            debug_symbols: ProgramDebugInfo { debug_infos: vec![DebugInfo::default()] },
            file_map: BTreeMap::default(),
            names: vec!["main".to_string()],
            brillig_names: Vec::new(),
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
        let output_file = temp_dir.path().join("test_filename::main::gates.svg");
        assert!(output_file.exists());
    }
}
