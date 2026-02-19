use std::path::PathBuf;

use acvm::compiler::find_dead_witnesses;
use clap::Args;

use crate::{Artifact, errors::CliError};

use super::parse_and_normalize_path;

/// Check for dead witnesses in a compiled program or contract artifact.
#[derive(Debug, Clone, Args)]
pub struct CheckDeadWitnessesCommand {
    /// Path to the JSON build artifact (program or contract).
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub artifact_path: PathBuf,
}

pub fn run(args: CheckDeadWitnessesCommand) -> Result<(), CliError> {
    let artifact = Artifact::read_from_file(&args.artifact_path)?;

    let circuits = match &artifact {
        Artifact::Program(program) => &program.bytecode.functions,
        Artifact::Contract(_) => {
            return Err(CliError::Generic(
                "contract artifacts are not yet supported for dead witness checking".to_string(),
            ));
        }
    };

    let mut has_dead = false;

    for (i, circuit) in circuits.iter().enumerate() {
        let dead_witnesses = find_dead_witnesses(circuit);
        if !dead_witnesses.is_empty() {
            eprintln!("func {i}: dead witnesses: {dead_witnesses:?}");
            has_dead = true;
        }
    }

    if has_dead {
        return Err(CliError::DeadWitnessesFound);
    }

    Ok(())
}
