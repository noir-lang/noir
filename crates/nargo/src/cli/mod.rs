use acvm::{acir::circuit::Circuit, hash_constraint_system, ProofSystemCompiler};
pub use check_cmd::check_from_path;
use clap::{Args, Parser, Subcommand};
use const_format::formatcp;
use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi, InputMap, MAIN_RETURN_NAME,
};
use noirc_driver::Driver;
use noirc_frontend::graph::{CrateName, CrateType};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
extern crate tempdir;
use tempdir::TempDir;

use crate::{
    constants::{ACIR_EXT, PK_EXT, VK_EXT},
    errors::CliError,
};

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

fn create_dir<P: AsRef<Path>>(dir_path: P) -> Result<PathBuf, std::io::Error> {
    let mut dir = std::path::PathBuf::new();
    dir.push(dir_path);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn create_named_dir(named_dir: &Path, name: &str) -> PathBuf {
    create_dir(named_dir).unwrap_or_else(|_| panic!("could not create the `{name}` directory"))
}

fn write_to_file(bytes: &[u8], path: &Path) -> String {
    let display = path.display();

    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {display}: {why}"),
        Ok(file) => file,
    };

    match file.write_all(bytes) {
        Err(why) => panic!("couldn't write to {display}: {why}"),
        Ok(_) => display.to_string(),
    }
}

pub fn read_inputs_from_file<P: AsRef<Path>>(
    path: P,
    file_name: &str,
    format: Format,
    abi: &Abi,
) -> Result<(InputMap, Option<InputValue>), CliError> {
    if abi.is_empty() {
        return Ok((BTreeMap::new(), None));
    }

    let file_path = {
        let mut dir_path = path.as_ref().to_path_buf();
        dir_path.push(file_name);
        dir_path.set_extension(format.ext());
        dir_path
    };
    if !file_path.exists() {
        return Err(CliError::MissingTomlFile(file_name.to_owned(), file_path));
    }

    let input_string = std::fs::read_to_string(file_path).unwrap();
    let mut input_map = format.parse(&input_string, abi)?;
    let return_value = input_map.remove(MAIN_RETURN_NAME);

    Ok((input_map, return_value))
}

pub fn write_inputs_to_file<P: AsRef<Path>>(
    input_map: &InputMap,
    return_value: &Option<InputValue>,
    path: P,
    file_name: &str,
    format: Format,
) -> Result<(), CliError> {
    let file_path = {
        let mut dir_path = path.as_ref().to_path_buf();
        dir_path.push(file_name);
        dir_path.set_extension(format.ext());
        dir_path
    };

    // If it exists, insert return value into input map so it's written to file.
    let serialized_output = if let Some(return_value) = return_value {
        let mut input_map = input_map.clone();
        input_map.insert(MAIN_RETURN_NAME.to_owned(), return_value.clone());
        format.serialize(&input_map)?
    } else {
        format.serialize(input_map)?
    };

    write_to_file(serialized_output.as_bytes(), &file_path);

    Ok(())
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

pub fn load_hex_data<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, CliError> {
    let hex_data: Vec<_> =
        std::fs::read(&path).map_err(|_| CliError::PathNotValid(path.as_ref().to_path_buf()))?;

    let raw_bytes = hex::decode(hex_data).map_err(CliError::HexArtifactNotValid)?;

    Ok(raw_bytes)
}

fn fetch_pk_and_vk<P: AsRef<Path>>(
    circuit: &Circuit,
    circuit_build_path: Option<P>,
    prove_circuit: bool,
    check_proof: bool,
) -> Result<(Vec<u8>, Vec<u8>), CliError> {
    let backend = crate::backends::ConcreteBackend;
    if let Some(circuit_build_path) = circuit_build_path {
        let mut acir_hash_path = PathBuf::new();
        acir_hash_path.push(circuit_build_path.as_ref());
        acir_hash_path.set_extension(ACIR_EXT.to_owned() + ".sha256");
        let expected_acir_hash = load_hex_data(acir_hash_path.clone())?;

        let new_acir_hash = hash_constraint_system(circuit);

        if new_acir_hash[..] != expected_acir_hash {
            return Err(CliError::MismatchedAcir(acir_hash_path));
        }

        // This flag exists to avoid an unnecessary read of the proving key during verification
        // as this method is used by both `nargo prove` and `nargo verify`
        let proving_key = if prove_circuit {
            let mut proving_key_path = PathBuf::new();
            proving_key_path.push(circuit_build_path.as_ref());
            proving_key_path.set_extension(PK_EXT);
            load_hex_data(proving_key_path)?
        } else {
            // We can return an empty Vec here as `prove_circuit` should only be false when running `nargo verify`
            vec![]
        };

        let verification_key = if check_proof {
            let mut verification_key_path = PathBuf::new();
            verification_key_path.push(circuit_build_path);
            verification_key_path.set_extension(VK_EXT);
            load_hex_data(verification_key_path)?
        } else {
            // We can return an empty Vec here as the verification key is used only is `check_proof` is true
            vec![]
        };

        Ok((proving_key, verification_key))
    } else {
        // If a path to the circuit's build dir has not been provided, run preprocess and generate the proving and verification keys
        let (proving_key, verification_key) = backend.preprocess(circuit.clone());
        Ok((proving_key, verification_key))
    }
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
    pub fn file_compiles<P: AsRef<Path>>(root_file: P) -> bool {
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
