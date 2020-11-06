
use thiserror::Error;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
use noirc_errors::Span;

#[derive(Error, Debug)]
pub enum ArrayError {
    #[error("Out of bounds")]
    OutOfBounds {  index : u128, bound : u128},
}

impl DiagnosableError for ArrayError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
              ArrayError::OutOfBounds{index, bound} => {
                Diagnostic::simple_error(format!("index out of bounds"), format!("out of bounds error, index is {} but length is {}",index, bound), Span::default())
            },
        }
    }
}

#[derive(Error, Debug)]
pub enum EnvironmentError {
    #[error("Cannot find Array")]
    ArrayNotFound {  found_type : String, name : String},
}

impl DiagnosableError for EnvironmentError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
              EnvironmentError::ArrayNotFound{found_type, name} => {
                Diagnostic::simple_error(format!("cannot find an array with name {}", name), format!("{} has type", found_type), Span::default())
            },
        }
    }
}

#[derive(Error, Debug)]
pub enum EvaluatorError {
    #[error("Environment errors")]
    EnvironmentError(EnvironmentError),
    #[error("Array errors")]
    ArrayError(ArrayError),
    #[error("Unstructured Error")]
    UnstructuredError { span: Span, message : String},
    #[error("Unsupported operation error")]
    UnsupportedOp { span: Span, op : String, first_type : String, second_type : String},
}


impl EvaluatorError {
    pub fn expected_type(expected_type : &'static str, found_type : &str) -> EvaluatorError {
        EvaluatorError::UnstructuredError{span : Default::default() , message : format!("Expected a {}, but found {}", expected_type, found_type)}
    }
}

impl DiagnosableError for EvaluatorError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
            EvaluatorError::EnvironmentError(err) => err.to_diagnostic(),
            EvaluatorError::ArrayError(err) => err.to_diagnostic(),
            EvaluatorError::UnstructuredError{span, message} => {
                Diagnostic::simple_error("".to_owned(), message.to_string(), *span)
            },
            EvaluatorError::UnsupportedOp {span, op, first_type, second_type} => {
                Diagnostic::simple_error("unsupported operation".to_owned(), format!("no support for {} with types {} and {}", op, first_type, second_type), *span)
            },
        }
    }
}