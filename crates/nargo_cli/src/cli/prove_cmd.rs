
use tracing::debug;



use clap::Args;

use super::{backend_vendor_cmd::{BackendOptions}, arguments::{ProofArtifact, VerificationKeyArtifact, WitnessArtifact, ProverMetaArtifact, VerifierMetaArtifact}};
use crate::cli::arguments::NargoConfig;

use crate::{
    errors::CliError, cli::backend_vendor_cmd::{self, execute_backend_cmd}, constants,
};

/// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {

    /// The name of the toml file which contains the inputs for the prover
    #[clap(flatten)]
    nargo_prover_meta: ProverMetaArtifact,

    /// The name of the toml file which contains the inputs for the verifier
    #[clap(flatten)]
    nargo_verifier_meta: VerifierMetaArtifact,

    #[clap(flatten)]
    pub(crate) proof_options: ProofArtifact,

    #[clap(flatten)]
    pub(crate) verification_key_options: VerificationKeyArtifact,

    #[clap(flatten)]
    pub(crate) witness_options: WitnessArtifact,

    #[clap(flatten)]
    backend_options: BackendOptions,

    #[clap(long)]
    package: Option<String>,
}

pub(crate) fn run<B: acvm::Backend>(
    _backend: &B,
    args: ProveCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {    


    debug!("Supplied arguments: {:?}", args);

    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args.backend_options)?;
    let mut raw_pass_through= args.backend_options.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(constants::PROVE_SUB_CMD)];
    backend_args.append(&mut raw_pass_through);

    execute_backend_cmd(&backend_executable_path, backend_args, &config).map_err(|e| { CliError::BackendVendorError(e)})
    
}

