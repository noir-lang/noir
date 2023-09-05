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

    assert!(backends.contains(&args.backend), "backend doesn't exist");

    set_active_backend(&args.backend);

    Ok(())
}
