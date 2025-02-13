use noirc_artifacts::{contract::ContractArtifact, program::ProgramArtifact};

pub mod errors;
pub mod fs;

/// A parsed JSON build artifact.
#[derive(Debug, Clone)]
pub enum Artifact {
    Program(ProgramArtifact),
    Contract(ContractArtifact),
}
