use std::{process::{Command, Stdio}, io::{BufReader, BufRead}};

// use acvm::Backend;
use clap::Args;
use tracing::debug;

use super::{NargoConfig, backend_vendor_cmd::BackendSubcommand};
use crate::{
    // constants::{PROOFS_DIR, PROVER_INPUT_FILE, TARGET_DIR, VERIFIER_INPUT_FILE},
    errors::CliError, cli::backend_vendor_cmd,
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

    // #[clap(flatten)]
    // compile_options: CompileOptions,

    #[clap(flatten)]
    backend_options: BackendSubcommand
}

pub(crate) fn run(
    args: ProveCommand,
    _config: NargoConfig,
) -> Result<i32, CliError> {    
    debug!("Args: {:?}", args);
    debug!("Cfg: {:?}", _config);

    let backend_executable_path = backend_vendor_cmd::resolve_backend(&args.backend_options, &_config)?;
    let mut raw_pass_through= args.backend_options.raw_pass_through.unwrap_or_default();
    let mut backend_args = vec!["prove".to_string()];
    backend_args.append(&mut raw_pass_through);

    let exit_code = backend_vendor_cmd::execute_backend_cmd(&backend_executable_path, backend_args).unwrap();

    Ok(exit_code)
}


