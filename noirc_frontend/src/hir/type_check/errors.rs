use thiserror::Error;
use noirc_errors::CustomDiagnostic as Diagnostic;
pub use noirc_errors::Span;

use crate::{Type, hir::lower::{HirBinaryOp, node_interner::NodeInterner}};

#[derive(Error, Debug, Clone)]
pub enum TypeCheckError {
    #[error("operator {op:?} cannot be used in a {place:?}")]
    OpCannotBeUsed {op : HirBinaryOp, place: &'static str, span : Span},
    #[error("type {typ:?} cannot be used in a {place:?}")]
    TypeCannotBeUsed {typ : Type, place: &'static str, span : Span},
    #[error("annotated type {ann_typ:?} is not the same as {expr_typ:?}")]
    TypeAnnotationMismatch {ann_typ : Type, ann_span : Span, expr_typ : Type, expr_span : Span},
    // Usually the type checker will return after the first encountered errors
    // Due to the fact that types depend on each other.
    // This is not the case in a CallExpression however, or more generally a list of expressions
    #[error("multiple errors when type checking list of expressions")]
    MultipleErrors(Vec<TypeCheckError>),
    #[error("error with additional context")]
    Context{err : Box<TypeCheckError>, ctx : &'static str}

}

impl TypeCheckError {
        pub fn into_diagnostics(self, interner : &NodeInterner) -> Vec<Diagnostic> {
            match self {
                TypeCheckError::TypeCannotBeUsed{typ, place, span} => {
                    vec![Diagnostic::simple_error(format!("the type {} cannot be used in a {}", &typ, place), format!(""), span)]
                },
                TypeCheckError::MultipleErrors(errors) => {
                    errors.into_iter().map(|err| err.into_diagnostics(interner)).flatten().collect()
                }
                TypeCheckError::Context { err, ctx } => {
                    let mut diags = err.into_diagnostics(interner);
                    
                    // Cannot add a single context to multiple errors
                    assert!(diags.len() == 1);
                    
                    let mut diag = diags.pop().unwrap();
                    diag.add_note(ctx.to_owned());
                    vec![diag]
                }
                TypeCheckError::OpCannotBeUsed { op, place, span } => {
                    vec![Diagnostic::simple_error(format!("the operator {:?} cannot be used in a {}", op, place), format!(""), span)]
                }
                TypeCheckError::TypeAnnotationMismatch { ann_typ, ann_span, expr_typ, expr_span } => {

                    let mut diag = Diagnostic::from_message(&format!("expected type {}, found type {}", ann_typ, expr_typ));
                    diag.add_secondary("type annotated here".to_owned(), ann_span);
                    diag.add_secondary("which does not match the expression type here".to_owned(), expr_span);
                    vec![diag]
                }
            }

         }

         pub fn add_context(self, ctx : &'static str) -> Option<Self> {
            match &self {
                TypeCheckError::OpCannotBeUsed { .. } |
                TypeCheckError::TypeAnnotationMismatch { .. } |
                TypeCheckError::TypeCannotBeUsed { .. } => Some(TypeCheckError::Context{err:Box::new(self), ctx}),
                // Cannot apply a context to multiple diagnostics
                TypeCheckError::MultipleErrors(_) => None,
                // Cannot append or overwrite a context
                TypeCheckError::Context { .. } => None
            }
         }
}