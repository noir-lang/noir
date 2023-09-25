use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Clone, Args)]
pub(crate) struct WriteVkCommand {
    #[clap(short = 'c')]
    pub(crate) crs_path: Option<PathBuf>,

    #[clap(short = 'b')]
    pub(crate) bytecode_path: PathBuf,

    #[clap(short = 'r')]
    pub(crate) is_recursive: bool,

    #[clap(short = 'o')]
    pub(crate) vk_path: PathBuf,
}

pub(crate) fn run(args: WriteVkCommand) {
    assert!(args.bytecode_path.is_file(), "Could not find bytecode file at provided path");

    std::fs::write(args.vk_path, "vk").unwrap();
}
