use codespan_reporting::files::{Error, Files, SimpleFile};
use noirc_driver::DebugFile;
use noirc_errors::{debug_info::DebugInfo, Location};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

use fm::{FileId, FileManager, PathString};

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

        let files_with_debug_symbols: BTreeSet<FileId> = debug_symbols
            .iter()
            .flat_map(|function_symbols| {
                function_symbols
                    .locations
                    .values()
                    .filter_map(|call_stack| call_stack.last().map(|location| location.file))
            })
            .collect();

        for file_id in files_with_debug_symbols {
            let file_source = file_manager.fetch_file(file_id).source();

            file_map.insert(
                file_id,
                DebugFile {
                    source: file_source.to_string(),
                    path: file_manager.path(file_id).to_path_buf(),
                },
            );
        }

        Self { debug_symbols, file_map }
    }

    /// Given a location, returns its file's source code
    pub fn location_source_code(&self, location: Location) -> &str {
        self.file_map[&location.file].source.as_str()
    }

    /// Given a location, returns the index of the line it starts at
    pub fn location_line_index(&self, location: Location) -> Result<usize, Error> {
        let location_start = location.span.start() as usize;
        Files::line_index(self, location.file, location_start)
    }

    /// Given a location, returns the line number it starts at
    pub fn location_line_number(&self, location: Location) -> Result<usize, Error> {
        let location_start = location.span.start() as usize;
        let line_index = Files::line_index(self, location.file, location_start);
        Files::line_number(self, location.file, line_index.unwrap())
    }

    /// Given a location, returns the column number it starts at
    pub fn location_column_number(&self, location: Location) -> Result<usize, Error> {
        let location_start = location.span.start() as usize;
        let line_index = Files::line_index(self, location.file, location_start);
        Files::column_number(self, location.file, line_index.unwrap(), location_start)
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
