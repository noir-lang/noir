use clap::{Args, Parser, Subcommand};
use const_format::formatcp;
use nargo_toml::find_package_root;
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use std::path::PathBuf;

use color_eyre::eyre;

mod fs;

mod check_cmd;
mod compile_cmd;
mod dap_cmd;
mod debug_cmd;
mod execute_cmd;
mod export_cmd;
mod fmt_cmd;
mod generate_completion_script_cmd;
mod info_cmd;
mod init_cmd;
mod lsp_cmd;
mod new_cmd;
mod test_cmd;

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
    Check(check_cmd::CheckCommand),
    Fmt(fmt_cmd::FormatCommand),
    #[command(alias = "build")]
    Compile(compile_cmd::CompileCommand),
    New(new_cmd::NewCommand),
    Init(init_cmd::InitCommand),
    Execute(execute_cmd::ExecuteCommand),
    #[command(hide = true)] // Hidden while the feature is being built out
    Export(export_cmd::ExportCommand),
    Debug(debug_cmd::DebugCommand),
    Test(test_cmd::TestCommand),
    Info(info_cmd::InfoCommand),
    Lsp(lsp_cmd::LspCommand),
    #[command(hide = true)]
    Dap(dap_cmd::DapCommand),
    GenerateCompletionScript(generate_completion_script_cmd::GenerateCompletionScriptCommand),
}

#[cfg(not(feature = "codegen-docs"))]
pub(crate) fn start_cli() -> eyre::Result<()> {
    let NargoCli { command, mut config } = NargoCli::parse();

    // If the provided `program_dir` is relative, make it absolute by joining it to the current directory.
    if !config.program_dir.is_absolute() {
        config.program_dir = std::env::current_dir().unwrap().join(config.program_dir);
    }

    // Search through parent directories to find package root if necessary.
    match &command {
        NargoCommand::Check(..)
        | NargoCommand::Fmt(..)
        | NargoCommand::Compile(..)
        | NargoCommand::Execute(..)
        | NargoCommand::Export(..)
        | NargoCommand::Debug(..)
        | NargoCommand::Test(..)
        | NargoCommand::Info(..) => {
            config.program_dir = find_package_root(&config.program_dir)?;
        }
        NargoCommand::New(..)
        | NargoCommand::Init(..)
        | NargoCommand::Lsp(..)
        | NargoCommand::Dap(..)
        | NargoCommand::GenerateCompletionScript(..) => (),
    }

    match command {
        NargoCommand::New(args) => new_cmd::run(args, config),
        NargoCommand::Init(args) => init_cmd::run(args, config),
        NargoCommand::Check(args) => check_cmd::run(args, config),
        NargoCommand::Compile(args) => compile_cmd::run(args, config),
        NargoCommand::Debug(args) => debug_cmd::run(args, config),
        NargoCommand::Execute(args) => execute_cmd::run(args, config),
        NargoCommand::Export(args) => export_cmd::run(args, config),
        NargoCommand::Test(args) => test_cmd::run(args, config),
        NargoCommand::Info(args) => info_cmd::run(args, config),
        NargoCommand::Lsp(args) => lsp_cmd::run(args, config),
        NargoCommand::Dap(args) => dap_cmd::run(args, config),
        NargoCommand::Fmt(args) => fmt_cmd::run(args, config),
        NargoCommand::GenerateCompletionScript(args) => generate_completion_script_cmd::run(args),
    }?;

    Ok(())
}

#[cfg(feature = "codegen-docs")]
pub(crate) fn start_cli() -> eyre::Result<()> {
    let markdown: String = clap_markdown::help_markdown::<NargoCli>();
    println!("{markdown}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    #[test]
    fn test_parse_invalid_expression_width() {
        let cmd = "nargo --program-dir . compile --expression-width 1";
        let res = super::NargoCli::try_parse_from(cmd.split_ascii_whitespace());

        let err = res.expect_err("should fail because of invalid width");
        assert!(err.to_string().contains("expression-width"));
        assert!(err
            .to_string()
            .contains(acvm::compiler::MIN_EXPRESSION_WIDTH.to_string().as_str()));
    }
}
