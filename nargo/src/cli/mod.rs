pub use build_cmd::build_from_path;
use clap::{App, Arg};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

mod build_cmd;
mod contract_cmd;
mod new_cmd;
mod prove_cmd;
mod verify_cmd;

const CONTRACT_DIR: &str = "contract";
const PROOFS_DIR: &str = "proofs";
const SRC_DIR: &str = "src";
const PKG_FILE: &str = "Nargo.toml";

pub fn start_cli() {
    let matches = App::new("nargo")
        .about("Noir's package manager")
        .version("0.1")
        .author("Kevaundray Wedderburn <kevtheappdev@gmail.com>")
        .subcommand(App::new("build").about("Builds the constraint system"))
        .subcommand(App::new("contract").about("Creates the smart contract code for circuit"))
        .subcommand(
            App::new("new")
                .about("Create a new binary project")
                .arg(
                    Arg::with_name("package_name")
                        .help("Name of the package")
                        .required(true),
                )
                .arg(
                    Arg::with_name("path")
                        .help("The path to save the new project")
                        .required(false),
                ),
        )
        .subcommand(
            App::new("verify")
                .about("Given a proof and a program, verify whether the proof is valid")
                .arg(
                    Arg::with_name("proof")
                        .help("The proof to verify")
                        .required(true),
                ),
        )
        .subcommand(
            App::new("prove")
                .about("Create proof for this program")
                .arg(
                    Arg::with_name("proof_name")
                        .help("The name of the proof")
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("new") => new_cmd::run(matches),
        Some("build") => build_cmd::run(matches),
        Some("contract") => contract_cmd::run(matches),
        Some("prove") => prove_cmd::run(matches),
        Some("verify") => verify_cmd::run(matches),
        None => println!("No subcommand was used"),
        Some(x) => println!("unknown command : {}", x),
    }
}

fn create_dir<P: AsRef<Path>>(dir_path: P) -> Result<PathBuf, std::io::Error> {
    let mut dir = std::path::PathBuf::new();
    dir.push(dir_path);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_to_file(bytes: &[u8], path: &Path) -> String {
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(bytes) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => display.to_string(),
    }
}
