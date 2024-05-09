use clap::Args;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Args)]
pub(crate) struct GatesCommand {
    #[clap(short = 'c')]
    pub(crate) crs_path: Option<PathBuf>,

    #[clap(short = 'b')]
    pub(crate) bytecode_path: PathBuf,
}

pub(crate) fn run(args: GatesCommand) {
    assert!(args.bytecode_path.is_file(), "Could not find bytecode file at provided path");

    let response: &str = r#"{ "functions": [{"acir_opcodes": 123, "circuit_size": 125 }] }"#;

    std::io::stdout().write_all(response.as_bytes()).unwrap();
}
