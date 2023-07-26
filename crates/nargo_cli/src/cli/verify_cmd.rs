

use super::arguments::VerificationKeyArtifact;
use super::backend_vendor_cmd::{BackendOptions};
use crate::cli::arguments::ProofArtifact;
use crate::cli::arguments::NargoConfig;
use crate::{
    constants::{self},
    errors::CliError, cli::backend_vendor_cmd::{execute_backend_cmd},
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
    backend_options: BackendOptions
}

pub(crate) fn run<B: Backend>(
    _backend: &B,
    args: VerifyCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {    

    debug!("Supplied Prove arguments: {:?}", args);

    let mut mut_config = config.clone();
    if Option::None == mut_config.nargo_proof_path {
        mut_config.nargo_proof_path = Some(args.proof_options.nargo_proof_path);
    }
    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args.backend_options)?;
    let mut raw_pass_through= args.backend_options.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(constants::VERIFY_SUB_CMD)];
    backend_args.append(&mut raw_pass_through);

    execute_backend_cmd(&backend_executable_path, backend_args, &config).map_err(|e| { CliError::Generic(e.to_string()) })
    
}
