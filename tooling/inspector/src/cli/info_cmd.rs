use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre;
use noir_artifact_cli::Artifact;
use noirc_artifacts_info::{InfoReport, count_opcodes_and_gates_in_program, show_info_report};

#[derive(Debug, Clone, Args)]
pub(crate) struct InfoCommand {
    /// The artifact to inspect
    artifact: PathBuf,

    /// Output a JSON formatted report. Changes to this format are not currently considered breaking.
    #[clap(long, hide = true)]
    json: bool,

    /// Name of the function to print, if the artifact is a contract.
    #[clap(long)]
    contract_fn: Option<String>,
}

pub(crate) fn run(args: InfoCommand) -> eyre::Result<()> {
    let artifact = Artifact::read_from_file(&args.artifact)?;

    let programs = match artifact {
        Artifact::Program(program) => {
            let package_name = args
                .artifact
                .with_extension("")
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "artifact".to_string());

            vec![count_opcodes_and_gates_in_program(program, package_name, None)]
        }
        Artifact::Contract(contract) => contract
            .functions
            .into_iter()
            .filter(|f| args.contract_fn.as_ref().map(|n| *n == f.name).unwrap_or(true))
            .map(|f| {
                let package_name = format!("{}::{}", contract.name, f.name);
                let program = f.into_compiled_program(
                    contract.noir_version.clone(),
                    contract.file_map.clone(),
                );
                count_opcodes_and_gates_in_program(program.into(), package_name, None)
            })
            .collect::<Vec<_>>(),
    };

    let info_report = InfoReport { programs };
    show_info_report(info_report, args.json);

    Ok(())
}
