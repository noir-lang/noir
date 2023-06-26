use std::collections::HashMap;

use super::{NargoConfig, backend_vendor_cmd::{BackendCommand, ContractArtifact}};
use crate::{
    constants::{self},
    errors::CliError, cli::backend_vendor_cmd::{execute_backend_cmd, VerificationKeyArtifact},
};

use acvm::Backend;
use clap::Args;
use nameof::name_of;
use tracing::debug;

use super::backend_vendor_cmd;

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct ContractCommand {

    #[clap(flatten)]
    pub(crate) contract_options: ContractArtifact,

    #[clap(flatten)]
    pub(crate) verification_key_options: VerificationKeyArtifact,

    #[clap(flatten)]
    backend_options: BackendCommand
}

pub(crate) fn run<B: Backend>(
    _backend: &B,
    mut args: ContractCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {    

    backend_vendor_cmd::configure_contract_artifact(&config, &mut args.contract_options);
    backend_vendor_cmd::configure_verification_key_artifact(&config, &mut args.verification_key_options);

    debug!("Supplied CodeGen arguments: {:?}", args);

    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args.backend_options)?;
    let mut raw_pass_through= args.backend_options.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(constants::CONTRACT_SUB_CMD)];
    backend_args.append(&mut raw_pass_through);

    let mut envs = HashMap::new();
    envs.insert(name_of!(nargo_artifact_path in NargoConfig).to_uppercase(), String::from(config.nargo_artifact_path.unwrap().as_os_str().to_str().unwrap()));
    envs.insert(name_of!(nargo_verification_key_path in VerificationKeyArtifact).to_uppercase(), String::from(args.verification_key_options.nargo_verification_key_path.unwrap().as_os_str().to_str().unwrap()));
    envs.insert(name_of!(nargo_contract_path in ContractArtifact).to_uppercase(), String::from(args.contract_options.nargo_contract_path.unwrap().as_os_str().to_str().unwrap()));
    
    execute_backend_cmd(&backend_executable_path, backend_args, &config.nargo_package_root, Some(envs)).map_err(|e| { CliError::BackendVendorError(e)})
}
