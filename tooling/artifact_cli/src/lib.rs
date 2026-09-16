#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

// Used by the `noir-execute` binary in `src/bin/execute.rs`, which this crate
// root's lint does not cover.
use const_format as _;
use tracing_subscriber as _;

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
