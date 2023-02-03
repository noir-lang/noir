use acvm::{acir::circuit::PublicInputs, FieldElement};
pub use check_cmd::check_from_path;
use clap::{App, AppSettings, Arg};
use const_format::formatcp;
use git_version::git_version;
use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi,
};
use noirc_driver::Driver;
use noirc_frontend::graph::{CrateName, CrateType};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
extern crate tempdir;
use tempdir::TempDir;

use crate::errors::CliError;

mod check_cmd;
mod compile_cmd;
mod contract_cmd;
mod gates_cmd;
mod new_cmd;
mod prove_cmd;
mod verify_cmd;

const SHORT_GIT_HASH: &str = git_version!(prefix = "git:");
const VERSION_STRING: &str = formatcp!("{} ({})", env!("CARGO_PKG_VERSION"), SHORT_GIT_HASH);

pub fn start_cli() {
    let allow_warnings = Arg::with_name("allow-warnings")
        .long("allow-warnings")
        .help("Issue a warning for each unused variable instead of an error");

    let show_ssa = Arg::with_name("show-ssa")
        .long("show-ssa")
        .help("Emit debug information for the intermediate SSA IR");

    let matches = App::new("nargo")
        .about("Noir's package manager")
        .version(VERSION_STRING)
        .author("The Noir Team <kevtheappdev@gmail.com>")
        .subcommand(
            App::new("check")
                .about("Checks the constraint system for errors")
                .arg(allow_warnings.clone()),
        )
        .subcommand(
            App::new("contract")
                .about("Generates a Solidity verifier smart contract for the program"),
        )
        .subcommand(
            App::new("new")
                .about("Create a new binary project")
                .arg(Arg::with_name("package_name").help("Name of the package").required(true))
                .arg(
                    Arg::with_name("path").help("The path to save the new project").required(false),
                ),
        )
        .subcommand(
            App::new("verify")
                .about("Given a proof and a program, verify whether the proof is valid")
                .arg(Arg::with_name("proof").help("The proof to verify").required(true))
                .arg(allow_warnings.clone()),
        )
        .subcommand(
            App::new("prove")
                .about("Create proof for this program")
                .arg(Arg::with_name("proof_name").help("The name of the proof"))
                .arg(show_ssa.clone())
                .arg(allow_warnings.clone()),
        )
        .subcommand(
            App::new("compile")
                .about("Compile the program and its secret execution trace into ACIR format")
                .arg(
                    Arg::with_name("circuit_name").help("The name of the ACIR file").required(true),
                )
                .arg(
                    Arg::with_name("witness")
                        .long("witness")
                        .help("Solve the witness and write it to file along with the ACIR"),
                )
                .arg(allow_warnings.clone()),
        )
        .subcommand(
            App::new("gates")
                .about("Counts the occurrences of different gates in circuit")
                .arg(show_ssa)
                .arg(allow_warnings),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let result = match matches.subcommand_name() {
        Some("new") => new_cmd::run(matches),
        Some("check") => check_cmd::run(matches),
        Some("contract") => contract_cmd::run(matches),
        Some("prove") => prove_cmd::run(matches),
        Some("compile") => compile_cmd::run(matches),
        Some("verify") => verify_cmd::run(matches),
        Some("gates") => gates_cmd::run(matches),
        Some(x) => Err(CliError::Generic(format!("unknown command : {x}"))),
        _ => unreachable!(),
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
    abi: Abi,
) -> Result<BTreeMap<String, InputValue>, CliError> {
    let file_path = {
        let mut dir_path = path.as_ref().to_path_buf();
        dir_path.push(file_name);
        dir_path.set_extension(format.ext());
        dir_path
    };
    if !file_path.exists() {
        return Err(CliError::MissingTomlFile(file_path));
    }

    let input_string = std::fs::read_to_string(file_path).unwrap();
    Ok(format.parse(&input_string, abi)?)
}

fn write_inputs_to_file<P: AsRef<Path>>(
    w_map: &BTreeMap<String, InputValue>,
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

    let serialized_output = format.serialize(w_map)?;
    write_to_file(serialized_output.as_bytes(), &file_path);

    Ok(())
}

// helper function which tests noir programs by trying to generate a proof and verify it
pub fn prove_and_verify(proof_name: &str, prg_dir: &Path, show_ssa: bool) -> bool {
    let tmp_dir = TempDir::new("p_and_v_tests").unwrap();
    let proof_path = match prove_cmd::prove_with_path(
        Some(proof_name),
        prg_dir,
        &tmp_dir.into_path(),
        show_ssa,
        false,
    ) {
        Ok(p) => p,
        Err(error) => {
            println!("{error}");
            return false;
        }
    };

    verify_cmd::verify_with_path(prg_dir, &proof_path.unwrap(), show_ssa, false).unwrap()
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

// Removes duplicates from the list of public input witnesses
fn dedup_public_input_indices(indices: PublicInputs) -> PublicInputs {
    let duplicates_removed: HashSet<_> = indices.0.into_iter().collect();
    PublicInputs(duplicates_removed.into_iter().collect())
}

// Removes duplicates from the list of public input witnesses and the
// associated list of duplicate values.
pub(crate) fn dedup_public_input_indices_values(
    indices: PublicInputs,
    values: Vec<FieldElement>,
) -> (PublicInputs, Vec<FieldElement>) {
    // Assume that the public input index lists and the values contain duplicates
    assert_eq!(indices.0.len(), values.len());

    let mut public_inputs_without_duplicates = Vec::new();
    let mut already_seen_public_indices = HashMap::new();

    for (index, value) in indices.0.iter().zip(values) {
        match already_seen_public_indices.get(index) {
            Some(expected_value) => {
                // The index has already been added
                // so lets check that the values already inserted is equal to the value, we wish to insert
                assert_eq!(*expected_value, value, "witness index {index:?} does not have a canonical map. The expected value is {expected_value}, the received value is {value}.")
            }
            None => {
                already_seen_public_indices.insert(*index, value);
                public_inputs_without_duplicates.push(value)
            }
        }
    }

    (
        PublicInputs(already_seen_public_indices.keys().copied().collect()),
        public_inputs_without_duplicates,
    )
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
