use thiserror::Error;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
use noirc_errors::Span;
use crate::ast::{Ident, Expression};

#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("Duplicate definition")]
    DuplicateDefinition { first_span: Span, second_span: Span, ident : String},
    #[error("Unused variables")]
    UnusedVariables { span: Span, ident : String},
    #[error("Could not find symbol in this scope")]
    Unresolved { span: Span, symbol_type : String, symbol : String},
    #[error("Unstructured")]
    Unstructured { span: Span, message : String},
}

impl ResolverError {
    pub fn from_ident(message : String, ident: &Ident) -> ResolverError {
        ResolverError::Unstructured{message, span :ident.0.span() }
    }
    pub fn from_expression(message : String, expr: &Expression) -> ResolverError {
        ResolverError::Unstructured{message, span :expr.span }
    }
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
                Diagnostic{
                    message : format!("duplicate definition of {:?} , first definition found at {}", first_span, ident),
                    span : *second_span
                }
            }
            ResolverError::UnusedVariables {span, ident} => {
                Diagnostic{
                    message : format!("unused variable {}", ident),
                    span : *span
                }
            }
            ResolverError::Unstructured {span, message} => {
                Diagnostic{
                    message : message.to_string(),
                    span : *span
                }
            }
            ResolverError::Unresolved {span, symbol_type, symbol} => {
                Diagnostic{
                    message : format!("cannot find {} `{}` in this scope ", symbol_type, symbol),
                    span : *span
                }
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum AnalyserError {
    #[error("Resolver Error")]
    ResolverError(ResolverError),
    #[error("Unstructured Error")]
    UnstructuredError { span: Span, message : String},
}

impl DiagnosableError for AnalyserError {
    fn to_diagnostic(&self) -> Diagnostic{
        match self {
            AnalyserError::ResolverError(res) => res.to_diagnostic(),
            AnalyserError::UnstructuredError{span, message} => {
                Diagnostic{
                    message : message.to_string(),
                    span : *span
                }
            },
        }
    }
}