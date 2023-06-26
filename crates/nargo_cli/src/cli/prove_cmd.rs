use nameof::name_of;
use tracing::debug;

use std::{collections::HashMap};

use clap::Args;





use super::{NargoConfig, backend_vendor_cmd::{BackendCommand, ProofArtifact, WitnessArtifact, VerificationKeyArtifact}};

use crate::{
    errors::CliError, cli::backend_vendor_cmd::{self, execute_backend_cmd}, constants,
};

/// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {
    /// The name of the proof
    // proof_name: Option<String>,

    /// The name of the circuit build files (ACIR, proving and verification keys)
    // circuit_name: Option<String>,

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = constants::PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the toml file which contains the inputs for the verifier
    #[clap(long, short, default_value = constants::VERIFIER_INPUT_FILE)]
    verifier_name: String,

    /// Verify proof after proving
    // #[arg(short, long)]
    // verify: bool,

    #[clap(flatten)]
    pub(crate) proof_options: ProofArtifact,

    #[clap(flatten)]
    pub(crate) verification_key_options: VerificationKeyArtifact,

    #[clap(flatten)]
    pub(crate) witness_options: WitnessArtifact,

    #[clap(flatten)]
    backend_options: BackendCommand


}

pub(crate) fn run<B: acvm::Backend>(
    _backend: &B,
    mut args: ProveCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {    

    backend_vendor_cmd::configure_proof_artifact(&config, &mut args.proof_options);

    backend_vendor_cmd::configure_verification_key_artifact(&config, &mut args.verification_key_options);

    backend_vendor_cmd::configure_witness_artifact(&config, &mut args.witness_options);

    debug!("Supplied Prove arguments: {:?}", args);

    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args.backend_options)?;
    let mut raw_pass_through= args.backend_options.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(constants::PROVE_SUB_CMD)];
    backend_args.append(&mut raw_pass_through);

    let mut envs = HashMap::new();
    envs.insert(name_of!(nargo_artifact_path in NargoConfig).to_uppercase(), String::from(config.nargo_artifact_path.unwrap().as_os_str().to_str().unwrap()));
    envs.insert(name_of!(nargo_proof_path in ProofArtifact).to_uppercase(), String::from(args.proof_options.nargo_proof_path.unwrap().as_os_str().to_str().unwrap()));
    envs.insert(name_of!(nargo_verification_key_path in VerificationKeyArtifact).to_uppercase(), String::from(args.verification_key_options.nargo_verification_key_path.unwrap().as_os_str().to_str().unwrap()));
    envs.insert(name_of!(nargo_witness_path in WitnessArtifact).to_uppercase(), String::from(args.witness_options.nargo_witness_path.unwrap().as_os_str().to_str().unwrap()));
    
    execute_backend_cmd(&backend_executable_path, backend_args, &config.nargo_package_root, Some(envs)).map_err(|e| { CliError::BackendVendorError(e)})
    
}

