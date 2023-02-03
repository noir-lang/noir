use std::path::{Path, PathBuf};

use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::input_parser::Format;

use super::execute_cmd::{execute_program, extract_public_inputs};
use super::{create_named_dir, write_inputs_to_file, write_to_file};
use crate::cli::dedup_public_input_indices;
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, VERIFIER_INPUT_FILE},
    errors::CliError,
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("prove").unwrap();
    let proof_name = args.value_of("proof_name");
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");

    prove(proof_name, show_ssa, allow_warnings)
}

fn prove(proof_name: Option<&str>, show_ssa: bool, allow_warnings: bool) -> Result<(), CliError> {
    let current_dir = std::env::current_dir().unwrap();

    let mut proof_dir = PathBuf::new();
    proof_dir.push(PROOFS_DIR);

    prove_with_path(proof_name, current_dir, proof_dir, show_ssa, allow_warnings)?;

    Ok(())
}

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: Option<&str>,
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
