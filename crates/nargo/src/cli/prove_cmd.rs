use std::path::{Path, PathBuf};

use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::input_parser::Format;

use super::execute_cmd::{execute_program, extract_public_inputs};
use super::{create_named_dir, write_inputs_to_file, write_to_file};
use crate::cli::{dedup_public_input_indices, load_hex_data};
use crate::constants::{ACIR_EXT, PK_EXT};
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("prove").unwrap();
    let proof_name = args.value_of("proof_name");
    let circuit_name = args.value_of("circuit_name").unwrap();
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");

    let program_dir =
        args.value_of("path").map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

    let mut proof_dir = program_dir.clone();
    proof_dir.push(PROOFS_DIR);

    let mut circuit_build_path = program_dir.clone();
    circuit_build_path.push(TARGET_DIR);
    circuit_build_path.push(circuit_name);

    prove_with_path(
        proof_name,
        program_dir,
        proof_dir,
        circuit_build_path,
        show_ssa,
        allow_warnings,
    )?;

    Ok(())
}

#[allow(deprecated)]
pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: Option<&str>,
    program_dir: P,
    proof_dir: P,
    circuit_build_path: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<Option<PathBuf>, CliError> {
    let mut acir_path = PathBuf::new();
    acir_path.push(circuit_build_path.as_ref());
    acir_path.set_extension(ACIR_EXT);
    let existing_acir =
        std::fs::read(&acir_path).map_err(|_| CliError::MissingAcir(acir_path.clone()))?;

    let mut compiled_program =
        super::compile_cmd::compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;

    let serialized = compiled_program.circuit.to_bytes();
    if serialized != existing_acir {
        return Err(CliError::MismatchedAcir(acir_path));
    }

    let (_, solved_witness) = execute_program(&program_dir, &compiled_program)?;

    // Write public inputs into Verifier.toml
    let public_inputs = extract_public_inputs(&compiled_program, &solved_witness)?;
    write_inputs_to_file(&public_inputs, &program_dir, VERIFIER_INPUT_FILE, Format::Toml)?;

    // Since the public outputs are added onto the public inputs list, there can be duplicates.
    // We keep the duplicates for when one is encoding the return values into the Verifier.toml,
    // however we must remove these duplicates when creating a proof.
    compiled_program.circuit.public_inputs =
        dedup_public_input_indices(compiled_program.circuit.public_inputs);

    let mut proving_key_path = PathBuf::new();
    proving_key_path.push(circuit_build_path);
    proving_key_path.set_extension(PK_EXT);
    let proving_key = load_hex_data(proving_key_path)?;

    let backend = crate::backends::ConcreteBackend;
    let proof = backend.prove_with_pk(compiled_program.circuit, solved_witness, proving_key);

    println!("Proof successfully created");
    if let Some(proof_name) = proof_name {
        let proof_path = save_proof_to_dir(proof, proof_name, proof_dir)?;

        println!("Proof saved to {}", proof_path.display());
        Ok(Some(proof_path))
    } else {
        println!("{}", hex::encode(&proof));
        Ok(None)
    }
}

fn save_proof_to_dir<P: AsRef<Path>>(
    proof: Vec<u8>,
    proof_name: &str,
    proof_dir: P,
) -> Result<PathBuf, CliError> {
    let mut proof_path = create_named_dir(proof_dir.as_ref(), "proof");
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    write_to_file(hex::encode(proof).as_bytes(), &proof_path);

    Ok(proof_path)
}
