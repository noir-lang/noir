use std::path::{Path, PathBuf};

use clap::Args;
use color_eyre::eyre::{self, Context};

use noirc_artifacts::debug::DebugArtifact;

use crate::flamegraph::{FlamegraphGenerator, InfernoFlamegraphGenerator};
use crate::fs::read_program_from_file;

#[derive(Debug, Clone, Args)]
pub(crate) struct OpcodesFlamegraphCommand {
    /// The path to the artifact JSON file
    #[clap(long, short)]
    artifact_path: String,

    /// The output folder for the flamegraph svg files
    #[clap(long, short)]
    output: String,
}

pub(crate) fn run(args: OpcodesFlamegraphCommand) -> eyre::Result<()> {
    run_with_generator(
        &PathBuf::from(args.artifact_path),
        &InfernoFlamegraphGenerator { count_name: "opcodes".to_string() },
        &PathBuf::from(args.output),
    )
}

fn run_with_generator<Generator: FlamegraphGenerator>(
    artifact_path: &Path,
    flamegraph_generator: &Generator,
    output_path: &Path,
) -> eyre::Result<()> {
    let mut program =
        read_program_from_file(artifact_path).context("Error reading program from file")?;

    let function_names = program.names.clone();

    let bytecode = std::mem::take(&mut program.bytecode);

    let debug_artifact: DebugArtifact = program.into();

    for (func_idx, (func_name, bytecode)) in
        function_names.into_iter().zip(bytecode.functions).enumerate()
    {
        println!("Opcode count: {}", bytecode.opcodes.len());

        flamegraph_generator.generate_flamegraph(
            bytecode.opcodes.iter().map(|_op| 1).collect(),
            bytecode.opcodes,
            &debug_artifact.debug_symbols[func_idx],
            &debug_artifact,
            artifact_path.to_str().unwrap(),
            &func_name,
            &Path::new(&output_path).join(Path::new(&format!("{}_opcodes.svg", &func_name))),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use acir::circuit::{Circuit, Opcode, Program};
    use color_eyre::eyre::{self};
    use fm::codespan_files::Files;
    use noirc_artifacts::program::ProgramArtifact;
    use noirc_errors::debug_info::{DebugInfo, ProgramDebugInfo};
    use std::{collections::BTreeMap, path::Path};

    #[derive(Default)]
    struct TestFlamegraphGenerator {}

    impl super::FlamegraphGenerator for TestFlamegraphGenerator {
        fn generate_flamegraph<'files, F>(
            &self,
            _samples_per_opcode: Vec<usize>,
            _opcodes: Vec<Opcode<F>>,
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
        };

        // Write the artifact to a file
        let artifact_file = std::fs::File::create(&artifact_path).unwrap();
        serde_json::to_writer(artifact_file, &artifact).unwrap();

        let flamegraph_generator = TestFlamegraphGenerator::default();

        super::run_with_generator(&artifact_path, &flamegraph_generator, temp_dir.path())
            .expect("should run without errors");

        // Check that the output file was written to
        let output_file = temp_dir.path().join("main_opcodes.svg");
        assert!(output_file.exists());
    }
}
