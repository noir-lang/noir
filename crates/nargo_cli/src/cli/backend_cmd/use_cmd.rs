use clap::Args;

use crate::{backends::set_active_backend, errors::CliError};

use super::ls_cmd::get_available_backends;

/// Select the currently active backend
#[derive(Debug, Clone, Args)]
pub(crate) struct UseCommand {
    backend: String,
}

pub(crate) fn run(args: UseCommand) -> Result<(), CliError> {
    let backends = get_available_backends();

    if !backends.contains(&args.backend) {
        // If its bb we re-download it
        if args.backend == "acvm-backend-barretenberg" {
            acvm_backend_barretenberg::get_bb();
        } else {
            return Err(CliError::Generic(format!("backend {} doesn't exist", args.backend)));
        }
    }

    set_active_backend(&args.backend);

    Ok(())
}
