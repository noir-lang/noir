use std::path::{Path, PathBuf};

use noirc_artifacts::trace::TraceArtifact;

use crate::errors::FilesystemError;

use super::write_to_file;

pub(crate) fn save_trace_to_file<P: AsRef<Path>>(trace: &TraceArtifact, trace_dir: P) -> PathBuf {
    let trace_path = trace_dir.as_ref().join("trace").with_extension("json");

    write_to_file(&serde_json::to_vec(trace).unwrap(), &trace_path);

    println!("Saved trace to {:?}", trace_path);

    trace_path
}

pub(crate) fn read_trace_from_file<P: AsRef<Path>>(
    trace_path: P,
) -> Result<TraceArtifact, FilesystemError> {
    let file_path = trace_path.as_ref().with_extension("json");

    let input_string =
        std::fs::read(&file_path).map_err(|_| FilesystemError::PathNotValid(file_path))?;
    let trace = serde_json::from_slice(&input_string)
        .map_err(|err| FilesystemError::ProgramSerializationError(err.to_string()))?;

    Ok(trace)
}
