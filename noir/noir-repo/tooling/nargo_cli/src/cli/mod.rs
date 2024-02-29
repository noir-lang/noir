use clap::{Args, Parser, Subcommand};
use const_format::formatcp;
use nargo_toml::find_package_root;
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use std::path::PathBuf;

use color_eyre::eyre;

use crate::backends::get_active_backend;

mod fs;

mod backend_cmd;
mod check_cmd;
mod codegen_verifier_cmd;
mod compile_cmd;
mod dap_cmd;
mod debug_cmd;
mod execute_cmd;
mod export_cmd;
mod fmt_cmd;
mod info_cmd;
mod init_cmd;
mod lsp_cmd;
mod new_cmd;
mod prove_cmd;
mod test_cmd;
mod verify_cmd;

const GIT_HASH: &str = env!("GIT_COMMIT");
const IS_DIRTY: &str = env!("GIT_DIRTY");
const NARGO_VERSION: &str = env!("CARGO_PKG_VERSION");

static VERSION_STRING: &str = formatcp!(
    "version = {}\nnoirc version = {}\n(git version hash: {}, is dirty: {})",
    NARGO_VERSION,
    NOIR_ARTIFACT_VERSION_STRING,
    GIT_HASH,
    IS_DIRTY
);

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
    Backend(backend_cmd::BackendCommand),
    Check(check_cmd::CheckCommand),
    Fmt(fmt_cmd::FormatCommand),
    CodegenVerifier(codegen_verifier_cmd::CodegenVerifierCommand),
    #[command(alias = "build")]
    Compile(compile_cmd::CompileCommand),
    New(new_cmd::NewCommand),
    Init(init_cmd::InitCommand),
    Execute(execute_cmd::ExecuteCommand),
    #[command(hide = true)] // Hidden while the feature is being built out
    Export(export_cmd::ExportCommand),
    #[command(hide = true)] // Hidden while the feature is being built out
    Debug(debug_cmd::DebugCommand),
    Prove(prove_cmd::ProveCommand),
    Verify(verify_cmd::VerifyCommand),
    Test(test_cmd::TestCommand),
    Info(info_cmd::InfoCommand),
    Lsp(lsp_cmd::LspCommand),
    #[command(hide = true)]
    Dap(dap_cmd::DapCommand),
}

#[cfg(not(feature = "codegen-docs"))]
pub(crate) fn start_cli() -> eyre::Result<()> {
    let NargoCli { command, mut config } = NargoCli::parse();

    // If the provided `program_dir` is relative, make it absolute by joining it to the current directory.
    if !config.program_dir.is_absolute() {
        config.program_dir = std::env::current_dir().unwrap().join(config.program_dir);
    }

    // Search through parent directories to find package root if necessary.
    if !matches!(
        command,
        NargoCommand::New(_)
            | NargoCommand::Init(_)
            | NargoCommand::Lsp(_)
            | NargoCommand::Backend(_)
            | NargoCommand::Dap(_)
    ) {
        config.program_dir = find_package_root(&config.program_dir)?;
    }

    let active_backend = get_active_backend();
    let backend = crate::backends::Backend::new(active_backend);

    match command {
        NargoCommand::New(args) => new_cmd::run(&backend, args, config),
        NargoCommand::Init(args) => init_cmd::run(args, config),
        NargoCommand::Check(args) => check_cmd::run(&backend, args, config),
        NargoCommand::Compile(args) => compile_cmd::run(&backend, args, config),
        NargoCommand::Debug(args) => debug_cmd::run(&backend, args, config),
        NargoCommand::Execute(args) => execute_cmd::run(&backend, args, config),
        NargoCommand::Export(args) => export_cmd::run(&backend, args, config),
        NargoCommand::Prove(args) => prove_cmd::run(&backend, args, config),
        NargoCommand::Verify(args) => verify_cmd::run(&backend, args, config),
        NargoCommand::Test(args) => test_cmd::run(&backend, args, config),
        NargoCommand::Info(args) => info_cmd::run(&backend, args, config),
        NargoCommand::CodegenVerifier(args) => codegen_verifier_cmd::run(&backend, args, config),
        NargoCommand::Backend(args) => backend_cmd::run(args),
        NargoCommand::Lsp(args) => lsp_cmd::run(&backend, args, config),
        NargoCommand::Dap(args) => dap_cmd::run(&backend, args, config),
        NargoCommand::Fmt(args) => fmt_cmd::run(args, config),
    }?;

    Ok(())
}

#[cfg(feature = "codegen-docs")]
pub(crate) fn start_cli() -> eyre::Result<()> {
    let markdown: String = clap_markdown::help_markdown::<NargoCli>();
    println!("{markdown}");
    Ok(())
}
