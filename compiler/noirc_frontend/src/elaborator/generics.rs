//! Generic parameter resolution and type parameter handling.

use std::rc::Rc;

use iter_extended::vecmap;
use noirc_errors::Location;
use rustc_hash::FxHashSet as HashSet;

use crate::{
    Kind, NamedGeneric, ResolvedGeneric, ResolvedGenerics, Type, TypeVariable,
    ast::{
        Ident, IdentOrQuotedType, Path, UnresolvedGeneric, UnresolvedGenerics,
        UnresolvedTraitConstraint, UnresolvedType, UnsupportedNumericGenericType, Visitor,
    },
    elaborator::types::{WildcardAllowed, WildcardDisallowedContext},
    hir::resolution::errors::ResolverError,
    node_interner::{DefinitionKind, NodeInterner, QuotedTypeId},
};

use super::Elaborator;

impl Elaborator<'_> {
    /// Runs `f` and if it modifies `self.generics`, `self.generics` is truncated
    /// back to the previous length.
    pub(super) fn recover_generics<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let generics_count = self.generics.len();
        let ret = f(self);
        self.generics.truncate(generics_count);
        ret
    }

    /// Add the given generics to scope.
    /// Each generic will have a fresh `Shared<TypeBinding>` associated with it.
    pub(super) fn add_generics(&mut self, generics: &UnresolvedGenerics) -> ResolvedGenerics {
        vecmap(generics, |generic| {
            let mut is_error = false;
            let (type_var, name) = match self.resolve_generic(generic) {
                Ok(values) => values,
                Err(error) => {
                    self.push_err(error);
                    is_error = true;
                    let id = self.interner.next_type_variable_id();
                    let kind = self.resolve_generic_kind(generic);
                    (TypeVariable::unbound(id, kind), Rc::new("(error)".into()))
                }
            };

            let location = generic.location();
            let name_owned = name.as_ref().clone();
            let resolved_generic = ResolvedGeneric { name, type_var, location };

            // Check for name collisions of this generic
            // Checking `is_error` here prevents DuplicateDefinition errors when
            // we have multiple generics from macros which fail to resolve and
            // are all given the same default name "(error)".
            if !is_error {
                if let Some(generic) = self.find_generic(&name_owned) {
                    self.push_err(ResolverError::DuplicateDefinition {
                        name: name_owned,
                        first_location: generic.location,
                        second_location: location,
                    });
                } else {
                    self.generics.push(resolved_generic.clone());
                }
            }

            resolved_generic
        })
    }

    pub(super) fn add_existing_generics(
        &mut self,
        unresolved_generics: &UnresolvedGenerics,
        generics: &ResolvedGenerics,
    ) {
        assert_eq!(unresolved_generics.len(), generics.len());

        for (unresolved_generic, generic) in unresolved_generics.iter().zip(generics) {
            self.add_existing_generic(unresolved_generic, unresolved_generic.location(), generic);
        }
    }

    pub(super) fn add_existing_generic(
        &mut self,
        unresolved_generic: &UnresolvedGeneric,
        location: Location,
        resolved_generic: &ResolvedGeneric,
    ) {
        if let Some(name) = unresolved_generic.ident().ident() {
            let name = name.as_str();

            if let Some(generic) = self.find_generic(name) {
                self.push_err(ResolverError::DuplicateDefinition {
                    name: name.to_string(),
                    first_location: generic.location,
                    second_location: location,
                });
            } else {
                self.generics.push(resolved_generic.clone());
            }
        }
    }

    pub(super) fn find_generic(&self, target_name: &str) -> Option<&ResolvedGeneric> {
        self.generics.iter().find(|generic| generic.name.as_ref() == target_name)
    }

    pub(super) fn resolve_generic(
        &mut self,
        generic: &UnresolvedGeneric,
    ) -> Result<(TypeVariable, Rc<String>), ResolverError> {
        // Map the generic to a fresh type variable
        match generic.ident() {
            IdentOrQuotedType::Ident(ident) => {
                let id = self.interner.next_type_variable_id();
                let kind = self.resolve_generic_kind(generic);
                let typevar = TypeVariable::unbound(id, kind);
                let name = Rc::new(ident.to_string());
                Ok((typevar, name))
            }
            IdentOrQuotedType::Quoted(id, location) => {
                match self.interner.get_quoted_type(*id).follow_bindings() {
                    Type::NamedGeneric(NamedGeneric { type_var, name, .. }) => {
                        Ok((type_var.clone(), name))
                    }
                    other => Err(ResolverError::MacroResultInGenericsListNotAGeneric {
                        location: *location,
                        typ: other.clone(),
                    }),
                }
            }
        }
    }

    /// Return the kind of an unresolved generic.
    /// If a numeric generic has been specified, resolve the annotated type to make
    /// sure only primitive numeric types are being used.
    pub(super) fn resolve_generic_kind(&mut self, generic: &UnresolvedGeneric) -> Kind {
        if let UnresolvedGeneric::Numeric { ident, typ } = generic {
            let unresolved_typ = typ.clone();
            let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::NumericGeneric);
            let typ = if unresolved_typ.is_type_expression() {
                self.resolve_type_with_kind(
                    unresolved_typ.clone(),
                    &Kind::numeric(Type::default_int_type()),
                    wildcard_allowed,
                )
            } else {
                self.resolve_type(unresolved_typ.clone(), wildcard_allowed)
            };
            if !matches!(typ, Type::FieldElement | Type::Integer(_, _)) {
                let unsupported_typ_err =
                    ResolverError::UnsupportedNumericGenericType(UnsupportedNumericGenericType {
                        name: ident.ident().map(|name| name.to_string()),
                        typ: typ.to_string(),
                        location: unresolved_typ.location,
                    });

                self.push_err(unsupported_typ_err);
            }
            Kind::numeric(typ)
        } else {
            Kind::Normal
        }
    }

    /// Check that all the generics show up in any of `types` (if they don't, we produce an error),
    /// or in any of a where clause associated type binding.
    pub(super) fn check_generics_appear_in_types(
        &mut self,
        generics: &[UnresolvedGeneric],
        types: &[&UnresolvedType],
        where_clause: &[UnresolvedTraitConstraint],
    ) {
        if generics.is_empty() {
            return;
        }

        // Turn each generic into an Ident
        let mut idents = HashSet::default();
        for generic in generics {
            match generic {
                UnresolvedGeneric::Variable(ident, _)
                | UnresolvedGeneric::Numeric { ident, typ: _ } => match ident {
                    IdentOrQuotedType::Ident(ident) => {
                        idents.insert(ident.clone());
                    }
                    IdentOrQuotedType::Quoted(quoted_type_id, location) => {
                        if let Type::NamedGeneric(NamedGeneric { name, .. }) =
                            self.interner.get_quoted_type(*quoted_type_id).follow_bindings()
                        {
                            idents.insert(Ident::new(name.to_string(), *location));
                        }
                    }
                },
            }
        }

        // Remove the ones that show up in `self_type`
        let mut visitor =
            RemoveGenericsAppearingInTypeVisitor { interner: self.interner, idents: &mut idents };
        for typ in types {
            typ.accept(&mut visitor);
        }

        // Removes the ones that show up in associated type bindings in the where clause
        for where_clause in where_clause {
            for (_name, typ) in &where_clause.trait_bound.trait_generics.named_args {
                typ.accept(&mut visitor);
            }
        }

        // The ones that remain are not mentioned in the impl: it's an error.
        for ident in idents {
            self.push_err(ResolverError::UnconstrainedTypeParameter { ident });
        }
    }

    pub(super) fn introduce_generics_into_scope(&mut self, all_generics: Vec<ResolvedGeneric>) {
        // Introduce all numeric generics into scope
        for generic in &all_generics {
            if let Kind::Numeric(typ) = &generic.kind() {
                let definition =
                    DefinitionKind::NumericGeneric(generic.type_var.clone(), typ.clone());
                let ident = Ident::new(generic.name.to_string(), generic.location);
                let hir_ident = self.add_variable_decl(
                    ident, false, // mutable
                    false, // allow_shadowing
                    false, // warn_if_unused
                    definition,
                );
                self.interner.push_definition_type(hir_ident.id, *typ.clone());
            }
        }

        self.generics = all_generics;
    }
}

struct RemoveGenericsAppearingInTypeVisitor<'interner, 'ident> {
    interner: &'interner NodeInterner,
    idents: &'ident mut HashSet<Ident>,
}

impl RemoveGenericsAppearingInTypeVisitor<'_, '_> {
    fn visit_type(&mut self, typ: &Type) {
        match typ {
            Type::Array(length, element) => {
                self.visit_type(length);
                self.visit_type(element);
            }
            Type::Vector(element) => {
                self.visit_type(element);
            }
            Type::FmtString(length, element) => {
                self.visit_type(length);
                self.visit_type(element);
            }
            Type::Tuple(items) | Type::DataType(_, items) | Type::Alias(_, items) => {
                for item in items {
                    self.visit_type(item);
                }
            }
            Type::TraitAsType(_, _, trait_generics) => {
                for generic in &trait_generics.ordered {
                    self.visit_type(generic);
                }
                for named_type in &trait_generics.named {
                    self.visit_type(&named_type.typ);
                }
            }
            Type::NamedGeneric(named_generic) => {
                let ident = Ident::new(named_generic.name.to_string(), Location::dummy());
                self.idents.remove(&ident);
            }
            Type::CheckedCast { from, to } => {
                self.visit_type(from);
                self.visit_type(to);
            }
            Type::Function(args, ret, env, _) => {
                for arg in args {
                    self.visit_type(arg);
                }
                self.visit_type(ret);
                self.visit_type(env);
            }
            Type::Reference(typ, _) => {
                self.visit_type(typ);
            }
            Type::InfixExpr(lhs, _, rhs, _) => {
                self.visit_type(lhs);
                self.visit_type(rhs);
            }
            Type::Unit
            | Type::Bool
            | Type::Integer(..)
            | Type::FieldElement
            | Type::String(_)
            | Type::Constant(..)
            | Type::Quoted(_)
            | Type::Forall(..)
            | Type::TypeVariable(_)
            | Type::Error => (),
        }
    }
}

impl Visitor for RemoveGenericsAppearingInTypeVisitor<'_, '_> {
    fn visit_path(&mut self, path: &Path) {
        if let Some(ident) = path.as_ident() {
            self.idents.remove(ident);
        }
    }

    fn visit_resolved_type(&mut self, quoted_type_id: QuotedTypeId, _: Location) {
        let typ = self.interner.get_quoted_type(quoted_type_id);
        self.visit_type(typ);
    }
}
