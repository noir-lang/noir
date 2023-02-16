use std::path::{Path, PathBuf};

use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::input_parser::Format;

use super::{
    create_named_dir, fetch_pk_and_vk, read_inputs_from_file, write_inputs_to_file, write_to_file,
};
use crate::{
    cli::{execute_cmd::execute_program, verify_cmd::verify_proof},
    constants::{PROOFS_DIR, PROOF_EXT, PROVER_INPUT_FILE, TARGET_DIR, VERIFIER_INPUT_FILE},
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

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: Option<&str>,
    program_dir: P,
    proof_dir: P,
    circuit_build_path: Option<P>,
    check_proof: bool,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<Option<PathBuf>, CliError> {
    let compiled_program =
        super::compile_cmd::compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
    let (proving_key, verification_key) = fetch_pk_and_vk(
        compiled_program.circuit.clone(),
        circuit_build_path.as_ref(),
        true,
        check_proof,
    )?;

    // Parse the initial witness values from Prover.toml
    let inputs_map = read_inputs_from_file(
        &program_dir,
        PROVER_INPUT_FILE,
        Format::Toml,
        &compiled_program.abi,
    )?;

    let (_, solved_witness) = execute_program(&compiled_program, &inputs_map)?;

    // Write public inputs into Verifier.toml
    let public_abi = compiled_program.abi.clone().public_abi();
    let public_inputs = public_abi.decode(&solved_witness)?;
    write_inputs_to_file(&public_inputs, &program_dir, VERIFIER_INPUT_FILE, Format::Toml)?;

    let backend = crate::backends::ConcreteBackend;
    let proof =
        backend.prove_with_pk(compiled_program.circuit.clone(), solved_witness, proving_key);

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
