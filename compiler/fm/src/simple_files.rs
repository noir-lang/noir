use std::ops::Range;

use codespan_reporting::files::{Error, Files, SimpleFile};

/// This is exactly the same as `codespan_reporting::files::SimpleFiles`, and in fact
/// it was copied from there. However, it also provides a `replace` method to allow
/// replacing the contents of a file. This is only used by LSP: when a file is modified
/// (but still unsaved) we'll replace its contents in the database, then run the compiler
/// on just that file. The original `SimpleFiles` does not provide such method.
#[derive(Debug, Clone)]
pub(crate) struct SimpleFiles<Name, Source> {
    files: Vec<SimpleFile<Name, Source>>,
}

impl<Name, Source> SimpleFiles<Name, Source>
where
    Name: std::fmt::Display + Clone,
    Source: AsRef<str>,
{
    /// Create a new files database.
    pub(crate) fn new() -> SimpleFiles<Name, Source> {
        SimpleFiles { files: Vec::new() }
    }

    /// Add a file to the database, returning the handle that can be used to
    /// refer to it again.
    pub(crate) fn add(&mut self, name: Name, source: Source) -> usize {
        let file_id = self.files.len();
        self.files.push(SimpleFile::new(name, source));
        file_id
    }

    /// Get the file corresponding to the given id.
    pub(crate) fn get(&self, file_id: usize) -> Result<&SimpleFile<Name, Source>, Error> {
        self.files.get(file_id).ok_or(Error::FileMissing)
    }

    /// Replaces the contents of the file with the given id.
    pub(crate) fn replace(&mut self, file_id: usize, source: Source) {
        let file = self.files.get_mut(file_id).unwrap();
        *file = SimpleFile::new(file.name().clone(), source);
    }
}

impl<'a, Name, Source> Files<'a> for SimpleFiles<Name, Source>
where
    Name: 'a + std::fmt::Display + Clone,
    Source: 'a + AsRef<str>,
{
    type FileId = usize;
    type Name = Name;
    type Source = &'a str;

    fn name(&self, file_id: usize) -> Result<Name, Error> {
        Ok(self.get(file_id)?.name().clone())
    }

    fn source(&self, file_id: usize) -> Result<&str, Error> {
        Ok(self.get(file_id)?.source().as_ref())
    }

    fn line_index(&self, file_id: usize, byte_index: usize) -> Result<usize, Error> {
        self.get(file_id)?.line_index((), byte_index)
    }

    fn line_range(&self, file_id: usize, line_index: usize) -> Result<Range<usize>, Error> {
        self.get(file_id)?.line_range((), line_index)
    }
}
