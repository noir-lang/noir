use clap::{Args, Parser, Subcommand};
use const_format::formatcp;
use tracing::debug;
use std::path::PathBuf;

use color_eyre::eyre;

use crate::{find_package_root, cli::fs::global_config};

use self::{backend_vendor_cmd::{BackendOptions}, install_cmd::InstallBackendCommand, prove_cmd::ProveCommand, verify_cmd::VerifyCommand, gates_cmd::GatesCommand, codegen_verifier_cmd::CodegenVerifierCommand};

mod fs;

mod arguments;
mod backend_vendor_cmd;
mod check_cmd;
mod codegen_verifier_cmd;
mod compile_cmd;
mod execute_cmd;
mod gates_cmd;
mod lsp_cmd;
mod new_cmd;
mod prove_cmd;
mod test_cmd;
mod verify_cmd;
mod install_cmd;

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
    config: arguments::NargoConfig,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum NargoCommand {
    Check(check_cmd::CheckCommand),
    CodegenVerifier(CodegenVerifierCommand),
    #[command(alias = "build")]
    Compile(compile_cmd::CompileCommand),
    New(new_cmd::NewCommand),
    Execute(execute_cmd::ExecuteCommand),
    /// Create proof for this program
    Prove(ProveCommand),
    /// Given a proof and a program, verify whether the proof is valid
    Verify(VerifyCommand),
    Test(test_cmd::TestCommand),
    /// Counts the occurrences of different gates in circuit
    Gates(GatesCommand),
    Lsp(lsp_cmd::LspCommand),
    /// Execute arbitrary backend subcommand, pass args behind `--`
    Backend(BackendOptions),
    InstallBackend(InstallBackendCommand),
}

pub fn start_cli() -> eyre::Result<()> {

    let NargoCli { command, mut config } = NargoCli::parse();

    let global_config = global_config::read_global_config_file();

    debug!("Global config: {:?}", global_config);

    // Search through parent directories to find package root if necessary.
    if !matches!(command, NargoCommand::New(_) | NargoCommand::InstallBackend(_)) {
        config.nargo_package_root = find_package_root(&config.nargo_package_root)?;
        debug!("Project root is {:?}", config.nargo_package_root);
    }

    backend_vendor_cmd::set_default_paths(&mut config);

    debug!("Nargo configuration: {:?}", config);

    let backend = crate::backends::ConcreteBackend::default();

    match command {
        NargoCommand::New(args) => new_cmd::run(&backend, args, config),
        NargoCommand::Check(args) => check_cmd::run(&backend, args, config),
        NargoCommand::Compile(args) => compile_cmd::run(&backend, args, config),
        NargoCommand::Execute(args) => execute_cmd::run(&backend, args, config),
        NargoCommand::Prove(args) => prove_cmd::run(&backend, args, config),
        NargoCommand::Verify(args) => verify_cmd::run(&backend, args, config),
        NargoCommand::Test(args) => test_cmd::run(&backend, args, config),
        NargoCommand::Gates(args) => gates_cmd::run(&backend, args, &config),
        NargoCommand::CodegenVerifier(args) => codegen_verifier_cmd::run(&backend, args, config),
        NargoCommand::Backend(args) => backend_vendor_cmd::run(&backend, args, config),
        NargoCommand::Lsp(args) => lsp_cmd::run(&backend, args, config),
        NargoCommand::InstallBackend(args) => install_cmd::run(&backend, args, config),
    }?;

    Ok(())
}

// FIXME: I not sure that this is the right place for this tests.
#[cfg(test)]
mod tests {
    use fm::FileManager;
    use noirc_driver::{check_crate, create_local_crate};
    use noirc_errors::reporter;
    use noirc_frontend::{
        graph::{CrateGraph, CrateType},
        hir::Context,
    };

    use std::path::{Path, PathBuf};

    const TEST_DATA_DIR: &str = "tests/compile_tests_data";

    /// Compiles a file and returns true if compilation was successful
    ///
    /// This is used for tests.
    fn file_compiles(root_dir: &Path, root_file: &Path) -> bool {
        let fm = FileManager::new(root_dir);
        let graph = CrateGraph::default();
        let mut context = Context::new(fm, graph);
        let crate_id = create_local_crate(&mut context, root_file, CrateType::Binary);

        let result = check_crate(&mut context, crate_id, false, false);
        let success = result.is_ok();

        let errors = match result {
            Ok(warnings) => warnings,
            Err(errors) => errors,
        };

        reporter::report_all(&context.file_manager, &errors, false);
        success
    }

    #[test]
    fn compilation_pass() {
        let pass_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/pass"));

        let paths = std::fs::read_dir(&pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(file_compiles(&pass_dir, &path), "path: {}", path.display());
        }
    }

    #[test]
    fn compilation_fail() {
        let fail_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{TEST_DATA_DIR}/fail"));

        let paths = std::fs::read_dir(&fail_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(!file_compiles(&fail_dir, &path), "path: {}", path.display());
        }
    }
}
