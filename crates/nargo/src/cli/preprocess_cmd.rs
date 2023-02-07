use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use std::path::{Path, PathBuf};

use crate::constants::TARGET_DIR;
use crate::errors::CliError;
use crate::{
    cli::compile_cmd::compile_circuit,
    constants::{PK_EXT, VK_EXT},
};

use super::{create_named_dir, write_to_file};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("preprocess").unwrap();
    // let key_name = args.value_of("key_name").unwrap();

    let allow_warnings = args.is_present("allow-warnings");
    preprocess("", allow_warnings)
}

pub fn preprocess(key_name: &str, allow_warnings: bool) -> Result<(), CliError> {
    let curr_dir = std::env::current_dir().unwrap();

    let mut preprocess_dir = PathBuf::new();
    preprocess_dir.push(TARGET_DIR);

    preprocess_with_path(curr_dir, preprocess_dir, key_name, allow_warnings)?;

    Ok(())
}

pub fn preprocess_with_path<P: AsRef<Path>>(
    program_dir: P,
    preprocess_dir: P,
    _key_name: &str, 
    allow_warnings: bool,
) -> Result<(PathBuf, PathBuf), CliError> {
    let compiled_program = compile_circuit(program_dir, false, allow_warnings)?;

    let backend = crate::backends::ConcreteBackend;

    let (proving_key, verification_key) = backend.preprocess(compiled_program.circuit);

    println!("Proving and verification key successfully created");
    // TODO: change this to use `key_name`. We will either have to remove the ability to not supply a proof name to
    // `nargo prove` or add an additional field to `nargo prove` that requires you to specify the key name too
    let pk_path = save_key_to_dir(proving_key.clone(), "proving_key", &preprocess_dir, true)?;
    println!("Proving key saved to {}", pk_path.display());
    let vk_path =
        save_key_to_dir(verification_key.clone(), "verification_key", preprocess_dir, false)?;
    println!("Verification key saved to {}", vk_path.display());

    Ok((pk_path, vk_path))
}

fn save_key_to_dir<P: AsRef<Path>>(
    key: Vec<u8>,
    key_name: &str,
    key_dir: P,
    is_proving_key: bool,
) -> Result<PathBuf, CliError> {
    let mut key_path = create_named_dir(key_dir.as_ref(), key_name);
    key_path.push(key_name);
    let extension = if is_proving_key { PK_EXT } else { VK_EXT };
    key_path.set_extension(extension);

    write_to_file(hex::encode(key).as_bytes(), &key_path);

    Ok(key_path)
}
