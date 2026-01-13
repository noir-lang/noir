use std::path::{Path, PathBuf};

use acir::AcirField;
use acir::circuit::brillig::BrilligFunctionId;
use acir::circuit::{Circuit, Opcode, OpcodeLocation};
use clap::Args;
use color_eyre::eyre::{self, Context};

use noir_artifact_cli::fs::artifact::read_program_from_file;
use noirc_artifacts::debug::DebugArtifact;

use crate::flamegraph::{CompilationSample, FlamegraphGenerator, InfernoFlamegraphGenerator};
use crate::opcode_formatter::{format_acir_opcode, format_brillig_opcode};

/// Generates a flamegraph mapping ACIR opcodes to their associated locations in the source code.
#[derive(Debug, Clone, Args)]
pub(crate) struct OpcodesFlamegraphCommand {
    /// The path to the artifact JSON file
    #[clap(long, short)]
    artifact_path: PathBuf,

    /// The output folder for the flamegraph svg files
    #[clap(long, short)]
    output: PathBuf,

    /// Whether to skip brillig functions
    #[clap(long, short, action)]
    skip_brillig: bool,
}

pub(crate) fn run(args: OpcodesFlamegraphCommand) -> eyre::Result<()> {
    run_with_generator(
        &args.artifact_path,
        &InfernoFlamegraphGenerator { count_name: "opcodes".to_string() },
        &args.output,
        args.skip_brillig,
    )
}

fn run_with_generator<Generator: FlamegraphGenerator>(
    artifact_path: &Path,
    flamegraph_generator: &Generator,
    output_path: &Path,
    skip_brillig: bool,
) -> eyre::Result<()> {
    let mut program =
        read_program_from_file(artifact_path).context("Error reading program from file")?;

    let bytecode = std::mem::take(&mut program.bytecode);
    let debug_artifact: DebugArtifact = program.into();

    for (func_idx, circuit) in bytecode.functions.iter().enumerate() {
        // We can have repeated names if there are functions with the same name in different
        // modules or functions that use generics. Thus, add the unique function index as a suffix.
        let function_name = if bytecode.functions.len() > 1 {
            format!("{}_{}", circuit.function_name.as_str(), func_idx)
        } else {
            circuit.function_name.to_owned()
        };

        println!("Opcode count for {}: {}", function_name, circuit.opcodes.len());

        let samples = circuit
            .opcodes
            .iter()
            .enumerate()
            .map(|(index, opcode)| CompilationSample {
                opcode: Some(format_acir_opcode(opcode)),
                call_stack: vec![OpcodeLocation::Acir(index)],
                count: 1,
                brillig_function_id: None,
            })
            .collect();

        flamegraph_generator.generate_flamegraph(
            samples,
            &debug_artifact.debug_symbols[func_idx],
            &debug_artifact,
            artifact_path.to_str().unwrap(),
            &function_name,
            &Path::new(&output_path)
                .join(Path::new(&format!("{}_acir_opcodes.svg", &function_name))),
        )?;
    }

    if skip_brillig {
        return Ok(());
    }

    for (brillig_fn_index, brillig_bytecode) in
        bytecode.unconstrained_functions.into_iter().enumerate()
    {
        let acir_location = locate_brillig_call(brillig_fn_index, &bytecode.functions);
        let Some((acir_fn_index, acir_opcode_index)) = acir_location else {
            continue;
        };

        // We can have repeated names if there are functions with the same name in different
        // modules or functions that use generics. Thus, add the unique function index as a suffix.
        let function_name =
            format!("{}_{}", brillig_bytecode.function_name.as_str(), brillig_fn_index);

        println!("Opcode count for {}_brillig: {}", function_name, brillig_bytecode.bytecode.len());

        let samples = brillig_bytecode
            .bytecode
            .into_iter()
            .enumerate()
            .map(|(brillig_index, opcode)| CompilationSample {
                opcode: Some(format_brillig_opcode(&opcode)),
                call_stack: vec![OpcodeLocation::Brillig {
                    acir_index: acir_opcode_index,
                    brillig_index,
                }],
                count: 1,
                brillig_function_id: Some(BrilligFunctionId(brillig_fn_index as u32)),
            })
            .collect();

        flamegraph_generator.generate_flamegraph(
            samples,
            &debug_artifact.debug_symbols[acir_fn_index],
            &debug_artifact,
            artifact_path.to_str().unwrap(),
            &function_name,
            &Path::new(&output_path)
                .join(Path::new(&format!("{function_name}_brillig_opcodes.svg"))),
        )?;
    }

    Ok(())
}

fn locate_brillig_call<F: AcirField>(
    brillig_fn_index: usize,
    acir_functions: &[Circuit<F>],
) -> Option<(usize, usize)> {
    for (acir_fn_index, acir_fn) in acir_functions.iter().enumerate() {
        for (acir_opcode_index, acir_opcode) in acir_fn.opcodes.iter().enumerate() {
            match acir_opcode {
                Opcode::BrilligCall { id, .. } if id.as_usize() == brillig_fn_index => {
                    return Some((acir_fn_index, acir_opcode_index));
                }
                _ => {}
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use acir::{
        FieldElement,
        circuit::{
            Circuit, ExpressionWidth, Opcode, Program,
            brillig::{BrilligBytecode, BrilligFunctionId},
        },
        native_types::Expression,
    };
    use color_eyre::eyre;
    use fm::codespan_files::Files;
    use noirc_artifacts::{
        debug::{DebugInfo, ProgramDebugInfo},
        program::ProgramArtifact,
    };
    use std::{collections::BTreeMap, path::Path};

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
            expression_width: ExpressionWidth::Bounded { width: 4 },
        };

        // Write the artifact to a file
        let artifact_file = std::fs::File::create(&artifact_path).unwrap();
        serde_json::to_writer(artifact_file, &artifact).unwrap();

        let flamegraph_generator = TestFlamegraphGenerator::default();

        super::run_with_generator(&artifact_path, &flamegraph_generator, temp_dir.path(), true)
            .expect("should run without errors");

        // Check that the output file was written to
        let output_file = temp_dir.path().join("main_acir_opcodes.svg");
        assert!(output_file.exists());
    }

    #[test]
    fn brillig_test() {
        let temp_dir = tempfile::tempdir().unwrap();

        let artifact_path = temp_dir.path().join("test.json");

        let acir: Vec<Opcode<FieldElement>> = vec![
            Opcode::BrilligCall {
                id: BrilligFunctionId(0),
                inputs: vec![],
                outputs: vec![],
                predicate: Expression::one(),
            },
            Opcode::BrilligCall {
                id: BrilligFunctionId(1),
                inputs: vec![],
                outputs: vec![],
                predicate: Expression::one(),
            },
            Opcode::BrilligCall {
                id: BrilligFunctionId(2),
                inputs: vec![],
                outputs: vec![],
                predicate: Expression::one(),
            },
        ];

        let artifact = ProgramArtifact {
            noir_version: "0.0.0".to_string(),
            hash: 27,
            abi: noirc_abi::Abi::default(),
            bytecode: Program {
                functions: vec![Circuit {
                    function_name: "main".to_string(),
                    opcodes: acir,
                    ..Circuit::default()
                }],
                unconstrained_functions: vec![
                    BrilligBytecode { function_name: "main".to_string(), bytecode: Vec::default() },
                    BrilligBytecode { function_name: "main".to_string(), bytecode: Vec::default() },
                    BrilligBytecode {
                        function_name: "main_1".to_string(),
                        bytecode: Vec::default(),
                    },
                ],
            },
            debug_symbols: ProgramDebugInfo { debug_infos: vec![DebugInfo::default()] },
            file_map: BTreeMap::default(),
            expression_width: ExpressionWidth::Bounded { width: 4 },
        };

        // Write the artifact to a file
        let artifact_file = std::fs::File::create(&artifact_path).unwrap();
        serde_json::to_writer(artifact_file, &artifact).unwrap();

        let flamegraph_generator = TestFlamegraphGenerator::default();

        super::run_with_generator(&artifact_path, &flamegraph_generator, temp_dir.path(), false)
            .expect("should run without errors");

        // Check that the output files were written
        let output_file = temp_dir.path().join("main_acir_opcodes.svg");
        assert!(output_file.exists());

        let output_file = temp_dir.path().join("main_0_brillig_opcodes.svg");
        assert!(output_file.exists());

        let output_file = temp_dir.path().join("main_1_brillig_opcodes.svg");
        assert!(output_file.exists());

        let output_file = temp_dir.path().join("main_1_2_brillig_opcodes.svg");
        assert!(output_file.exists());
    }
}
