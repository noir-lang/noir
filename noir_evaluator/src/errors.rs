
use thiserror::Error;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
use noirc_errors::Span;

#[derive(Error, Debug)]
pub enum RuntimeErrorKind {

    // Array errors
    #[error("Out of bounds")]
    ArrayOutOfBounds {  index : u128, bound : u128},
    
    // Environment errors
    #[error("Cannot find Array")]
    ArrayNotFound {  found_type : String, name : String},
    
    #[error("Unstructured Error")]
    UnstructuredError { span: Span, message : String},
    #[error("Unsupported operation error")]
    UnsupportedOp { span: Span, op : String, first_type : String, second_type : String},
}
impl RuntimeErrorKind {
    pub fn into_err(self, file_id : usize) -> RuntimeError {
        RuntimeError {
            kind : self,
            file_id
        }
    }
}
#[derive(Debug)]
pub struct RuntimeError {
    kind: RuntimeErrorKind,
    file_id : usize
}


impl RuntimeErrorKind {
    pub fn expected_type(expected_type : &'static str, found_type : &str) -> RuntimeErrorKind {
        RuntimeErrorKind::UnstructuredError{span : Default::default() , message : format!("Expected a {}, but found {}", expected_type, found_type)}
    }
}

impl DiagnosableError for RuntimeError {
    fn to_diagnostic(&self) -> Diagnostic{
        match &self.kind {
            RuntimeErrorKind::ArrayOutOfBounds{index, bound} => {
                Diagnostic::simple_error(format!("index out of bounds"), format!("out of bounds error, index is {} but length is {}",index, bound), Span::default())
            },
            RuntimeErrorKind::ArrayNotFound{found_type, name} => {
                Diagnostic::simple_error(format!("cannot find an array with name {}", name), format!("{} has type", found_type), Span::default())
            },
            RuntimeErrorKind::UnstructuredError{span, message} => {
                Diagnostic::simple_error("".to_owned(), message.to_string(), *span)
            },
            RuntimeErrorKind::UnsupportedOp {span, op, first_type, second_type} => {
                Diagnostic::simple_error("unsupported operation".to_owned(), format!("no support for {} with types {} and {}", op, first_type, second_type), *span)
            },
        }
    }
}