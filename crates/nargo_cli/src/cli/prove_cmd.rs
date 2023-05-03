use std::path::{Path, PathBuf};

use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::ops::{preprocess_program, prove_execution};
use noirc_abi::input_parser::Format;
use noirc_driver::{CompileOptions, CompiledProgram};

use super::NargoConfig;
use super::{
    compile_cmd::compile_circuit,
    fs::{
        inputs::{read_inputs_from_file, write_inputs_to_file},
        program::read_program_from_file,
        proof::save_proof_to_dir,
    },
};
use crate::{
    cli::{execute_cmd::execute_program, verify_cmd::verify_proof},
    constants::{PROOFS_DIR, PROVER_INPUT_FILE, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError,
};

/// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {
    /// The name of the proof
    proof_name: Option<String>,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: Option<String>,

    /// Verify proof after proving
    #[arg(short, long)]
    verify: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: ProveCommand, config: NargoConfig) -> Result<(), CliError> {
    let proof_dir = config.program_dir.join(PROOFS_DIR);

    let circuit_build_path = args
        .circuit_name
        .map(|circuit_name| config.program_dir.join(TARGET_DIR).join(circuit_name));

    prove_with_path(
        args.proof_name,
        config.program_dir,
        proof_dir,
        circuit_build_path,
        args.verify,
        &args.compile_options,
    )?;

    Ok(())
}

pub(crate) fn prove_with_path<P: AsRef<Path>>(
    proof_name: Option<String>,
    program_dir: P,
    proof_dir: P,
    circuit_build_path: Option<PathBuf>,
    check_proof: bool,
    compile_options: &CompileOptions,
) -> Result<Option<PathBuf>, CliError> {
    let backend = crate::backends::ConcreteBackend::default();

    let preprocessed_program = match circuit_build_path {
        Some(circuit_build_path) => read_program_from_file(circuit_build_path)?,
        None => {
            let compiled_program =
                compile_circuit(&backend, program_dir.as_ref(), compile_options)?;
            preprocess_program(&backend, compiled_program)?
        }
    };

    let PreprocessedProgram { abi, bytecode, proving_key, verification_key, .. } =
        preprocessed_program;
    let compiled_program = CompiledProgram { abi, circuit: bytecode };

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) = read_inputs_from_file(
        &program_dir,
        PROVER_INPUT_FILE,
        Format::Toml,
        &compiled_program.abi,
    )?;

    let solved_witness = execute_program(&backend, &compiled_program, &inputs_map)?;

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

    let proof = prove_execution(&backend, &compiled_program.circuit, solved_witness, &proving_key)?;

    if check_proof {
        let no_proof_name = "".into();
        verify_proof(
            &backend,
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
