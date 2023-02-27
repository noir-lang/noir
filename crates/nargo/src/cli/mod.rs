use clap::{Args, Parser, Subcommand};
use const_format::formatcp;
use noirc_abi::InputMap;
use noirc_driver::Driver;
use noirc_frontend::graph::{CrateName, CrateType};
use std::path::{Path, PathBuf};
extern crate tempdir;
use tempdir::TempDir;

mod fs;

mod check_cmd;
mod compile_cmd;
mod contract_cmd;
mod execute_cmd;
mod gates_cmd;
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
#[command(author, version=VERSION_STRING, about, long_about = None)]
struct NargoCli {
    #[command(subcommand)]
    command: NargoCommand,

    #[clap(flatten)]
    config: NargoConfig,
}

#[non_exhaustive]
#[derive(Args, Clone, Debug)]
pub(crate) struct NargoConfig {
    #[arg(short, long, hide=true, default_value_os_t = std::env::current_dir().unwrap())]
    program_dir: PathBuf,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
enum NargoCommand {
    Check(check_cmd::CheckCommand),
    Contract(contract_cmd::ContractCommand),
    Compile(compile_cmd::CompileCommand),
    New(new_cmd::NewCommand),
    Execute(execute_cmd::ExecuteCommand),
    Prove(prove_cmd::ProveCommand),
    Verify(verify_cmd::VerifyCommand),
    Test(test_cmd::TestCommand),
    Gates(gates_cmd::GatesCommand),
}

pub fn start_cli() {
    let matches = NargoCli::parse();

    let result = match matches.command {
        NargoCommand::New(args) => new_cmd::run(args, matches.config),
        NargoCommand::Check(args) => check_cmd::run(args, matches.config),
        NargoCommand::Compile(args) => compile_cmd::run(args, matches.config),
        NargoCommand::Execute(args) => execute_cmd::run(args, matches.config),
        NargoCommand::Prove(args) => prove_cmd::run(args, matches.config),
        NargoCommand::Verify(args) => verify_cmd::run(args, matches.config),
        NargoCommand::Test(args) => test_cmd::run(args, matches.config),
        NargoCommand::Gates(args) => gates_cmd::run(args, matches.config),
        NargoCommand::Contract(args) => contract_cmd::run(args, matches.config),
    };
    if let Err(err) = result {
        err.write()
    }
}

// helper function which tests noir programs by trying to generate a proof and verify it
pub fn prove_and_verify(proof_name: &str, prg_dir: &Path, show_ssa: bool) -> bool {
    let tmp_dir = TempDir::new("p_and_v_tests").unwrap();
    match prove_cmd::prove_with_path(
        Some(proof_name.to_owned()),
        prg_dir,
        &tmp_dir.into_path(),
        None,
        true,
        show_ssa,
        false,
    ) {
        Ok(_) => true,
        Err(error) => {
            println!("{error}");
            false
        }
    }
}

fn add_std_lib(driver: &mut Driver) {
    let path_to_std_lib_file = path_to_stdlib().join("lib.nr");
    let std_crate = driver.create_non_local_crate(path_to_std_lib_file, CrateType::Library);
    let std_crate_name = "std";
    driver.propagate_dep(std_crate, &CrateName::new(std_crate_name).unwrap());
}

fn path_to_stdlib() -> PathBuf {
    dirs::config_dir().unwrap().join("noir-lang").join("std/src")
}

// FIXME: I not sure that this is the right place for this tests.
#[cfg(test)]
mod tests {
    use noirc_driver::Driver;
    use noirc_frontend::graph::CrateType;

    use std::path::{Path, PathBuf};

    const TEST_DATA_DIR: &str = "tests/compile_tests_data";

    /// Compiles a file and returns true if compilation was successful
    ///
    /// This is used for tests.
    fn file_compiles<P: AsRef<Path>>(root_file: P) -> bool {
        let mut driver = Driver::new(&acvm::Language::R1CS);
        driver.create_local_crate(&root_file, CrateType::Binary);
        super::add_std_lib(&mut driver);
        driver.file_compiles()
    }

    #[test]
    fn compilation_pass() {
        let mut pass_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        pass_dir.push(&format!("{TEST_DATA_DIR}/pass"));

        let paths = std::fs::read_dir(pass_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(file_compiles(&path), "path: {}", path.display());
        }
    }

    #[test]
    fn compilation_fail() {
        let mut fail_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fail_dir.push(&format!("{TEST_DATA_DIR}/fail"));

        let paths = std::fs::read_dir(fail_dir).unwrap();
        for path in paths.flatten() {
            let path = path.path();
            assert!(!file_compiles(&path), "path: {}", path.display());
        }
    }
}
