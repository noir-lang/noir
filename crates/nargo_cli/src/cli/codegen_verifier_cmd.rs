

use super::{backend_vendor_cmd::{BackendOptions}, arguments::{VerificationKeyArtifact, ContractArtifact}};
use crate::{
    constants::{self},
    errors::CliError, cli::backend_vendor_cmd::{execute_backend_cmd},
};
use crate::cli::arguments::NargoConfig;

use acvm::Backend;
use clap::Args;

use tracing::debug;

use super::backend_vendor_cmd;

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct CodegenVerifierCommand {

    #[clap(flatten)]
    pub(crate) contract_options: ContractArtifact,

    #[clap(flatten)]
    pub(crate) verification_key_options: VerificationKeyArtifact,

    #[clap(flatten)]
    backend_options: BackendOptions
}

pub(crate) fn run<B: Backend>(
    _backend: &B,
    args: CodegenVerifierCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {    

    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args.backend_options)?;
    let mut raw_pass_through= args.backend_options.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(constants::CONTRACT_SUB_CMD)];
    backend_args.append(&mut raw_pass_through);

    execute_backend_cmd(&backend_executable_path, backend_args, &config).map_err(|e| { CliError::BackendVendorError(e)})
}
