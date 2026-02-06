use std::collections::HashMap;

use fm::FileId;
use noirc_span::Span;
use rangemap::RangeMap;

use crate::Location;

/// Maps function location ranges to their names, to be used when showing call stack frames
/// during error reporting.
#[derive(Default)]
pub struct FunctionLocations {
    files: HashMap<FileId, RangeMap<u32, String>>,
}

impl FunctionLocations {
    pub fn new() -> Self {
        Self { files: HashMap::new() }
    }

    /// Maps a location range to a function name.
    pub fn insert(&mut self, location: Location, name: String) {
        let range_map = self.files.entry(location.file).or_default();
        range_map.insert(location.span.start()..location.span.end(), name);
    }

    /// Returns the function name, if any, associated with the given location.
    pub fn lookup(&self, location: Location) -> Option<&str> {
        self.files
            .get(&location.file)
            .and_then(|range_map| range_map.get(&location.span.start()))
            .map(|str| str.as_str())
    }

    /// Returns all registered function names in the given file, along with the range they are defined in.
    pub fn all_in_file(&mut self, file: FileId) -> impl Iterator<Item = (&str, Span)> {
        self.files
            .entry(file)
            .or_default()
            .iter()
            .map(|(range, name)| (name.as_str(), Span::from(range.start..range.end)))
    }
}
