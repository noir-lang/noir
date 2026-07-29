use std::collections::BTreeMap;
use std::path::PathBuf;

use acir::FieldElement;
use acir::circuit::{Program, display_program};
use clap::Args;
use color_eyre::eyre;
use fm::FileId;
use noir_artifact_cli::Artifact;
use noirc_artifacts::annotations::program_opcode_annotations;
use noirc_artifacts::debug::{DebugFile, ProgramDebugInfo};

#[derive(Debug, Clone, Args)]
pub(crate) struct PrintAcirCommand {
    /// The artifact to print
    artifact: PathBuf,

    /// Name of the function to print, if the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,

    /// Annotate each opcode with the Noir source location and code snippet it was
    /// compiled from, using the debug symbols embedded in the artifact.
    #[clap(long)]
    with_locations: bool,
}

pub(crate) fn run(args: PrintAcirCommand) -> eyre::Result<()> {
    let artifact = Artifact::read_from_file(&args.artifact)?;

    match artifact {
        Artifact::Program(program) => {
            println!("Compiled ACIR for main:");
            print_program(
                &program.bytecode,
                &program.debug_symbols,
                &program.file_map,
                args.with_locations,
            );
        }
        Artifact::Contract(contract) => {
            println!("Compiled circuits for contract '{}':", contract.name);
            for function in contract
                .functions
                .into_iter()
                .filter(|f| args.contract_fn.as_ref().is_none_or(|n| *n == f.name))
            {
                println!("Compiled ACIR for function '{}':", function.name);
                print_program(
                    &function.bytecode,
                    &function.debug_symbols,
                    &contract.file_map,
                    args.with_locations,
                );
            }
        }
    }

    Ok(())
}

fn print_program(
    program: &Program<FieldElement>,
    debug_symbols: &ProgramDebugInfo,
    file_map: &BTreeMap<FileId, DebugFile>,
    with_locations: bool,
) {
    let annotations = with_locations
        .then(|| program_opcode_annotations(program, &debug_symbols.debug_infos, file_map));
    println!("{}", AnnotatedProgram { program, annotations });
}

/// Displays a [Program], attaching source-location annotations to its ACIR opcodes
/// when present.
struct AnnotatedProgram<'a> {
    program: &'a Program<FieldElement>,
    annotations: Option<Vec<BTreeMap<usize, String>>>,
}

impl std::fmt::Display for AnnotatedProgram<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_program(self.program, None, self.annotations.as_deref(), f)
    }
}
