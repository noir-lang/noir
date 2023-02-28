use std::{
    error::Error,
    path::{Path, PathBuf},
};

use acvm::ProofSystemCompiler;
use clap::Args;
use iter_extended::try_btree_map;
use noirc_abi::{
    input_parser::{Format, InputValue},
};

use super::fs::{
    inputs::{read_inputs_from_file, write_inputs_to_file},
    keys::fetch_pk_and_vk,
    program::read_program_from_file,
    proof::save_proof_to_dir,
};
use super::NargoConfig;
use crate::{
    cli::{execute_cmd::execute_program, verify_cmd::verify_proof},
    constants::{PROOFS_DIR, PROVER_INPUT_FILE, TARGET_DIR, VERIFIER_INPUT_FILE},
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

    #[arg(short, long, value_parser = parse_key_val::<String, String>)]
    inputs: Option<Vec<(String, String)>>,
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
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
        args.inputs,
    )?;

    Ok(())
}

pub(crate) fn prove_with_path<P: AsRef<Path>>(
    proof_name: Option<String>,
    program_dir: P,
    proof_dir: P,
    circuit_build_path: Option<PathBuf>,
    check_proof: bool,
    show_ssa: bool,
    allow_warnings: bool,
    inputs: Option<Vec<(String, String)>>,
) -> Result<Option<PathBuf>, CliError> {
    let (compiled_program, proving_key, verification_key) = match circuit_build_path {
        Some(circuit_build_path) => {
            let compiled_program = read_program_from_file(&circuit_build_path)?;

            let (proving_key, verification_key) =
                fetch_pk_and_vk(&compiled_program.circuit, circuit_build_path, true, true)?;
            (compiled_program, proving_key, verification_key)
        }
        None => {
            let compiled_program = super::compile_cmd::compile_circuit(
                program_dir.as_ref(),
                show_ssa,
                allow_warnings,
            )?;

            let backend = crate::backends::ConcreteBackend;
            let (proving_key, verification_key) = backend.preprocess(&compiled_program.circuit);
            (compiled_program, proving_key, verification_key)
        }
    };

    // Parse the initial witness values from Prover.toml
    let inputs_map = if let Some(inputs) = inputs {
        // TODO: move this to an appropiate place
        try_btree_map(inputs, |(key, value)| {
            let abi_map = &compiled_program.abi.to_btree_map();
            InputValue::try_from_cli_args(value, &abi_map[&key])
                .map(|input_value| (key, input_value))
        })?
    } else {
        let (map, _) = read_inputs_from_file(&program_dir, PROVER_INPUT_FILE, Format::Toml, &compiled_program.abi)?
        map
    };

    let solved_witness = execute_program(&compiled_program, &inputs_map)?;

    // Write public inputs into Verifier.toml
    let public_abi = compiled_program.abi.clone().public_abi();
    let (public_inputs, return_value) = public_abi.decode(&solved_witness)?;

    write_inputs_to_file(
        &public_inputs,
        &return_value,
        &program_dir,
        VERIFIER_INPUT_FILE,
        Format::Toml,
    )?;

    let backend = crate::backends::ConcreteBackend;
    let proof = backend.prove_with_pk(&compiled_program.circuit, solved_witness, &proving_key);

    if check_proof {
        let no_proof_name = "".into();
        verify_proof(
            &compiled_program,
            public_inputs,
            return_value,
            &proof,
            &verification_key,
            no_proof_name,
        )?;
    }

    let proof_path = if let Some(proof_name) = proof_name {
        Some(save_proof_to_dir(&proof, &proof_name, proof_dir)?)
    } else {
        println!("{}", hex::encode(&proof));
        None
    };

    Ok(proof_path)
}
