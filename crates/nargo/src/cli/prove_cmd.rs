use std::path::{Path, PathBuf};

use acvm::ProofSystemCompiler;
use clap::Args;
use noirc_abi::input_parser::Format;

use super::execute_cmd::{execute_program, extract_public_inputs};
use super::{create_named_dir, write_inputs_to_file, write_to_file, NargoConfig};
use crate::cli::dedup_public_input_indices;
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, VERIFIER_INPUT_FILE},
    errors::CliError,
};

/// Create proof for this program
#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {
    /// The name of the proof
    proof_name: Option<String>,

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
        args.show_ssa,
        args.allow_warnings,
    )?;

    Ok(())
}

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: Option<String>,
    program_dir: P,
    proof_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<Option<PathBuf>, CliError> {
    let mut compiled_program =
        super::compile_cmd::compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
    let (_, solved_witness) = execute_program(&program_dir, &compiled_program)?;

    // Write public inputs into Verifier.toml
    let public_inputs = extract_public_inputs(&compiled_program, &solved_witness)?;
    write_inputs_to_file(&public_inputs, &program_dir, VERIFIER_INPUT_FILE, Format::Toml)?;

    // Since the public outputs are added onto the public inputs list, there can be duplicates.
    // We keep the duplicates for when one is encoding the return values into the Verifier.toml,
    // however we must remove these duplicates when creating a proof.
    compiled_program.circuit.public_inputs =
        dedup_public_input_indices(compiled_program.circuit.public_inputs);

    let backend = crate::backends::ConcreteBackend;
    let proof = backend.prove_with_meta(compiled_program.circuit, solved_witness);

    println!("Proof successfully created");
    if let Some(proof_name) = proof_name {
        let proof_path = save_proof_to_dir(proof, &proof_name, proof_dir)?;

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
