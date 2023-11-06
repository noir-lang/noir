use backend_interface::Backend;
use clap::Args;

use crate::errors::CliError;

use super::NargoConfig;

#[derive(Debug, Clone, Args)]
pub(crate) struct DapCommand;

pub(crate) fn run(
    _backend: &Backend,
    _args: DapCommand,
    _config: NargoConfig,
) -> Result<(), CliError> {
    Ok(())
}
