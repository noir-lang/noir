use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {
    #[clap(short = 'c')]
    pub(crate) crs_path: Option<PathBuf>,

    #[clap(short = 'b')]
    pub(crate) bytecode_path: PathBuf,

    #[clap(short = 'w')]
    pub(crate) witness_path: PathBuf,

    #[clap(short = 'o')]
    pub(crate) proof_path: PathBuf,
}

pub(crate) fn run(args: ProveCommand) {
    assert!(args.bytecode_path.is_file(), "Could not find bytecode file at provided path");
    assert!(args.witness_path.is_file(), "Could not find witness file at provided path");

    std::fs::write(args.proof_path, "proof").unwrap();
}
