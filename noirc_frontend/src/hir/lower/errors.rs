use noirc_errors::CustomDiagnostic as Diagnostic;
pub use noirc_errors::Span;
use thiserror::Error;

use super::node_interner::{IdentId, NodeInterner};

#[derive(Error, Debug, Clone)]
pub enum ResolverError {
    #[error("Duplicate definition")]
    DuplicateDefinition {
        first_ident: IdentId,
        second_ident: IdentId,
    },
    #[error("Unused variable")]
    UnusedVariable { ident_id: IdentId },
    #[error("Could not find variable in this scope")]
    VariableNotDeclared { name: String, span: Span },
    #[error("path is not an identifier")]
    PathIsNotIdent { span: Span },
    #[error("could not resolve path")]
    PathUnresolved { span: Span, name: String },
    #[error("could not resolve path")]
    Expected {
        span: Span,
        expected: String,
        got: String,
    },
}

impl ResolverError {
    /// Only user errors can be transformed into a Diagnostic
    /// ICEs will make the compiler panic, as they could affect the
    /// soundness of the generated program
    pub fn into_diagnostic(self, interner: &NodeInterner) -> Diagnostic {
        match self {
            ResolverError::DuplicateDefinition {
                first_ident,
                second_ident,
            } => {
                let first_span = interner.ident_span(&first_ident);
                let second_span = interner.ident_span(&second_ident);

                let name = interner.ident_name(&first_ident);

                let mut diag = Diagnostic::simple_error(
                    format!("duplicate definitions of {} found", name),
                    format!("first definition found here"),
                    first_span,
                );
                diag.add_secondary(format!("second definition found here"), second_span);
                diag
            }
            ResolverError::UnusedVariable { ident_id } => {
                let name = interner.ident_name(&ident_id);
                let span = interner.ident_span(&ident_id);

                let mut diag = Diagnostic::simple_error(
                    format!("unused variable {}", name),
                    format!("unused variable "),
                    span,
                );

                diag.add_note("A new variable usually means a constraint has been added and is being unused. \n For this reason, it is almost always a bug to declare a variable and not use it.".to_owned());
                diag
            }
            ResolverError::VariableNotDeclared { name, span } => Diagnostic::simple_error(
                format!("cannot find `{}` in this scope ", name),
                format!("not found in this scope"),
                span,
            ),
            ResolverError::PathIsNotIdent { span } => Diagnostic::simple_error(
                format!("cannot use path as an identifier"),
                String::new(),
                span,
            ),
            ResolverError::PathUnresolved { span, name } => Diagnostic::simple_error(
                format!("could not resolve path : {}", name),
                String::new(),
                span,
            ),
            ResolverError::Expected {
                span,
                expected,
                got,
            } => Diagnostic::simple_error(
                format!("expected {} got {}", expected, got),
                String::new(),
                span,
            ),
        }
    }
}
