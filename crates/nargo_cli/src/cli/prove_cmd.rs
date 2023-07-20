
use tracing::debug;



use clap::Args;

use super::{NargoConfig, backend_vendor_cmd::{BackendOptions, ProofArtifact, WitnessArtifact, VerificationKeyArtifact}};

use crate::{
    errors::CliError, cli::backend_vendor_cmd::{self, execute_backend_cmd}, constants,
};

/// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
pub(crate) struct ProveCommand {

    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, default_value = constants::PROVER_INPUT_FILE)]
    nargo_prover_meta: String,

    /// The name of the toml file which contains the inputs for the verifier
    #[clap(long, default_value = constants::VERIFIER_INPUT_FILE)]
    nargo_verifier_meta: String,

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
    backend_options: BackendOptions,

    #[clap(long)]
    package: Option<String>,
}

pub(crate) fn run<B: acvm::Backend>(
    _backend: &B,
    args: BackendOptions,
    config: NargoConfig,
) -> Result<(), CliError<B>> {    


    debug!("Supplied arguments: {:?}", args);

    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args)?;
    let mut raw_pass_through= args.backend_arguments.unwrap_or_default();
    let mut backend_args = vec![String::from(constants::PROVE_SUB_CMD)];
    backend_args.append(&mut raw_pass_through);

    execute_backend_cmd(&backend_executable_path, backend_args, &config).map_err(|e| { CliError::BackendVendorError(e)})
    
}

