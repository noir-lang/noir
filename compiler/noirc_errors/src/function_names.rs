use std::collections::HashMap;

use fm::FileId;
use rangemap::RangeMap;

use crate::Location;

/// Maps function location ranges to their names, to be used when showing call stack frames
/// during error reporting.
#[derive(Default)]
pub struct FunctionNames {
    files: HashMap<FileId, RangeMap<u32, String>>,
}

impl FunctionNames {
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
}
