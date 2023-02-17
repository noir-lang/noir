use std::path::{Path, PathBuf};

use acvm::ProofSystemCompiler;
use clap::Args;
use noirc_abi::input_parser::Format;

use super::{
    create_named_dir, fetch_pk_and_vk, read_inputs_from_file, write_inputs_to_file, write_to_file,
    NargoConfig,
};
use crate::{
    cli::{execute_cmd::execute_program, verify_cmd::verify_proof},
    constants::{PROOFS_DIR, PROOF_EXT, PROVER_INPUT_FILE, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};

// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {
    /// The name of the proof
    proof_name: Option<String>,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: Option<String>,

    /// Verify proof after proving
    #[arg(short, long)]
    verify: bool,

    /// Issue a warning for each unused variable instead of an error
    #[arg(short, long)]
    allow_warnings: bool,

    /// Emit debug information for the intermediate SSA IR
    #[arg(short, long)]
    show_ssa: bool,
}

pub(crate) fn run(args: ProveCommand, config: NargoConfig) -> Result<(), CliError> {
    let mut proof_dir = config.program_dir.clone();
    proof_dir.push(PROOFS_DIR);

    let circuit_build_path = if let Some(circuit_name) = args.circuit_name {
        let mut circuit_build_path = config.program_dir.clone();
        circuit_build_path.push(TARGET_DIR);
        circuit_build_path.push(circuit_name);
        Some(circuit_build_path)
    } else {
        None
    };

    prove_with_path(
        args.proof_name,
        config.program_dir,
        proof_dir,
        circuit_build_path,
        args.verify,
        args.show_ssa,
        args.allow_warnings,
    )?;

    Ok(())
}

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: Option<String>,
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

    let solved_witness = execute_program(&compiled_program, &inputs_map)?;

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
        let proof_path = save_proof_to_dir(&proof, &proof_name, proof_dir)?;

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
