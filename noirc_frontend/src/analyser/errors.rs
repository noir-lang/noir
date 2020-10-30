use thiserror::Error;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
pub use noirc_errors::Span;
use crate::ast::{Ident, Expression, Type};

#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("Duplicate definition")]
    DuplicateDefinition { first_span: Span, second_span: Span, ident : String},
    #[error("Unused variables")]
    UnusedVariables { span: Span, ident : String},
    #[error("Could not find symbol in this scope")]
    Unresolved { span: Span, symbol_type : String, symbol : String},
}

impl Into<AnalyserError> for ResolverError {
    fn into(self) -> AnalyserError {
        AnalyserError::ResolverError(self)
    }
}

impl DiagnosableError for ResolverError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
            ResolverError::DuplicateDefinition {first_span, second_span, ident} => {
                let mut diag = Diagnostic::simple_error(format!("duplicate definitions of {} found", ident), format!("first definition found here"), *first_span);
                diag.add_secondary(format!("second definition found here"), *second_span);
                diag
            }
            ResolverError::UnusedVariables {span, ident} => {
                let mut diag = Diagnostic::simple_error(format!("unused variable {}", ident), format!("unused variable "), *span);
                diag.add_note("A new variable usually means a constraint has been added and is being unused. \n For this reason, it is almost always a bug to declare a variable and not use it.".to_owned());
                diag
            }
            ResolverError::Unresolved {span, symbol_type, symbol} => {
                Diagnostic::simple_error(format!("cannot find {} `{}` in this scope ", symbol_type, symbol), format!("not found in this scope"), *span)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Array is not homogenous")]
    NonHomogenousArray { first_span: Span, first_type : String, first_index : usize, second_span: Span, second_type : String, second_index : usize},
}

impl DiagnosableError for TypeError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
            TypeError::NonHomogenousArray{ first_span, first_type ,first_index,second_span, second_type, second_index} => {
                let mut diag = Diagnostic::simple_error(format!("Non homogenous array found at indices ({},{})", first_index, second_index), format!("found type {}", first_type), *first_span);
                diag.add_secondary(format!("but then found type {}", second_type), *second_span);
                diag.add_note("elements in an array must have the same type".to_owned());
                diag
            }
        }
    }
}



#[derive(Error, Debug)]
pub enum AnalyserError {
    #[error("Resolver Error")]
    ResolverError(ResolverError),
    #[error("Type Error")]
    TypeError(TypeError),
    #[error("Unstructured")]
    Unstructured { span: Span, message : String},
}

impl DiagnosableError for AnalyserError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
            AnalyserError::ResolverError(res) => res.to_diagnostic(),
            AnalyserError::TypeError(res) => res.to_diagnostic(),
            AnalyserError::Unstructured{span, message} => {
                Diagnostic::simple_error("".to_owned(), message.to_string(), *span)
            },
        }
    }
}

impl AnalyserError {
    pub fn from_ident(message : String, ident: &Ident) -> AnalyserError {
        AnalyserError::Unstructured{message, span :ident.0.span() }
    }
    pub fn from_expression(message : String, expr: &Expression) -> AnalyserError {
        AnalyserError::Unstructured{message, span :expr.span }
    }
}