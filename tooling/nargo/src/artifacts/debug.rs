use codespan_reporting::files::{Error, Files, SimpleFile};
use noirc_driver::{CompiledContract, CompiledProgram, DebugFile};
use noirc_errors::{debug_info::DebugInfo, Location};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

pub use super::debug_vars::{DebugVars, StackFrame};
use super::{contract::ContractArtifact, program::ProgramArtifact};
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
                    .flat_map(|call_stack| call_stack.iter().map(|location| location.file))
            })
            .collect();

        for file_id in files_with_debug_symbols {
            let file_path = file_manager.path(file_id).expect("file should exist");
            let file_source = file_manager.fetch_file(file_id).expect("file should exist");

            file_map.insert(
                file_id,
                DebugFile { source: file_source.to_string(), path: file_path.to_path_buf() },
            );
        }

        Self { debug_symbols, file_map }
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

    /// Given a location, returns the index of the line it ends at
    pub fn location_end_line_index(&self, location: Location) -> Result<usize, Error> {
        let location_end = location.span.end() as usize;
        self.line_index(location.file, location_end)
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

        let line_length =
            if line_span.end > line_span.start { line_span.end - (line_span.start + 1) } else { 0 };
        let start_in_line = location_start - line_span.start;

        // The location might continue beyond the line,
        // so we need a bounds check
        let end_in_line = location_end - line_span.start;
        let end_in_line = std::cmp::min(end_in_line, line_length);

        Ok(Range { start: start_in_line, end: end_in_line })
    }

    /// Given a location, returns a Span relative to its last line's
    /// position in the file. This is useful when processing a file's
    /// contents on a per-line-basis.
    pub fn location_in_end_line(&self, location: Location) -> Result<Range<usize>, Error> {
        let end_line_index = self.location_end_line_index(location)?;
        let line_span = self.line_range(location.file, end_line_index)?;
        let location_end = location.span.end() as usize;
        let end_in_line = location_end - line_span.start;
        Ok(Range { start: 0, end: end_in_line })
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
        DebugArtifact { debug_symbols: compiled_program.debug, file_map: compiled_program.file_map }
    }
}

impl From<ProgramArtifact> for DebugArtifact {
    fn from(program_artifact: ProgramArtifact) -> Self {
        DebugArtifact {
            debug_symbols: program_artifact.debug_symbols.debug_infos,
            file_map: program_artifact.file_map,
        }
    }
}

impl From<CompiledContract> for DebugArtifact {
    fn from(compiled_artifact: CompiledContract) -> Self {
        let all_functions_debug: Vec<DebugInfo> = compiled_artifact
            .functions
            .into_iter()
            .flat_map(|contract_function| contract_function.debug)
            .collect();

        DebugArtifact { debug_symbols: all_functions_debug, file_map: compiled_artifact.file_map }
    }
}

impl From<ContractArtifact> for DebugArtifact {
    fn from(compiled_artifact: ContractArtifact) -> Self {
        let all_functions_debug: Vec<DebugInfo> = compiled_artifact
            .functions
            .into_iter()
            .flat_map(|contract_function| contract_function.debug_symbols.debug_infos)
            .collect();

        DebugArtifact { debug_symbols: all_functions_debug, file_map: compiled_artifact.file_map }
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

#[cfg(test)]
mod tests {
    use crate::artifacts::debug::DebugArtifact;
    use acvm::acir::circuit::OpcodeLocation;
    use fm::FileManager;
    use noirc_errors::{debug_info::DebugInfo, Location, Span};
    use std::collections::BTreeMap;
    use std::ops::Range;
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    // Returns the absolute path to the file
    fn create_dummy_file(dir: &TempDir, file_name: &Path) -> PathBuf {
        let file_path = dir.path().join(file_name);
        let _file = std::fs::File::create(&file_path).unwrap();
        file_path
    }

    // Tests that location_in_line correctly handles
    // locations spanning multiple lines.
    // For example, given the snippet:
    // ```
    // permute(
    //    consts::x5_2_config(),
    //    state);
    // ```
    // We want location_in_line to return the range
    // containing `permute(`
    #[test]
    fn location_in_line_stops_at_end_of_line() {
        let source_code = r##"pub fn main(mut state: [Field; 2]) -> [Field; 2] {
    state = permute(
        consts::x5_2_config(),
        state);

    state
}"##;

        let dir = tempdir().unwrap();
        let file_name = Path::new("main.nr");
        create_dummy_file(&dir, file_name);

        let mut fm = FileManager::new(dir.path());
        let file_id = fm.add_file_with_source(file_name, source_code.to_string()).unwrap();

        // Location of
        // ```
        // permute(
        //      consts::x5_2_config(),
        //      state)
        // ```
        let loc = Location::new(Span::inclusive(63, 116), file_id);

        // We don't care about opcodes in this context,
        // we just use a dummy to construct debug_symbols
        let mut opcode_locations = BTreeMap::<OpcodeLocation, Vec<Location>>::new();
        opcode_locations.insert(OpcodeLocation::Acir(42), vec![loc]);

        let debug_symbols = vec![DebugInfo::new(
            opcode_locations,
            BTreeMap::default(),
            BTreeMap::default(),
            BTreeMap::default(),
        )];
        let debug_artifact = DebugArtifact::new(debug_symbols, &fm);

        let location_in_line = debug_artifact.location_in_line(loc).expect("Expected a range");
        assert_eq!(location_in_line, Range { start: 12, end: 20 });
    }
}
