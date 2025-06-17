#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

use noirc_artifacts::{contract::ContractArtifact, program::ProgramArtifact};

pub mod commands;
pub mod errors;
pub mod execution;
pub mod fs;

/// A parsed JSON build artifact.
#[derive(Debug, Clone)]
pub enum Artifact {
    Program(ProgramArtifact),
    Contract(ContractArtifact),
}
