use fm::PathString;
use std::path::PathBuf;

/// A location in the source code: filename and line number (1-indexed).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SourceLocation {
    pub(crate) filepath: PathString,
    pub(crate) line_number: isize,
}

impl SourceLocation {
    /// Creates a source location that represents an unknown place in the source code.
    pub(crate) fn create_unknown() -> SourceLocation {
        SourceLocation { filepath: PathString::from_path(PathBuf::from("?")), line_number: -1 }
    }
}
