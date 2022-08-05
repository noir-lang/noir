use noirc_errors::CustomDiagnostic as Diagnostic;
pub use noirc_errors::Span;
use thiserror::Error;

use crate::Ident;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ResolverError {
    #[error("Duplicate definition")]
    DuplicateDefinition { name: String, first_span: Span, second_span: Span },
    #[error("Unused variable")]
    UnusedVariable { ident: Ident },
    #[error("Could not find variable in this scope")]
    VariableNotDeclared { name: String, span: Span },
    #[error("path is not an identifier")]
    PathIsNotIdent { span: Span },
    #[error("could not resolve path")]
    PathUnresolved { span: Span, name: String, segment: Ident },
    #[error("Expected")]
    Expected { span: Span, expected: String, got: String },
    #[error("Duplicate field in constructor")]
    DuplicateField { field: Ident },
    #[error("No such field in struct")]
    NoSuchField { field: Ident, struct_definition: Ident },
    #[error("Missing fields from struct")]
    MissingFields { span: Span, missing_fields: Vec<String>, struct_definition: Ident },
    #[error("Unneeded 'mut', pattern is already marked as mutable")]
    UnnecessaryMut { first_mut: Span, second_mut: Span },
}

impl ResolverError {
    /// Only user errors can be transformed into a Diagnostic
    /// ICEs will make the compiler panic, as they could affect the
    /// soundness of the generated program
    pub fn into_diagnostic(self) -> Diagnostic {
        match self {
            ResolverError::DuplicateDefinition { name, first_span, second_span } => {
                let mut diag = Diagnostic::simple_error(
                    format!("duplicate definitions of {} found", name),
                    "first definition found here".to_string(),
                    first_span,
                );
                diag.add_secondary("second definition found here".to_string(), second_span);
                diag
            }
            ResolverError::UnusedVariable { ident } => {
                let name = &ident.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("unused variable {}", name),
                    "unused variable ".to_string(),
                    ident.0.span(),
                );

                diag.add_note("A new variable usually means a constraint has been added and is being unused. \n For this reason, it is almost always a bug to declare a variable and not use it.".to_owned());
                diag
            }
            ResolverError::VariableNotDeclared { name, span } => Diagnostic::simple_error(
                format!("cannot find `{}` in this scope ", name),
                "not found in this scope".to_string(),
                span,
            ),
            ResolverError::PathIsNotIdent { span } => Diagnostic::simple_error(
                "cannot use path as an identifier".to_string(),
                String::new(),
                span,
            ),
            ResolverError::PathUnresolved { span, name, segment } => {
                let mut diag = Diagnostic::simple_error(
                    format!("could not resolve path '{}'", name),
                    String::new(),
                    span,
                );
                // XXX: When the secondary and primary labels have spans that
                // overlap, you cannot differentiate between them.
                // This error is an example of this.
                diag.add_secondary(
                    format!("could not resolve `{}` in path", &segment.0.contents),
                    segment.0.span(),
                );

                diag
            }
            ResolverError::Expected { span, expected, got } => Diagnostic::simple_error(
                format!("expected {} got {}", expected, got),
                String::new(),
                span,
            ),
            ResolverError::DuplicateField { field } => Diagnostic::simple_error(
                format!("duplicate field {}", field),
                String::new(),
                field.span(),
            ),
            ResolverError::NoSuchField { field, struct_definition } => {
                let mut error = Diagnostic::simple_error(
                    format!("no such field {} defined in struct {}", field, struct_definition),
                    String::new(),
                    field.span(),
                );

                error.add_secondary(
                    format!("{} defined here with no {} field", struct_definition, field),
                    struct_definition.span(),
                );
                error
            }
            ResolverError::MissingFields { span, missing_fields, struct_definition } => {
                let plural = if missing_fields.len() != 1 { "s" } else { "" };
                let missing_fields = missing_fields.join(", ");

                let mut error = Diagnostic::simple_error(
                    format!("missing field{}: {}", plural, missing_fields),
                    String::new(),
                    span,
                );

                error.add_secondary(
                    format!("{} defined here", struct_definition),
                    struct_definition.span(),
                );
                error
            }
            ResolverError::UnnecessaryMut { first_mut, second_mut } => {
                let mut error = Diagnostic::simple_error(
                    "'mut' here is not necessary".to_owned(),
                    "".to_owned(),
                    second_mut,
                );
                error.add_secondary(
                    "Pattern was already made mutable from this 'mut'".to_owned(),
                    first_mut,
                );
                error
            }
        }
    }
}
