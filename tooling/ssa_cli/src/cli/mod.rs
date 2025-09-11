use std::io::Read;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, command};
use color_eyre::eyre::{self, Context};
use const_format::formatcp;
use noir_artifact_cli::commands::parse_and_normalize_path;

mod interpret_cmd;
mod transform_cmd;
mod validate_cmd;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
static VERSION_STRING: &str = formatcp!("version = {}\n", PKG_VERSION,);

#[derive(Parser, Debug)]
#[command(name="noir-ssa", author, version=VERSION_STRING, about, long_about = None)]
struct SsaCli {
    #[command(subcommand)]
    command: SsaCommand,

    #[command(flatten)]
    args: SsaArgs,
}

#[derive(Args, Clone, Debug)]
struct SsaArgs {
    /// Path to the source SSA.
    ///
    /// If empty, the SSA will be read from stdin.
    #[clap(long, short, global = true, value_parser = parse_and_normalize_path)]
    source_path: Option<PathBuf>,
}

#[derive(Subcommand, Clone, Debug)]
enum SsaCommand {
    Interpret(interpret_cmd::InterpretCommand),
    Transform(transform_cmd::TransformCommand),
    /// Parse and validate the source SSA, printing errors to stderr
    Validate,
}

pub(crate) fn start_cli() -> eyre::Result<()> {
    let SsaCli { command, args } = SsaCli::parse();

    let src = read_source(args.source_path)?;

    Ok(())
}

/// Read the SSA from a file or stdin.
fn read_source(path: Option<PathBuf>) -> eyre::Result<String> {
    if let Some(path) = path {
        std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("error reading source from {}", path.to_string_lossy()))
    } else {
        let mut src = String::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_string(&mut src)?;
        Ok(src)
    }
}
