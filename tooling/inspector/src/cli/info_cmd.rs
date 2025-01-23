use std::path::PathBuf;

use acir::circuit::ExpressionWidth;
use clap::Args;
use color_eyre::eyre;
use noirc_artifacts::program::ProgramArtifact;
use noirc_artifacts_info::{count_opcodes_and_gates_in_program, show_info_report, InfoReport};

#[derive(Debug, Clone, Args)]
pub(crate) struct InfoCommand {
    /// The artifact to inspect
    artifact: PathBuf,

    /// Output a JSON formatted report. Changes to this format are not currently considered breaking.
    #[clap(long, hide = true)]
    json: bool,
}

pub(crate) fn run(args: InfoCommand) -> eyre::Result<()> {
    let file = std::fs::File::open(args.artifact.clone())?;
    let artifact: ProgramArtifact = serde_json::from_reader(file)?;

    let package_name = args
        .artifact
        .with_extension("")
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "artifact".to_string());

    let expression_width = ExpressionWidth::Unbounded;
    let program_info =
        count_opcodes_and_gates_in_program(artifact, package_name.to_string(), expression_width);

    let info_report = InfoReport { programs: vec![program_info] };
    show_info_report(info_report, args.json);

    Ok(())
}
