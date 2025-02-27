use crate::ast::{Ident, ItemVisibility, Path, UnsupportedNumericGenericType};
use crate::hir::resolution::import::PathResolutionError;
use crate::hir::type_check::generics::TraitGenerics;

use noirc_errors::{CustomDiagnostic as Diagnostic, Location};
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
    StructField,
    EnumVariant,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DefCollectorErrorKind {
    #[error("Duplicate {typ}")]
    Duplicate { typ: DuplicateType, first_def: Ident, second_def: Ident },
    #[error("Unresolved import")]
    UnresolvedModuleDecl { mod_name: Ident, expected_path: String, alternative_path: String },
    #[error("Overlapping imports")]
    OverlappingModuleDecls { mod_name: Ident, expected_path: String, alternative_path: String },
    #[error("Path resolution error")]
    PathResolutionError(PathResolutionError),
    #[error("Cannot re-export {item_name} because it has less visibility than this use statement")]
    CannotReexportItemWithLessVisibility { item_name: Ident, desired_visibility: ItemVisibility },
    #[error("Non-struct type used in impl")]
    NonStructTypeInImpl { location: Location },
    #[error("Cannot implement trait on a mutable reference type")]
    MutableReferenceInTraitImpl { location: Location },
    #[error("Impl for type `{typ}` overlaps with existing impl")]
    OverlappingImpl { typ: crate::Type, location: Location, prev_location: Location },
    #[error("Cannot `impl` a type defined outside the current crate")]
    ForeignImpl { location: Location, type_name: String },
    #[error("Method is not defined in trait")]
    MethodNotInTrait { trait_name: Ident, impl_method: Ident },
    #[error("Only traits can be implemented")]
    NotATrait { not_a_trait_name: Path },
    #[error("Trait not found")]
    TraitNotFound { trait_path: Path },
    #[error("Missing Trait method implementation")]
    TraitMissingMethod { trait_name: Ident, method_name: Ident, trait_impl_location: Location },
    #[error("Module is already part of the crate")]
    ModuleAlreadyPartOfCrate { mod_name: Ident, location: Location },
    #[error("Module was originally declared here")]
    ModuleOriginallyDefined { mod_name: Ident, location: Location },
    #[error("Either the type or the trait must be from the same crate as the trait implementation")]
    TraitImplOrphaned { location: Location },
    #[error("impl has stricter requirements than trait")]
    ImplIsStricterThanTrait {
        constraint_typ: crate::Type,
        constraint_name: String,
        constraint_generics: TraitGenerics,
        constraint_location: Location,
        trait_method_name: String,
        trait_method_location: Location,
    },
    #[error("{0}")]
    UnsupportedNumericGenericType(#[from] UnsupportedNumericGenericType),
    #[error("The `#[test]` attribute may only be used on a non-associated function")]
    TestOnAssociatedFunction { location: Location },
    #[error("The `#[export]` attribute may only be used on a non-associated function")]
    ExportOnAssociatedFunction { location: Location },
}

impl DefCollectorErrorKind {
    pub fn location(&self) -> Location {
        match self {
            DefCollectorErrorKind::Duplicate { first_def: ident, .. }
            | DefCollectorErrorKind::UnresolvedModuleDecl { mod_name: ident, .. }
            | DefCollectorErrorKind::CannotReexportItemWithLessVisibility {
                item_name: ident,
                ..
            }
            | DefCollectorErrorKind::MethodNotInTrait { impl_method: ident, .. }
            | DefCollectorErrorKind::OverlappingModuleDecls { mod_name: ident, .. } => {
                ident.location()
            }
            DefCollectorErrorKind::PathResolutionError(path_resolution_error) => {
                path_resolution_error.location()
            }
            DefCollectorErrorKind::ImplIsStricterThanTrait {
                trait_method_location: location,
                ..
            }
            | DefCollectorErrorKind::TestOnAssociatedFunction { location }
            | DefCollectorErrorKind::ExportOnAssociatedFunction { location }
            | DefCollectorErrorKind::NonStructTypeInImpl { location }
            | DefCollectorErrorKind::MutableReferenceInTraitImpl { location }
            | DefCollectorErrorKind::OverlappingImpl { location, .. }
            | DefCollectorErrorKind::ModuleAlreadyPartOfCrate { location, .. }
            | DefCollectorErrorKind::ModuleOriginallyDefined { location, .. }
            | DefCollectorErrorKind::TraitImplOrphaned { location }
            | DefCollectorErrorKind::TraitMissingMethod { trait_impl_location: location, .. }
            | DefCollectorErrorKind::ForeignImpl { location, .. } => *location,
            DefCollectorErrorKind::NotATrait { not_a_trait_name: path }
            | DefCollectorErrorKind::TraitNotFound { trait_path: path } => path.location,
            DefCollectorErrorKind::UnsupportedNumericGenericType(
                unsupported_numeric_generic_type,
            ) => unsupported_numeric_generic_type.ident.location(),
        }
    }
}

impl<'a> From<&'a UnsupportedNumericGenericType> for Diagnostic {
    fn from(error: &'a UnsupportedNumericGenericType) -> Diagnostic {
        let name = &error.ident.0.contents;
        let typ = &error.typ;

        Diagnostic::simple_error(
            format!(
                "{name} has a type of {typ}. The only supported numeric generic types are `u1`, `u8`, `u16`, and `u32`."
            ),
            "Unsupported numeric generic type".to_string(),
            error.ident.0.location(),
        )
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
            DuplicateType::StructField => write!(f, "struct field"),
            DuplicateType::EnumVariant => write!(f, "enum variant"),
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
                    let first_location = first_def.0.location();
                    let second_location = second_def.0.location();
                    let mut diag = Diagnostic::simple_error(
                        primary_message,
                        format!("First {} found here", &typ),
                        first_location,
                    );
                    diag.add_secondary(format!("Second {} found here", &typ), second_location);
                    diag
                }
            }
            DefCollectorErrorKind::UnresolvedModuleDecl { mod_name, expected_path, alternative_path } => {
                let location = mod_name.0.location();
                let mod_name = &mod_name.0.contents;

                Diagnostic::simple_error(
                    format!("No module `{mod_name}` at path `{expected_path}` or `{alternative_path}`"),
                    String::new(),
                    location,
                )
            }
            DefCollectorErrorKind::OverlappingModuleDecls { mod_name, expected_path, alternative_path } => {
                let location = mod_name.0.location();
                let mod_name = &mod_name.0.contents;

                Diagnostic::simple_error(
                    format!("Overlapping modules `{mod_name}` at  path `{expected_path}` and `{alternative_path}`"),
                    String::new(),
                    location,
                )
            }
            DefCollectorErrorKind::PathResolutionError(error) => error.into(),
            DefCollectorErrorKind::CannotReexportItemWithLessVisibility{item_name, desired_visibility} => {
                Diagnostic::simple_error(
                    format!("cannot re-export {item_name} because it has less visibility than this use statement"),
                    format!("consider marking {item_name} as {desired_visibility}"),
                    item_name.location())
            }
            DefCollectorErrorKind::NonStructTypeInImpl { location } => Diagnostic::simple_error(
                "Non-struct type used in impl".into(),
                "Only struct types may have implementation methods".into(),
                *location,
            ),
            DefCollectorErrorKind::MutableReferenceInTraitImpl { location } => Diagnostic::simple_error(
                "Trait impls are not allowed on mutable reference types".into(),
                "Try using a struct type here instead".into(),
                *location,
            ),
            DefCollectorErrorKind::OverlappingImpl { location, typ, prev_location } => {
                let mut diagnostic = Diagnostic::simple_error(
                    format!("Impl for type `{typ}` overlaps with existing impl"),
                    "Overlapping impl".into(),
                    *location,
                );
                diagnostic.add_secondary("Previous impl defined here".into(), *prev_location);
                diagnostic
            }
            DefCollectorErrorKind::ForeignImpl { location, type_name } => Diagnostic::simple_error(
                "Cannot `impl` a type that was defined outside the current crate".into(),
                format!("{type_name} was defined outside the current crate"),
                *location,
            ),
            DefCollectorErrorKind::TraitNotFound { trait_path } => Diagnostic::simple_error(
                format!("Trait {trait_path} not found"),
                "".to_string(),
                trait_path.location,
            ),
            DefCollectorErrorKind::MethodNotInTrait { trait_name, impl_method } => {
                let trait_name = &trait_name.0.contents;
                let impl_method_location = impl_method.location();
                let impl_method_name = &impl_method.0.contents;
                let primary_message = format!("Method with name `{impl_method_name}` is not part of trait `{trait_name}`, therefore it can't be implemented");
                Diagnostic::simple_error(primary_message, "".to_owned(), impl_method_location)
            }
            DefCollectorErrorKind::TraitMissingMethod {
                trait_name,
                method_name,
                trait_impl_location,
            } => {
                let trait_name = &trait_name.0.contents;
                let impl_method_name = &method_name.0.contents;
                let primary_message = format!(
                    "Method `{impl_method_name}` from trait `{trait_name}` is not implemented"
                );
                Diagnostic::simple_error(
                    primary_message,
                    format!("Please implement {impl_method_name} here"),
                    *trait_impl_location,
                )
            }
            DefCollectorErrorKind::NotATrait { not_a_trait_name } => {
                let location = not_a_trait_name.location;
                Diagnostic::simple_error(
                    format!("{not_a_trait_name} is not a trait, therefore it can't be implemented"),
                    String::new(),
                    location,
                )
            }
            DefCollectorErrorKind::ModuleAlreadyPartOfCrate { mod_name, location } => {
                let message = format!("Module '{mod_name}' is already part of the crate");
                let secondary = String::new();
                Diagnostic::simple_error(message, secondary, *location)
            }
            DefCollectorErrorKind::ModuleOriginallyDefined { mod_name, location } => {
                let message = format!("Note: {mod_name} was originally declared here");
                let secondary = String::new();
                Diagnostic::simple_error(message, secondary, *location)
            }
            DefCollectorErrorKind::TraitImplOrphaned { location } => Diagnostic::simple_error(
                "Orphaned trait implementation".into(),
                "Either the type or the trait must be from the same crate as the trait implementation".into(),
                *location,
            ),
            DefCollectorErrorKind::ImplIsStricterThanTrait { constraint_typ, constraint_name, constraint_generics, constraint_location, trait_method_name, trait_method_location } => {
                let constraint = format!("{}{}", constraint_name, constraint_generics);

                let mut diag = Diagnostic::simple_error(
                    "impl has stricter requirements than trait".to_string(),
                    format!("impl has extra requirement `{constraint_typ}: {constraint}`"),
                    *constraint_location,
                );
                diag.add_secondary(format!("definition of `{trait_method_name}` from trait"), *trait_method_location);
                diag
            }
            DefCollectorErrorKind::UnsupportedNumericGenericType(err) => err.into(),
            DefCollectorErrorKind::TestOnAssociatedFunction { location } => Diagnostic::simple_error(
                "The `#[test]` attribute is disallowed on `impl` methods".into(),
                String::new(),
                *location,
            ),
            DefCollectorErrorKind::ExportOnAssociatedFunction { location } => Diagnostic::simple_error(
                "The `#[export]` attribute is disallowed on `impl` methods".into(),
                String::new(),
                *location,
            ),
        }
    }
}
