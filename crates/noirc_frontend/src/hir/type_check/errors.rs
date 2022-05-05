use noirc_errors::CustomDiagnostic as Diagnostic;
pub use noirc_errors::Span;
use thiserror::Error;

use crate::hir_def::expr::HirBinaryOp;
use crate::hir_def::types::Type;
use crate::node_interner::NodeInterner;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TypeCheckError {
    #[error("operator {op:?} cannot be used in a {place:?}")]
    OpCannotBeUsed { op: HirBinaryOp, place: &'static str, span: Span },
    #[error("type {typ:?} cannot be used in a {place:?}")]
    TypeCannotBeUsed { typ: Type, place: &'static str, span: Span },
    #[error("expected type {expected_typ:?} is not the same as {expr_typ:?}")]
    TypeMismatch { expected_typ: String, expr_typ: String, expr_span: Span },
    #[error("expected {expected:?} found {found:?}")]
    ArityMisMatch { expected: u16, found: u16, span: Span },
    #[error("return type in a function cannot be public")]
    PublicReturnType { typ: Type, span: Span },
    // XXX: unstructured errors are not ideal for testing.
    // They will be removed in a later iteration
    #[error("unstructured msg: {msg:?}")]
    Unstructured { msg: String, span: Span },
    #[error("error with additional context")]
    Context { err: Box<TypeCheckError>, ctx: &'static str },
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
    pub fn into_diagnostic(self, interner: &NodeInterner) -> Diagnostic {
        match self {
            TypeCheckError::TypeCannotBeUsed { typ, place, span } => Diagnostic::simple_error(
                format!("the type {} cannot be used in a {}", &typ, place),
                String::new(),
                span,
            ),
            TypeCheckError::Context { err, ctx } => {
                let mut diag = err.into_diagnostic(interner);
                diag.add_note(ctx.to_owned());
                diag
            }
            TypeCheckError::OpCannotBeUsed { op, place, span } => Diagnostic::simple_error(
                format!("the operator {:?} cannot be used in a {}", op, place),
                String::new(),
                span,
            ),
            TypeCheckError::TypeMismatch { expected_typ, expr_typ, expr_span } => {
                Diagnostic::simple_error(
                    format!("expected type {}, found type {}", expected_typ, expr_typ),
                    String::new(),
                    expr_span,
                )
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
                diag
            }
            TypeCheckError::ArityMisMatch { expected, found, span } => {
                let plural = if expected == 1 { "" } else { "s" };
                let msg = format!("expected {} argument{}, but found {}", expected, plural, found);
                Diagnostic::simple_error(msg, String::new(), span)
            }
            TypeCheckError::Unstructured { msg, span } => {
                Diagnostic::simple_error(msg, String::new(), span)
            }
            TypeCheckError::PublicReturnType { typ, span } => Diagnostic::simple_error(
                "functions cannot declare a public return type".to_string(),
                format!("return type is {}", typ),
                span,
            ),
        }
    }

    pub fn add_context(self, ctx: &'static str) -> Self {
        TypeCheckError::Context { err: Box::new(self), ctx }
    }
}
