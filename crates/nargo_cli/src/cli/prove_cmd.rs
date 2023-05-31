use std::path::{Path, PathBuf};

use acvm::Backend;
use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::ops::{preprocess_program, prove_execution, verify_proof};
use noirc_abi::input_parser::Format;
use noirc_driver::CompileOptions;

use super::NargoConfig;
use super::{
    compile_cmd::compile_circuit,
    fs::{
        common_reference_string::{
            read_cached_common_reference_string, update_common_reference_string,
            write_cached_common_reference_string,
        },
        inputs::{read_inputs_from_file, write_inputs_to_file},
        program::read_program_from_file,
        proof::save_proof_to_dir,
    },
};
use crate::{
    cli::execute_cmd::execute_program,
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

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: ProveCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let proof_dir = config.program_dir.join(PROOFS_DIR);

    let circuit_build_path = args
        .circuit_name
        .map(|circuit_name| config.program_dir.join(TARGET_DIR).join(circuit_name));

    prove_with_path(
        backend,
        args.proof_name,
        config.program_dir,
        proof_dir,
        circuit_build_path,
        args.verify,
        &args.compile_options,
    )?;

    Ok(())
}

pub(crate) fn prove_with_path<B: Backend, P: AsRef<Path>>(
    backend: &B,
    proof_name: Option<String>,
    program_dir: P,
    proof_dir: P,
    circuit_build_path: Option<PathBuf>,
    check_proof: bool,
    compile_options: &CompileOptions,
) -> Result<Option<PathBuf>, CliError<B>> {
    let common_reference_string = read_cached_common_reference_string();

    let (common_reference_string, preprocessed_program) = match circuit_build_path {
        Some(circuit_build_path) => {
            let program = read_program_from_file(circuit_build_path)?;
            let common_reference_string = update_common_reference_string(
                backend,
                &common_reference_string,
                &program.bytecode,
            )
            .map_err(CliError::CommonReferenceStringError)?;
            (common_reference_string, program)
        }
        None => {
            let program = compile_circuit(backend, program_dir.as_ref(), compile_options)?;
            let common_reference_string =
                update_common_reference_string(backend, &common_reference_string, &program.circuit)
                    .map_err(CliError::CommonReferenceStringError)?;
            let program = preprocess_program(backend, &common_reference_string, program)
                .map_err(CliError::ProofSystemCompilerError)?;
            (common_reference_string, program)
        }
    };

    write_cached_common_reference_string(&common_reference_string);

    let PreprocessedProgram { abi, bytecode, proving_key, verification_key, .. } =
        preprocessed_program;

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&program_dir, PROVER_INPUT_FILE, Format::Toml, &abi)?;

    let solved_witness = execute_program(backend, bytecode.clone(), &abi, &inputs_map)?;

    // Write public inputs into Verifier.toml
    let public_abi = abi.public_abi();
    let (public_inputs, return_value) = public_abi.decode(&solved_witness)?;

    write_inputs_to_file(
        &public_inputs,
        &return_value,
        &program_dir,
        VERIFIER_INPUT_FILE,
        Format::Toml,
    )?;

    let proof =
        prove_execution(backend, &common_reference_string, &bytecode, solved_witness, &proving_key)
            .map_err(CliError::ProofSystemCompilerError)?;

    if check_proof {
        let public_inputs = public_abi.encode(&public_inputs, return_value)?;
        let valid_proof = verify_proof(
            backend,
            &common_reference_string,
            &bytecode,
            &proof,
            public_inputs,
            &verification_key,
        )
        .map_err(CliError::ProofSystemCompilerError)?;

        if !valid_proof {
            return Err(CliError::InvalidProof("".into()));
        }
    }

    let proof_path = if let Some(proof_name) = proof_name {
        Some(save_proof_to_dir(&proof, &proof_name, proof_dir)?)
    } else {
        println!("{}", hex::encode(&proof));
        None
    };

    Ok(proof_path)
}
