use crate::Span;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::{SimpleFiles, SimpleFile};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term;

use codespan::{ByteIndex, ByteOffset, RawOffset};
use codespan::{Span as ByteSpan};

pub struct FileMap(SimpleFiles<String, String>);

#[derive(Copy, Clone)]
pub struct FileID(pub usize);

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

pub struct Reporter;

impl Reporter {
    pub fn with_diagnostics(file_id : FileID, files : &FileMap, diagnostics : &Vec<CustomDiagnostic>) {       
        // Convert each Custom Diagnostic into a diagnostic
        let diagnostics : Vec<_> = diagnostics.into_iter().map(|cd| {

            let start_span = cd.span.start.to_byte_index().to_usize();
            let end_span = cd.span.end.to_byte_index().to_usize() + 1;
            
            Diagnostic::error()
                // .with_code("E01")
                .with_labels(vec![
        Label::secondary(file_id.0, start_span..end_span).with_message(&cd.message),
        ])
        
        }).collect();

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();

        for diagnostic in diagnostics {
            term::emit(&mut writer.lock(), &config, &files.0, &diagnostic).unwrap();
        }
}
}