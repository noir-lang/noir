use std::path::{Path, PathBuf};

use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::input_parser::Format;
use noirc_driver::CompiledProgram;
use sha2::{Digest, Sha256};

use super::{
    create_named_dir, dedup_public_input_indices, load_hex_data, read_inputs_from_file,
    write_inputs_to_file, write_to_file,
};
use crate::{
    cli::{
        execute_cmd::{execute_program, extract_public_inputs},
        verify_cmd::verify_proof,
    },
    constants::{
        ACIR_EXT, PK_EXT, PROOFS_DIR, PROOF_EXT, PROVER_INPUT_FILE, TARGET_DIR,
        VERIFIER_INPUT_FILE, VK_EXT,
    },
    errors::CliError,
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("prove").unwrap();
    let proof_name = args.value_of("proof_name");
    let circuit_name = args.value_of("circuit_name");
    let check_proof = args.is_present("verify");
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");

    let program_dir =
        args.value_of("path").map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

    let mut proof_dir = program_dir.clone();
    proof_dir.push(PROOFS_DIR);

    let circuit_build_path = if let Some(circuit_name) = circuit_name {
        let mut circuit_build_path = program_dir.clone();
        circuit_build_path.push(TARGET_DIR);
        circuit_build_path.push(circuit_name);
        Some(circuit_build_path)
    } else {
        None
    };

    prove_with_path(
        proof_name,
        program_dir,
        proof_dir,
        circuit_build_path,
        check_proof,
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
    circuit_build_path: Option<P>,
    check_proof: bool,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<Option<PathBuf>, CliError> {
    let (compiled_program, proving_key, verification_key) = fetch_build_artifacts(
        &program_dir,
        circuit_build_path.as_ref(),
        check_proof,
        show_ssa,
        allow_warnings,
    )?;

    // Parse the initial witness values from Prover.toml
    let inputs_map = read_inputs_from_file(
        &program_dir,
        PROVER_INPUT_FILE,
        Format::Toml,
        compiled_program.abi.as_ref().unwrap().clone(),
    )?;

    let (_, solved_witness) = execute_program(&compiled_program, &inputs_map)?;

    // Write public inputs into Verifier.toml
    let public_inputs = extract_public_inputs(&compiled_program, &solved_witness)?;
    write_inputs_to_file(&public_inputs, &program_dir, VERIFIER_INPUT_FILE, Format::Toml)?;

    // Since the public outputs are added onto the public inputs list, there can be duplicates.
    // We keep the duplicates for when one is encoding the return values into the Verifier.toml,
    // however we must remove these duplicates when creating a proof.
    let mut prover_circuit = compiled_program.circuit.clone();
    prover_circuit.public_inputs = dedup_public_input_indices(prover_circuit.public_inputs);

    let backend = crate::backends::ConcreteBackend;
    let proof = backend.prove_with_pk(prover_circuit, solved_witness, proving_key);

    println!("Proof successfully created");
    if check_proof {
        let valid_proof = verify_proof(compiled_program, public_inputs, &proof, verification_key)?;
        println!("Proof verified : {valid_proof}");
        if !valid_proof {
            return Err(CliError::Generic("Could not verify generated proof".to_owned()));
        }
    }

    let proof_path = if let Some(proof_name) = proof_name {
        let proof_path = save_proof_to_dir(&proof, proof_name, proof_dir)?;

        println!("Proof saved to {}", proof_path.display());
        Some(proof_path)
    } else {
        println!("{}", hex::encode(&proof));
        None
    };

    Ok(proof_path)
}

fn save_proof_to_dir<P: AsRef<Path>>(
    proof: &[u8],
    proof_name: &str,
    proof_dir: P,
) -> Result<PathBuf, CliError> {
    let mut proof_path = create_named_dir(proof_dir.as_ref(), "proof");
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    write_to_file(hex::encode(proof).as_bytes(), &proof_path);

    Ok(proof_path)
}

#[allow(deprecated)]
fn fetch_build_artifacts<P: AsRef<Path>>(
    program_dir: P,
    circuit_build_path: Option<P>,
    check_proof: bool,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<(CompiledProgram, Vec<u8>, Vec<u8>), CliError> {
    let backend = crate::backends::ConcreteBackend;
    if let Some(circuit_build_path) = circuit_build_path {
        let mut acir_hash_path = PathBuf::new();
        acir_hash_path.push(circuit_build_path.as_ref());
        acir_hash_path.set_extension(ACIR_EXT.to_owned() + ".sha256");
        let expected_acir_hash = load_hex_data(acir_hash_path.clone())?;

        let compiled_program =
            super::compile_cmd::compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
        let serialized = compiled_program.circuit.to_bytes();

        let mut hasher = Sha256::new();
        hasher.update(serialized);
        let new_acir_hash = hasher.finalize();

        if new_acir_hash[..] != expected_acir_hash {
            return Err(CliError::MismatchedAcir(acir_hash_path));
        }
        let mut proving_key_path = PathBuf::new();
        proving_key_path.push(circuit_build_path.as_ref());
        proving_key_path.set_extension(PK_EXT);
        let proving_key = load_hex_data(proving_key_path)?;

        let verification_key = if check_proof {
            let mut verification_key_path = PathBuf::new();
            verification_key_path.push(circuit_build_path);
            verification_key_path.set_extension(VK_EXT);
            load_hex_data(verification_key_path)?
        } else {
            // We can return an empty Vec here as the verification key is used only is `check_proof` is true
            vec![]
        };

        Ok((compiled_program, proving_key, verification_key))
    } else {
        let compiled_program =
            super::compile_cmd::compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
        let (proving_key, verification_key) = backend.preprocess(compiled_program.circuit.clone());
        Ok((compiled_program, proving_key, verification_key))
    }
}
