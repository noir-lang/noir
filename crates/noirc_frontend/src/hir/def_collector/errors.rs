use crate::hir::resolution::import::PathResolutionError;
use crate::Ident;

use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::FileDiagnostic;
use noirc_errors::Span;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DefCollectorErrorKind {
    #[error("duplicate function found in namespace")]
    DuplicateFunction { first_def: Ident, second_def: Ident },
    #[error("duplicate function found in namespace")]
    DuplicateModuleDecl { first_def: Ident, second_def: Ident },
    #[error("duplicate import")]
    DuplicateImport { first_def: Ident, second_def: Ident },
    #[error("duplicate global found in namespace")]
    DuplicateGlobal { first_def: Ident, second_def: Ident },
    #[error("unresolved import")]
    UnresolvedModuleDecl { mod_name: Ident },
    #[error("path resolution error")]
    PathResolutionError(PathResolutionError),
    #[error("Non-struct type used in impl")]
    NonStructTypeInImpl { span: Span },
    #[error("Cannot `impl` a type defined outside the current crate")]
    ForeignImpl { span: Span, type_name: String },
}

impl DefCollectorErrorKind {
    pub fn into_file_diagnostic(self, file: fm::FileId) -> FileDiagnostic {
        Diagnostic::from(self).in_file(file)
    }
}

impl From<DefCollectorErrorKind> for Diagnostic {
    fn from(error: DefCollectorErrorKind) -> Diagnostic {
        match error {
            DefCollectorErrorKind::DuplicateFunction { first_def, second_def } => {
                let first_span = first_def.0.span();
                let second_span = second_def.0.span();
                let func_name = &first_def.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("duplicate definitions of {func_name} function found"),
                    "first definition found here".to_string(),
                    first_span,
                );
                diag.add_secondary("second definition found here".to_string(), second_span);
                diag
            }
            DefCollectorErrorKind::DuplicateModuleDecl { first_def, second_def } => {
                let first_span = first_def.0.span();
                let second_span = second_def.0.span();
                let mod_name = &first_def.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("module {mod_name} has been declared twice"),
                    "first declaration found here".to_string(),
                    first_span,
                );
                diag.add_secondary("second declaration found here".to_string(), second_span);
                diag
            }
            DefCollectorErrorKind::DuplicateImport { first_def, second_def } => {
                let first_span = first_def.0.span();
                let second_span = second_def.0.span();
                let import_name = &first_def.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("the name `{import_name}` is defined multiple times"),
                    "first import found here".to_string(),
                    first_span,
                );
                diag.add_secondary("second import found here".to_string(), second_span);
                diag
            }
            DefCollectorErrorKind::DuplicateGlobal { first_def, second_def } => {
                let first_span = first_def.0.span();
                let second_span = second_def.0.span();
                let import_name = &first_def.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("the name `{import_name}` is defined multiple times"),
                    "first global declaration found here".to_string(),
                    first_span,
                );
                diag.add_secondary("second global declaration found here".to_string(), second_span);
                diag
            }
            DefCollectorErrorKind::UnresolvedModuleDecl { mod_name } => {
                let span = mod_name.0.span();
                let mod_name = &mod_name.0.contents;

                Diagnostic::simple_error(
                    format!("could not resolve module `{mod_name}` "),
                    String::new(),
                    span,
                )
            }
            DefCollectorErrorKind::PathResolutionError(error) => error.into(),
            DefCollectorErrorKind::NonStructTypeInImpl { span } => Diagnostic::simple_error(
                "Non-struct type used in impl".into(),
                "Only struct types may have implementation methods".into(),
                span,
            ),
            DefCollectorErrorKind::ForeignImpl { span, type_name } => Diagnostic::simple_error(
                "Cannot `impl` a type that was defined outside the current crate".into(),
                format!("{type_name} was defined outside the current crate"),
                span,
            ),
        }
    }
}
