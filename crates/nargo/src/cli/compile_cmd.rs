use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::input_parser::Format;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use super::{add_std_lib, create_named_dir, read_inputs_from_file, write_to_file};
use crate::{
    cli::execute_cmd::save_witness_to_dir,
    constants::{ACIR_EXT, PK_EXT, PROVER_INPUT_FILE, TARGET_DIR, VK_EXT},
    errors::CliError,
    resolver::Resolver,
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("compile").unwrap();
    let circuit_name = args.value_of("circuit_name").unwrap();
    let witness = args.is_present("witness");
    let allow_warnings = args.is_present("allow-warnings");
    let program_dir =
        args.value_of("path").map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

    let mut circuit_path = program_dir.clone();
    circuit_path.push(TARGET_DIR);

    generate_circuit_and_witness_to_disk(
        circuit_name,
        program_dir,
        circuit_path,
        witness,
        allow_warnings,
    )
    .map(|_| ())
}

#[allow(deprecated)]
pub fn generate_circuit_and_witness_to_disk<P: AsRef<Path>>(
    circuit_name: &str,
    program_dir: P,
    circuit_dir: P,
    generate_witness: bool,
    allow_warnings: bool,
) -> Result<PathBuf, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), false, allow_warnings)?;

    preprocess_with_path(circuit_name, circuit_dir.as_ref(), compiled_program.circuit.clone())?;

    let mut circuit_path = create_named_dir(circuit_dir.as_ref(), "target");
    circuit_path.push(circuit_name);
    circuit_path.set_extension(ACIR_EXT);

    let serialized = compiled_program.circuit.to_bytes();
    let path = write_to_file(serialized.as_slice(), &circuit_path);
    println!("Generated ACIR code into {path}");

    let mut hasher = Sha256::new();
    hasher.update(serialized);
    let acir_hash = hasher.finalize();
    circuit_path.set_extension(ACIR_EXT.to_owned() + ".sha256");
    write_to_file(hex::encode(acir_hash).as_bytes(), &circuit_path);

    if generate_witness {
        // Parse the initial witness values from Prover.toml
        let inputs_map = read_inputs_from_file(
            program_dir,
            PROVER_INPUT_FILE,
            Format::Toml,
            compiled_program.abi.as_ref().unwrap().clone(),
        )?;

        let (_, solved_witness) =
            super::execute_cmd::execute_program(&compiled_program, &inputs_map)?;

        circuit_path.pop();
        dbg!(circuit_path.clone());
        save_witness_to_dir(solved_witness, circuit_name, &circuit_path)?;
    }

    // The circuit path is used when combining proving and verification for integration tests.
    // The prove command sets different file extensions on the returned circuit path to access the necessary build artifacts (ACIR checksum, proving and verification keys)
    // We reset the circuit path to end with just the circuit name to avoid any errors with setting extensions when reading the ACIR checksum and keys
    circuit_path.pop();
    circuit_path.push(circuit_name);
    Ok(circuit_path)
}

pub fn compile_circuit<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<noirc_driver::CompiledProgram, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(program_dir.as_ref(), backend.np_language())?;
    add_std_lib(&mut driver);

    driver
        .into_compiled_program(backend.np_language(), show_ssa, allow_warnings)
        .map_err(|_| std::process::exit(1))
}

pub fn preprocess_with_path<P: AsRef<Path>>(
    key_name: &str,
    preprocess_dir: P,
    circuit: acvm::acir::circuit::Circuit,
) -> Result<(PathBuf, PathBuf), CliError> {
    let backend = crate::backends::ConcreteBackend;

    let (proving_key, verification_key) = backend.preprocess(circuit);

    println!("Proving and verification key successfully created");
    let pk_path = save_key_to_dir(proving_key, key_name, &preprocess_dir, true)?;
    println!("Proving key saved to {}", pk_path.display());
    let vk_path = save_key_to_dir(verification_key, key_name, preprocess_dir, false)?;
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
