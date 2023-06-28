

use super::{NargoConfig, backend_vendor_cmd::{BackendOptions, ProofArtifact}};
use crate::{
    constants::{self},
    errors::CliError, cli::backend_vendor_cmd::{execute_backend_cmd, VerificationKeyArtifact},
};

use acvm::Backend;
// use acvm::Backend;
use clap::Args;

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
    backend_options: BackendOptions
}

pub(crate) fn run<B: Backend>(
    _backend: &B,
    args: BackendOptions,
    config: NargoConfig,
) -> Result<(), CliError<B>> {    

    debug!("Supplied Prove arguments: {:?}", args);

    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args)?;
    let mut raw_pass_through= args.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(constants::VERIFY_SUB_CMD)];
    backend_args.append(&mut raw_pass_through);

    execute_backend_cmd(&backend_executable_path, backend_args, &config).map_err(|e| { CliError::Generic(e.to_string()) })
    
}
