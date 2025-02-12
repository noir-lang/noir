use clap::{Args, Parser, Subcommand};
use const_format::formatcp;
use nargo::workspace::Workspace;
use nargo_toml::{
    get_package_manifest, resolve_workspace_from_toml, ManifestError, PackageSelection,
};
use noirc_driver::{CrateName, NOIR_ARTIFACT_VERSION_STRING};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use color_eyre::eyre;

use crate::errors::CliError;

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
    #[arg(long, hide = true, global = true, default_value = "./", value_parser = parse_path)]
    program_dir: PathBuf,

    /// Override the default target directory.
    #[arg(long, hide = true, global = true, value_parser = parse_path)]
    target_dir: Option<PathBuf>,
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

/// Commands that can execute on the workspace level, or be limited to a selected package.
trait WorkspaceCommand {
    /// Indicate which package the command will be applied to.
    fn package_selection(&self) -> PackageSelection;
    /// The kind of lock the command needs to take out on the selected packages.
    fn lock_type(&self) -> LockType;
}

/// What kind of lock to take out on the (selected) workspace members.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)] // Not using `Shared` at the moment, e.g. while we `debug` we can `compile` a different version.
enum LockType {
    /// For commands that write artifacts.
    Exclusive,
    /// For commands that read artifacts, but never write them.
    Shared,
    /// For commands that cannot interfere with others.
    None,
}

#[cfg(not(feature = "codegen-docs"))]
#[tracing::instrument(level = "trace")]
pub(crate) fn start_cli() -> eyre::Result<()> {
    let NargoCli { command, config } = NargoCli::parse();

    match command {
        NargoCommand::New(args) => new_cmd::run(args, config),
        NargoCommand::Init(args) => init_cmd::run(args, config),
        NargoCommand::Check(args) => with_workspace(args, config, check_cmd::run),
        NargoCommand::Compile(args) => with_workspace(args, config, compile_cmd::run),
        NargoCommand::Debug(args) => with_workspace(args, config, debug_cmd::run),
        NargoCommand::Execute(args) => with_workspace(args, config, execute_cmd::run),
        NargoCommand::Export(args) => with_workspace(args, config, export_cmd::run),
        NargoCommand::Test(args) => with_workspace(args, config, test_cmd::run),
        NargoCommand::Info(args) => with_workspace(args, config, info_cmd::run),
        NargoCommand::Lsp(_) => lsp_cmd::run(),
        NargoCommand::Dap(args) => dap_cmd::run(args),
        NargoCommand::Fmt(args) => with_workspace(args, config, fmt_cmd::run),
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

/// Read a given program directory into a workspace.
fn read_workspace(
    program_dir: &Path,
    selection: PackageSelection,
) -> Result<Workspace, ManifestError> {
    let toml_path = get_package_manifest(program_dir)?;

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
    )?;

    Ok(workspace)
}

/// Find the root directory, parse the workspace, lock the packages, then execute the command.
fn with_workspace<C, R>(cmd: C, config: NargoConfig, run: R) -> Result<(), CliError>
where
    C: WorkspaceCommand,
    R: FnOnce(C, Workspace) -> Result<(), CliError>,
{
    // All commands need to run on the workspace level, because that's where the `target` directory is.
    let workspace_dir = nargo_toml::find_root(&config.program_dir, true)?;
    let package_dir = nargo_toml::find_root(&config.program_dir, false)?;
    // Check if we're running inside the directory of a package, without having selected the entire workspace
    // or a specific package; if that's the case then parse the package name to select it in the workspace.
    let selection = match cmd.package_selection() {
        PackageSelection::DefaultOrAll if workspace_dir != package_dir => {
            let package = read_workspace(&package_dir, PackageSelection::DefaultOrAll)?;
            let package = package.into_iter().next().expect("there should be exactly 1 package");
            PackageSelection::Selected(package.name.clone())
        }
        other => other,
    };
    // Parse the top level workspace with the member selected.
    let mut workspace = read_workspace(&workspace_dir, selection)?;
    // Optionally override the target directory. It's only done here because most commands like the LSP and DAP
    // don't read or write artifacts, so they don't use the target directory.
    workspace.target_dir = config.target_dir.clone();
    // Lock manifests if the command needs it.
    let _locks = match cmd.lock_type() {
        LockType::None => None,
        typ => Some(lock_workspace(&workspace, typ == LockType::Exclusive)?),
    };
    run(cmd, workspace)
}

/// Lock the (selected) packages in the workspace.
/// The lock taken can be shared for commands that only read the artifacts,
/// or exclusive for the ones that (might) write artifacts as well.
fn lock_workspace(workspace: &Workspace, exclusive: bool) -> Result<Vec<impl Drop>, CliError> {
    use fs2::FileExt as _;

    struct LockedFile(File);

    impl Drop for LockedFile {
        fn drop(&mut self) {
            let _ = self.0.unlock();
        }
    }

    let mut locks = Vec::new();
    for pkg in workspace.into_iter() {
        let toml_path = get_package_manifest(&pkg.root_dir)?;
        let path_display = toml_path.display();

        let file = File::open(&toml_path)
            .unwrap_or_else(|e| panic!("Expected {path_display} to exist: {e}"));

        if exclusive {
            if file.try_lock_exclusive().is_err() {
                eprintln!("Waiting for lock on {path_display}...");
            }
            file.lock_exclusive().unwrap_or_else(|e| panic!("Failed to lock {path_display}: {e}"));
        } else {
            if file.try_lock_shared().is_err() {
                eprintln!("Waiting for lock on {path_display}...",);
            }
            file.lock_shared().unwrap_or_else(|e| panic!("Failed to lock {path_display}: {e}"));
        }

        locks.push(LockedFile(file));
    }
    Ok(locks)
}

/// Parses a path and turns it into an absolute one by joining to the current directory.
fn parse_path(path: &str) -> Result<PathBuf, String> {
    use fm::NormalizePath;
    let mut path: PathBuf = path.parse().map_err(|e| format!("failed to parse path: {e}"))?;
    if !path.is_absolute() {
        path = std::env::current_dir().unwrap().join(path).normalize();
    }
    Ok(path)
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
