use crate::{hir::resolution::import::ImportDirective, Ident};

use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::DiagnosableError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DefCollectorErrorKind {
    #[error("duplicate function found in namespace")]
    DuplicateFunction { first_def: Ident, second_def: Ident },
    #[error("duplicate function found in namespace")]
    DuplicateModuleDecl { first_def: Ident, second_def: Ident },
    #[error("duplicate import")]
    DuplicateImport { first_def: Ident, second_def: Ident },
    #[error("unresolved import")]
    UnresolvedModuleDecl { mod_name: Ident },
    #[error("unresolved import")]
    UnresolvedImport { import: ImportDirective },
}

impl DiagnosableError for DefCollectorErrorKind {
    fn to_diagnostic(&self) -> Diagnostic {
        match self {
            DefCollectorErrorKind::DuplicateFunction {
                first_def,
                second_def,
            } => {
                let first_span = first_def.0.span();
                let second_span = second_def.0.span();
                let func_name = &first_def.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("duplicate definitions of {} function found", func_name),
                    format!("first definition found here"),
                    first_span,
                );
                diag.add_secondary(format!("second definition found here"), second_span);
                diag
            }
            DefCollectorErrorKind::DuplicateModuleDecl {
                first_def,
                second_def,
            } => {
                let first_span = first_def.0.span();
                let second_span = second_def.0.span();
                let mod_name = &first_def.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("module {} has been declared twice", mod_name),
                    format!("first declaration found here"),
                    first_span,
                );
                diag.add_secondary(format!("second declaration found here"), second_span);
                diag
            }
            DefCollectorErrorKind::DuplicateImport {
                first_def,
                second_def,
            } => {
                let first_span = first_def.0.span();
                let second_span = second_def.0.span();
                let import_name = &first_def.0.contents;

                let mut diag = Diagnostic::simple_error(
                    format!("the name `{}` is defined multiple times", import_name),
                    format!("first import found here"),
                    first_span,
                );
                diag.add_secondary(format!("second import found here"), second_span);
                diag
            }
            DefCollectorErrorKind::UnresolvedModuleDecl { mod_name } => {
                let span = mod_name.0.span();
                let mod_name = &mod_name.0.contents;

                let diag = Diagnostic::simple_error(
                    format!("could not resolve module `{}` ", mod_name),
                    format!(""),
                    span,
                );
                diag
            }
            DefCollectorErrorKind::UnresolvedImport { import } => {
                let mut span = import.path.span();
                if let Some(alias) = &import.alias {
                    span = span.merge(alias.0.span())
                }

                let diag = Diagnostic::simple_error(
                    format!("could not resolve import {}", &import.path.as_string()),
                    format!(""),
                    span,
                );
                diag
            }
        }
    }
}
