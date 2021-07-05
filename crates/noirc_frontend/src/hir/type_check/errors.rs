use noirc_errors::CustomDiagnostic as Diagnostic;
pub use noirc_errors::Span;
use thiserror::Error;

use crate::node_interner::NodeInterner;
use crate::{hir_def::expr::HirBinaryOp, Type};

#[derive(Error, Debug, Clone)]
pub enum TypeCheckError {
    #[error("operator {op:?} cannot be used in a {place:?}")]
    OpCannotBeUsed {
        op: HirBinaryOp,
        place: &'static str,
        span: Span,
    },
    #[error("type {typ:?} cannot be used in a {place:?}")]
    TypeCannotBeUsed {
        typ: Type,
        place: &'static str,
        span: Span,
    },
    #[error("expected type {expected_typ:?} is not the same as {expr_typ:?}")]
    TypeMismatch {
        expected_typ: String,
        expr_typ: String,
        expr_span: Span,
    },
    #[error("expected {expected:?} found {found:?}")]
    ArityMisMatch {
        expected: u16,
        found: u16,
        span: Span,
    },
    #[error("return type in a function cannot be public")]
    PublicReturnType { typ: Type, span: Span },
    // XXX: unstructured errors are not ideal for testing.
    // They will be removed in a later iteration
    #[error("unstructured msg: {msg:?}")]
    Unstructured { msg: String, span: Span },
    // Usually the type checker will return after the first encountered errors
    // Due to the fact that types depend on each other.
    // This is not the case in a CallExpression however, or more generally a list of expressions
    #[error("multiple errors when type checking list of expressions")]
    MultipleErrors(Vec<TypeCheckError>),
    #[error("error with additional context")]
    Context {
        err: Box<TypeCheckError>,
        ctx: &'static str,
    },
    #[error("Array is not homogeneous")]
    NonHomogeneousArray {
        first_span: Span,
        first_type: String,
        first_index: usize,
        second_span: Span,
        second_type: String,
        second_index: usize,
    },
}

impl TypeCheckError {
    pub fn into_diagnostics(self, interner: &NodeInterner) -> Vec<Diagnostic> {
        match self {
            TypeCheckError::TypeCannotBeUsed { typ, place, span } => {
                vec![Diagnostic::simple_error(
                    format!("the type {} cannot be used in a {}", &typ, place),
                    format!(""),
                    span,
                )]
            }
            TypeCheckError::MultipleErrors(errors) => errors
                .into_iter()
                .flat_map(|err| err.into_diagnostics(interner))
                .collect(),
            TypeCheckError::Context { err, ctx } => {
                let mut diags = err.into_diagnostics(interner);

                // Cannot add a single context to multiple errors
                assert!(diags.len() == 1);

                let mut diag = diags.pop().unwrap();
                diag.add_note(ctx.to_owned());
                vec![diag]
            }
            TypeCheckError::OpCannotBeUsed { op, place, span } => {
                vec![Diagnostic::simple_error(
                    format!("the operator {:?} cannot be used in a {}", op, place),
                    format!(""),
                    span,
                )]
            }
            TypeCheckError::TypeMismatch {
                expected_typ,
                expr_typ,
                expr_span,
            } => {
                vec![Diagnostic::simple_error(
                    format!("expected type {}, found type {}", expected_typ, expr_typ),
                    format!(""),
                    expr_span,
                )]
            }
            TypeCheckError::NonHomogeneousArray {
                first_span,
                first_type,
                first_index,
                second_span,
                second_type,
                second_index,
            } => {
                let mut diag = Diagnostic::simple_error(
                    format!(
                        "Non homogeneous array, different element types found at indices ({},{})",
                        first_index, second_index
                    ),
                    format!("found type {}", first_type),
                    first_span,
                );
                diag.add_secondary(format!("but then found type {}", second_type), second_span);
                vec![diag]
            }
            TypeCheckError::ArityMisMatch {
                expected,
                found,
                span,
            } => {
                vec![Diagnostic::simple_error(
                    format!("expected {} number of arguments, found {}", expected, found),
                    format!(""),
                    span,
                )]
            }
            TypeCheckError::Unstructured { msg, span } => {
                vec![Diagnostic::simple_error(msg, format!(""), span)]
            }
            TypeCheckError::PublicReturnType { typ, span } => {
                vec![Diagnostic::simple_error(
                    "functions cannot declare a public return type".to_string(),
                    format!("return type is {}", typ),
                    span,
                )]
            }
        }
    }

    pub fn add_context(self, ctx: &'static str) -> Option<Self> {
        match &self {
            TypeCheckError::OpCannotBeUsed { .. }
            | TypeCheckError::Unstructured { .. }
            | TypeCheckError::TypeMismatch { .. }
            | TypeCheckError::NonHomogeneousArray { .. }
            | TypeCheckError::PublicReturnType { .. }
            | TypeCheckError::ArityMisMatch { .. }
            | TypeCheckError::TypeCannotBeUsed { .. } => Some(TypeCheckError::Context {
                err: Box::new(self),
                ctx,
            }),
            // Cannot apply a context to multiple diagnostics
            TypeCheckError::MultipleErrors(_) => None,
            // Cannot append or overwrite a context
            TypeCheckError::Context { .. } => None,
        }
    }
}
