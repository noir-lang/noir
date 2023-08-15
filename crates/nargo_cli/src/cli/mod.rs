use clap::{Args, Parser, Subcommand};
use const_format::formatcp;
use nargo_toml::find_package_root;
use std::path::PathBuf;

use color_eyre::eyre;

mod fs;

mod check_cmd;
mod codegen_verifier_cmd;
mod compile_cmd;
mod execute_cmd;
mod info_cmd;
mod init_cmd;
mod lsp_cmd;
mod new_cmd;
mod prove_cmd;
mod test_cmd;
mod verify_cmd;

const GIT_HASH: &str = env!("GIT_COMMIT");
const IS_DIRTY: &str = env!("GIT_DIRTY");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

static VERSION_STRING: &str =
    formatcp!("{} (git version hash: {}, is dirty: {})", CARGO_PKG_VERSION, GIT_HASH, IS_DIRTY);

#[derive(Parser, Debug)]
#[command(name="nargo", author, version=VERSION_STRING, about, long_about = None)]
struct NargoCli {
    #[command(subcommand)]
    command: NargoCommand,

    #[clap(flatten)]
    config: NargoConfig,
}

#[non_exhaustive]
#[derive(Args, Clone, Debug)]
pub(crate) struct NargoConfig {
    // REMINDER: Also change this flag in the LSP test lens if renamed
    #[arg(long, hide = true, global = true, default_value = "./")]
    program_dir: PathBuf,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum NargoCommand {
    Check(check_cmd::CheckCommand),
    CodegenVerifier(codegen_verifier_cmd::CodegenVerifierCommand),
    #[command(alias = "build")]
    Compile(compile_cmd::CompileCommand),
    New(new_cmd::NewCommand),
    Init(init_cmd::InitCommand),
    Execute(execute_cmd::ExecuteCommand),
    Prove(prove_cmd::ProveCommand),
    Verify(verify_cmd::VerifyCommand),
    Test(test_cmd::TestCommand),
    Info(info_cmd::InfoCommand),
    Lsp(lsp_cmd::LspCommand),
}

pub(crate) fn start_cli() -> eyre::Result<()> {
    let NargoCli { command, mut config } = NargoCli::parse();

    // If the provided `program_dir` is relative, make it absolute by joining it to the current directory.
    if !config.program_dir.is_absolute() {
        config.program_dir = std::env::current_dir().unwrap().join(config.program_dir)
    }

    // Search through parent directories to find package root if necessary.
    if !matches!(command, NargoCommand::New(_) | NargoCommand::Init(_) | NargoCommand::Lsp(_)) {
        config.program_dir = find_package_root(&config.program_dir)?;
    }

    let backend = crate::backends::ConcreteBackend::default();

    match command {
        NargoCommand::New(args) => new_cmd::run(&backend, args, config),
        NargoCommand::Init(args) => init_cmd::run(&backend, args, config),
        NargoCommand::Check(args) => check_cmd::run(&backend, args, config),
        NargoCommand::Compile(args) => compile_cmd::run(&backend, args, config),
        NargoCommand::Execute(args) => execute_cmd::run(&backend, args, config),
        NargoCommand::Prove(args) => prove_cmd::run(&backend, args, config),
        NargoCommand::Verify(args) => verify_cmd::run(&backend, args, config),
        NargoCommand::Test(args) => test_cmd::run(&backend, args, config),
        NargoCommand::Info(args) => info_cmd::run(&backend, args, config),
        NargoCommand::CodegenVerifier(args) => codegen_verifier_cmd::run(&backend, args, config),
        NargoCommand::Lsp(args) => lsp_cmd::run(&backend, args, config),
    }?;

    Ok(())
}
