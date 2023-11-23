use clap::Args;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Args)]
pub(crate) struct ContractCommand {
    #[clap(short = 'c')]
    pub(crate) crs_path: Option<PathBuf>,

    #[clap(short = 'k')]
    pub(crate) vk_path: PathBuf,

    #[clap(short = 'o')]
    pub(crate) contract_path: PathBuf,
}

pub(crate) fn run(args: ContractCommand) {
    assert!(args.vk_path.is_file(), "Could not find vk file at provided path");

    std::io::stdout().write_all(b"contract VerifierContract {}").unwrap();
}
