use crate::hir::resolution::import::PathResolutionError;
use crate::Ident;
use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::FileDiagnostic;
use noirc_errors::Span;
use thiserror::Error;

use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum DuplicateType {
    Function,
    Module,
    Global,
    TypeDefinition,
    Import,
    Trait,
    TraitImplementation,
}

#[derive(Error, Debug)]
pub enum DefCollectorErrorKind {
    #[error("duplicate {typ} found in namespace")]
    Duplicate { typ: DuplicateType, first_def: Ident, second_def: Ident },
    #[error("unresolved import")]
    UnresolvedModuleDecl { mod_name: Ident, expected_path: String },
    #[error("path resolution error")]
    PathResolutionError(PathResolutionError),
    #[error("Non-struct type used in impl")]
    NonStructTypeInImpl { span: Span },
    #[error("Non-struct type used in trait impl")]
    NonStructTraitImpl { trait_ident: Ident, span: Span },
    #[error("Cannot `impl` a type defined outside the current crate")]
    ForeignImpl { span: Span, type_name: String },
    #[error("Mismatch number of parameters in of trait implementation")]
    MismatchTraitImplementationNumParameters {
        actual_num_parameters: usize,
        expected_num_parameters: usize,
        trait_name: String,
        method_name: String,
        span: Span,
    },
    #[error("Method is not defined in trait")]
    MethodNotInTrait { trait_name: Ident, impl_method: Ident },
    #[error("Only traits can be implemented")]
    NotATrait { not_a_trait_name: Ident },
    #[error("Trait not found")]
    TraitNotFound { trait_ident: Ident },
    #[error("Missing Trait method implementation")]
    TraitMissingMethod { trait_name: Ident, method_name: Ident, trait_impl_span: Span },
    #[error("Module is already part of the crate")]
    ModuleAlreadyPartOfCrate { mod_name: Ident, span: Span },
    #[error("Module was originally declared here")]
    ModuleOrignallyDefined { mod_name: Ident, span: Span },
    #[cfg(feature = "aztec")]
    #[error("Aztec dependency not found. Please add aztec as a dependency in your Cargo.toml")]
    AztecNotFound {},
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
            DuplicateType::TraitImplementation => write!(f, "trait implementation"),
            DuplicateType::Import => write!(f, "import"),
        }
    }
}

impl From<DefCollectorErrorKind> for Diagnostic {
    fn from(error: DefCollectorErrorKind) -> Diagnostic {
        match error {
            DefCollectorErrorKind::Duplicate { typ, first_def, second_def } => {
                let primary_message = format!(
                    "Duplicate definitions of {} with name {} found",
                    &typ, &first_def.0.contents
                );
                {
                    let first_span = first_def.0.span();
                    let second_span = second_def.0.span();
                    let mut diag = Diagnostic::simple_error(
                        primary_message,
                        format!("First {} found here", &typ),
                        first_span,
                    );
                    diag.add_secondary(format!("Second {} found here", &typ), second_span);
                    diag
                }
            }
            DefCollectorErrorKind::UnresolvedModuleDecl { mod_name, expected_path } => {
                let span = mod_name.0.span();
                let mod_name = &mod_name.0.contents;

                Diagnostic::simple_error(
                    format!("No module `{mod_name}` at path `{expected_path}`"),
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
            DefCollectorErrorKind::NonStructTraitImpl { trait_ident, span } => {
                Diagnostic::simple_error(
                    format!("Only struct types may implement trait `{trait_ident}`"),
                    "Only struct types may implement traits".into(),
                    span,
                )
            }
            DefCollectorErrorKind::ForeignImpl { span, type_name } => Diagnostic::simple_error(
                "Cannot `impl` a type that was defined outside the current crate".into(),
                format!("{type_name} was defined outside the current crate"),
                span,
            ),
            DefCollectorErrorKind::TraitNotFound { trait_ident } => Diagnostic::simple_error(
                format!("Trait {trait_ident} not found"),
                "".to_string(),
                trait_ident.span(),
            ),
            DefCollectorErrorKind::MismatchTraitImplementationNumParameters {
                expected_num_parameters,
                actual_num_parameters,
                trait_name,
                method_name,
                span,
            } => {
                let primary_message = format!(
                    "Method `{method_name}` of trait `{trait_name}` needs {expected_num_parameters} parameters, but has {actual_num_parameters}");
                Diagnostic::simple_error(primary_message, "".to_string(), span)
            }
            DefCollectorErrorKind::MethodNotInTrait { trait_name, impl_method } => {
                let trait_name = trait_name.0.contents;
                let impl_method_span = impl_method.span();
                let impl_method_name = impl_method.0.contents;
                let primary_message = format!("Method with name `{impl_method_name}` is not part of trait `{trait_name}`, therefore it can't be implemented");
                Diagnostic::simple_error(primary_message, "".to_owned(), impl_method_span)
            }
            DefCollectorErrorKind::TraitMissingMethod {
                trait_name,
                method_name,
                trait_impl_span,
            } => {
                let trait_name = trait_name.0.contents;
                let impl_method_name = method_name.0.contents;
                let primary_message = format!(
                    "Method `{impl_method_name}` from trait `{trait_name}` is not implemented"
                );
                Diagnostic::simple_error(
                    primary_message,
                    format!("Please implement {impl_method_name} here"),
                    trait_impl_span,
                )
            }
            DefCollectorErrorKind::NotATrait { not_a_trait_name } => {
                let span = not_a_trait_name.0.span();
                let name = &not_a_trait_name.0.contents;
                Diagnostic::simple_error(
                    format!("{name} is not a trait, therefore it can't be implemented"),
                    String::new(),
                    span,
                )
            }
            DefCollectorErrorKind::ModuleAlreadyPartOfCrate { mod_name, span } => {
                let message = format!("Module '{mod_name}' is already part of the crate");
                let secondary = String::new();
                Diagnostic::simple_error(message, secondary, span)
            }
            DefCollectorErrorKind::ModuleOrignallyDefined { mod_name, span } => {
                let message = format!("Note: {mod_name} was originally declared here");
                let secondary = String::new();
                Diagnostic::simple_error(message, secondary, span)
            }
            #[cfg(feature = "aztec")]
            DefCollectorErrorKind::AztecNotFound {} => Diagnostic::from_message(
                "Aztec dependency not found. Please add aztec as a dependency in your Cargo.toml",
            ),
        }
    }
}
