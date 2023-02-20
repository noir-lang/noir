use acvm::ProofSystemCompiler;
use acvm::{acir::circuit::Circuit, hash_constraint_system};
use noirc_abi::input_parser::Format;
use std::path::{Path, PathBuf};

use clap::Args;

use crate::{
    cli::{execute_cmd::save_witness_to_dir, read_inputs_from_file},
    constants::{ACIR_EXT, PK_EXT, PROVER_INPUT_FILE, TARGET_DIR, VK_EXT},
    errors::CliError,
    resolver::Resolver,
};

use super::{add_std_lib, create_named_dir, write_to_file, NargoConfig};

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct CompileCommand {
    /// The name of the ACIR file
    circuit_name: String,

    /// Solve the witness and write it to file along with the ACIR
    #[arg(short, long)]
    witness: bool,

    /// Issue a warning for each unused variable instead of an error
    #[arg(short, long)]
    allow_warnings: bool,
}

pub(crate) fn run(args: CompileCommand, config: NargoConfig) -> Result<(), CliError> {
    let mut circuit_path = config.program_dir.clone();
    circuit_path.push(TARGET_DIR);

    generate_circuit_and_witness_to_disk(
        &args.circuit_name,
        config.program_dir,
        circuit_path,
        args.witness,
        args.allow_warnings,
    )
    .map(|_| ())
}

pub fn generate_circuit_and_witness_to_disk<P: AsRef<Path>>(
    circuit_name: &str,
    program_dir: P,
    circuit_dir: P,
    generate_witness: bool,
    allow_warnings: bool,
) -> Result<PathBuf, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), false, allow_warnings)?;

    preprocess_with_path(circuit_name, circuit_dir.as_ref(), compiled_program.circuit.clone())?;

    let mut circuit_path =
        save_acir_to_dir(compiled_program.circuit.clone(), circuit_name, circuit_dir.as_ref());
    println!("Generated ACIR code into {}", circuit_path.display());

    if generate_witness {
        // Parse the initial witness values from Prover.toml
        let inputs_map = read_inputs_from_file(
            program_dir,
            PROVER_INPUT_FILE,
            Format::Toml,
            &compiled_program.abi,
        )?;

        let solved_witness = super::execute_cmd::execute_program(&compiled_program, &inputs_map)?;

        circuit_path.pop();
        save_witness_to_dir(solved_witness, circuit_name, &circuit_path)?;
    }

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
    circuit: Circuit,
) -> Result<(PathBuf, PathBuf), CliError> {
    let backend = crate::backends::ConcreteBackend;

    let (proving_key, verification_key) = backend.preprocess(circuit);

    let pk_path = save_key_to_dir(proving_key, key_name, &preprocess_dir, true)?;
    println!("Proving key saved to {}", pk_path.display());
    let vk_path = save_key_to_dir(verification_key, key_name, preprocess_dir, false)?;
    println!("Verification key saved to {}", vk_path.display());

    Ok((pk_path, vk_path))
}

fn save_acir_to_dir<P: AsRef<Path>>(
    circuit: Circuit,
    circuit_name: &str,
    circuit_dir: P,
) -> PathBuf {
    let mut circuit_path = create_named_dir(circuit_dir.as_ref(), "target");
    circuit_path.push(circuit_name);

    // Save a checksum of the circuit to compare against during proving and verification
    let acir_hash = hash_constraint_system(&circuit);
    circuit_path.set_extension(ACIR_EXT.to_owned() + ".sha256");
    write_to_file(hex::encode(acir_hash).as_bytes(), &circuit_path);

    let mut serialized = Vec::new();
    circuit.write(&mut serialized).expect("could not serialize circuit");

    circuit_path.set_extension(ACIR_EXT);
    write_to_file(serialized.as_slice(), &circuit_path);

    circuit_path
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
