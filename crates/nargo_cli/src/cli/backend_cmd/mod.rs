use clap::{Args, Subcommand};

use crate::errors::CliError;

mod ls_cmd;
mod use_cmd;

#[non_exhaustive]
#[derive(Args, Clone, Debug)]
pub(crate) struct BackendCommand {
    #[command(subcommand)]
    command: BackendCommands,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
pub(crate) enum BackendCommands {
    Ls(ls_cmd::LsCommand),
    Use(use_cmd::UseCommand),
}

pub(crate) fn run(cmd: BackendCommand) -> Result<(), CliError> {
    let BackendCommand { command } = cmd;

    match command {
        BackendCommands::Ls(args) => ls_cmd::run(args),
        BackendCommands::Use(args) => use_cmd::run(args),
    }?;

    Ok(())
}
