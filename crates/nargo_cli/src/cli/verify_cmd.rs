use std::collections::HashMap;

use super::{NargoConfig, backend_vendor_cmd::{BackendCommand, ProofArtifact}};
use crate::{
    constants::{self},
    errors::CliError, cli::backend_vendor_cmd::{execute_backend_cmd, VerificationKeyArtifact},
};

use acvm::Backend;
// use acvm::Backend;
use clap::Args;
use nameof::name_of;
use tracing::debug;

use super::backend_vendor_cmd;

/// Given a proof and a program, verify whether the proof is valid
#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyCommand {
    #[clap(flatten)]
    pub(crate) proof_options: ProofArtifact,

    #[clap(flatten)]
    pub(crate) verification_key_options: VerificationKeyArtifact,

    #[clap(flatten)]
    backend_options: BackendCommand
}

pub(crate) fn run<B: Backend>(
    _backend: &B,
    mut args: VerifyCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {    

    backend_vendor_cmd::configure_proof_artifact(&config, &mut args.proof_options);
    backend_vendor_cmd::configure_verification_key_artifact(&config, &mut args.verification_key_options);

    debug!("Supplied Prove arguments: {:?}", args);

    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args.backend_options)?;
    let mut raw_pass_through= args.backend_options.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(constants::VERIFY_SUB_CMD)];
    backend_args.append(&mut raw_pass_through);

    let mut envs = HashMap::new();
    // envs.insert(name_of!(nargo_artifact_path in NargoConfig).to_uppercase(), String::from(config.nargo_artifact_path.unwrap().as_os_str().to_str().unwrap()));
    envs.insert(name_of!(nargo_proof_path in ProofArtifact).to_uppercase(), String::from(args.proof_options.nargo_proof_path.unwrap().as_os_str().to_str().unwrap()));
    envs.insert(name_of!(nargo_verification_key_path in VerificationKeyArtifact).to_uppercase(), String::from(args.verification_key_options.nargo_verification_key_path.unwrap().as_os_str().to_str().unwrap()));
    
    execute_backend_cmd(&backend_executable_path, backend_args, &config.nargo_package_root, Some(envs)).map_err(|e| { CliError::Generic(e.to_string()) })
    
}
