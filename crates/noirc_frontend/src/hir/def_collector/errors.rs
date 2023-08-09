use crate::hir::resolution::import::PathResolutionError;
use crate::Ident;

use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::FileDiagnostic;
use noirc_errors::Span;
use thiserror::Error;

use std::fmt;

#[derive(Debug)]
pub enum DuplicateType {
    Function,
    Module,
    Global,
    TypeDefinition,
    Import,
    Trait,
}

#[derive(Error, Debug)]
pub enum DefCollectorErrorKind {
    #[error("duplicate {typ} found in namespace")]
    Duplicate { typ: DuplicateType, first_def: Ident, second_def: Ident },
    #[error("unresolved import")]
    UnresolvedModuleDecl { mod_name: Ident },
    #[error("path resolution error")]
    PathResolutionError(PathResolutionError),
    #[error("Non-struct type used in impl")]
    NonStructTypeInImpl { span: Span },
    #[error("Cannot `impl` a type defined outside the current crate")]
    ForeignImpl { span: Span, type_name: String },
    #[error("Mismatch signature of trait")]
    MismatchTraitSignature { primary_message: String, secondary_message: String, span: Span },
    #[error("Method is not defined in trait")]
    MethodNotInTrait {
        trait_name: String,
        trait_span: Span,
        impl_method_name: String,
        impl_method_span: Span,
    },
    #[error("Not a trait")]
    NotATrait { primary_message: String, secondary_message: String, span: Span },
    #[error("Trait {trait_name} not found")]
    TraitNotFound { trait_name: String, span: Span },
}

impl DefCollectorErrorKind {
    pub fn into_file_diagnostic(self, file: fm::FileId) -> FileDiagnostic {
        Diagnostic::from(self).in_file(file)
    }
}

impl fmt::Display for DuplicateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DuplicateType::Function => write!(f, "function"),
            DuplicateType::Module => write!(f, "module"),
            DuplicateType::Global => write!(f, "global"),
            DuplicateType::TypeDefinition => write!(f, "type definition"),
            DuplicateType::Trait => write!(f, "trait definition"),
            DuplicateType::Import => write!(f, "import"),
        }
    }
}

impl From<DefCollectorErrorKind> for Diagnostic {
    fn from(error: DefCollectorErrorKind) -> Diagnostic {
        match error {
            DefCollectorErrorKind::Duplicate { typ, first_def, second_def } => {
                let primary_message = format!(
                    "duplicate definitions of {} with name {} found",
                    &typ, &first_def.0.contents
                );
                {
                    let first_span = first_def.0.span();
                    let second_span = second_def.0.span();
                    let mut diag = Diagnostic::simple_error(
                        primary_message,
                        format!("first {} found here", &typ),
                        first_span,
                    );
                    diag.add_secondary(format!("second {} found here", &typ), second_span);
                    diag
                }
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
            DefCollectorErrorKind::TraitNotFound { trait_name, span } => Diagnostic::simple_error(
                format!("Trait {} not found", trait_name),
                "".to_string(),
                span,
            ),
            DefCollectorErrorKind::MismatchTraitSignature {
                primary_message,
                secondary_message,
                span,
            } => Diagnostic::simple_error(primary_message, secondary_message, span),
            DefCollectorErrorKind::MethodNotInTrait {
                trait_name,
                trait_span: _,
                impl_method_name,
                impl_method_span,
            } => {
                let primary_message = format!("method with name {impl_method_name} is not part of trait {trait_name}, therefore it can't be implemented");
                Diagnostic::simple_error(primary_message, "".to_owned(), impl_method_span)
            }
            DefCollectorErrorKind::NotATrait { primary_message, secondary_message, span } => {
                Diagnostic::simple_error(primary_message, secondary_message, span)
            }
        }
    }
}
