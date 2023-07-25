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
    #[error("duplicate type definition found in namespace")]
    DuplicateTypeDef { first_def: Ident, second_def: Ident },
    #[error("duplicate trait definition found in namespace")]
    DuplicateTraitDef { first_def: Ident, second_def: Ident },
    #[error("duplicate module found in namespace")]
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
    #[error("Feature not implemented")]
    SimpleError { primary_message: String, secondary_message: String, span: Span },
}

impl DefCollectorErrorKind {
    pub fn into_file_diagnostic(self, file: fm::FileId) -> FileDiagnostic {
        Diagnostic::from(self).in_file(file)
    }
}

fn report_duplicate(
    primary_message: String,
    duplicate_type: &str,
    first_def: Ident,
    second_def: Ident,
) -> Diagnostic {
    let first_span = first_def.0.span();
    let second_span = second_def.0.span();
    let mut diag = Diagnostic::simple_error(
        primary_message,
        format!("first {} found here", duplicate_type),
        first_span,
    );
    diag.add_secondary(format!("second {} found here", duplicate_type), second_span);
    diag
}

impl From<DefCollectorErrorKind> for Diagnostic {
    fn from(error: DefCollectorErrorKind) -> Diagnostic {
        match error {
            DefCollectorErrorKind::DuplicateFunction { first_def, second_def } => {
                let primary_message =
                    format!("duplicate definitions of {} function found", &first_def.0.contents);
                report_duplicate(primary_message, "function definition", first_def, second_def)
            }
            DefCollectorErrorKind::DuplicateTypeDef { first_def, second_def } => {
                let primary_message =
                    format!("duplicate definitions of {} type found", &first_def.0.contents);
                report_duplicate(primary_message, "type definition", first_def, second_def)
            }
            DefCollectorErrorKind::DuplicateTraitDef { first_def, second_def } => {
                let primary_message =
                    format!("duplicate definitions of {} trait found", &first_def.0.contents);
                report_duplicate(primary_message, "trait definition", first_def, second_def)
            }
            DefCollectorErrorKind::DuplicateModuleDecl { first_def, second_def } => {
                let primary_message =
                    format!("module {} has been declared twice", &first_def.0.contents);
                report_duplicate(primary_message, "module declaration", first_def, second_def)
            }
            DefCollectorErrorKind::DuplicateImport { first_def, second_def } => {
                let primary_message =
                    format!("module `{}` is imported multiple times", &first_def.0.contents);
                report_duplicate(primary_message, "import", first_def, second_def)
            }
            DefCollectorErrorKind::DuplicateGlobal { first_def, second_def } => {
                let primary_message =
                    format!("the global `{}` is defined multiple times", &first_def.0.contents);
                report_duplicate(primary_message, "global declaration", first_def, second_def)
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
            DefCollectorErrorKind::SimpleError { primary_message, secondary_message, span } => {
                Diagnostic::simple_error(primary_message, secondary_message, span)
            }
        }
    }
}
