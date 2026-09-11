use std::path::{Path, PathBuf};

use acir::FieldElement;
use acvm::acir::native_types::{WitnessMap, WitnessStack};

use crate::{
    errors::{CliError, FilesystemError},
    fs::artifact::write_to_file,
};

/// Write `witness.gz` to the output directory.
pub fn save_witness_to_dir(
    witnesses: &WitnessStack<FieldElement>,
    witness_name: &str,
    witness_dir: &Path,
) -> Result<PathBuf, CliError> {
    std::fs::create_dir_all(witness_dir)?;

    let witness_path = witness_dir.join(witness_name).with_extension("gz");

    let buf: Vec<u8> = witnesses.serialize().map_err(|e| {
        FilesystemError::OutputWitnessCreationFailed(witness_path.clone(), format!("{e:?}"))
    })?;

    write_to_file(buf.as_slice(), &witness_path).map_err(|e| {
        FilesystemError::OutputWitnessCreationFailed(witness_path.clone(), e.to_string())
    })?;

    Ok(witness_path)
}

pub fn load_witness_from_file(witness_path: &Path) -> Result<WitnessStack<FieldElement>, CliError> {
    let witness_data = std::fs::read(witness_path)?;

    Ok(WitnessStack::deserialize(&witness_data).map_err(|e| {
        FilesystemError::InvalidInputFile(witness_path.to_path_buf(), e.to_string())
    })?)
}

/// Read the initial witness to execute a circuit with out of a witness file.
///
/// An empty witness stack is well-formed — [`load_witness_from_file`] accepts it and `check-witness`
/// validates every item it holds, which is none — it just has no witness to execute with.
pub fn load_initial_witness_from_file(
    witness_path: &Path,
) -> Result<WitnessMap<FieldElement>, CliError> {
    let mut witness_stack = load_witness_from_file(witness_path)?;

    let Some(stack_item) = witness_stack.pop() else {
        return Err(FilesystemError::EmptyWitnessFile(witness_path.to_path_buf()).into());
    };

    Ok(stack_item.witness)
}

#[cfg(test)]
mod tests {
    use acvm::acir::native_types::{Witness, WitnessMap};

    use super::*;

    fn write_witness_stack(stack: &WitnessStack<FieldElement>, dir: &Path) -> PathBuf {
        save_witness_to_dir(stack, "witness", dir).expect("witness stack should serialize")
    }

    #[test]
    fn reads_the_initial_witness_off_the_stack() {
        let mut witness_map = WitnessMap::new();
        witness_map.insert(Witness(0), FieldElement::from(7u128));

        let mut stack = WitnessStack::default();
        stack.push(0, witness_map.clone());

        let dir = tempfile::tempdir().unwrap();
        let path = write_witness_stack(&stack, dir.path());

        assert_eq!(load_initial_witness_from_file(&path).unwrap(), witness_map);
    }

    /// An empty witness stack is well-formed, it simply holds no witness to execute with. Reading
    /// one has to be an input error rather than a panic.
    #[test]
    fn reports_an_empty_witness_stack_as_an_input_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_witness_stack(&WitnessStack::<FieldElement>::default(), dir.path());

        // The stack is only empty, not corrupt, so it still loads.
        assert!(load_witness_from_file(&path).unwrap().peek().is_none());

        let error = load_initial_witness_from_file(&path).unwrap_err();
        assert!(
            matches!(error, CliError::FilesystemError(FilesystemError::EmptyWitnessFile(_))),
            "unexpected error: {error}"
        );
    }
}
