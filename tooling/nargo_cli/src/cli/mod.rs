use clap::{Args, Parser, Subcommand};
use const_format::formatcp;
use nargo_toml::PackageSelection;
use noirc_driver::{CrateName, NOIR_ARTIFACT_VERSION_STRING};
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

/// Options for commands that work on either workspace or package scope.
#[derive(Args, Clone, Debug, Default)]
pub(crate) struct PackageOptions {
    /// The name of the package to run the command on.
    /// By default run on the first one found moving up along the ancestors of the current directory.
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Run on all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,
}

impl PackageOptions {
    /// Decide which package to run the command on:
    /// * `package` if non-empty
    /// * all packages if `workspace` is `true`
    /// * otherwise the default package
    pub(crate) fn package_selection(&self) -> PackageSelection {
        let default_selection =
            if self.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };

        self.package.clone().map_or(default_selection, PackageSelection::Selected)
    }

    /// Whether we need to look for the package manifest at the workspace level.
    /// If a package is specified, it might not be the current package.
    fn is_workspace_rooted(&self) -> bool {
        self.workspace || self.package.is_some()
    }
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
    if let Some(workspace) = manifest_scope(&command) {
        config.program_dir = nargo_toml::find_root(&config.program_dir, workspace)?;
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

/// Some commands have package options, which we use here to decide whether to
/// alter `--program-dir` to point at a manifest, depending on whether we want
/// to work on a specific package or the entire workspace.
///
/// Returns:
/// * `None` if the command does not need a manifest to be found
/// * `Some(true)` if the command runs on the workspace level
/// * `Some(false)` if the command runs on the current package
fn manifest_scope(cmd: &NargoCommand) -> Option<bool> {
    match &cmd {
        NargoCommand::Check(cmd) => Some(cmd.package_options.is_workspace_rooted()),
        NargoCommand::Compile(cmd) => Some(cmd.package_options.is_workspace_rooted()),
        NargoCommand::Execute(cmd) => Some(cmd.package_options.is_workspace_rooted()),
        NargoCommand::Export(cmd) => Some(cmd.package_options.is_workspace_rooted()),
        NargoCommand::Test(cmd) => Some(cmd.package_options.is_workspace_rooted()),
        NargoCommand::Info(cmd) => Some(cmd.package_options.is_workspace_rooted()),
        NargoCommand::Debug(cmd) => Some(cmd.package.is_some()),
        NargoCommand::Fmt(..) => Some(true),
        NargoCommand::New(..)
        | NargoCommand::Init(..)
        | NargoCommand::Lsp(..)
        | NargoCommand::Dap(..)
        | NargoCommand::GenerateCompletionScript(..) => None,
    }
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
