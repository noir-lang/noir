use clap::Args;
use noirc_frontend::graph::CrateName;

use crate::errors::CliError;

use super::NargoConfig;

/// Perform formal verification on a program
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "fv")]
pub(crate) struct FormalVerifyCommand {
    /// The name of the package to formally verify
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// formally verify all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,
}

pub(crate) fn run(args: FormalVerifyCommand, config: NargoConfig) -> Result<(), CliError> {
    println!("Hello, this feature is not implemented yet");
    Err(CliError::Generic("Not implemented yet".to_string()))
}