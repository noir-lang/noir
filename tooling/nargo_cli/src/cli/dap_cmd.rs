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
    noir_debugger::start_dap_server().map_err(CliError::DapError)
}
