use std::path::PathBuf;

use acvm::acir::circuit::ExpressionWidth;
use clap::Args;
use noirc_artifacts::program::ProgramArtifact;
use noirc_driver::{parse_expression_width, DEFAULT_EXPRESSION_WIDTH};

use crate::{
    errors::CliError,
    program_info::{count_opcodes_and_gates_in_program, show_info_report, InfoReport},
};

/// Provides detailed information on a build artifact.
///
/// Current information provided per circuit:
/// 1. The number of ACIR opcodes
/// 2. Counts the final number gates in the circuit used by a backend
#[derive(Debug, Clone, Args)]
pub(crate) struct InspectCommand {
    /// The artifact to inspect
    artifact: PathBuf,

    /// Output a JSON formatted report. Changes to this format are not currently considered breaking.
    #[clap(long, hide = true)]
    json: bool,

    /// Specify the backend expression width that should be assumed
    #[arg(long, value_parser = parse_expression_width)]
    expression_width: Option<ExpressionWidth>,

    /// Display the ACIR for compiled circuit
    #[arg(long)]
    print_acir: bool,
}

pub(crate) fn run(args: InspectCommand) -> Result<(), CliError> {
    let file = match std::fs::File::open(args.artifact.clone()) {
        Ok(file) => file,
        Err(err) => {
            return Err(CliError::Generic(format!(
                "Cannot open file `{}`: {}",
                args.artifact.to_string_lossy(),
                err,
            )));
        }
    };
    let artifact: ProgramArtifact = match serde_json::from_reader(file) {
        Ok(artifact) => artifact,
        Err(error) => {
            return Err(CliError::Generic(format!("Cannot deserialize artifact: {}", error)));
        }
    };

    if args.print_acir {
        println!("Compiled ACIR for main:");
        println!("{}", artifact.bytecode);
    }

    let package_name = args
        .artifact
        .with_extension("")
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "artifact".to_string());

    let expression_width = args.expression_width.unwrap_or(DEFAULT_EXPRESSION_WIDTH);
    let program_info =
        count_opcodes_and_gates_in_program(artifact, package_name.to_string(), expression_width);

    let info_report = InfoReport { programs: vec![program_info] };
    show_info_report(info_report, args.json);

    Ok(())
}
