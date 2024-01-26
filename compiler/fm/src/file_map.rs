use codespan_reporting::files::{Error, Files, SimpleFile, SimpleFiles};
use noirc_errors::SrcId;
use std::collections::HashMap;
use std::{ops::Range, path::PathBuf};

// XXX: File and FileMap serve as opaque types, so that the rest of the library does not need to import the dependency
// or worry about when we change the dep

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
#[derive(Debug, Clone)]
pub struct FileMap {
    files: SimpleFiles<PathString, String>,
    name_to_id: HashMap<PathString, SrcId>,
}

pub struct File<'input>(&'input SimpleFile<PathString, String>);

impl<'input> File<'input> {
    pub fn source(self) -> &'input str {
        self.0.source()
    }
}

impl FileMap {
    pub fn add_file(&mut self, file_name: PathString, code: String) -> SrcId {
        let file_id = SrcId(self.files.add(file_name.clone(), code));
        self.name_to_id.insert(file_name, file_id);
        file_id
    }

    pub fn get_file(&self, file_id: SrcId) -> Option<File> {
        self.files.get(file_id.0).map(File).ok()
    }

    pub fn get_file_id(&self, file_name: &PathString) -> Option<SrcId> {
        self.name_to_id.get(file_name).cloned()
    }

    pub fn all_file_ids(&self) -> impl Iterator<Item = &SrcId> {
        self.name_to_id.values()
    }
}
impl Default for FileMap {
    fn default() -> Self {
        FileMap { files: SimpleFiles::new(), name_to_id: HashMap::new() }
    }
}

impl<'a> Files<'a> for FileMap {
    type FileId = SrcId;
    type Name = PathString;
    type Source = &'a str;

    fn name(&self, file_id: Self::FileId) -> Result<Self::Name, Error> {
        Ok(self.files.get(file_id.into())?.name().clone())
    }

    fn source(&'a self, file_id: Self::FileId) -> Result<Self::Source, Error> {
        Ok(self.files.get(file_id.into())?.source().as_ref())
    }

    fn line_index(&self, file_id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        self.files.get(file_id.into())?.line_index((), byte_index)
    }

    fn line_range(&self, file_id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error> {
        self.files.get(file_id.into())?.line_range((), line_index)
    }
}
