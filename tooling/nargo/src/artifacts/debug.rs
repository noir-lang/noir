use codespan_reporting::files::{Error, Files, SimpleFile};
use noirc_driver::{DebugFile,CompiledProgram};
use noirc_errors::debug_info::DebugInfo;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ops::Range,
};

use fm::{FileId, FileManager, PathString};
pub use super::debug_vars::DebugVars;

/// A Debug Artifact stores, for a given program, the debug info for every function
/// along with a map of file Id to the source code so locations in debug info can be mapped to source code they point to.
#[derive(Debug, Serialize, Deserialize)]
pub struct DebugArtifact {
    pub debug_symbols: Vec<DebugInfo>,
    pub file_map: BTreeMap<FileId, DebugFile>,
}

impl DebugArtifact {
    pub fn new(debug_symbols: Vec<DebugInfo>, file_manager: &FileManager) -> Self {
        let mut file_map = BTreeMap::new();

        let file_ids: Vec<FileId> = debug_symbols
            .iter()
            .flat_map(|debug_info| debug_info.get_file_ids())
            .collect();

        for file_id in file_ids.iter() {
            let file_source = file_manager.fetch_file(*file_id).source();

            file_map.insert(
                *file_id,
                DebugFile {
                    source: file_source.to_string(),
                    path: file_manager.path(*file_id).to_path_buf(),
                },
            );
        }

        Self { debug_symbols, file_map }
    }

    pub fn from_program(program: &CompiledProgram) -> Self {
        Self {
            debug_symbols: vec![program.debug.clone()],
            file_map: program.file_map.clone(),
        }
    }
}

impl<'a> Files<'a> for DebugArtifact {
    type FileId = FileId;
    type Name = PathString;
    type Source = &'a str;

    fn name(&self, file_id: Self::FileId) -> Result<Self::Name, Error> {
        self.file_map.get(&file_id).ok_or(Error::FileMissing).map(|file| file.path.clone().into())
    }

    fn source(&'a self, file_id: Self::FileId) -> Result<Self::Source, Error> {
        self.file_map.get(&file_id).ok_or(Error::FileMissing).map(|file| file.source.as_ref())
    }

    fn line_index(&self, file_id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        self.file_map.get(&file_id).ok_or(Error::FileMissing).and_then(|file| {
            SimpleFile::new(PathString::from(file.path.clone()), file.source.clone())
                .line_index((), byte_index)
        })
    }

    fn line_range(&self, file_id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error> {
        self.file_map.get(&file_id).ok_or(Error::FileMissing).and_then(|file| {
            SimpleFile::new(PathString::from(file.path.clone()), file.source.clone())
                .line_range((), line_index)
        })
    }
}
