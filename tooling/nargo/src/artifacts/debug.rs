use codespan_reporting::files::{Error, Files, SimpleFile};
use noirc_driver::{CompiledContract, CompiledProgram, DebugFile};
use noirc_errors::{debug_info::DebugInfo, Location};
use noirc_evaluator::errors::SsaReport;
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
    pub warnings: Vec<SsaReport>,
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
                    .flat_map(|call_stack| call_stack.iter().map(|location| location.file))
            })
            .collect();

        for file_id in files_with_debug_symbols {
            let file_source = file_manager.fetch_file(file_id);

            file_map.insert(
                file_id,
                DebugFile {
                    source: file_source.to_string(),
                    path: file_manager.path(file_id).to_path_buf(),
                },
            );
        }

        Self { debug_symbols, file_map, warnings: Vec::new() }
    }

    /// Given a location, returns its file's source code
    pub fn location_source_code(&self, location: Location) -> Result<&str, Error> {
        self.source(location.file)
    }

    /// Given a location, returns the index of the line it starts at
    pub fn location_line_index(&self, location: Location) -> Result<usize, Error> {
        let location_start = location.span.start() as usize;
        self.line_index(location.file, location_start)
    }

    /// Given a location, returns the line number it starts at
    pub fn location_line_number(&self, location: Location) -> Result<usize, Error> {
        let location_start = location.span.start() as usize;
        let line_index = self.line_index(location.file, location_start)?;
        self.line_number(location.file, line_index)
    }

    /// Given a location, returns the column number it starts at
    pub fn location_column_number(&self, location: Location) -> Result<usize, Error> {
        let location_start = location.span.start() as usize;
        let line_index = self.line_index(location.file, location_start)?;
        self.column_number(location.file, line_index, location_start)
    }

    /// Given a location, returns a Span relative to its line's
    /// position in the file. This is useful when processing a file's
    /// contents on a per-line-basis.
    pub fn location_in_line(&self, location: Location) -> Result<Range<usize>, Error> {
        let location_start = location.span.start() as usize;
        let location_end = location.span.end() as usize;
        let line_index = self.line_index(location.file, location_start)?;
        let line_span = self.line_range(location.file, line_index)?;

        let start_in_line = location_start - line_span.start;
        let end_in_line = location_end - line_span.start;

        Ok(Range { start: start_in_line, end: end_in_line })
    }

    /// Given a location, returns the last line index
    /// of its file
    pub fn last_line_index(&self, location: Location) -> Result<usize, Error> {
        let source = self.source(location.file)?;
        self.line_index(location.file, source.len())
    }
}

impl From<CompiledProgram> for DebugArtifact {
    fn from(compiled_program: CompiledProgram) -> Self {
        DebugArtifact {
            debug_symbols: vec![compiled_program.debug],
            file_map: compiled_program.file_map,
            warnings: compiled_program.warnings,
        }
    }
}

impl From<&CompiledContract> for DebugArtifact {
    fn from(compiled_artifact: &CompiledContract) -> Self {
        let all_functions_debug: Vec<DebugInfo> = compiled_artifact
            .functions
            .iter()
            .map(|contract_function| contract_function.debug.clone())
            .collect();

        DebugArtifact {
            debug_symbols: all_functions_debug,
            file_map: compiled_artifact.file_map.clone(),
            warnings: compiled_artifact.warnings.clone(),
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
