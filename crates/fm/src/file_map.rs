use codespan_reporting::files::{SimpleFile, SimpleFiles};
use std::path::PathBuf;

use crate::FileManager;

// XXX: File and FileMap serve as opaque types, so that the rest of the library does not need to import the dependency
// or worry about when we change the dep

#[derive(Clone, Debug)]
pub struct PathString(PathBuf);

impl std::fmt::Display for PathString {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        f.write_str(&self.0.to_string_lossy())
    }
}

impl PathString {
    pub fn from_path(p: PathBuf) -> Self {
        PathString(p)
    }
}
impl From<PathBuf> for PathString {
    fn from(pb: PathBuf) -> PathString {
        PathString::from_path(pb)
    }
}
impl From<&PathBuf> for PathString {
    fn from(pb: &PathBuf) -> PathString {
        PathString::from(pb.to_owned())
    }
}
#[derive(Debug)]
pub struct FileMap(SimpleFiles<PathString, String>);

// XXX: Note that we derive Default here due to ModuleOrigin requiring us to set a FileId
#[derive(Default, Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct FileId(usize);

impl FileId {
    //XXX: find a way to remove the need for this. Errors do not need to attach their FileIds immediately!
    pub fn as_usize(&self) -> usize {
        self.0
    }

    pub fn dummy() -> FileId {
        FileId(0)
    }
}

pub struct File<'input>(&'input SimpleFile<PathString, String>);

impl<'input> File<'input> {
    pub fn get_source(self) -> &'input str {
        self.0.source()
    }
}

impl FileMap {
    pub fn new() -> Self {
        FileMap(SimpleFiles::new())
    }

    pub fn add_file(&mut self, file_name: PathString, code: String) -> FileId {
        let file_id = self.0.add(file_name, code);
        FileId(file_id)
    }
    pub fn get_file(&self, file_id: FileId) -> Option<File> {
        self.0.get(file_id.0).map(File)
    }
}

impl Default for FileMap {
    fn default() -> Self {
        Self::new()
    }
}

impl FileManager {
    // Needed as code_span dep requires underlying SimpleFiles
    pub fn as_simple_files(&self) -> &SimpleFiles<PathString, String> {
        &self.file_map.0
    }
}
