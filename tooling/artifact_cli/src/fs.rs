use std::path::Path;

use color_eyre::eyre::{self, Context};
use noirc_artifacts::{contract::ContractArtifact, program::ProgramArtifact};

/// A parsed JSON build artifact.
#[derive(Debug, Clone)]
pub enum Artifact {
    Program(ProgramArtifact),
    Contract(ContractArtifact),
}

impl Artifact {
    /// Try to parse an artifact as a binary program or a contract
    pub fn read_from_file(path: &Path) -> eyre::Result<Artifact> {
        let json = std::fs::read(path)
            .with_context(|| format!("failed to read artifact from '{}'", path.display()))?;

        let as_program = || serde_json::from_slice::<ProgramArtifact>(&json).map(Artifact::Program);
        let as_contract =
            || serde_json::from_slice::<ContractArtifact>(&json).map(Artifact::Contract);

        Ok(as_program().or_else(|e| as_contract().map_err(|_| e))?)
    }
}
