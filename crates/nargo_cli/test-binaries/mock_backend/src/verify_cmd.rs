use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyCommand {
    #[clap(short = 'c')]
    pub(crate) crs_path: Option<PathBuf>,

    #[clap(short = 'p')]
    pub(crate) proof_path: PathBuf,

    #[clap(short = 'k')]
    pub(crate) vk_path: PathBuf,

    #[clap(short = 'r')]
    pub(crate) is_recursive: bool,
}

pub(crate) fn run(args: VerifyCommand) {
    assert!(args.vk_path.is_file(), "Could not find verification key file at provided path");
    assert!(args.proof_path.is_file(), "Could not find proof file at provided path");

    std::fs::write(args.proof_path, "proof").unwrap();
}
