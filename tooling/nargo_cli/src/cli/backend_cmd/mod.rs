use clap::{Args, Subcommand};

use crate::errors::CliError;

mod current_cmd;
mod install_cmd;
mod ls_cmd;
mod uninstall_cmd;
mod use_cmd;

#[non_exhaustive]
#[derive(Args, Clone, Debug)]
/// Install and select custom backends used to generate and verify proofs.
pub(crate) struct BackendCommand {
    #[command(subcommand)]
    command: BackendCommands,
}

#[non_exhaustive]
#[derive(Subcommand, Clone, Debug)]
pub(crate) enum BackendCommands {
    Current(current_cmd::CurrentCommand),
    Ls(ls_cmd::LsCommand),
    Use(use_cmd::UseCommand),
    Install(install_cmd::InstallCommand),
    Uninstall(uninstall_cmd::UninstallCommand),
}

pub(crate) fn run(cmd: BackendCommand) -> Result<(), CliError> {
    let BackendCommand { command } = cmd;

    match command {
        BackendCommands::Current(args) => current_cmd::run(args),
        BackendCommands::Ls(args) => ls_cmd::run(args),
        BackendCommands::Use(args) => use_cmd::run(args),
        BackendCommands::Install(args) => install_cmd::run(args),
        BackendCommands::Uninstall(args) => uninstall_cmd::run(args),
    }?;

    Ok(())
}
