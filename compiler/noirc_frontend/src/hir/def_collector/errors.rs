use crate::hir::resolution::import::PathResolutionError;
use crate::Ident;
use crate::UnresolvedType;

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
    TraitImplementation,
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
    #[error("Non-struct type used in trait impl")]
    NonStructTraitImpl { trait_ident: Ident, span: Span },
    #[error("Cannot `impl` a type defined outside the current crate")]
    ForeignImpl { span: Span, type_name: String },
    #[error("Mismatch signature of trait")]
    MismatchTraitImlementationParameter {
        trait_name: String,
        impl_method: String,
        parameter: Ident,
        expected_type: UnresolvedType,
    },
    #[error("Mismatch return type of trait implementation")]
    MismatchTraitImplementationReturnType { trait_name: String, impl_ident: Ident },
    #[error("Mismatch number of parameters in of trait implementation")]
    MismatchTraitImplementationNumParameters {
        actual_num_parameters: usize,
        expected_num_parameters: usize,
        trait_name: String,
        impl_ident: Ident,
    },
    #[error("Method is not defined in trait")]
    MethodNotInTrait { trait_name: Ident, impl_method: Ident },
    #[error("Only traits can be implemented")]
    NotATrait { not_a_trait_name: Ident },
    #[error("Trait not found")]
    TraitNotFound { trait_ident: Ident },
    #[error("Missing Trait method implementation")]
    TraitMissedMethodImplementation { trait_name: Ident, method_name: Ident, trait_impl_span: Span },
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
            DefCollectorErrorKind::MismatchTraitImplementationReturnType {
                trait_name,
                impl_ident,
            } => {
                let span = impl_ident.span();
                let method_name = impl_ident.0.contents;
                Diagnostic::simple_error(
                    format!("Mismatch return type of method with name {method_name} that implements trait {trait_name}"),
                    "".to_string(),
                    span,
                )
            }
            DefCollectorErrorKind::MismatchTraitImplementationNumParameters {
                expected_num_parameters,
                actual_num_parameters,
                trait_name,
                impl_ident,
            } => {
                let method_name = impl_ident.0.contents.clone();
                let primary_message = format!(
                    "Mismatch - expected {expected_num_parameters} arguments, but got {actual_num_parameters} of trait `{trait_name}` implementation `{method_name}`");
                Diagnostic::simple_error(primary_message, "".to_string(), impl_ident.span())
            }
            DefCollectorErrorKind::MismatchTraitImlementationParameter {
                trait_name,
                impl_method,
                parameter,
                expected_type,
            } => {
                let primary_message = format!(
                    "Mismatch signature of method {impl_method} that implements trait {trait_name}"
                );
                let secondary_message =
                    format!("`{}: {}` expected", parameter.0.contents, expected_type,);
                let span = parameter.span();
                Diagnostic::simple_error(primary_message, secondary_message, span)
            }
            DefCollectorErrorKind::MethodNotInTrait { trait_name, impl_method } => {
                let trait_name = trait_name.0.contents;
                let impl_method_span = impl_method.span();
                let impl_method_name = impl_method.0.contents;
                let primary_message = format!("method with name {impl_method_name} is not part of trait {trait_name}, therefore it can't be implemented");
                Diagnostic::simple_error(primary_message, "".to_owned(), impl_method_span)
            }
            DefCollectorErrorKind::TraitMissedMethodImplementation {
                trait_name,
                method_name,
                trait_impl_span,
            } => {
                let trait_name = trait_name.0.contents;
                let impl_method_name = method_name.0.contents;
                let primary_message = format!(
                    "method `{impl_method_name}` from trait `{trait_name}` is not implemented"
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
        }
    }
}
