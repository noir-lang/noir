use std::collections::HashMap;

// use super::compile_cmd::compile_circuit;
// use super::fs::{
//     common_reference_string::{
//         read_cached_common_reference_string, update_common_reference_string,
//         write_cached_common_reference_string,
//     },
//     inputs::read_inputs_from_file,
//     load_hex_data,
//     program::read_program_from_file,
// };
use super::{NargoConfig, backend_vendor_cmd::{BackendCommand, ProofArtifact}};
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, TARGET_DIR, VERIFIER_INPUT_FILE, self},
    errors::CliError, cli::backend_vendor_cmd::execute_backend_cmd,
};

// use acvm::Backend;
use clap::Args;
use nameof::name_of;
use tracing::debug;
// use nargo::artifacts::program::PreprocessedProgram;
// use nargo::ops::{preprocess_program, verify_proof};
// use noirc_abi::input_parser::Format;
// use noirc_driver::CompileOptions;
// use std::path::{Path, PathBuf};

use super::backend_vendor_cmd;

/// Given a proof and a program, verify whether the proof is valid
#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyCommand {
    // / The proof to verify
    // proof: String,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    // circuit_name: Option<String>,

    // #[clap(flatten)]
    // compile_options: CompileOptions,
    #[clap(flatten)]
    pub(crate) proof_options: ProofArtifact,

    #[clap(flatten)]
    backend_options: BackendCommand
}

pub(crate) fn run(
    mut args: VerifyCommand,
    config: NargoConfig,
) -> Result<i32, CliError> {    

    backend_vendor_cmd::configure_proof_artifact(&config, &mut args.proof_options);

    debug!("Supplied Prove arguments: {:?}", args);

    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args.backend_options, &config)?;
    let mut raw_pass_through= args.backend_options.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(constants::VERIFY_SUB_CMD)];
    backend_args.append(&mut raw_pass_through);

    let mut envs = HashMap::new();
    // envs.insert(name_of!(nargo_artifact_path in NargoConfig).to_uppercase(), String::from(config.nargo_artifact_path.unwrap().as_os_str().to_str().unwrap()));
    envs.insert(name_of!(nargo_proof_path in ProofArtifact).to_uppercase(), String::from(args.proof_options.nargo_proof_path.unwrap().as_os_str().to_str().unwrap()));
    envs.insert(name_of!(nargo_verification_key_path in ProofArtifact).to_uppercase(), String::from(args.proof_options.nargo_verification_key_path.unwrap().as_os_str().to_str().unwrap()));
    let exit_code = execute_backend_cmd(&backend_executable_path, backend_args, &config.nargo_package_root, Some(envs));

    match exit_code {
        Ok(code) => {
            if code > 0 {
                Err(CliError::Generic(format!("Backend exited with failure code: {}", code)))
            } else {
                Ok(code)
            }
        },
        Err(err) => Err(err),
    }
    
}

// pub(crate) fn run(
//     // backend: &B,
//     args: VerifyCommand,
//     config: NargoConfig,
// ) -> Result<(i32), CliError> {
//     // let proof_path =
//     //     config.program_dir.join(PROOFS_DIR).join(&args.proof).with_extension(PROOF_EXT);

//     // let circuit_build_path = args
//     //     .circuit_name
//     //     .map(|circuit_name| config.program_dir.join(TARGET_DIR).join(circuit_name));

//     // verify_with_path(
//     //     backend,
//     //     &config.program_dir,
//     //     proof_path,
//     //     circuit_build_path.as_ref(),
//     //     &args.compile_options,
//     // )

//     let backend_executable_path = backend_vendor_cmd::resolve_backend(&args.backend_options, &config)?;
//     let mut raw_pass_through= args.backend_options.backend_arguments.unwrap_or_default();
//     let mut backend_args = vec![String::from("verify")];
//     backend_args.append(&mut raw_pass_through);

//     let exit_code = backend_vendor_cmd::execute_backend_cmd(&backend_executable_path, backend_args).unwrap();

//     Ok(exit_code)

// }
// fn verify_with_path<B: Backend, P: AsRef<Path>>(
//     backend: &B,
//     program_dir: P,
//     proof_path: PathBuf,
//     circuit_build_path: Option<P>,
//     compile_options: &CompileOptions,
// ) -> Result<(), CliError> {

// fn verify_with_path<B: Backend, P: AsRef<Path>>(
//     backend: &B,
//     program_dir: P,
//     proof_path: PathBuf,
//     circuit_build_path: Option<P>,
//     compile_options: &CompileOptions,
// ) -> Result<(), CliError> {
//     let common_reference_string = read_cached_common_reference_string();

//     let (common_reference_string, preprocessed_program) = match circuit_build_path {
//         Some(circuit_build_path) => {
//             let program = read_program_from_file(circuit_build_path)?;
//             let common_reference_string = update_common_reference_string(
//                 backend,
//                 &common_reference_string,
//                 &program.bytecode,
//             )
//             .map_err(CliError::CommonReferenceStringError)?;
//             (common_reference_string, program)
//         }
//         None => {
//             let program = compile_circuit(backend, program_dir.as_ref(), compile_options)?;
//             let common_reference_string =
//                 update_common_reference_string(backend, &common_reference_string, &program.circuit)
//                     .map_err(CliError::CommonReferenceStringError)?;
//             let program = preprocess_program(backend, &common_reference_string, program)
//                 .map_err(CliError::ProofSystemCompilerError)?;
//             (common_reference_string, program)
//         }
//     };

//     write_cached_common_reference_string(&common_reference_string);

//     let PreprocessedProgram { abi, bytecode, verification_key, .. } = preprocessed_program;

//     // Load public inputs (if any) from `VERIFIER_INPUT_FILE`.
//     let public_abi = abi.public_abi();
//     let (public_inputs_map, return_value) =
//         read_inputs_from_file(program_dir, VERIFIER_INPUT_FILE, Format::Toml, &public_abi)?;

//     let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;
//     let proof = load_hex_data(&proof_path)?;

//     let valid_proof = verify_proof(
//         backend,
//         &common_reference_string,
//         &bytecode,
//         &proof,
//         public_inputs,
//         &verification_key,
//     )
//     .map_err(CliError::ProofSystemCompilerError)?;

//     if valid_proof {
//         Ok(())
//     } else {
//         Err(CliError::InvalidProof(proof_path))
//     }
// }
