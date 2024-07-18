use crate::ast::{Ident, Path, UnresolvedTypeData};
use crate::hir::resolution::import::PathResolutionError;

use noirc_errors::CustomDiagnostic as Diagnostic;
use noirc_errors::FileDiagnostic;
use noirc_errors::Span;
use thiserror::Error;

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DuplicateType {
    Function,
    Module,
    Global,
    TypeDefinition,
    Import,
    Trait,
    TraitImplementation,
    TraitAssociatedType,
    TraitAssociatedConst,
    TraitAssociatedFunction,
}

#[derive(Error, Debug, Clone)]
pub enum DefCollectorErrorKind {
    #[error("duplicate {typ} found in namespace")]
    Duplicate { typ: DuplicateType, first_def: Ident, second_def: Ident },
    #[error("unresolved import")]
    UnresolvedModuleDecl { mod_name: Ident, expected_path: String, alternative_path: String },
    #[error("overlapping imports")]
    OverlappingModuleDecls { mod_name: Ident, expected_path: String, alternative_path: String },
    #[error("path resolution error")]
    PathResolutionError(PathResolutionError),
    #[error("Non-struct type used in impl")]
    NonStructTypeInImpl { span: Span },
    #[error("Cannot implement trait on a mutable reference type")]
    MutableReferenceInTraitImpl { span: Span },
    #[error("Impl for type `{typ}` overlaps with existing impl")]
    OverlappingImpl { span: Span, typ: crate::Type },
    #[error("Previous impl defined here")]
    OverlappingImplNote { span: Span },
    #[error("Cannot `impl` a type defined outside the current crate")]
    ForeignImpl { span: Span, type_name: String },
    #[error("Mismatched number of generics in {location}")]
    MismatchGenericCount {
        actual_generic_count: usize,
        expected_generic_count: usize,
        location: &'static str,
        origin: String,
        span: Span,
    },
    #[error("Method is not defined in trait")]
    MethodNotInTrait { trait_name: Ident, impl_method: Ident },
    #[error("Only traits can be implemented")]
    NotATrait { not_a_trait_name: Path },
    #[error("Trait not found")]
    TraitNotFound { trait_path: Path },
    #[error("Missing Trait method implementation")]
    TraitMissingMethod { trait_name: Ident, method_name: Ident, trait_impl_span: Span },
    #[error("Module is already part of the crate")]
    ModuleAlreadyPartOfCrate { mod_name: Ident, span: Span },
    #[error("Module was originally declared here")]
    ModuleOriginallyDefined { mod_name: Ident, span: Span },
    #[error(
        "Either the type or the trait must be from the same crate as the trait implementation"
    )]
    TraitImplOrphaned { span: Span },
    #[error("macro error : {0:?}")]
    MacroError(MacroError),
    #[error("The only supported types of numeric generics are integers, fields, and booleans")]
    UnsupportedNumericGenericType { ident: Ident, typ: UnresolvedTypeData },
    #[error("impl has stricter requirements than trait")]
    ImplIsStricterThanTrait {
        constraint_typ: crate::Type,
        constraint_name: String,
        constraint_generics: Vec<crate::Type>,
        constraint_span: Span,
        trait_method_name: String,
        trait_method_span: Span,
    },
}

/// An error struct that macro processors can return.
#[derive(Debug, Clone)]
pub struct MacroError {
    pub primary_message: String,
    pub secondary_message: Option<String>,
    pub span: Option<Span>,
}

impl DefCollectorErrorKind {
    pub fn into_file_diagnostic(&self, file: fm::FileId) -> FileDiagnostic {
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
            DuplicateType::TraitAssociatedType => write!(f, "trait associated type"),
            DuplicateType::TraitAssociatedConst => write!(f, "trait associated constant"),
            DuplicateType::TraitAssociatedFunction => write!(f, "trait associated function"),
        }
    }
}

impl<'a> From<&'a DefCollectorErrorKind> for Diagnostic {
    fn from(error: &'a DefCollectorErrorKind) -> Diagnostic {
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
            DefCollectorErrorKind::UnresolvedModuleDecl { mod_name, expected_path, alternative_path } => {
                let span = mod_name.0.span();
                let mod_name = &mod_name.0.contents;

                Diagnostic::simple_error(
                    format!("No module `{mod_name}` at path `{expected_path}` or `{alternative_path}`"),
                    String::new(),
                    span,
                )
            }
            DefCollectorErrorKind::OverlappingModuleDecls { mod_name, expected_path, alternative_path } => {
                let span = mod_name.0.span();
                let mod_name = &mod_name.0.contents;

                Diagnostic::simple_error(
                    format!("Overlapping modules `{mod_name}` at  path `{expected_path}` and `{alternative_path}`"),
                    String::new(),
                    span,
                )
            }
            DefCollectorErrorKind::PathResolutionError(error) => error.into(),
            DefCollectorErrorKind::NonStructTypeInImpl { span } => Diagnostic::simple_error(
                "Non-struct type used in impl".into(),
                "Only struct types may have implementation methods".into(),
                *span,
            ),
            DefCollectorErrorKind::MutableReferenceInTraitImpl { span } => Diagnostic::simple_error(
                "Trait impls are not allowed on mutable reference types".into(),
                "Try using a struct type here instead".into(),
                *span,
            ),
            DefCollectorErrorKind::OverlappingImpl { span, typ } => {
                Diagnostic::simple_error(
                    format!("Impl for type `{typ}` overlaps with existing impl"),
                    "Overlapping impl".into(),
                    *span,
                )
            }
            DefCollectorErrorKind::OverlappingImplNote { span } => {
                // This should be a note or part of the previous error eventually.
                // This must be an error to appear next to the previous OverlappingImpl
                // error since we sort warnings first.
                Diagnostic::simple_error(
                    "Previous impl defined here".into(),
                    "Previous impl defined here".into(),
                    *span,
                )
            }
            DefCollectorErrorKind::ForeignImpl { span, type_name } => Diagnostic::simple_error(
                "Cannot `impl` a type that was defined outside the current crate".into(),
                format!("{type_name} was defined outside the current crate"),
                *span,
            ),
            DefCollectorErrorKind::TraitNotFound { trait_path } => Diagnostic::simple_error(
                format!("Trait {trait_path} not found"),
                "".to_string(),
                trait_path.span(),
            ),
            DefCollectorErrorKind::MismatchGenericCount {
                actual_generic_count,
                expected_generic_count,
                location,
                origin,
                span,
            } => {
                let plural = if *expected_generic_count == 1 { "" } else { "s" };
                let primary_message = format!(
                    "`{origin}` expects {expected_generic_count} generic{plural}, but {location} has {actual_generic_count}");
                Diagnostic::simple_error(primary_message, "".to_string(), *span)
            }
            DefCollectorErrorKind::MethodNotInTrait { trait_name, impl_method } => {
                let trait_name = &trait_name.0.contents;
                let impl_method_span = impl_method.span();
                let impl_method_name = &impl_method.0.contents;
                let primary_message = format!("Method with name `{impl_method_name}` is not part of trait `{trait_name}`, therefore it can't be implemented");
                Diagnostic::simple_error(primary_message, "".to_owned(), impl_method_span)
            }
            DefCollectorErrorKind::TraitMissingMethod {
                trait_name,
                method_name,
                trait_impl_span,
            } => {
                let trait_name = &trait_name.0.contents;
                let impl_method_name = &method_name.0.contents;
                let primary_message = format!(
                    "Method `{impl_method_name}` from trait `{trait_name}` is not implemented"
                );
                Diagnostic::simple_error(
                    primary_message,
                    format!("Please implement {impl_method_name} here"),
                    *trait_impl_span,
                )
            }
            DefCollectorErrorKind::NotATrait { not_a_trait_name } => {
                let span = not_a_trait_name.span();
                Diagnostic::simple_error(
                    format!("{not_a_trait_name} is not a trait, therefore it can't be implemented"),
                    String::new(),
                    span,
                )
            }
            DefCollectorErrorKind::ModuleAlreadyPartOfCrate { mod_name, span } => {
                let message = format!("Module '{mod_name}' is already part of the crate");
                let secondary = String::new();
                Diagnostic::simple_error(message, secondary, *span)
            }
            DefCollectorErrorKind::ModuleOriginallyDefined { mod_name, span } => {
                let message = format!("Note: {mod_name} was originally declared here");
                let secondary = String::new();
                Diagnostic::simple_error(message, secondary, *span)
            }
            DefCollectorErrorKind::TraitImplOrphaned { span } => Diagnostic::simple_error(
                "Orphaned trait implementation".into(),
                "Either the type or the trait must be from the same crate as the trait implementation".into(),
                *span,
            ),
            DefCollectorErrorKind::MacroError(macro_error) => {
                Diagnostic::simple_error(macro_error.primary_message.clone(), macro_error.secondary_message.clone().unwrap_or_default(), macro_error.span.unwrap_or_default())
            },
            DefCollectorErrorKind::UnsupportedNumericGenericType { ident, typ } => {
                let name = &ident.0.contents;

                Diagnostic::simple_error(
                    format!("{name} has a type of {typ}. The only supported types of numeric generics are integers and fields"),
                    "Unsupported numeric generic type".to_string(),
                    ident.0.span(),
                )
            }
            DefCollectorErrorKind::ImplIsStricterThanTrait { constraint_typ, constraint_name, constraint_generics, constraint_span, trait_method_name, trait_method_span } => {
                let mut constraint_name_with_generics = constraint_name.to_owned();
                if !constraint_generics.is_empty() {
                    constraint_name_with_generics.push('<');
                    for generic in constraint_generics.iter() {
                        constraint_name_with_generics.push_str(generic.to_string().as_str());
                    }
                    constraint_name_with_generics.push('>');
                }

                let mut diag = Diagnostic::simple_error(
                    "impl has stricter requirements than trait".to_string(),
                    format!("impl has extra requirement `{constraint_typ}: {constraint_name_with_generics}`"),
                    *constraint_span,
                );
                diag.add_secondary(format!("definition of `{trait_method_name}` from trait"), *trait_method_span);
                diag
            }
        }
    }
}
