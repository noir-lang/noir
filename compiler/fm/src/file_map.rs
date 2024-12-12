use codespan_reporting::files::{Error, Files, SimpleFile, SimpleFiles};
use serde::{Deserialize, Serialize};
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
    name_to_id: HashMap<PathString, FileId>,
    current_dir: Option<PathBuf>,
}

// XXX: Note that we derive Default here due to ModuleOrigin requiring us to set a FileId
#[derive(
    Default, Debug, Clone, PartialEq, Eq, Copy, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
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
    pub fn source(self) -> &'input str {
        self.0.source()
    }
}

impl FileMap {
    pub fn add_file(&mut self, file_name: PathString, code: String) -> FileId {
        let file_id = FileId(self.files.add(file_name.clone(), code));
        self.name_to_id.insert(file_name, file_id);
        file_id
    }

    pub fn get_file(&self, file_id: FileId) -> Option<File> {
        self.files.get(file_id.0).map(File).ok()
    }

    pub fn get_file_id(&self, file_name: &PathString) -> Option<FileId> {
        self.name_to_id.get(file_name).cloned()
    }

    pub fn all_file_ids(&self) -> impl Iterator<Item = &FileId> {
        self.name_to_id.values()
    }

    pub fn get_name(&self, file_id: FileId) -> Result<PathString, Error> {
        let name = self.files.get(file_id.as_usize())?.name().clone();

        // See if we can make the file name a bit shorter/easier to read if it starts with the current directory
        if let Some(current_dir) = &self.current_dir {
            if let Ok(name_without_prefix) = name.0.strip_prefix(current_dir) {
                return Ok(PathString::from_path(name_without_prefix.to_path_buf()));
            }
        }

        Ok(name)
    }
}
impl Default for FileMap {
    fn default() -> Self {
        FileMap {
            files: SimpleFiles::new(),
            name_to_id: HashMap::new(),
            current_dir: std::env::current_dir().ok(),
        }
    }
}

impl<'a> Files<'a> for FileMap {
    type FileId = FileId;
    type Name = PathString;
    type Source = &'a str;

    fn name(&self, file_id: Self::FileId) -> Result<Self::Name, Error> {
        self.get_name(file_id)
    }

    fn source(&'a self, file_id: Self::FileId) -> Result<Self::Source, Error> {
        Ok(self.files.get(file_id.as_usize())?.source().as_ref())
    }

    fn line_index(&self, file_id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        self.files.get(file_id.as_usize())?.line_index((), byte_index)
    }

    fn line_range(&self, file_id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error> {
        self.files.get(file_id.as_usize())?.line_range((), line_index)
    }
}
