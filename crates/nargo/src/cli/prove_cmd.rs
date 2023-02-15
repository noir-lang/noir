use std::path::{Path, PathBuf};

use acvm::ProofSystemCompiler;
use clap::Args;
use noirc_abi::input_parser::Format;

use super::{
    create_named_dir, dedup_public_input_indices, read_inputs_from_file, write_inputs_to_file,
    write_to_file, NargoConfig,
};
use crate::{
    cli::{
        execute_cmd::{execute_program, extract_public_inputs},
        verify_cmd::verify_proof,
    },
    constants::{PROOFS_DIR, PROOF_EXT, PROVER_INPUT_FILE, VERIFIER_INPUT_FILE},
    errors::CliError,
};

// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {
    /// The name of the proof
    proof_name: Option<String>,

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

    prove_with_path(
        args.proof_name,
        config.program_dir,
        proof_dir,
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
    check_proof: bool,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<Option<PathBuf>, CliError> {
    let compiled_program =
        super::compile_cmd::compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;

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
    let proof = backend.prove_with_meta(prover_circuit, solved_witness);

    println!("Proof successfully created");
    if check_proof {
        let valid_proof = verify_proof(compiled_program, public_inputs, &proof)?;
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
