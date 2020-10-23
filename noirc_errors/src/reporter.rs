use crate::Span;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::{SimpleFiles, SimpleFile};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term;

use codespan::{ByteIndex, ByteOffset, RawOffset};
use codespan::{Span as ByteSpan};

pub struct FileMap(SimpleFiles<String, String>);

pub struct FileID(usize);

pub struct File<'input>(&'input SimpleFile<String, String>);

impl<'input> File<'input> {
    pub fn get_source(self) -> &'input str {
        self.0.source()
    }
}

impl FileMap {
    pub fn new() -> Self {
        FileMap(SimpleFiles::new())
    }

    pub fn add_file(&mut self, file_name : String, code: String) -> FileID {
        let file_id = self.0.add(file_name, code);
        FileID(file_id)
    }
    pub fn get_file(&mut self, file_id : FileID) -> Option<File> {
        match self.0.get(file_id.0) {
            Some(source) => Some(File(source)),
            None => None
        }
    }

}

/// Diagnostics 
pub struct CustomDiagnostic {
    // Diagnostic message
    pub message : String, 
    // Span of token which is giving the error message
    pub span : Span,
}

pub struct Reporter<'input> {
    file_map : &'input FileMap,
    diagnostics : Vec<CustomDiagnostic>
}

#[test]
fn test_reporting() {
    // unimplemented!();
}