use thiserror::Error;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
pub use noirc_errors::Span;
use crate::ast::{Ident, Expression, Type};

use super::resolve::Resolver;

#[derive(Error, Debug)]
pub enum ResolverErrorKind {
    #[error("Duplicate definition")]
    DuplicateDefinition { first_span: Span, second_span: Span, ident : String},
    #[error("Unused variables")]
    UnusedVariables { span: Span, ident : String},
    #[error("Could not find symbol in this scope")]
    Unresolved { span: Span, symbol_type : String, symbol : String},
}

impl ResolverErrorKind {
    pub fn into_err(self,file_id : usize) -> ResolverError {
        ResolverError {
            kind: self,
            file_id,
        }
    }
}

#[derive(Error,Debug)]
pub struct ResolverError {
    kind : ResolverErrorKind, 
    file_id : usize,
}


impl Into<AnalyserError> for ResolverError {
    fn into(self) -> AnalyserError {
        AnalyserError::Resolver(self)
    }
}

impl std::fmt::Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl DiagnosableError for ResolverError {
    fn to_diagnostic(&self) -> Diagnostic{
        match &self.kind {
            ResolverErrorKind::DuplicateDefinition {first_span, second_span, ident} => {
                let mut diag = Diagnostic::simple_error(self.file_id,format!("duplicate definitions of {} found", ident), format!("first definition found here"), *first_span);
                diag.add_secondary(format!("second definition found here"), *second_span);
                diag
            }
            ResolverErrorKind::UnusedVariables {span, ident} => {
                let mut diag = Diagnostic::simple_error(self.file_id,format!("unused variable {}", ident), format!("unused variable "), *span);
                diag.add_note("A new variable usually means a constraint has been added and is being unused. \n For this reason, it is almost always a bug to declare a variable and not use it.".to_owned());
                diag
            }
            ResolverErrorKind::Unresolved {span, symbol_type, symbol} => {
                Diagnostic::simple_error(self.file_id,format!("cannot find {} `{}` in this scope ", symbol_type, symbol), format!("not found in this scope"), *span)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum TypeErrorKind {
    #[error("Array is not homogenous")]
    NonHomogenousArray { first_span: Span, first_type : String, first_index : usize, second_span: Span, second_type : String, second_index : usize},
}

impl TypeErrorKind {
    pub fn into_err(self,file_id : usize) -> TypeError {
        TypeError {
            kind: self,
            file_id,
        }
    }
}

#[derive(Error, Debug)]
pub struct TypeError {
    kind : TypeErrorKind,
    file_id : usize,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}


impl Into<AnalyserError> for TypeError {
    fn into(self) -> AnalyserError {
        AnalyserError::Type(self)
    }
}

impl DiagnosableError for TypeError {
    fn to_diagnostic(&self) -> Diagnostic{
        match &self.kind {
            TypeErrorKind::NonHomogenousArray{ first_span, first_type ,first_index,second_span, second_type, second_index} => {
                let mut diag = Diagnostic::simple_error(self.file_id,format!("Non homogenous array found at indices ({},{})", first_index, second_index), format!("found type {}", first_type), *first_span);
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
    Resolver(ResolverError),
    #[error("Type Error")]
    Type(TypeError),
    #[error("Unstructured")]
    Unstructured { span: Span, message : String, file_id : usize},
}

impl DiagnosableError for AnalyserError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
            AnalyserError::Resolver(res) => res.to_diagnostic(),
            AnalyserError::Type(res) => res.to_diagnostic(),
            AnalyserError::Unstructured{span, message, file_id} => {
                Diagnostic::simple_error(*file_id, "".to_owned(), message.to_string(), *span)
            },
        }
    }
}

impl AnalyserError {
    pub fn from_ident(file_id : usize, message : String, ident: &Ident) -> AnalyserError {
        AnalyserError::Unstructured{file_id,message, span :ident.0.span() }
    }
    pub fn from_expression(file_id : usize, message : String, expr: &Expression) -> AnalyserError {
        AnalyserError::Unstructured{file_id, message, span :expr.span }
    }
    pub fn type_mismatch(file_id : usize, expected : &Type, got : &Type, expr_span : Span) -> AnalyserError {
        // Type does not have span information, so we take it as a parameter for now
        let message = format!("Type mismatch: expected {}, got {}", expected, got);
        return AnalyserError::Unstructured{file_id, message, span :expr_span }
    }  
}