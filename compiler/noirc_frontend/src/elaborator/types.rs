//! Type resolution, unification, and method resolution (for both types and traits).
mod similarly_named_types;

use std::{borrow::Cow, collections::BTreeSet, rc::Rc};

use acvm::{AcirField, FieldElement};
use im::HashSet;
use iter_extended::vecmap;
use itertools::Itertools;
use noirc_errors::Location;
use num_bigint::BigInt;
use rustc_hash::FxHashMap as HashMap;

pub(crate) use similarly_named_types::SimilarlyNamedType;

use crate::{
    BinaryTypeOperator, Kind, ResolvedGeneric, Type, TypeBinding, TypeBindings, UnificationError,
    ast::{
        AsTraitPath, BinaryOpKind, GenericTypeArgs, Ident, IntegerBitSize, PathKind, UnaryOp,
        UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression, WILDCARD_TYPE,
    },
    elaborator::{Turbofish, UnstableFeature, path_resolution::PathResolution},
    hir::{
        comptime::{Integer, Value, bigint_to_field, evaluate_cast_one_step},
        def_collector::dc_crate::CompilationError,
        def_map::{ModuleDefId, ModuleId, Namespace, fully_qualified_module_path},
        resolution::{
            errors::ResolverError,
            import::PathResolutionError,
            visibility::{item_in_module_is_visible, trait_visibility_for_method_is_satisfied},
        },
        type_check::{
            Source, TypeCheckError,
            generics::{Generic, TraitGenerics},
        },
    },
    hir_def::{
        expr::{
            HirBinaryOp, HirCallExpression, HirExpression, HirIdent, HirLiteral, HirMemberAccess,
            HirMethodReference, HirPrefixExpression, HirTraitMethodReference, ImplKind, TraitItem,
        },
        function::FuncMeta,
        stmt::HirStatement,
        traits::{NamedType, ResolvedTraitBound, Trait, TraitConstraint},
    },
    modules::{get_ancestor_module_reexport, module_def_id_is_visible},
    node_interner::{
        DependencyId, ExprId, FuncId, GlobalValue, TraitId, TraitImplId, TraitImplKind,
        TraitItemId, TraitLookupMode,
    },
    shared::Signedness,
};

use super::{
    Elaborator, PathResolutionTarget, UnsafeBlockStatus, lints,
    path_resolution::{PathResolutionItem, PathResolutionMode, TypedPath, TypedPathSegment},
    variable::VariableResolution,
};

pub const SELF_TYPE_NAME: &str = "Self";

#[derive(Debug)]
struct TraitPathResolution {
    method: TraitPathResolutionMethod,
    item: Option<PathResolutionItem>,
    errors: Vec<PathResolutionError>,
}

#[derive(Debug)]
enum TraitPathResolutionMethod {
    NotATraitMethod(FuncId),
    TraitItem(TraitItem),
    MultipleTraitsInScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WildcardAllowed {
    Yes,
    No(WildcardDisallowedContext),
}

/// Context for positions where `impl Trait` is not allowed as a type.
/// `impl Trait` is only meaningful in function signatures (parameters and return types).
///
/// This context is stored on the `Elaborator` and checked in the `TraitAsType` arm of
/// type resolution. The variant is used to produce a position-specific error message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplTraitDisallowedContext {
    StructField,
    EnumVariant,
    Global,
    TypeAlias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WildcardDisallowedContext {
    AssociatedType,
    Cast,
    EnumVariant,
    FunctionReturn,
    FunctionParameter,
    Global,
    ImplType,
    NumericGeneric,
    QuotedAsType,
    StructField,
    TraitAsType,
    TraitBound,
    TraitConstraint,
    TraitImplType,
    TypeAlias,
}

impl Elaborator<'_> {
    /// Resolves an [`UnresolvedType`] to a [Type] with [`Kind::Normal`] and marks it, and any generic types it contains, as _referenced_.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn resolve_type(
        &mut self,
        typ: UnresolvedType,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        self.resolve_type_inner(
            typ,
            &Kind::Normal,
            PathResolutionMode::MarkAsReferenced,
            wildcard_allowed,
        )
    }

    /// Resolves an [`UnresolvedType`] to a [Type] with [`Kind::Normal`] and marks it, and any generic types it contains, as _used_.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn use_type(
        &mut self,
        typ: UnresolvedType,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        self.use_type_with_kind(typ, &Kind::Normal, wildcard_allowed)
    }

    /// Resolves an [`UnresolvedType`] to a [Type] and marks it, and any generic types it contains, as _used_.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn use_type_with_kind(
        &mut self,
        typ: UnresolvedType,
        kind: &Kind,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        self.resolve_type_inner(typ, kind, PathResolutionMode::MarkAsUsed, wildcard_allowed)
    }

    /// Translates an [`UnresolvedType`] to a [Type] with a given [Kind] and [`PathResolutionMode`].
    ///
    /// Pushes an error if the resolved type is invalid.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_type_inner(
        &mut self,
        typ: UnresolvedType,
        kind: &Kind,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        let location = typ.location;
        let resolved_type = self.resolve_type_with_kind_inner(typ, kind, mode, wildcard_allowed);
        if !self.in_comptime_context && resolved_type.is_nested_vector() {
            self.push_err(ResolverError::NestedVectors { location });
        }
        resolved_type
    }

    /// Rebind the free named generics of a type spliced in from a `quote { $typ }` to the
    /// same-named generics in scope at the splice site.
    ///
    /// A `Type` interpolated into a quote (e.g. one obtained from `TypeDefinition::fields_as_written`)
    /// is already resolved: its named generics carry the type variables of the definition it came
    /// from. When such a type is spliced into a new generic scope (e.g. a generated
    /// `impl<Context> ..`), a textually-written `Context` resolves by name to the new scope's
    /// generic, so the spliced type's `Context` must too; otherwise two identically-named generics
    /// with different type variables fail to unify.
    ///
    /// Only ordinary named generics are rebound. Associated-type and associated-constant
    /// projections are also modeled as `NamedGeneric`s, but their name is the projection itself
    /// (e.g. `<T as Deserialize>::N`); rebinding one to a same-named projection in scope would
    /// conflate distinct projections (e.g. a field's `<[T; N] as Deserialize>::N` with the
    /// enclosing impl's own `Self::N`, yielding a cyclic associated constant).
    fn rebind_resolved_type_generics(&self, typ: Type) -> Type {
        let mut bindings = TypeBindings::default();
        typ.visit(&mut |typ| {
            if let Type::NamedGeneric(named) = typ
                && !named.is_associated()
                && let TypeBinding::Unbound(id, kind) = &*named.type_var.borrow()
                && let Some(generic) = self.find_generic(named.name.as_str())
                && generic.type_var.id() != *id
            {
                let replacement = generic.clone().into_named_generic(None);
                bindings.insert(*id, (named.type_var.clone(), kind.clone(), replacement));
            }
            true
        });

        if bindings.is_empty() { typ } else { typ.substitute(&bindings) }
    }

    /// Resolves an [`UnresolvedType`] to a [Type] with a given [Kind] and marks it, and any generic types it contains, as _referenced_.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn resolve_type_with_kind(
        &mut self,
        typ: UnresolvedType,
        kind: &Kind,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        self.resolve_type_inner(typ, kind, PathResolutionMode::MarkAsReferenced, wildcard_allowed)
    }

    /// Translates an [`UnresolvedType`] into a [Type] with a given [Kind] and [`PathResolutionMode`].
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_type_with_kind_inner(
        &mut self,
        typ: UnresolvedType,
        kind: &Kind,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        use crate::ast::UnresolvedTypeData::*;

        let location = typ.location;
        let (named_path_location, is_self_type_name, is_synthetic) =
            if let Named(ref named_path, _, synthetic) = typ.typ {
                (
                    Some(named_path.last_ident().location()),
                    named_path.last_ident().is_self_type_name(),
                    synthetic,
                )
            } else {
                (None, false, false)
            };

        let resolved_type = match typ.typ {
            Array(size, elem) => {
                let elem = Box::new(self.resolve_type_with_kind_inner(
                    *elem,
                    kind,
                    mode,
                    wildcard_allowed,
                ));
                let size =
                    self.convert_expression_type(size, &Kind::u32(), location, wildcard_allowed);
                Type::Array(elem, Box::new(size))
            }
            Vector(elem) => {
                let elem = Box::new(self.resolve_type_with_kind_inner(
                    *elem,
                    kind,
                    mode,
                    wildcard_allowed,
                ));
                Type::Vector(elem)
            }
            Expression(expr) => {
                self.convert_expression_type(expr, kind, location, wildcard_allowed)
            }
            Unit => Type::Unit,
            Error => Type::Error,
            Named(path, args, _) => {
                let path = self.validate_path(path);
                self.resolve_named_type(path, args, mode, wildcard_allowed)
            }
            TraitAsType(path, args) => {
                if let Some(context) = self.impl_trait_is_disallowed {
                    self.push_err(ResolverError::ImplTraitTypeDisallowed {
                        location: path.location,
                        context,
                    });
                    return Type::Error;
                }
                self.use_unstable_feature(UnstableFeature::TraitAsType, path.location);
                let path = self.validate_path(path);
                self.resolve_trait_as_type(path, args, mode)
            }

            Tuple(fields) => Type::Tuple(vecmap(fields, |field| {
                self.resolve_type_with_kind_inner(field, kind, mode, wildcard_allowed)
            })),
            Function(args, ret, env, unconstrained) => {
                let args = vecmap(args, |arg| {
                    self.resolve_type_with_kind_inner(arg, kind, mode, wildcard_allowed)
                });
                let ret =
                    Box::new(self.resolve_type_with_kind_inner(*ret, kind, mode, wildcard_allowed));
                let env_location = env.location;

                let env =
                    Box::new(self.resolve_type_with_kind_inner(*env, kind, mode, wildcard_allowed));

                match env.follow_bindings_shallow().into_owned() {
                    Type::Unit | Type::Tuple(_) | Type::NamedGeneric(_) => {
                        Type::Function(args, ret, env, unconstrained)
                    }
                    typ => {
                        self.push_err(ResolverError::InvalidClosureEnvironment {
                            typ,
                            location: env_location,
                        });
                        Type::Error
                    }
                }
            }
            Reference(element, mutable) => Type::Reference(
                Box::new(self.resolve_type_with_kind_inner(*element, kind, mode, wildcard_allowed)),
                mutable,
            ),
            Parenthesized(typ) => {
                self.resolve_type_with_kind_inner(*typ, kind, mode, wildcard_allowed)
            }
            Resolved(id) => {
                let typ = self.interner.get_quoted_type(id).clone();
                self.rebind_resolved_type_generics(typ)
            }
            AsTraitPath(path) => self.resolve_as_trait_path(*path, mode, wildcard_allowed),
            Interned(id) => {
                let typ = self.interner.get_unresolved_type_data(id).clone();
                return self.resolve_type_with_kind_inner(
                    UnresolvedType { typ, location },
                    kind,
                    mode,
                    wildcard_allowed,
                );
            }
        };

        let location = named_path_location.unwrap_or(typ.location);
        match resolved_type {
            Type::DataType(ref data_type, _) => {
                // Record the location of the type reference
                self.interner.push_type_ref_location(&resolved_type, location);
                if !is_synthetic {
                    self.interner.add_type_reference(
                        data_type.borrow().id,
                        location,
                        is_self_type_name,
                    );
                }
            }
            Type::Alias(ref alias_type, _) => {
                self.interner.add_alias_reference(alias_type.borrow().id, location);
            }
            _ => (),
        }

        self.check_type_kind(resolved_type, kind, location)
    }

    /// Resolve `Self::Foo` to an associated type on the current trait or trait impl.
    /// Also searches parent traits.
    #[tracing::instrument(level = "trace", skip_all)]
    fn lookup_associated_type_on_self(&mut self, path: &TypedPath) -> Option<Type> {
        if path.segments.len() == 2 && path.first_name() == Some(SELF_TYPE_NAME) {
            let name = path.last_name();

            // Inside a trait definition (not an impl): check this trait and its parent traits.
            if self.current_trait_impl.is_none()
                && let Some(trait_id) = self.current_trait
            {
                let mut found = self.lookup_associated_type_in_parent_traits(trait_id, name);
                match found.len() {
                    0 => {}
                    1 => return Some(found.remove(0).1),
                    _ => {
                        let location = path.location;
                        let trait_names: Vec<_> = found
                            .iter()
                            .map(|(id, _)| self.interner.get_trait(*id).name.to_string())
                            .collect();
                        let ident = Ident::new(name.to_string(), location);
                        self.push_err(PathResolutionError::MultipleTraitsInScope {
                            ident,
                            traits: trait_names,
                        });
                        return Some(Type::Error);
                    }
                }
            }

            // Inside a trait impl: check the impl's own types, then parent trait impls.
            if let Some(impl_id) = self.current_trait_impl {
                if let Some(typ) = self.interner.find_associated_type_for_impl(impl_id, name) {
                    return Some(typ.clone());
                }

                if let Some(trait_id) = self.current_trait
                    && let Some(typ) = self.lookup_associated_type_in_parent_impls(
                        trait_id,
                        name,
                        &mut BTreeSet::new(),
                    )
                {
                    return Some(typ);
                }
            }
        }
        None
    }

    /// Search for an associated type in a trait and its parent trait hierarchy.
    /// Used inside trait definitions to resolve `Self::Foo` when `Foo` may be
    /// defined on a parent trait.
    fn lookup_associated_type_in_parent_traits(
        &self,
        trait_id: TraitId,
        name: &str,
    ) -> Vec<(TraitId, Type)> {
        let mut found = Vec::new();
        let mut visited = BTreeSet::new();
        self.collect_associated_type_in_parent_traits(trait_id, name, &mut found, &mut visited);
        found
    }

    /// Recursively collect all (`trait_id`, type) pairs for a named associated type
    /// across a trait and its parent hierarchy. Skip traits already
    /// visited in `visited`.
    fn collect_associated_type_in_parent_traits(
        &self,
        trait_id: TraitId,
        name: &str,
        found: &mut Vec<(TraitId, Type)>,
        visited: &mut BTreeSet<TraitId>,
    ) {
        if !visited.insert(trait_id) {
            return;
        }

        let the_trait = self.interner.get_trait(trait_id);
        if let Some(typ) = the_trait.get_associated_type(name) {
            let typ =
                typ.clone().into_named_generic(Some((SELF_TYPE_NAME, the_trait.name.as_str())));
            found.push((trait_id, typ));
        }

        let parent_trait_ids: Vec<_> =
            the_trait.parent_bounds().map(|bound| bound.trait_id).collect();
        for parent_id in parent_trait_ids {
            self.collect_associated_type_in_parent_traits(parent_id, name, found, visited);
        }
    }

    /// Search for an associated type in parent 'trait impls'.
    fn lookup_associated_type_in_parent_impls(
        &self,
        trait_id: TraitId,
        name: &str,
        visited: &mut BTreeSet<TraitId>,
    ) -> Option<Type> {
        if !visited.insert(trait_id) {
            return None;
        }

        let the_trait = self.interner.get_trait(trait_id);
        let parent_bounds: Vec<_> = the_trait.parent_bounds().cloned().collect();
        let self_type = self.self_type.as_ref()?;

        for parent_bound in &parent_bounds {
            let result = self.interner.try_lookup_trait_implementation(
                self_type,
                parent_bound.trait_id,
                &parent_bound.trait_generics.ordered,
                &parent_bound.trait_generics.named,
                TraitLookupMode::Default,
            );

            match result {
                Ok((
                    TraitImplKind::Normal(parent_impl_id)
                    | TraitImplKind::Prepared(parent_impl_id, _),
                    _,
                    _,
                )) => {
                    if let Some(typ) =
                        self.interner.find_associated_type_for_impl(parent_impl_id, name)
                    {
                        return Some(typ.clone());
                    }
                }
                Ok((TraitImplKind::Assumed { trait_generics, .. }, _, _)) => {
                    for named in &trait_generics.named {
                        if named.name.as_str() == name {
                            return Some(named.typ.clone());
                        }
                    }
                }
                _ => {
                    // Lookup failed (e.g. self type is too generic). Fall back to
                    // searching the parent trait's definition for the associated type.
                    let found =
                        self.lookup_associated_type_in_parent_traits(parent_bound.trait_id, name);
                    if let Some((_, typ)) = found.into_iter().next() {
                        return Some(typ);
                    }
                }
            }

            // Recurse into grandparent traits
            if let Some(typ) =
                self.lookup_associated_type_in_parent_impls(parent_bound.trait_id, name, visited)
            {
                return Some(typ);
            }
        }

        None
    }

    /// Resolve `T::Foo` to an associated type on a generic type parameter with trait bounds.
    ///
    /// For example, in `impl<T: Baz> Foo for T { type Bar = T::Qux; }`, this resolves `T::Qux`
    /// by finding that `T` has a bound `Baz` which defines the associated type `Qux`.
    /// Also searches parent traits.
    #[tracing::instrument(level = "trace", skip_all)]
    fn lookup_associated_type_on_generic(&mut self, path: &TypedPath) -> Option<Type> {
        if self.trait_bounds.is_empty() {
            return None;
        }

        if path.segments.len() != 2 {
            return None;
        }

        let type_name = path.segments[0].ident.as_str();
        let assoc_name = path.last_name();

        // Check if first segment is a generic parameter
        self.find_generic(type_name)?;

        // Search trait bounds for this generic to find the associated type directly.
        // Parent associated types are expected to be in `self.trait_bounds` already,
        // added during function elaboration.
        let mut found_types = Vec::new();
        let mut seen_traits = BTreeSet::new();

        for constraint in &self.trait_bounds {
            if let Type::NamedGeneric(generic) = &constraint.typ
                && generic.name.as_ref() == type_name
            {
                for named_generic in &constraint.trait_bound.trait_generics.named {
                    if named_generic.name.as_str() == assoc_name {
                        let trait_id = constraint.trait_bound.trait_id;
                        // Skip duplicates.
                        if seen_traits.insert(trait_id) {
                            found_types.push((trait_id, named_generic.typ.clone()));
                        }
                    }
                }
            }
        }

        match found_types.len() {
            0 => None, // Fall through to normal resolution
            1 => Some(found_types.remove(0).1),
            _ => {
                // Multiple traits have this associated type - ambiguous
                let location = path.location;
                let trait_names: Vec<_> = found_types
                    .iter()
                    .map(|(id, _)| self.interner.get_trait(*id).name.to_string())
                    .collect();
                let ident = Ident::new(assoc_name.to_string(), location);
                self.push_err(PathResolutionError::MultipleTraitsInScope {
                    ident,
                    traits: trait_names,
                });
                Some(Type::Error)
            }
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_named_type(
        &mut self,
        path: TypedPath,
        args: GenericTypeArgs,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        let location = path.location;
        let typ = self.resolve_named_type_helper(path, args, mode, wildcard_allowed);
        self.check_comptime_type_in_non_comptime_item(&typ, location);
        typ
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_named_type_helper(
        &mut self,
        path: TypedPath,
        args: GenericTypeArgs,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        // Check generics and associated types first.
        if let Some(typ) = self.lookup_generic_or_associated_type(&path, &args) {
            return typ;
        }

        let location = path.location;

        // Check for removed types and give a helpful error message
        if path.segments.len() == 1 {
            let name = path.segments[0].ident.as_str();
            if name == "u1" || name == "i1" {
                self.push_err(ResolverError::RemovedType {
                    location,
                    typ: name.to_string(),
                    replacement: "bool".to_string(),
                });
                return Type::Error;
            }
        }

        // Check if the path is a type variable first. We currently disallow generics on type
        // variables since we do not support higher-kinded types.
        if let Some(typ) = self.lookup_type_variable(&path, &args, wildcard_allowed) {
            return typ;
        }

        // Check type aliases before globals: a type alias in the types namespace should
        // take priority over a global with the same name in the values namespace.
        if let Some(type_alias) = self.lookup_type_alias(path.clone(), mode) {
            let id = type_alias.borrow().id;
            let (args, _) =
                self.resolve_type_args_inner(args, id, location, mode, wildcard_allowed);

            if let Some(item) = self.current_item {
                self.interner.add_type_alias_dependency(item, id);
            }

            // Collecting Type Alias references [Location]s to be used by LSP in order
            // to resolve the definition of the type alias
            self.interner.add_type_alias_ref(id, location);

            // Because there is no ordering to when type aliases (and other globals) are resolved,
            // it is possible for one to refer to an Error type and issue no error if it is set
            // equal to another type alias. Fixing this fully requires an analysis to create a DFG
            // of definition ordering. There is no such check here, so a type alias pointing at a
            // not-yet-resolved (or erroring) alias can still silently resolve to an Error type.
            return Type::Alias(type_alias, args);
        }

        match self.resolve_path_or_error_inner(path.clone(), PathResolutionTarget::Type, mode) {
            Ok(PathResolutionItem::Type(type_id)) => {
                let data_type = self.get_type(type_id);

                if self.resolving_ids.contains(&data_type.borrow().id) {
                    self.push_err(ResolverError::SelfReferentialType {
                        location: data_type.borrow().name.location(),
                    });

                    return Type::Error;
                }

                let (args, _) = self.resolve_type_args_inner(
                    args,
                    data_type.borrow(),
                    location,
                    mode,
                    wildcard_allowed,
                );

                if let Some(current_item) = self.current_item {
                    let dependency_id = data_type.borrow().id;
                    self.interner.add_type_dependency(current_item, dependency_id);
                }

                Type::DataType(data_type, args)
            }
            Ok(PathResolutionItem::PrimitiveType(primitive_type)) => {
                self.instantiate_primitive_type(primitive_type, args, location, wildcard_allowed)
            }
            Ok(PathResolutionItem::TraitAssociatedType(associated_type_id)) => {
                if wildcard_allowed == WildcardAllowed::No(WildcardDisallowedContext::ImplType) {
                    self.push_err(ResolverError::TraitImplOnAssociatedType { location });
                } else {
                    let associated_type =
                        self.interner.get_trait_associated_type(associated_type_id);
                    let trait_ = self.interner.get_trait(associated_type.trait_id);
                    self.push_err(ResolverError::AmbiguousAssociatedType {
                        trait_name: trait_.name.to_string(),
                        associated_type_name: associated_type.name.to_string(),
                        location,
                    });
                }

                Type::Error
            }
            Ok(item) => {
                // Fall back to the numeric-global shortcut so that `global N: u32 = 5`
                // used in a type position like `[u8; N]` still resolves. A name that
                // also exists in the types namespace as a real type takes priority via
                // the match arms above.
                if args.is_empty()
                    && let Some(typ) = self.lookup_global_type(&path, mode)
                {
                    return typ;
                }

                self.push_err(ResolverError::Expected {
                    expected: "type",
                    found: item.description(self.interner),
                    location,
                });

                Type::Error
            }
            Err(err) => {
                if args.is_empty()
                    && let Some(typ) = self.lookup_global_type(&path, mode)
                {
                    return typ;
                }

                self.push_err(err);

                Type::Error
            }
        }
    }

    /// Reports an error if `typ` is a comptime-only type and we are not in a comptime item
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn check_comptime_type_in_non_comptime_item(
        &mut self,
        typ: &Type,
        location: Location,
    ) {
        if self.in_comptime_context() {
            return;
        }

        let Some(item) = self.current_item else {
            // Early return if we're not actually inside any item.
            return;
        };

        let typ_is_comptime_only = match typ {
            Type::Quoted(_) => true,
            Type::DataType(data_type, _) => data_type.borrow().comptime,
            Type::Alias(type_alias, _) => type_alias.borrow().comptime,
            _ => false,
        };

        // We return early if we are in a comptime context, so if we find a comptime-only type here
        // it means we are trying to use it in a non-comptime item, which is an error.
        if typ_is_comptime_only {
            let item = match item {
                DependencyId::Function(_) => "function",
                DependencyId::Global(_) => "global",
                DependencyId::Alias(_) => "type alias",
                DependencyId::DataType(type_id) => {
                    if self.interner.get_type(type_id).borrow().is_struct() {
                        "struct"
                    } else {
                        "enum"
                    }
                }
                DependencyId::Trait(_) | DependencyId::Variable(_) => {
                    unreachable!(
                        "Unexpected current item when checking for comptime type usage: {:?}",
                        self.current_item
                    )
                }
            };

            let typ = typ.to_string();
            self.push_err(ResolverError::ComptimeTypeInNonComptimeItem { location, typ, item });
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn lookup_type_variable(
        &mut self,
        path: &TypedPath,
        args: &GenericTypeArgs,
        wildcard_allowed: WildcardAllowed,
    ) -> Option<Type> {
        if path.segments.len() != 1 {
            return None;
        }

        let name = path.last_name();
        match name {
            SELF_TYPE_NAME => {
                let self_type = self.self_type.clone()?;
                if !args.is_empty() {
                    self.push_err(ResolverError::GenericsOnSelfType { location: path.location });
                }
                Some(self_type)
            }
            WILDCARD_TYPE => {
                if !args.is_empty() {
                    self.push_err(ResolverError::GenericsOnWildcardType {
                        location: path.location,
                    });
                }

                match wildcard_allowed {
                    WildcardAllowed::Yes => {}
                    WildcardAllowed::No(reason) => {
                        self.push_err(ResolverError::WildcardTypeDisallowed {
                            location: path.location,
                            context: reason,
                        });
                    }
                }

                Some(self.interner.next_type_variable_with_kind(Kind::Any))
            }
            _ => None,
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_trait_as_type(
        &mut self,
        path: TypedPath,
        args: GenericTypeArgs,
        mode: PathResolutionMode,
    ) -> Type {
        // Fetch information needed from the trait as the closure for resolving all the `args`
        // requires exclusive access to `self`
        let location = path.location;
        self.use_unstable_feature(UnstableFeature::TraitAsType, location);
        let trait_as_type_info = self.lookup_trait_or_error(path).map(|trait_| trait_.id);

        if let Some(id) = trait_as_type_info {
            let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::TraitAsType);
            let (ordered, named) =
                self.resolve_type_args_inner(args, id, location, mode, wildcard_allowed);
            let name = self.interner.get_trait(id).name.to_string();
            let generics = TraitGenerics { ordered, named };
            Type::TraitAsType(id, Rc::new(name), generics)
        } else {
            Type::Error
        }
    }

    /// Resolves the ordered and named [`GenericTypeArgs`] into [Type]s and associated [`NamedType`]s,
    /// marking all of them as _used_.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn use_type_args(
        &mut self,
        args: GenericTypeArgs,
        item: impl Generic,
        location: Location,
    ) -> (Vec<Type>, Vec<NamedType>) {
        let mode = PathResolutionMode::MarkAsUsed;
        let wildcard_allowed = WildcardAllowed::Yes;
        self.resolve_type_args_inner(args, item, location, mode, wildcard_allowed)
    }

    /// Resolves the ordered and named [`GenericTypeArgs`] into [Type]s and associated [`NamedType`]s.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_type_args_inner(
        &mut self,
        args: GenericTypeArgs,
        item: impl Generic,
        location: Location,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
    ) -> (Vec<Type>, Vec<NamedType>) {
        let allow_implicit_named_args = true;
        self.resolve_type_or_trait_args_inner(
            args,
            item,
            location,
            allow_implicit_named_args,
            mode,
            wildcard_allowed,
        )
    }

    /// Matches [`GenericTypeArgs::ordered_args`] to the [`Generic::generic_kinds`] of a [Generic] type,
    /// resolving them to [Type]s with the given [`PathResolutionMode`]. If the type accepts named
    /// generic arguments, those are resolved as well and returned as associated [`NamedType`]s.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_type_or_trait_args_inner(
        &mut self,
        mut args: GenericTypeArgs,
        item: impl Generic,
        location: Location,
        allow_implicit_named_args: bool,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
    ) -> (Vec<Type>, Vec<NamedType>) {
        let expected_kinds = item.generic_kinds(self.interner);

        if args.ordered_args.len() != expected_kinds.len() {
            self.push_err(TypeCheckError::GenericCountMismatch {
                item: item.item_name(self.interner),
                expected: expected_kinds.len(),
                found: args.ordered_args.len(),
                location,
            });
            let error_type = UnresolvedTypeData::Error.with_location(location);
            args.ordered_args.resize(expected_kinds.len(), error_type);
        }

        let ordered_args = expected_kinds.iter().zip_eq(args.ordered_args);
        let ordered = vecmap(ordered_args, |(kind, typ)| {
            self.resolve_type_with_kind_inner(typ, kind, mode, wildcard_allowed)
        });

        let mut associated = Vec::new();

        if item.accepts_named_type_args() {
            associated = self.resolve_associated_type_args(
                args.named_args,
                item,
                location,
                allow_implicit_named_args,
                mode,
                wildcard_allowed,
            );
        } else if !args.named_args.is_empty() {
            let item_kind = item.item_kind();
            self.push_err(ResolverError::NamedTypeArgs { location, item_kind });
        }

        (ordered, associated)
    }

    /// Assuming that a [Generic] type accepts named type arguments, ie. has associated types,
    /// go through a list of named [`UnresolvedType`]s and match them up to the named generics of the type,
    /// returning the resolved [`NamedType`]s and pushing errors for any unexpected, duplicate or missing entries.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_associated_type_args(
        &mut self,
        args: Vec<(Ident, UnresolvedType)>,
        item: impl Generic,
        location: Location,
        allow_implicit_named_args: bool,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
    ) -> Vec<NamedType> {
        let mut seen_args = HashMap::default();
        let mut required_args = item.named_generics(self.interner);
        let mut resolved = Vec::with_capacity(required_args.len());

        // Go through each argument to check if it is in our required_args list.
        // If it is remove it from the list, otherwise issue an error.
        for (name, typ) in args {
            let index = required_args.iter().position(|item| item.name.as_ref() == name.as_str());

            let Some(index) = index else {
                if let Some(prev_location) = seen_args.get(name.as_str()).copied() {
                    self.push_err(TypeCheckError::DuplicateNamedTypeArg { name, prev_location });
                } else {
                    let item = item.item_name(self.interner);
                    self.push_err(TypeCheckError::NoSuchNamedTypeArg { name, item });
                }
                continue;
            };

            // Remove the argument from the required list so we remember that we already have it
            let expected = required_args.remove(index);
            seen_args.insert(name.to_string(), name.location());

            let typ =
                self.resolve_type_with_kind_inner(typ, &expected.kind(), mode, wildcard_allowed);
            resolved.push(NamedType { name, typ });
        }

        // Anything that hasn't been removed yet is missing.
        // Fill it in to avoid a panic if we allow named args to be elided, otherwise error.
        for generic in required_args {
            let name = generic.name.clone();

            if allow_implicit_named_args {
                let name = Ident::new(name.as_ref().clone(), location);
                let typ = self.interner.next_type_variable();
                resolved.push(NamedType { name, typ });
            } else {
                let item = item.item_name(self.interner);
                self.push_err(TypeCheckError::MissingNamedTypeArg { item, location, name });
            }
        }
        resolved
    }

    /// Look up a path as a generic type parameter or an associated type
    /// Returns `None` if the path doesn't match any of these.
    #[tracing::instrument(level = "trace", skip_all)]
    fn lookup_generic_or_associated_type(
        &mut self,
        path: &TypedPath,
        args: &GenericTypeArgs,
    ) -> Option<Type> {
        if path.segments.len() == 1 {
            let name = path.last_name();
            if let Some(generic) = self.find_generic(name) {
                let generic = generic.clone();
                // A generic type parameter cannot take generic arguments since we don't support
                // higher-kinded types, so reject any that were given (in either `T<..>` or the
                // `T::<..>` turbofish form).
                let last_segment = path.segments.last();
                let turbofish = last_segment.is_some_and(|segment| segment.generics.is_some());
                if !args.is_empty() || turbofish {
                    let location = if turbofish {
                        last_segment.map_or(path.location, |segment| segment.turbofish_location())
                    } else {
                        path.location
                    };
                    self.push_err(ResolverError::GenericsOnGeneric { location });
                }
                return Some(generic.into_named_generic(None));
            }
        } else if let Some(typ) = self.lookup_associated_type_on_self(path) {
            self.error_if_generics_on_associated_type(path);
            return Some(typ);
        } else if let Some(typ) = self.lookup_associated_type_on_generic(path) {
            self.error_if_generics_on_associated_type(path);
            return Some(typ);
        }

        None
    }

    /// Associated types cannot carry turbofish generics; report an error if the path's last
    /// segment has any.
    fn error_if_generics_on_associated_type(&mut self, path: &TypedPath) {
        if let Some(last_segment) = path.segments.last()
            && last_segment.generics.is_some()
        {
            self.push_err(ResolverError::GenericsOnAssociatedType {
                location: last_segment.turbofish_location(),
            });
        }
    }

    /// Look up a path as a global used as a numeric type (e.g. `global N: u32 = 5;`
    /// Resolves `path` as a numeric global used in type position (e.g. an array length).
    ///
    /// Returns `None` *without* emitting a diagnostic when `path` does not resolve to a global,
    /// leaving it to the caller to surface the original path-resolution error. `None` is also
    /// returned *with* a diagnostic already pushed for globals that exist but cannot be used as a
    /// numeric type (unresolved, non-integral, or not fitting their type).
    #[tracing::instrument(level = "trace", skip_all)]
    fn lookup_global_type(&mut self, path: &TypedPath, mode: PathResolutionMode) -> Option<Type> {
        match self.resolve_path_inner(path.clone(), PathResolutionTarget::Value, mode) {
            Ok(PathResolution { item: PathResolutionItem::Global(id), errors }) => {
                self.push_errors(errors);

                if let Some(current_item) = self.current_item {
                    self.interner.add_global_dependency(current_item, id);
                }

                let reference_location = path.location;
                self.interner.add_global_reference(id, reference_location);
                let opt_global_let_statement = self.interner.get_global_let_statement(id);
                let typ = opt_global_let_statement
                    .as_ref()
                    .map_or(Type::u32(), |let_statement| let_statement.r#type.clone());

                let Some(stmt) = opt_global_let_statement else {
                    if self.elaborate_global_if_unresolved(&id) {
                        return self.lookup_global_type(path, mode);
                    } else {
                        let path = path.clone();
                        self.push_err(ResolverError::NoSuchNumericTypeVariable { path });
                        return None;
                    }
                };

                let rhs = stmt.expression;
                let location = self.interner.expr_location(&rhs);

                let GlobalValue::Resolved(global_value) = &self.interner.get_global(id).value
                else {
                    self.push_err(ResolverError::UnevaluatedGlobalType { location });
                    return None;
                };

                let Some(global_value) = global_value.as_integer() else {
                    let global_value = global_value.clone();
                    self.push_err(ResolverError::NonIntegralGlobalType { location, global_value });
                    return None;
                };

                if global_value.get_type().unify(&typ).is_err() {
                    let global_value = *global_value;
                    self.push_err(ResolverError::GlobalDoesNotFitItsType {
                        location,
                        global_value,
                        typ,
                    });
                    return None;
                }

                Some(Type::Constant(*global_value))
            }
            // Not a global: defer to the caller to report the original path-resolution error.
            _ => None,
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn convert_expression_type(
        &mut self,
        expr: UnresolvedTypeExpression,
        expected_kind: &Kind,
        location: Location,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        match expr {
            UnresolvedTypeExpression::Variable(path) => {
                let mut ab = GenericTypeArgs::default();
                // Use generics from path, if they exist
                if let Some(last_segment) = path.segments.last()
                    && let Some(generics) = &last_segment.generics
                {
                    ab.ordered_args = generics.clone();
                }
                let path = self.validate_path(path);
                let mode = PathResolutionMode::MarkAsReferenced;
                let mut typ = self.resolve_named_type(path, ab, mode, wildcard_allowed);
                if let Type::Alias(alias, ref vec) = typ {
                    if alias.borrow().numeric_expr.is_none() {
                        self.push_err(ResolverError::InvalidNumericAliasExpression { location });
                        return Type::Error;
                    }
                    typ = alias.borrow().get_type(vec);
                }
                self.check_type_kind(typ, expected_kind, location)
            }
            UnresolvedTypeExpression::Constant(int, suffix, _span) => {
                // Default type constants to u32 if not specified
                let suffix = suffix.unwrap_or(crate::token::IntegerTypeSuffix::U32);
                let typ = suffix.as_type();

                if !self.check_kind(Kind::numeric(typ.clone()), expected_kind, location) {
                    return Type::Error;
                }

                let Some(int) = Integer::try_from_bigint_and_type_suffix(&int, suffix) else {
                    let min = typ.integral_minimum_size().unwrap();
                    let max = typ.integral_maximum_size().unwrap();
                    self.push_err(TypeCheckError::IntegerLiteralDoesNotFitItsType {
                        expr: int,
                        ty: typ,
                        range: format!("{min}..={max}"),
                        location,
                    });
                    return Type::Error;
                };

                Type::Constant(int)
            }
            UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, location) => {
                let (lhs_location, rhs_location) = (lhs.location(), rhs.location());
                let lhs = self.convert_expression_type(
                    *lhs,
                    expected_kind,
                    lhs_location,
                    wildcard_allowed,
                );
                let rhs = self.convert_expression_type(
                    *rhs,
                    expected_kind,
                    rhs_location,
                    wildcard_allowed,
                );

                match (lhs, rhs) {
                    (Type::Constant(lhs), Type::Constant(rhs)) => {
                        if lhs.get_type().unify(&rhs.get_type()).is_err() {
                            self.push_err(TypeCheckError::TypeKindMismatch {
                                expected_kind: lhs.numeric_kind(),
                                expr_kind: rhs.numeric_kind(),
                                expr_location: location,
                            });
                            return Type::Error;
                        }
                        match op.function(lhs, rhs, location) {
                            Ok(result) => Type::Constant(result),
                            Err(err) => {
                                let err = Box::new(err);
                                let error =
                                    ResolverError::BinaryOpError { lhs, op, rhs, err, location };
                                self.push_err(error);
                                Type::Error
                            }
                        }
                    }
                    (lhs, rhs) => {
                        let infix = Type::infix_expr(Box::new(lhs), op, Box::new(rhs));
                        Type::CheckedCast { from: Box::new(infix.clone()), to: Box::new(infix) }
                            .canonicalize()
                    }
                }
            }
            UnresolvedTypeExpression::Negation(rhs, location) => {
                // Fold `-<magnitude>` of a signed integer literal into a single constant so the
                // range check sees the negative value instead of rejecting the magnitude (e.g.
                // `128` is out of `i8` range, but `-128` is `i8::MIN`).
                if let UnresolvedTypeExpression::Constant(int, Some(suffix), const_location) =
                    rhs.as_ref()
                    && matches!(
                        suffix,
                        crate::token::IntegerTypeSuffix::I8
                            | crate::token::IntegerTypeSuffix::I16
                            | crate::token::IntegerTypeSuffix::I32
                            | crate::token::IntegerTypeSuffix::I64
                    )
                {
                    let folded = UnresolvedTypeExpression::Constant(
                        -(int.clone()),
                        Some(*suffix),
                        *const_location,
                    );
                    return self.convert_expression_type(
                        folded,
                        expected_kind,
                        location,
                        wildcard_allowed,
                    );
                }

                let rhs_location = rhs.location();
                let rhs = self.convert_expression_type(
                    *rhs,
                    expected_kind,
                    rhs_location,
                    wildcard_allowed,
                );

                match rhs {
                    Type::Constant(rhs) => {
                        if let Some(result) = -rhs {
                            Type::Constant(result)
                        } else {
                            self.push_err(TypeCheckError::InvalidUnaryOp {
                                typ: rhs.get_type().to_string(),
                                operator: "-",
                                location,
                            });
                            Type::Error
                        }
                    }
                    rhs => {
                        let kind = rhs.kind().into_numeric_type_or_error();
                        let int = Integer::try_from_bigint(&BigInt::ZERO, &kind)
                            .unwrap_or_else(|| Integer::Field(FieldElement::zero()));
                        let zero = Type::Constant(int);
                        let sub = BinaryTypeOperator::Subtraction;
                        let infix = Box::new(Type::infix_expr(Box::new(zero), sub, Box::new(rhs)));
                        Type::CheckedCast { from: infix.clone(), to: infix }.canonicalize()
                    }
                }
            }
            UnresolvedTypeExpression::AsTraitPath(path) => {
                let mode = PathResolutionMode::MarkAsReferenced;
                let typ = self.resolve_as_trait_path(*path, mode, wildcard_allowed);
                self.check_type_kind(typ, expected_kind, location)
            }
        }
    }

    /// Checks that the type's [Kind] matches the expected kind, issuing an error if it does not.
    /// Returns `typ` unless an error occurs - in which case [`Type::Error`] is returned.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn check_type_kind(
        &mut self,
        typ: Type,
        expected_kind: &Kind,
        location: Location,
    ) -> Type {
        if typ.has_cyclic_alias() {
            self.push_err(TypeCheckError::CyclicType { typ, location });
            return Type::Error;
        }

        if self.check_kind(typ.kind(), expected_kind, location) { typ } else { Type::Error }
    }

    /// Checks that `expr_kind` matches `expected_kind`, issuing an error if it does not.
    /// Returns `true` if the kinds unify.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn check_kind(
        &mut self,
        expr_kind: Kind,
        expected_kind: &Kind,
        location: Location,
    ) -> bool {
        if !expr_kind.unifies(expected_kind) {
            self.push_err(TypeCheckError::TypeKindMismatch {
                expected_kind: expected_kind.clone(),
                expr_kind,
                expr_location: location,
            });
            false
        } else {
            true
        }
    }

    /// Resolve `<{object} as {trait}>::{ident}` to a [Type] of the `{ident}`.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_as_trait_path(
        &mut self,
        path: AsTraitPath,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        let location = path.trait_path.location;
        let trait_path = self.validate_path(path.trait_path.clone());
        let Some(trait_id) = self.resolve_trait_by_path(trait_path) else {
            // Error should already be pushed in the None case
            return Type::Error;
        };

        if let Some(typ) =
            self.try_resolve_self_as_trait_path(&path, mode, wildcard_allowed, trait_id)
        {
            return typ;
        }

        let (ordered, named) = self.use_type_args(path.trait_generics.clone(), trait_id, location);
        let object_type = self.use_type(path.typ.clone(), wildcard_allowed);

        match self.interner.lookup_trait_implementation(&object_type, trait_id, &ordered, &named) {
            Ok((impl_kind, instantiation_bindings)) => {
                let typ = self.get_associated_type_from_trait_impl(path, impl_kind);
                typ.substitute(&instantiation_bindings)
            }
            Err(constraints) => {
                self.push_trait_constraint_error(&object_type, constraints, location);
                Type::Error
            }
        }
    }

    /// Try to resolve an [`AsTraitPath`] as `<Self as {trait}>::{ident}` to the [Type] of the `{ident}`.
    ///
    /// If it's a different pattern then returns `None`.
    #[tracing::instrument(level = "trace", skip_all)]
    fn try_resolve_self_as_trait_path(
        &mut self,
        path: &AsTraitPath,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
        trait_id: TraitId,
    ) -> Option<Type> {
        // Only applies if the path refers to the current trait.
        let current_trait = self.current_trait?;

        if trait_id != current_trait {
            return None;
        }

        // See if we are dealing with `<Self as {trait}>::{ident}`.
        // If so, redirect to how we deal with `Self::{ident}`.
        let UnresolvedTypeData::Named(object_path, object_generics, _) = &path.typ.typ else {
            return None;
        };

        // Only applies if the object refers to `Self` and nothing else.
        if object_path.segments.len() != 1
            || !object_path.segments[0].ident.is_self_type_name()
            || !object_generics.is_empty()
        {
            return None;
        }

        // Only works if all the trait generics in the path are the same as the trait itself.
        let location = path.trait_path.location;
        let (ordered, named) = self.use_type_args(path.trait_generics.clone(), trait_id, location);

        let trait_generic_ids: Vec<_> =
            vecmap(&self.interner.get_trait(current_trait).generics, |g| g.type_var.id());

        if ordered.len() != trait_generic_ids.len() {
            return None;
        }
        let ordered_match_trait_generics =
            ordered.iter().zip_eq(&trait_generic_ids).all(|(typ, trait_gen_id)| {
                matches!(typ, Type::NamedGeneric(ng) if ng.type_var.id() == *trait_gen_id)
            });
        if !ordered_match_trait_generics
            || !named.iter().all(|typ| matches!(typ.typ, Type::TypeVariable(_)))
        {
            return None;
        }

        // Remove the trait from the path.
        let self_and_impl = object_path.clone().join(path.impl_item.clone());
        let self_and_impl = self.validate_path(self_and_impl);

        // Resolved as a named type, which is what `Self::{ident}` would be.
        let typ = self.resolve_named_type(
            self_and_impl,
            // The call to `lookup_generic_or_associated_type` which calls `lookup_associated_type_on_self`
            // will only be made if the args are empty, and that's what we tried to ascertain before
            // by checking that none of them are bound to a concrete type.
            GenericTypeArgs::default(),
            mode,
            wildcard_allowed,
        );
        Some(typ)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn get_associated_type_from_trait_impl(
        &mut self,
        path: AsTraitPath,
        impl_kind: TraitImplKind,
    ) -> Type {
        let associated_types = match impl_kind {
            TraitImplKind::Assumed { trait_generics, .. } => Cow::Owned(trait_generics.named),
            TraitImplKind::Normal(impl_id) | TraitImplKind::Prepared(impl_id, _) => {
                Cow::Borrowed(self.interner.get_associated_types_for_impl(impl_id))
            }
        };

        match associated_types.iter().find(|named| named.name == path.impl_item) {
            Some(generic) => generic.typ.clone(),
            None => {
                let name = path.impl_item.clone();
                let item = format!("<{} as {}>", path.typ, path.trait_path);
                self.push_err(TypeCheckError::NoSuchNamedTypeArg { name, item });
                Type::Error
            }
        }
    }

    /// Reduce an associated-type projection `<object_type as trait>::assoc_name` over a rigid
    /// object type to the type defined by the matching impl. Assumed (`where` clause) impls are
    /// ignored, so the answer is the single ground truth from the real impl. Returns `None` when
    /// the object type contains unbound type variables (unification could still change which
    /// impl matches) or no such impl/associated type is found - in particular for a bare generic
    /// like `T`, which no real impl matches and only a `where` clause hypothesis can answer.
    pub(super) fn normalize_rigid_associated_type(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        ordered: &[Type],
        assoc_name: &str,
    ) -> Option<Type> {
        if object_type.contains_unbound_type_variable() {
            return None;
        }
        let (impl_kind, instantiation_bindings) = self
            .interner
            .lookup_trait_implementation_ignoring_assumed(object_type, trait_id, ordered, &[])
            .ok()?;
        let associated_types = match impl_kind {
            TraitImplKind::Assumed { .. } => unreachable!(
                "lookup_trait_implementation_ignoring_assumed should ignore assumed impls"
            ),
            TraitImplKind::Normal(impl_id) | TraitImplKind::Prepared(impl_id, _) => {
                self.interner.get_associated_types_for_impl(impl_id)
            }
        };
        let typ =
            associated_types.iter().find(|named| named.name.as_str() == assoc_name)?.typ.clone();
        Some(typ.substitute(&instantiation_bindings).follow_bindings())
    }

    /// This resolves `Self::some_static_method`, inside an impl block (where we don't have a concrete `self_type`)
    /// or inside a trait default method.
    ///
    /// Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    /// E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_static_method_by_self(
        &self,
        path: &TypedPath,
        trait_id: TraitId,
    ) -> Option<TraitPathResolution> {
        // Reached only for a plain `Self::…` prefix, so the only thing left to distinguish is the
        // single-segment `Self::item` form (a longer `Self::A::b` is left to the bounds fallback).
        debug_assert!(path.kind == PathKind::Plain && path.segments[0].ident.is_self_type_name());
        if path.segments.len() != 2 {
            return None;
        }

        let method = &path.segments[1].ident;
        let the_trait = self.interner.get_trait(trait_id);
        // Allow referring to trait constants via Self:: as well
        let definition = the_trait.find_method_or_constant(method.as_str(), self.interner)?;
        let constraint = the_trait.as_constraint(path.location);
        let trait_method = TraitItem { definition, constraint, assumed: true };
        let method = TraitPathResolutionMethod::TraitItem(trait_method);
        Some(TraitPathResolution { method, item: None, errors: Vec::new() })
    }

    /// Resolves the last segment of a `Trait::item` path against a prefix already resolved to a
    /// trait. The segment is either a trait static method (`Trait::method`) or an associated
    /// constant (`Trait::CONST`); returns `None` if it is neither.
    ///
    /// The method is looked up directly on the trait rather than by re-resolving the whole path:
    /// the trait was already marked when its prefix was resolved, so only the method's own
    /// reference/dependency is recorded here (as inherent-method resolution does). An associated
    /// constant is not a function, so it could not be resolved as a path anyway; its `TraitItem`
    /// is lowered to the selected impl's value during monomorphization.
    pub(super) fn resolve_trait_item_on_prefix(
        &mut self,
        trait_id: TraitId,
        turbofish: Option<Turbofish>,
        last_segment: &TypedPathSegment,
        resolution: PathResolution,
        location: Location,
    ) -> Option<VariableResolution> {
        let the_trait = self.interner.get_trait(trait_id);
        let name = last_segment.ident.as_str();
        let method_func_id = the_trait.method_ids.get(name).copied();
        let associated_constant = the_trait.associated_constant_ids.get(name).copied();
        let constraint = the_trait.as_constraint(location);

        let trait_resolution = if let Some(func_id) = method_func_id {
            let definition = self.interner.function_definition_id(func_id);
            self.record_direct_method_reference(func_id, &last_segment.ident);
            let trait_item = TraitItem { definition, constraint, assumed: false };
            let item = PathResolutionItem::TraitFunction(trait_id, turbofish, func_id);
            Some(TraitPathResolution {
                method: TraitPathResolutionMethod::TraitItem(trait_item),
                item: Some(item),
                errors: resolution.errors,
            })
        } else if let Some(definition) = associated_constant {
            let trait_item = TraitItem { definition, constraint, assumed: true };
            Some(TraitPathResolution {
                method: TraitPathResolutionMethod::TraitItem(trait_item),
                item: None,
                errors: resolution.errors,
            })
        } else {
            None
        };

        // A trait prefix can only carry a trait method or associated constant; if the last segment
        // is neither, it names nothing.
        self.variable_from_trait_resolution_or_unresolved(location, last_segment, trait_resolution)
    }

    /// Resolves `T::item` to a method or associated constant of a generic `T`, given the `T: Trait`
    /// bounds matched in scope (and the trait/supertrait hierarchy they reach). Reports an error and
    /// resolves to nothing identifiable when the item is ambiguous across in-scope traits.
    /// E.g. `t.method()` with `where T: Foo<Bar>` in scope returns `(Foo::method, T, vec![Bar])`.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_bounded_generic_item(
        &mut self,
        bounds: Vec<TraitConstraint>,
        last_segment: &TypedPathSegment,
        turbofish: Option<Turbofish>,
        location: Location,
    ) -> Option<VariableResolution> {
        let method_name = last_segment.ident.as_str();

        let mut matches = Vec::new();
        let mut visited = BTreeSet::new();
        for constraint in bounds {
            let the_trait = self.interner.get_trait(constraint.trait_bound.trait_id);
            matches.extend(self.find_methods_or_constants_in_trait(
                method_name,
                constraint,
                the_trait,
                &mut visited,
            ));
        }

        let trait_resolution = if matches.len() == 1 {
            let method = matches.remove(0).0;

            // A turbofish on the generic itself (e.g. `T::<u32>::method`) is not allowed.
            if let Some(turbofish) = &turbofish {
                self.push_err(PathResolutionError::TurbofishNotAllowedOnItem {
                    item: "generic parameter".to_string(),
                    location: turbofish.location,
                });
            }

            Some(TraitPathResolution { method, item: None, errors: Vec::new() })
        } else if matches.len() > 1 {
            let ident = Ident::new(method_name.to_string(), location);
            let traits =
                vecmap(matches, |(_, trait_id)| self.fully_qualified_trait_path_by_id(trait_id));
            let errors = vec![PathResolutionError::MultipleTraitsInScope { ident, traits }];
            Some(TraitPathResolution {
                method: TraitPathResolutionMethod::MultipleTraitsInScope,
                item: None,
                errors,
            })
        } else {
            None
        };

        // The generic is in scope; the last segment must be a method or associated constant
        // reached through one of its bounds. If it is neither, it names nothing.
        self.variable_from_trait_resolution_or_unresolved(location, last_segment, trait_resolution)
    }

    fn find_methods_or_constants_in_trait(
        &self,
        method_name: &str,
        constraint: TraitConstraint,
        the_trait: &Trait,
        visited: &mut BTreeSet<TraitId>,
    ) -> Vec<(TraitPathResolutionMethod, TraitId)> {
        // Skip if we've already visited this trait.
        if !visited.insert(the_trait.id) {
            return Vec::new();
        }

        let mut matches = Vec::new();

        let parent_constraints = vecmap(the_trait.parent_bounds(), |trait_bound| TraitConstraint {
            typ: constraint.typ.clone(),
            trait_bound: trait_bound.clone(),
        });

        if let Some(definition) = the_trait.find_method_or_constant(method_name, self.interner) {
            let trait_item = TraitItem { definition, constraint, assumed: true };
            let method = TraitPathResolutionMethod::TraitItem(trait_item);
            matches.push((method, the_trait.id));
        }

        for constraint in parent_constraints {
            let parent_trait = self.interner.get_trait(constraint.trait_bound.trait_id);
            matches.extend(self.find_methods_or_constants_in_trait(
                method_name,
                constraint,
                parent_trait,
                visited,
            ));
        }

        matches
    }

    /// Resolves a path of the form `Type::method` or `Type::<turbofish>::method`.
    /// Lazy-aware wrapper around [`crate::node_interner::NodeInterner::lookup_direct_method`].
    /// Resolves each candidate's meta first so that the type-aware lookup (which reads
    /// `function_meta` directly via `Methods::method_matches`) doesn't ICE on a
    /// still-deferred meta.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn lookup_direct_method(
        &mut self,
        typ: &Type,
        method_name: &str,
        check_self_param: bool,
    ) -> Option<FuncId> {
        self.resolve_method_candidate_metas(typ, method_name);
        self.interner.lookup_direct_method(typ, method_name, check_self_param)
    }

    /// Lazy-aware wrapper around [`crate::node_interner::NodeInterner::lookup_trait_methods`].
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn lookup_trait_methods(
        &mut self,
        typ: &Type,
        method_name: &str,
        has_self_arg: bool,
    ) -> Vec<(FuncId, TraitId, Type)> {
        self.resolve_method_candidate_metas(typ, method_name);
        self.interner.lookup_trait_methods(typ, method_name, has_self_arg)
    }

    /// Lazy-aware wrapper around [`crate::node_interner::NodeInterner::lookup_generic_methods`].
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn lookup_generic_methods(
        &mut self,
        typ: &Type,
        method_name: &str,
        has_self_arg: bool,
    ) -> Vec<(FuncId, TraitId, Type)> {
        for func_id in self.interner.generic_method_candidate_ids(method_name) {
            self.define_function_meta_if_undefined(func_id);
        }
        self.interner.lookup_generic_methods(typ, method_name, has_self_arg)
    }

    fn resolve_method_candidate_metas(&mut self, typ: &Type, method_name: &str) {
        for func_id in self.interner.method_candidate_ids(typ, method_name) {
            self.define_function_meta_if_undefined(func_id);
        }
    }

    /// Resolves `Type::method` (or `Type::<..>::method`) given a prefix already resolved to a type,
    /// type alias, or primitive type.
    ///
    /// When turbofish generics are present, uses type-directed lookup to select the correct impl
    /// (e.g. `S::<u32, u64>::foo` picks the impl whose self type unifies with `S<u32, u64>`).
    /// Without turbofish, returns `None` so the caller falls back to module-based lookup, which
    /// handles `Self::method`, visibility checks, and associated constants correctly.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_method_on_type_prefix(
        &mut self,
        last_segment: TypedPathSegment,
        turbofish: Option<Turbofish>,
        is_self_prefix: bool,
        path_resolution: PathResolution,
        location: Location,
    ) -> Option<VariableResolution> {
        // `Self::method` must anchor on the impl's own `self_type` and produce a `SelfMethod`, so it
        // gets dedicated handling (in `resolve_self_or_inherent_method`) distinct from a plain
        // `TypeName::method`.
        let mut errors = Vec::new();
        let Some(typ) = self.path_resolution_item_to_type(
            &path_resolution.item,
            turbofish.clone(),
            &mut errors,
        ) else {
            // Both callers pass a prefix already known to name a concrete type — the `Type` prefix
            // kind is classified as exactly a type/alias/primitive, and `Self` (the other caller)
            // resolves to its concrete self type — so this conversion cannot fail.
            unreachable!(
                "a type-prefix path must resolve to a type, got {:?}",
                path_resolution.item
            );
        };

        // Kept for the value fallback below (the method-lookup branches consume `turbofish`).
        let fallback_turbofish = turbofish.clone();

        let method_name = last_segment.ident.as_str();

        let check_self_param = false;
        let direct_method = self.lookup_direct_method(&typ, method_name, check_self_param);

        // Resolve inherent methods (`TypeName::method`, and `Self::method`) through this
        // type-directed lookup. Names that aren't an inherent method here (associated constants,
        // trait methods not in scope) fall through to trait-method resolution.
        let trait_resolution = if turbofish.is_some() {
            self.resolve_turbofish_type_method(
                &typ,
                direct_method,
                &last_segment,
                turbofish,
                path_resolution,
                errors,
                location,
            )
        } else if let Some(direct_method) = direct_method {
            Some(self.resolve_self_or_inherent_method(
                &typ,
                direct_method,
                is_self_prefix,
                &last_segment,
                turbofish,
                path_resolution,
                errors,
            ))
        } else {
            self.resolve_qualified_trait_method(
                &typ,
                None,
                &last_segment,
                turbofish,
                path_resolution,
                errors,
                location,
            )
        };

        // The last segment isn't an inherent or qualified trait method on the type; it may still be
        // an enum variant or an associated constant accessed as `Type::CONST`, resolved directly on
        // the type (or, if the prefix was a broken type alias whose type is `Type::Error`, reported
        // there as an unresolved member). The error is reported if it is none of these.
        match trait_resolution {
            Some(resolution) => self.variable_from_trait_resolution(location, resolution),
            None => self.resolve_value_in_type(&last_segment, &typ, fallback_turbofish),
        }
    }

    /// Resolves a turbofished `TypeName::<..>::method` path. Resolves to the single inherent method
    /// matching the turbofish type if there is one, reports an error if the name is a method on the
    /// type but none matches the turbofish, and otherwise defers to trait-method resolution.
    #[allow(clippy::too_many_arguments)]
    fn resolve_turbofish_type_method(
        &mut self,
        typ: &Type,
        direct_method: Option<FuncId>,
        last_segment: &TypedPathSegment,
        turbofish: Option<Turbofish>,
        path_resolution: PathResolution,
        generics_errors: Vec<CompilationError>,
        location: Location,
    ) -> Option<TraitPathResolution> {
        let method_name = last_segment.ident.as_str();

        if let Some(func_id) = direct_method {
            self.push_errors(generics_errors);
            return Some(self.resolve_direct_method(
                path_resolution.item,
                turbofish,
                func_id,
                &last_segment.ident,
                path_resolution.errors,
            ));
        }

        let has_self_arg = false;
        let type_trait_methods = self.lookup_trait_methods(typ, method_name, has_self_arg);

        // If no method matches the turbofish type but the name is a known method for this type
        // (just incompatible), report an error. If the name isn't a method at all (e.g. it's an
        // associated constant), return None and let the fallback handle it.
        if type_trait_methods.is_empty() && self.interner.has_method_with_name(typ, method_name) {
            self.push_errors(generics_errors);
            let mut errors = path_resolution.errors;
            let available_impls = self
                .interner
                .get_direct_method_impl_types(typ, method_name)
                .into_iter()
                .map(|t| t.to_string())
                .collect();
            errors.push(PathResolutionError::UnresolvedMethodForType {
                typ: typ.to_string(),
                ident: last_segment.ident.clone(),
                available_impls,
            });
            return Some(TraitPathResolution {
                method: TraitPathResolutionMethod::MultipleTraitsInScope,
                item: None,
                errors,
            });
        }

        self.resolve_qualified_trait_method(
            typ,
            Some(type_trait_methods),
            last_segment,
            turbofish,
            path_resolution,
            generics_errors,
            location,
        )
    }

    /// Resolves a non-turbofished `TypeName::method` (or `Self::method`) path to an inherent method.
    /// `Self::method` anchors on the impl's own concrete self type; `TypeName::method` is reported as
    /// ambiguous (Rust's E0034) when more than one non-overlapping inherent impl provides it.
    #[allow(clippy::too_many_arguments)]
    fn resolve_self_or_inherent_method(
        &mut self,
        typ: &Type,
        direct_method: FuncId,
        is_self_prefix: bool,
        last_segment: &TypedPathSegment,
        turbofish: Option<Turbofish>,
        path_resolution: PathResolution,
        generics_errors: Vec<CompilationError>,
    ) -> TraitPathResolution {
        let method_name = last_segment.ident.as_str();

        if is_self_prefix {
            // `Self::method` anchors on the impl's own concrete self type, so pick the matching
            // impl among any non-overlapping inherent impls and resolve to a `SelfMethod` (so
            // the impl's generics — not the path's fresh ones — anchor the call).
            let func_id = self
                .self_type
                .clone()
                .and_then(|self_type| self.lookup_direct_method(&self_type, method_name, true))
                .unwrap_or(direct_method);
            self.push_errors(generics_errors);
            let mut errors = path_resolution.errors;
            self.record_direct_method_reference(func_id, &last_segment.ident);
            self.push_direct_method_visibility_error(func_id, &last_segment.ident, &mut errors);
            let method = TraitPathResolutionMethod::NotATraitMethod(func_id);
            let item = Some(PathResolutionItem::SelfMethod(func_id));
            return TraitPathResolution { method, item, errors };
        }

        // `TypeName::method` is ambiguous when more than one non-overlapping inherent impl
        // defines a `method` applicable to `typ` (mirroring Rust's E0034): the path names more
        // than one function and nothing here disambiguates. Require a method call
        // (`value.method(..)`) or turbofish (`TypeName::<..>::method(..)`) instead.
        let impl_types = self.interner.matching_direct_method_types(typ, method_name);
        if impl_types.len() >= 2 {
            self.push_errors(generics_errors);
            let mut errors = path_resolution.errors;
            errors.push(PathResolutionError::MultipleApplicableMethods {
                ident: last_segment.ident.clone(),
                impl_types: vecmap(&impl_types, |typ| typ.to_string()),
            });
            return TraitPathResolution {
                method: TraitPathResolutionMethod::MultipleTraitsInScope,
                item: None,
                errors,
            };
        }

        self.push_errors(generics_errors);
        self.resolve_direct_method(
            path_resolution.item,
            turbofish,
            direct_method,
            &last_segment.ident,
            path_resolution.errors,
        )
    }

    /// Resolves the head of a `Type::method` path to the concrete receiver [Type] whose methods we
    /// look up, applying any `turbofish` generics. Returns `None` (so the caller falls back to other
    /// resolution strategies) when the path doesn't name a type, type alias, or primitive type.
    fn path_resolution_item_to_type(
        &mut self,
        item: &PathResolutionItem,
        turbofish: Option<Turbofish>,
        errors: &mut Vec<CompilationError>,
    ) -> Option<Type> {
        let typ = match item {
            PathResolutionItem::Type(type_id) => {
                let generics =
                    self.resolve_struct_id_turbofish_generics(*type_id, turbofish, errors);
                let datatype = self.get_type(*type_id);
                Type::DataType(datatype, generics)
            }
            PathResolutionItem::TypeAlias(type_alias_id) => {
                let generics = self.resolve_type_alias_id_turbofish_generics(
                    *type_alias_id,
                    turbofish,
                    errors,
                );
                let type_alias = self.interner.get_type_alias(*type_alias_id);
                let type_alias = type_alias.borrow();
                type_alias.get_type(&generics)
            }
            PathResolutionItem::PrimitiveType(primitive_type) => {
                let (typ, _) = self.instantiate_primitive_type_with_turbofish(
                    *primitive_type,
                    turbofish,
                    errors,
                );
                typ
            }
            PathResolutionItem::Module(..)
            | PathResolutionItem::Trait(..)
            | PathResolutionItem::TraitAssociatedType(..)
            | PathResolutionItem::Global(..)
            | PathResolutionItem::EnumVariant(..)
            | PathResolutionItem::ModuleFunction(..)
            | PathResolutionItem::Method(..)
            | PathResolutionItem::SelfMethod(..)
            | PathResolutionItem::TypeAliasFunction(..)
            | PathResolutionItem::TraitFunction(..)
            | PathResolutionItem::TypeTraitFunction(..)
            | PathResolutionItem::PrimitiveFunction(..)
            | PathResolutionItem::TraitConstant(..) => {
                return None;
            }
        };
        Some(typ)
    }

    /// Resolves `Type::method` to a trait method on `typ`, given the trait methods already looked up
    /// for this type (or `None` to look them up here). Returns `None` if `method` isn't a trait method
    /// on `typ` at all; otherwise resolves to the single matching method, or reports an ambiguity when
    /// more than one trait in scope provides it.
    #[allow(clippy::too_many_arguments)]
    fn resolve_qualified_trait_method(
        &mut self,
        typ: &Type,
        trait_methods: Option<Vec<(FuncId, TraitId, Type)>>,
        last_segment: &TypedPathSegment,
        turbofish: Option<Turbofish>,
        path_resolution: PathResolution,
        generics_errors: Vec<CompilationError>,
        location: Location,
    ) -> Option<TraitPathResolution> {
        let PathResolution { item: path_resolution_item, errors: path_resolution_errors } =
            path_resolution;
        let method_name = last_segment.ident.as_str();

        let has_self_arg = false;
        let trait_methods = trait_methods
            .unwrap_or_else(|| self.lookup_trait_methods(typ, method_name, has_self_arg));

        if trait_methods.is_empty() {
            return None;
        }

        let (hir_method_reference, error) =
            self.get_trait_method_in_scope(&trait_methods, method_name, last_segment.location);
        let Some(hir_method_reference) = hir_method_reference else {
            // The method matches multiple traits (in scope, or none in scope), so there's no single
            // method to resolve to. Report the ambiguity here rather than deferring to module-based
            // resolution (which no longer holds trait methods).
            self.push_errors(generics_errors);
            let mut errors = path_resolution_errors;
            errors.extend(error);
            return Some(TraitPathResolution {
                method: TraitPathResolutionMethod::MultipleTraitsInScope,
                item: None,
                errors,
            });
        };

        match hir_method_reference {
            HirMethodReference::FuncId(func_id) => {
                // It could happen that we find a single function (one in a trait impl)
                let mut errors = path_resolution_errors;
                if let Some(error) = error {
                    errors.push(error);
                }

                Some(Self::type_method_or_trait_method_func_id_resolution(
                    path_resolution_item,
                    turbofish,
                    func_id,
                    errors,
                ))
            }
            HirMethodReference::TraitItemId(HirTraitMethodReference {
                definition,
                trait_id,
                ..
            }) => {
                // In this case turbofish won't be resolved again, so we can commit the errors
                self.push_errors(generics_errors);

                let trait_ = self.interner.get_trait(trait_id);

                let mut constraint = trait_.as_constraint(location);
                constraint.typ = typ.clone();

                let trait_method = TraitItem { definition, constraint, assumed: false };
                let func_id = hir_method_reference.func_id(self.interner)?;
                let item = PathResolutionItem::TypeTraitFunction(typ.clone(), trait_id, func_id);

                let mut errors = path_resolution_errors;
                if let Some(error) = error {
                    errors.push(error);
                }

                if !trait_visibility_for_method_is_satisfied(
                    func_id,
                    self.module_id(),
                    self.interner,
                    self.def_maps,
                ) {
                    errors.push(PathResolutionError::Private(last_segment.ident.clone()));
                }

                let method = TraitPathResolutionMethod::TraitItem(trait_method);
                Some(TraitPathResolution { method, item: Some(item), errors })
            }
        }
    }

    /// Builds the resolution for an inherent `TypeName::method` (or turbofished `TypeName::<..>::method`)
    /// call, reporting a `Private` error if `func_id` is not visible from the current module.
    fn resolve_direct_method(
        &mut self,
        path_resolution_item: PathResolutionItem,
        turbofish: Option<Turbofish>,
        func_id: FuncId,
        method_ident: &Ident,
        mut errors: Vec<PathResolutionError>,
    ) -> TraitPathResolution {
        self.record_direct_method_reference(func_id, method_ident);
        self.push_direct_method_visibility_error(func_id, method_ident, &mut errors);
        Self::type_method_or_trait_method_func_id_resolution(
            path_resolution_item,
            turbofish,
            func_id,
            errors,
        )
    }

    /// Records the dependency and the LSP reference (at the method name) for an inherent method
    /// resolved here. Inherent methods aren't in the module scope that would otherwise record this.
    fn record_direct_method_reference(&mut self, func_id: FuncId, method_ident: &Ident) {
        if let Some(current_item) = self.current_item {
            self.interner.add_function_dependency(current_item, func_id);
        }
        self.interner.add_function_reference(func_id, method_ident.location());
    }

    /// Pushes a `Private` error onto `errors` if the inherent method `func_id` is not visible from
    /// the current module (checked against the impl's defining module, not the type's module).
    fn push_direct_method_visibility_error(
        &self,
        func_id: FuncId,
        method_ident: &Ident,
        errors: &mut Vec<PathResolutionError>,
    ) {
        let visibility = self.interner.function_visibility(func_id);
        if let Some(func_meta) = self.interner.try_function_meta(&func_id) {
            let source_module =
                ModuleId { krate: func_meta.source_crate, local_id: func_meta.source_module };
            if !item_in_module_is_visible(
                self.def_maps,
                self.module_id(),
                source_module,
                visibility,
            ) {
                errors.push(PathResolutionError::Private(method_ident.clone()));
            }
        }
    }

    fn type_method_or_trait_method_func_id_resolution(
        path_resolution_item: PathResolutionItem,
        turbofish: Option<Turbofish>,
        func_id: FuncId,
        errors: Vec<PathResolutionError>,
    ) -> TraitPathResolution {
        let item = match path_resolution_item {
            PathResolutionItem::Type(type_id) => {
                PathResolutionItem::Method(type_id, turbofish, func_id)
            }
            PathResolutionItem::TypeAlias(type_alias_id) => {
                PathResolutionItem::TypeAliasFunction(type_alias_id, turbofish, func_id)
            }
            PathResolutionItem::PrimitiveType(primitive_type) => {
                PathResolutionItem::PrimitiveFunction(primitive_type, turbofish, func_id)
            }
            _ => unreachable!("An early return should have triggered before in this case"),
        };
        let method = TraitPathResolutionMethod::NotATraitMethod(func_id);
        TraitPathResolution { method, item: Some(item), errors }
    }

    /// `Self::…` inside a trait impl. `Self` is the impl's concrete type with associated items, so
    /// the last segment may be an associated-type method, an associated constant, or a method on a
    /// primitive `Self` (none of which a plain type-prefix resolution reaches); otherwise it is a
    /// plain method on the self type, resolved like `Type::method`.
    pub(super) fn resolve_self_in_trait_impl(
        &mut self,
        path: TypedPath,
        self_type: Type,
        trait_impl_id: TraitImplId,
        last_segment: TypedPathSegment,
        turbofish: Option<Turbofish>,
    ) -> Option<VariableResolution> {
        if let Some((expr_id, typ)) = self.resolve_variable_as_self_method_or_associated_constant(
            &path,
            self_type,
            trait_impl_id,
        ) {
            return Some(VariableResolution::Expression(expr_id, typ));
        }
        self.resolve_self_as_concrete_type(path, last_segment, turbofish)
    }

    /// `Self::…` inside a trait definition. `Self` is the trait, so the last segment resolves to an
    /// assumed constraint on the current trait, falling back to a supertrait reached through it.
    pub(super) fn resolve_self_in_trait(
        &mut self,
        path: TypedPath,
        trait_id: TraitId,
        last_segment: TypedPathSegment,
        turbofish: Option<Turbofish>,
    ) -> Option<VariableResolution> {
        if let Some(resolution) = self.resolve_trait_static_method_by_self(&path, trait_id) {
            return self.variable_from_trait_resolution(path.location, resolution);
        }
        // Fall back to a supertrait reached through the assumed `Self` bound.
        if let Some(bounds) = self.matching_generic_bounds(&path) {
            return self.resolve_bounded_generic_item(
                bounds,
                &last_segment,
                turbofish,
                path.location,
            );
        }
        // `Self` here is the trait itself, so the last segment can only be a trait static method or
        // associated constant (handled above); anything else names nothing. Report the last
        // segment rather than the in-scope `Self`.
        self.push_err(PathResolutionError::Unresolved(last_segment.ident.clone()));
        None
    }

    /// Resolve `Self::method` (or `Self::AssocType::method`) by resolving the `Self` prefix as a
    /// type and looking the last segment up on it, exactly as `Type::method` does. A non-method
    /// (e.g. `Self::Variant`) falls back to a value lookup of the whole path.
    pub(super) fn resolve_self_as_concrete_type(
        &mut self,
        path: TypedPath,
        last_segment: TypedPathSegment,
        turbofish: Option<Turbofish>,
    ) -> Option<VariableResolution> {
        let mut prefix = path.clone();
        prefix.pop();
        match self.use_path_as_type(prefix) {
            Ok(type_resolution) => self.resolve_method_on_type_prefix(
                last_segment,
                turbofish,
                true, // is_self_prefix
                type_resolution,
                path.location,
            ),
            // The `Self` prefix itself doesn't resolve as a type (e.g. `Self::not_a_type::method`):
            // report that resolution failure, which already points at the offending segment, rather
            // than re-resolving the whole path as a value just to rediscover it.
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    /// Turn a [`TraitPathResolution`] into the [`VariableResolution`] a prefixed path resolves to,
    /// pushing the resolution's errors. Returns `None` for an ambiguous trait method
    /// (`MultipleTraitsInScope`), whose error was already reported, so the caller produces an error
    /// expression rather than falling back to a value lookup.
    fn variable_from_trait_resolution(
        &mut self,
        location: Location,
        resolution: TraitPathResolution,
    ) -> Option<VariableResolution> {
        self.push_errors(resolution.errors);
        let item = resolution.item;
        match resolution.method {
            TraitPathResolutionMethod::NotATraitMethod(func_id) => {
                let ident = HirIdent {
                    location,
                    id: self.interner.function_definition_id(func_id),
                    impl_kind: ImplKind::NotATraitMethod,
                };
                Some(VariableResolution::Ident(ident, item))
            }
            TraitPathResolutionMethod::TraitItem(trait_item) => {
                let ident = HirIdent {
                    location,
                    id: trait_item.definition,
                    impl_kind: ImplKind::TraitItem(trait_item),
                };
                Some(VariableResolution::Ident(ident, item))
            }
            TraitPathResolutionMethod::MultipleTraitsInScope => None,
        }
    }

    /// Map a trait/bounded-generic prefix's last-segment resolution to a [`VariableResolution`]: a
    /// `Some` trait resolution becomes the trait item (or `None` for an already-reported
    /// ambiguity), while a `None` means the last segment is not a method or associated constant of
    /// the trait, which is the only thing such a prefix can carry, so report it as unresolved
    /// directly. A value lookup would fail anyway, and on a trait or generic prefix it blames the
    /// (in-scope) prefix segment rather than the missing item.
    fn variable_from_trait_resolution_or_unresolved(
        &mut self,
        location: Location,
        last_segment: &TypedPathSegment,
        trait_resolution: Option<TraitPathResolution>,
    ) -> Option<VariableResolution> {
        match trait_resolution {
            Some(resolution) => self.variable_from_trait_resolution(location, resolution),
            None => {
                self.push_err(PathResolutionError::Unresolved(last_segment.ident.clone()));
                None
            }
        }
    }

    /// Unify two types, modifying both in the process.
    ///
    /// Pushes an error on failure.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn unify(
        &mut self,
        actual: &Type,
        expected: &Type,
        make_error: impl FnOnce(&Elaborator) -> TypeCheckError,
    ) {
        if let Err(UnificationError) = actual.unify(expected) {
            let error = make_error(self);
            self.push_err(error);
        }
    }

    /// Wrapper of [`Type::unify_with_coercions`], pushing any unification errors.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn unify_with_coercions(
        &mut self,
        actual: &Type,
        expected: &Type,
        expression: ExprId,
        location: Location,
        make_error: impl FnOnce(&Elaborator) -> CompilationError,
    ) {
        let mut errors = Vec::new();
        actual.unify_with_coercions(expected, expression, location, self, &mut errors, make_error);

        // When passing lambdas to unconstrained functions that don't explicitly state
        // that they expect unconstrained lambdas, ignore the coercion.
        if self.in_unconstrained_args {
            errors.retain(|err| {
                !matches!(err, CompilationError::TypeError(TypeCheckError::UnsafeFn { .. }))
            });
        }

        self.push_errors(errors);
    }

    pub(super) fn unify_or_type_mismatch(
        &mut self,
        actual: &Type,
        expected: &Type,
        location: Location,
    ) {
        self.unify(actual, expected, |elaborator| {
            elaborator.new_type_mismatch_error(actual, expected, location)
        });
    }

    pub(super) fn unify_with_reference_coercion(
        &mut self,
        actual: &Type,
        expected: &Type,
        location: Location,
    ) {
        if !actual.try_reference_coercion(expected) {
            self.unify_or_type_mismatch(actual, expected, location);
        }
    }

    pub(super) fn unify_or_type_mismatch_with_source(
        &mut self,
        actual: &Type,
        expected: &Type,
        source: Source,
        location: Location,
    ) {
        self.unify(actual, expected, |elaborator| {
            elaborator.new_type_mismatch_with_source_error(actual, expected, source, location)
        });
    }

    pub(crate) fn new_type_mismatch_error(
        &self,
        actual: &Type,
        expected: &Type,
        location: Location,
    ) -> TypeCheckError {
        TypeCheckError::TypeMismatch {
            expected_typ: expected.to_string(),
            expr_typ: actual.to_string(),
            expr_location: location,
            similarly_named_types: self.compute_similarly_named_types(actual, expected),
        }
    }

    pub(crate) fn new_type_mismatch_with_source_error(
        &self,
        actual: &Type,
        expected: &Type,
        source: Source,
        location: Location,
    ) -> TypeCheckError {
        TypeCheckError::TypeMismatchWithSource {
            expected: expected.to_string(),
            actual: actual.to_string(),
            source,
            location,
            similarly_named_types: self.compute_similarly_named_types(actual, expected),
        }
    }

    /// Return a fresh integer or field type variable and log it
    /// in `self.type_variables` to default it later.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn polymorphic_integer_or_field(&mut self) -> Type {
        let typ = Type::polymorphic_integer_or_field(self.interner);
        self.push_defaultable_type_variable(typ.clone());
        typ
    }

    /// Return a fresh integer type variable and log it
    /// in `self.type_variables` to default it later.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn polymorphic_integer(&mut self) -> Type {
        let typ = Type::polymorphic_integer(self.interner);
        self.push_defaultable_type_variable(typ.clone());
        typ
    }

    /// Return a fresh integer type variable and log it
    /// in `self.type_variables` to default it later.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn type_variable_with_kind(&mut self, type_var_kind: Kind) -> Type {
        let typ = Type::type_variable_with_kind(self.interner, type_var_kind);
        self.push_defaultable_type_variable(typ.clone());
        typ
    }

    /// Translates a (possibly Unspecified) `UnresolvedType` to a Type.
    /// Any `UnresolvedType::Unspecified` encountered are replaced with fresh type variables.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_inferred_type(
        &mut self,
        typ: Option<UnresolvedType>,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        match typ {
            Some(typ) => self.use_type(typ, wildcard_allowed),
            None => self.interner.next_type_variable_with_kind(Kind::Any),
        }
    }

    /// Insert as many dereference operations as necessary to automatically dereference a method
    /// call object to its base value type T.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn insert_auto_dereferences(&mut self, object: ExprId, typ: Type) -> (ExprId, Type) {
        if !matches!(typ.follow_bindings_shallow().as_ref(), Type::Reference(..)) {
            return (object, typ);
        }

        let Type::Reference(element, _mut) = typ.follow_bindings() else {
            unreachable!("`typ` was just checked to be a reference");
        };
        let location = self.interner.id_location(object);

        let object = self.interner.push_expr_full(
            HirExpression::Prefix(HirPrefixExpression::new(
                UnaryOp::Dereference { implicitly_added: true },
                object,
            )),
            location,
            element.as_ref().clone(),
        );

        // Recursively dereference to allow for converting &mut &mut T to T
        self.insert_auto_dereferences(object, *element)
    }

    /// Given a method object: `(*foo).bar` of a method call `(*foo).bar.baz()`, remove the
    /// implicitly added dereference operator if one is found.
    ///
    /// Returns `Some(new_expr_id)` if a dereference was removed and None otherwise.
    #[tracing::instrument(level = "trace", skip_all)]
    fn try_remove_implicit_dereference(&mut self, object: ExprId) -> Option<ExprId> {
        match self.interner.expression(&object) {
            HirExpression::MemberAccess(mut access) => {
                let new_lhs = self.try_remove_implicit_dereference(access.lhs)?;
                access.lhs = new_lhs;
                access.is_offset = true;

                // `object` will have a different type now, which will be filled in
                // later when type checking the method call as a function call.
                self.interner.replace_expr(&object, HirExpression::MemberAccess(access));
                Some(object)
            }
            HirExpression::Prefix(prefix) => match prefix.operator {
                // Found a dereference we can remove. Now just replace it with its rhs to remove it.
                UnaryOp::Dereference { implicitly_added: true } => Some(prefix.rhs),
                _ => None,
            },
            _ => None,
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn bind_function_type_impl(
        &mut self,
        fn_params: &[Type],
        fn_ret: &Type,
        callsite_args: &[(Type, ExprId, Location)],
        location: Location,
    ) -> Type {
        if fn_params.len() != callsite_args.len() {
            self.push_err(TypeCheckError::ParameterCountMismatch {
                expected: fn_params.len(),
                found: callsite_args.len(),
                location,
            });
            return Type::Error;
        }

        for (param, (arg, arg_expr_id, arg_location)) in fn_params.iter().zip_eq(callsite_args) {
            self.unify_with_coercions(arg, param, *arg_expr_id, *arg_location, |elaborator| {
                CompilationError::TypeError(elaborator.new_type_mismatch_error(
                    arg,
                    param,
                    *arg_location,
                ))
            });
        }

        fn_ret.clone()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn bind_function_type(
        &mut self,
        function: Type,
        args: Vec<(Type, ExprId, Location)>,
        location: Location,
    ) -> Type {
        // Could do a single unification for the entire function type, but matching beforehand
        // lets us issue a more precise error on the individual argument that fails to type check.
        match function.follow_bindings_shallow().as_ref() {
            Type::TypeVariable(binding) if binding.kind().is_normal_or_any() => {
                if let TypeBinding::Bound(typ) = &*binding.borrow() {
                    return self.bind_function_type(typ.clone(), args, location);
                }

                let ret = self.interner.next_type_variable();
                let args = vecmap(args, |(arg, _, _)| arg);
                let env_type = self.interner.next_type_variable();
                let expected =
                    Type::Function(args, Box::new(ret.clone()), Box::new(env_type), false);

                let expected_kind = expected.kind();
                if let Err(error) = binding.try_bind(expected, &expected_kind, location) {
                    self.push_err(error);
                }
                ret
            }
            // The closure env is ignored on purpose: call arguments never place
            // constraints on closure environments.
            Type::Function(parameters, ret, _env, _unconstrained) => {
                self.bind_function_type_impl(parameters, ret, &args, location)
            }
            Type::Error => Type::Error,
            found => {
                self.push_err(TypeCheckError::ExpectedFunction { found: found.clone(), location });
                Type::Error
            }
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn check_cast(
        &mut self,
        from_expr_id: &ExprId,
        from: &Type,
        to: &Type,
        location: Location,
    ) -> Type {
        let to = to.follow_bindings();
        let from_follow_bindings = from.follow_bindings();

        use HirExpression::Literal;
        let from_value_opt = match self.interner.expression(from_expr_id) {
            Literal(HirLiteral::Integer(field)) => Some(field),
            _ => None,
        };

        let from_is_polymorphic = match from_follow_bindings {
            Type::Integer(..) | Type::FieldElement | Type::Bool => false,

            Type::TypeVariable(ref var) if var.is_integer() || var.is_integer_or_field() => true,
            Type::TypeVariable(_) => {
                // NOTE: in reality the expected type can also include bool, but for the compiler's simplicity
                // we only allow integer types. If a bool is in `from` it will need an explicit type annotation.
                let expected = self.polymorphic_integer_or_field();
                self.unify(from, &expected, |_| TypeCheckError::InvalidCast {
                    from: from.clone(),
                    location,
                    reason: "casting from a non-integral type is unsupported".into(),
                });
                true
            }
            Type::Error => return Type::Error,
            from => {
                let reason = "casting from this type is unsupported".into();
                self.push_err(TypeCheckError::InvalidCast { from, location, reason });
                return Type::Error;
            }
        };

        // Warn if a user casts to an integer from a negative field literal.
        // `-1 as i8 == 0`, not `-1` which can be confusing.
        if let Some(value) = &from_value_opt
            && *value < BigInt::ZERO
            && to.is_integer()
            && (from_follow_bindings.is_field() || from_follow_bindings.is_bindable())
            && let Ok(Value::Integer(result)) =
                evaluate_cast_one_step(&to, location, Value::field(bigint_to_field(value)))
        {
            self.push_err(TypeCheckError::NegativeLiteralCastToInteger {
                value: value.clone(),
                result: result.to_string(),
                to: to.clone(),
                location,
            });
        }

        // when casting a polymorphic value to a specifically sized type,
        // check that it fits or throw a warning
        if let (Some(from_value), Some(to_maximum_size)) =
            (from_value_opt, to.integral_maximum_size())
            && from_is_polymorphic
            && from_value >= BigInt::ZERO
            && from_value <= BigInt::from(u128::MAX)
            && from_value > BigInt::from(to_maximum_size)
        {
            let from = from.clone();
            let to = to.clone();
            let reason = format!(
                "casting untyped value ({from_value}) to a type with a maximum size ({to_maximum_size}) that's smaller than it"
            );
            // we warn that the 'to' type is too small for the value
            self.push_err(TypeCheckError::DownsizingCast { from, to, location, reason });
        }

        match to {
            Type::Integer(sign, bits) => Type::Integer(sign, bits),
            Type::FieldElement => {
                // Deferred to the end of the function so the source type's bindings have
                // settled. If `from` is still a polymorphic type variable at this point it
                // may later be constrained to a signed integer, which only an end-of-function
                // check can detect.
                self.push_signed_to_field_cast(from.clone(), location);

                Type::FieldElement
            }
            Type::Bool => {
                // Deferred for the same reason as the FieldElement branch above: `from` may
                // still be polymorphic here and only later resolve to a numeric type.
                self.push_numeric_to_bool_cast(from.clone(), location);

                Type::Bool
            }
            Type::Error => Type::Error,
            _ => {
                self.push_err(TypeCheckError::UnsupportedCast { location });
                Type::Error
            }
        }
    }

    /// Checks that two integer operands of a binary operator agree in both
    /// signedness and bit width, reporting the first mismatch as an error.
    fn check_integer_operands_match(
        sign_x: Signedness,
        bit_width_x: IntegerBitSize,
        sign_y: Signedness,
        bit_width_y: IntegerBitSize,
        location: Location,
    ) -> Result<(), TypeCheckError> {
        if sign_x != sign_y {
            return Err(TypeCheckError::IntegerSignedness { sign_x, sign_y, location });
        }
        if bit_width_x != bit_width_y {
            return Err(TypeCheckError::IntegerBitWidth { bit_width_x, bit_width_y, location });
        }
        Ok(())
    }

    /// Given a binary comparison operator and another type. This method will produce the output type
    /// and a boolean indicating whether to use the trait impl corresponding to the operator
    /// or not. A value of false indicates the caller to use a primitive operation for this
    /// operator, while a true value indicates a user-provided trait impl is required.
    #[tracing::instrument(level = "trace", skip_all)]
    fn comparator_operand_type_rules(
        &mut self,
        lhs_type: &Type,
        rhs_type: &Type,
        op: &HirBinaryOp,
        location: Location,
    ) -> Result<(Type, bool), TypeCheckError> {
        use Type::*;

        match (lhs_type, rhs_type) {
            // Avoid reporting errors multiple times
            (Error, _) | (_, Error) => Ok((Bool, false)),
            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                self.comparator_operand_type_rules(&alias, other, op, location)
            }

            // Matches on TypeVariable must be first to follow any type
            // bindings.
            (TypeVariable(var), other) | (other, TypeVariable(var)) => {
                if let TypeBinding::Bound(binding) = &*var.borrow() {
                    return self.comparator_operand_type_rules(other, binding, op, location);
                }

                let use_impl = self.bind_type_variables_for_infix(lhs_type, op, rhs_type, location);
                Ok((Bool, use_impl))
            }
            (Integer(sign_x, bit_width_x), Integer(sign_y, bit_width_y)) => {
                Self::check_integer_operands_match(
                    *sign_x,
                    *bit_width_x,
                    *sign_y,
                    *bit_width_y,
                    location,
                )?;
                Ok((Bool, false))
            }
            (FieldElement, FieldElement) => {
                if op.kind.is_valid_for_field_type() {
                    Ok((Bool, false))
                } else {
                    Err(TypeCheckError::FieldComparison { location })
                }
            }

            // <= and friends are technically valid for booleans, just not very useful
            (Bool, Bool) => Ok((Bool, false)),

            (lhs, rhs) => {
                self.unify_or_type_mismatch_with_source(rhs, lhs, Source::Binary, op.location);
                Ok((Bool, true))
            }
        }
    }

    /// Handles the `TypeVariable` case for checking binary operators.
    /// Returns true if we should use the impl for the operator instead of the primitive
    /// version of it.
    #[tracing::instrument(level = "trace", skip_all)]
    fn bind_type_variables_for_infix(
        &mut self,
        lhs_type: &Type,
        op: &HirBinaryOp,
        rhs_type: &Type,
        location: Location,
    ) -> bool {
        self.unify(lhs_type, rhs_type, |elaborator| {
            elaborator.new_type_mismatch_with_source_error(
                rhs_type,
                lhs_type,
                Source::Binary,
                location,
            )
        });

        let use_impl = !lhs_type.is_numeric_value();

        // If this operator isn't valid for fields we have to possibly narrow
        // Kind::IntegerOrField to Kind::Integer.
        // Doing so also ensures a type error if Field is used.
        // The is_numeric check is to allow impls for custom types to bypass this.
        if !op.kind.is_valid_for_field_type() && lhs_type.is_numeric_value() {
            let target = self.polymorphic_integer();

            use crate::ast::BinaryOpKind::*;
            use TypeCheckError::*;
            self.unify(lhs_type, &target, |_| match op.kind {
                Less | LessEqual | Greater | GreaterEqual => FieldComparison { location },
                And | Or | Xor | ShiftRight | ShiftLeft => FieldBitwiseOp { location },
                Modulo => FieldModulo { location },
                other => unreachable!("Operator {other:?} should be valid for Field"),
            });
        }

        use_impl
    }

    /// Given a binary operator and another type, this method will produce the output type
    /// and a boolean indicating whether to use the trait impl corresponding to the operator
    /// or not. A value of false indicates the caller to use a primitive operation for this
    /// operator, while a true value indicates a user-provided trait impl is required.
    ///
    /// Returns an `Err` if the operator cannot be applied on the argument types,
    /// or if the arguments are incompatible with each other.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn infix_operand_type_rules(
        &mut self,
        lhs_type: &Type,
        op: &HirBinaryOp,
        rhs_type: &Type,
        location: Location,
    ) -> Result<(Type, bool), TypeCheckError> {
        if op.kind.is_comparator() {
            return self.comparator_operand_type_rules(lhs_type, rhs_type, op, location);
        }

        use Type::*;
        match (lhs_type, rhs_type) {
            // An error type on either side will always return an error
            (Error, _) | (_, Error) => Ok((Error, false)),
            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                self.infix_operand_type_rules(&alias, op, other, location)
            }

            // Matches on TypeVariable must be first so that we follow any type
            // bindings.
            (TypeVariable(int), other) | (other, TypeVariable(int)) => {
                if let TypeBinding::Bound(binding) = &*int.borrow() {
                    return self.infix_operand_type_rules(binding, op, other, location);
                }
                let use_impl = self.bind_type_variables_for_infix(lhs_type, op, rhs_type, location);
                Ok((other.clone(), use_impl))
            }
            (Integer(sign_x, bit_width_x), Integer(sign_y, bit_width_y)) => {
                Self::check_integer_operands_match(
                    *sign_x,
                    *bit_width_x,
                    *sign_y,
                    *bit_width_y,
                    location,
                )?;
                Ok((Integer(*sign_x, *bit_width_x), false))
            }
            // The result of two Fields is always a witness
            (FieldElement, FieldElement) => {
                if !op.kind.is_valid_for_field_type() {
                    if op.kind == BinaryOpKind::Modulo {
                        return Err(TypeCheckError::FieldModulo { location });
                    } else {
                        return Err(TypeCheckError::FieldBitwiseOp { location });
                    }
                }
                Ok((FieldElement, false))
            }

            (Bool, Bool) => match op.kind {
                BinaryOpKind::Add
                | BinaryOpKind::Subtract
                | BinaryOpKind::Multiply
                | BinaryOpKind::Divide
                | BinaryOpKind::ShiftRight
                | BinaryOpKind::ShiftLeft
                | BinaryOpKind::Modulo => {
                    Err(TypeCheckError::InvalidBoolInfixOp { op: op.kind, location })
                }
                BinaryOpKind::Equal
                | BinaryOpKind::NotEqual
                | BinaryOpKind::Less
                | BinaryOpKind::LessEqual
                | BinaryOpKind::Greater
                | BinaryOpKind::GreaterEqual
                | BinaryOpKind::And
                | BinaryOpKind::Or
                | BinaryOpKind::Xor => Ok((Bool, false)),
            },

            (lhs, rhs) => {
                self.unify_or_type_mismatch_with_source(rhs, lhs, Source::Binary, op.location);
                Ok((lhs.clone(), true))
            }
        }
    }

    /// Given a unary operator and a type, this method will produce the output type
    /// and a boolean indicating whether to use the trait impl corresponding to the operator
    /// or not. A value of false indicates to the caller to use a primitive operation for this
    /// operator, while a true value indicates a user-provided trait impl is required.
    ///
    /// Returns `Err` if the type cannot be used with the given unary operator.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn prefix_operand_type_rules(
        &mut self,
        op: &UnaryOp,
        rhs_type: &Type,
        location: Location,
    ) -> Result<(Type, bool), TypeCheckError> {
        use Type::*;

        match op {
            UnaryOp::Minus | UnaryOp::Not => {
                match rhs_type {
                    // An error type will always return an error
                    Error => Ok((Error, false)),
                    Alias(alias, args) => {
                        let alias = alias.borrow().get_type(args);
                        self.prefix_operand_type_rules(op, &alias, location)
                    }

                    // Matches on TypeVariable must be first so that we follow any type
                    // bindings.
                    TypeVariable(int) => {
                        if let TypeBinding::Bound(binding) = &*int.borrow() {
                            return self.prefix_operand_type_rules(op, binding, location);
                        }

                        // The `!` prefix operator is not valid for Field, so if this is a numeric
                        // type we constrain it to just (non-Field) integer types.
                        if matches!(op, UnaryOp::Not) && rhs_type.is_numeric_value() {
                            let integer_type = Type::polymorphic_integer(self.interner);
                            self.unify(rhs_type, &integer_type, |_| {
                                TypeCheckError::InvalidUnaryOp {
                                    typ: rhs_type.to_string(),
                                    operator: "!",
                                    location,
                                }
                            });
                        }

                        Ok((rhs_type.clone(), !rhs_type.is_numeric_value()))
                    }
                    Integer(sign_x, bit_width_x) => {
                        if *op == UnaryOp::Minus && *sign_x == Signedness::Unsigned {
                            return Err(TypeCheckError::InvalidUnaryOp {
                                typ: rhs_type.to_string(),
                                operator: "-",
                                location,
                            });
                        }
                        Ok((Integer(*sign_x, *bit_width_x), false))
                    }
                    // The result of a Field is always a witness
                    FieldElement => {
                        if *op == UnaryOp::Not {
                            return Err(TypeCheckError::FieldNot { location });
                        }
                        Ok((FieldElement, false))
                    }

                    Bool => {
                        if *op == UnaryOp::Minus {
                            return Err(TypeCheckError::InvalidUnaryOp {
                                typ: rhs_type.to_string(),
                                operator: "-",
                                location,
                            });
                        }
                        Ok((Bool, false))
                    }

                    _ => Ok((rhs_type.clone(), true)),
                }
            }
            UnaryOp::Reference { mutable } => {
                let typ = Reference(Box::new(rhs_type.follow_bindings()), *mutable);
                Ok((typ, false))
            }
            UnaryOp::Dereference { implicitly_added: _ } => {
                let element_type = self.interner.next_type_variable();
                let make_expected = |mutable| Reference(Box::new(element_type.clone()), mutable);

                let immutable = make_expected(false);
                let mutable = make_expected(true);

                // Both `&mut T` and `&T` should coerce to an expected `&T`.
                if !rhs_type.try_reference_coercion(&immutable) {
                    self.unify_or_type_mismatch(rhs_type, &mutable, location);
                }
                Ok((element_type, false))
            }
        }
    }

    /// Prerequisite: the operator's trait constraint has already been solved via the trait
    /// constraint machinery (see `check_trait_constraints`).
    ///
    /// Although by this point the operator is expected to already have a trait impl,
    /// we still need to match the operator's type against the method's instantiated type
    /// to ensure the instantiation bindings are correct and the monomorphizer can
    /// re-apply the needed bindings.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn type_check_operator_method(
        &mut self,
        expr_id: ExprId,
        trait_method_id: TraitItemId,
        object_type: &Type,
        return_type: &Type,
        location: Location,
        is_ord: bool,
    ) {
        let method_type = self.interner.definition_type(trait_method_id.item_id);
        let (method_type, bindings) = method_type.instantiate(self.interner);

        match method_type {
            Type::Function(mut args, ret, env, _unconstrained) => {
                assert!(
                    !args.is_empty(),
                    "type_check_operator_method ICE: expected operator method to have at least one argument type"
                );

                self.unify_or_type_mismatch(&env, &Type::Unit, location);

                // Uses of `Ord` that return `bool`, e.g. `<`, `<=`, etc., are expected to have
                // a `return_type` of `bool`, but have a `ret` of type `std::cmp::Ordering`
                // from being based on `Ord::cmp(self, other: Self) -> Ordering`
                if is_ord {
                    let mut ordering_type_path_segments = vec![];
                    let ordering_type_path_kind = if self.crate_id.is_stdlib() {
                        PathKind::Crate
                    } else {
                        ordering_type_path_segments.push(TypedPathSegment::without_generics(
                            Ident::new("std".to_string(), location),
                            location,
                        ));
                        PathKind::Absolute
                    };
                    ordering_type_path_segments.push(TypedPathSegment::without_generics(
                        Ident::new("cmp".to_string(), location),
                        location,
                    ));
                    ordering_type_path_segments.push(TypedPathSegment::without_generics(
                        Ident::new("Ordering".to_string(), location),
                        location,
                    ));
                    let ordering_type_path = TypedPath {
                        segments: ordering_type_path_segments,
                        kind: ordering_type_path_kind,
                        location,
                        kind_location: location,
                    };
                    let ordering_type = self.resolve_named_type(
                        ordering_type_path,
                        GenericTypeArgs::default(),
                        PathResolutionMode::MarkAsReferenced,
                        WildcardAllowed::No(WildcardDisallowedContext::FunctionReturn),
                    );

                    self.unify_or_type_mismatch(return_type, &Type::Bool, location);
                    self.unify_or_type_mismatch(&ret, &ordering_type, location);
                } else {
                    self.unify_or_type_mismatch(&ret, return_type, location);
                }

                let expected_object_type = args.pop().unwrap_or_else(|| {
                    unreachable!("ICE: expected operator method on {object_type} to take arguments, but found no arguments")
                });
                for arg in args {
                    self.unify_or_type_mismatch(&arg, &expected_object_type, location);
                }

                self.unify_or_type_mismatch(object_type, &expected_object_type, location);
            }
            Type::Error => {
                self.push_err(TypeCheckError::expecting_other_error(
                    "type_check_operator_method: encountered method_type of type 'error'",
                    location,
                ));
            }
            other => {
                unreachable!(
                    "Expected operator method on {object_type} to have a function type, but found {other}"
                )
            }
        }

        // The expr_id is freshly created by the caller and the trait constraint is deferred
        // to function end (check_and_pop_function_context), so no impl should be selected yet.
        assert!(
            self.interner.get_selected_impl_for_expression(expr_id).is_none(),
            "type_check_operator_method: expected no impl to be selected yet for this expression"
        );

        self.interner.store_instantiation_bindings(expr_id, bindings);
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn type_check_member_access(
        &mut self,
        mut access: HirMemberAccess,
        expr_id: ExprId,
        lhs_type: Type,
        location: Location,
    ) -> Type {
        let access_lhs = &mut access.lhs;

        let dereference_lhs = |this: &mut Self, lhs_type, element, _is_mutable| {
            let old_lhs = *access_lhs;
            let old_location = this.interner.id_location(old_lhs);
            let location = Location::new(location.span, old_location.file);

            *access_lhs = this.interner.push_expr_full(
                HirExpression::Prefix(HirPrefixExpression::new(
                    UnaryOp::Dereference { implicitly_added: true },
                    old_lhs,
                )),
                location,
                element,
            );

            this.interner.push_expr_type(old_lhs, lhs_type);
        };

        // If this access is just a field offset, we want to avoid dereferencing
        let dereference_lhs = (!access.is_offset).then_some(dereference_lhs);

        match self.check_field_access(&lhs_type, access.rhs.as_str(), location, dereference_lhs) {
            Some((element_type, index)) => {
                self.interner.set_field_index(expr_id, index);
                // We must update `access` in case we added any dereferences to it
                self.interner.replace_expr(&expr_id, HirExpression::MemberAccess(access));
                element_type
            }
            None => Type::Error,
        }
    }

    /// Type checks a field access, adding dereference operators as necessary
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn check_field_access(
        &mut self,
        lhs_type: &Type,
        field_name: &str,
        location: Location,
        dereference_lhs: Option<impl FnMut(&mut Self, Type, Type, bool /* mutable */)>,
    ) -> Option<(Type, usize)> {
        let lhs_type = lhs_type.follow_bindings();

        match &lhs_type {
            Type::DataType(s, args) => {
                let type_id = s.borrow().id;
                self.define_struct_fields_if_undefined(type_id);
                let s = s.borrow();
                if let Some((field, visibility, index)) = s.get_field(field_name, args) {
                    self.interner.add_struct_member_reference(s.id, index, location);

                    self.check_struct_field_visibility(&s, field_name, visibility, location);

                    return Some((field, index));
                }
            }
            Type::Tuple(elements) => {
                if let Ok(index) = field_name.parse::<usize>() {
                    let length = elements.len();
                    if index < length {
                        return Some((elements[index].clone(), index));
                    } else {
                        self.push_err(TypeCheckError::TupleIndexOutOfBounds {
                            index,
                            lhs_type,
                            length,
                            location,
                        });
                        return None;
                    }
                }
            }
            // If the lhs is a reference we automatically transform `lhs.field` into `(*lhs).field`
            Type::Reference(element, mutable) => {
                if let Some(mut dereference_lhs) = dereference_lhs {
                    dereference_lhs(self, lhs_type.clone(), element.as_ref().clone(), *mutable);
                    return self.check_field_access(
                        element,
                        field_name,
                        location,
                        Some(dereference_lhs),
                    );
                } else {
                    let (element, index) =
                        self.check_field_access(element, field_name, location, dereference_lhs)?;
                    return Some((Type::Reference(Box::new(element), *mutable), index));
                }
            }
            _ => (),
        }

        // If we get here the type has no field named 'access.rhs'.
        // Now we specialize the error message based on whether we know the object type in question yet.
        if let Type::TypeVariable(..) = &lhs_type {
            self.push_err(TypeCheckError::TypeAnnotationsNeededForFieldAccess { location });
        } else if lhs_type != Type::Error {
            self.push_err(TypeCheckError::AccessUnknownMember {
                lhs_type,
                field_name: field_name.to_string(),
                location,
            });
        }

        None
    }

    /// Try to look up a method on a [Type] by name:
    /// * if the object type is generic, look it up in the trait constraints
    /// * otherwise look it up directly on the type, or in traits the type implements
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn lookup_method(
        &mut self,
        object_type: &Type,
        method_name: &str,
        location: Location,
        object_location: Location,
        check_self_param: bool,
    ) -> Option<HirMethodReference> {
        match object_type.follow_bindings() {
            // TODO(https://github.com/noir-lang/noir/issues/10518): We should allow method calls on
            // `impl Trait`s eventually. For now it is fine since they are only allowed on return types.
            Type::TraitAsType(..) => {
                self.push_err(TypeCheckError::UnresolvedMethodCall {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    location,
                });
                None
            }
            Type::NamedGeneric(_) => self.lookup_method_in_trait_constraints(
                object_type,
                method_name,
                location,
                object_location,
            ),
            // `DefCollectorErrorKind::ReferenceInTraitImpl`: "Trait impls are not allowed on reference types"
            // References to another type should resolve to methods of their element type.
            // This may be a data type or a primitive type.
            Type::Reference(element, _mutable) => self.lookup_method(
                &element,
                method_name,
                location,
                object_location,
                check_self_param,
            ),

            // If we fail to resolve the object to a data type, we have no way of type
            // checking its arguments as we can't even resolve the name of the function
            Type::Error => None,

            // The type variable must be unbound at this point since follow_bindings was called
            Type::TypeVariable(var) if var.kind() == Kind::Normal => {
                self.push_err(TypeCheckError::TypeAnnotationsNeededForMethodCall { location });
                None
            }

            other => self.lookup_type_or_primitive_method(
                &other,
                method_name,
                location,
                object_location,
                check_self_param,
            ),
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn lookup_type_or_primitive_method(
        &mut self,
        object_type: &Type,
        method_name: &str,
        location: Location,
        object_location: Location,
        check_self_param: bool,
    ) -> Option<HirMethodReference> {
        // First search in the type methods. A directly-defined (inherent) method that is
        // visible from here always wins. If it exists but is not visible, we do not commit to
        // it yet: an accessible trait method may exist that should be called instead. We keep
        // the inherent method around as a fallback so that, if no trait method resolves, the
        // caller still reports the appropriate "private" visibility error against it.
        let direct_method = self.lookup_direct_method(object_type, method_name, check_self_param);
        if let Some(method_id) = direct_method
            && self.method_call_is_visible(method_id, object_type)
        {
            return Some(HirMethodReference::FuncId(method_id));
        }

        // Next lookup all matching trait methods.
        let trait_methods = self.lookup_trait_methods(object_type, method_name, check_self_param);

        // If there's at least one matching trait method we need to see if only one is in scope.
        if !trait_methods.is_empty() {
            return self.return_trait_method_in_scope(&trait_methods, method_name, location);
        }

        // If we couldn't find any trait methods, search in
        // impls for all types `T`, e.g. `impl<T> Foo for T`
        let generic_methods =
            self.lookup_generic_methods(object_type, method_name, check_self_param);
        if !generic_methods.is_empty() {
            return self.return_trait_method_in_scope(&generic_methods, method_name, location);
        }

        // No trait method applies. Fall back to the inaccessible inherent method (if any) so the
        // caller reports its visibility error rather than a misleading "method not found".
        if let Some(method_id) = direct_method {
            return Some(HirMethodReference::FuncId(method_id));
        }

        // It could be that this type is a composite type that is bound to a trait,
        // for example `x: (T, U) ... where (T, U): SomeTrait`
        // (so this case is a generalization of the NamedGeneric case)
        self.lookup_method_in_trait_constraints(object_type, method_name, location, object_location)
    }

    /// Given a list of functions and the trait they belong to, returns the one function
    /// that is in scope.
    #[tracing::instrument(level = "trace", skip_all)]
    fn return_trait_method_in_scope(
        &mut self,
        trait_methods: &[(FuncId, TraitId, Type)],
        method_name: &str,
        location: Location,
    ) -> Option<HirMethodReference> {
        let (method, error) = self.get_trait_method_in_scope(trait_methods, method_name, location);
        if let Some(error) = error {
            self.push_err(error);
        }
        method
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn get_trait_method_in_scope(
        &mut self,
        trait_methods: &[(FuncId, TraitId, Type)],
        method_name: &str,
        location: Location,
    ) -> (Option<HirMethodReference>, Option<PathResolutionError>) {
        let module_id = self.module_id();
        let module_data = self.get_module(module_id);

        // Only keep unique trait IDs: multiple trait methods might come from the same trait
        // but implemented with different generics (like `Convert<Field>` and `Convert<i32>`).
        let traits: HashSet<TraitId> =
            trait_methods.iter().map(|(_, trait_id, _)| *trait_id).collect();

        let traits_in_scope: Vec<_> = traits
            .iter()
            .filter_map(|trait_id| {
                module_data.find_trait_in_scope(*trait_id).map(|name| (*trait_id, name.clone()))
            })
            .collect();

        for (_, trait_name) in &traits_in_scope {
            self.usage_tracker.mark_as_used(module_id, trait_name, Namespace::Type);
        }

        if traits_in_scope.is_empty() {
            if traits.len() == 1 {
                // This is the backwards-compatible case where there's a single trait but it's not in scope
                let trait_id = *traits.iter().next().unwrap();
                let trait_name = self.fully_qualified_trait_path_by_id(trait_id);
                let method =
                    self.trait_hir_method_reference(trait_id, trait_methods, method_name, location);
                let error = PathResolutionError::TraitMethodNotInScope {
                    ident: Ident::new(method_name.into(), location),
                    trait_name,
                };
                return (Some(method), Some(error));
            } else {
                let traits =
                    vecmap(traits, |trait_id| self.fully_qualified_trait_path_by_id(trait_id));
                let method_not_found = None;
                let error = PathResolutionError::UnresolvedWithPossibleTraitsToImport {
                    ident: Ident::new(method_name.into(), location),
                    traits,
                };
                return (method_not_found, Some(error));
            }
        }

        if traits_in_scope.len() > 1 {
            let traits = vecmap(&traits_in_scope, |(trait_id, _)| {
                self.fully_qualified_trait_path_by_id(*trait_id)
            });
            let method_not_found = None;
            let error = PathResolutionError::MultipleTraitsInScope {
                ident: Ident::new(method_name.into(), location),
                traits,
            };
            return (method_not_found, Some(error));
        }

        let trait_id = traits_in_scope[0].0;
        let method =
            self.trait_hir_method_reference(trait_id, trait_methods, method_name, location);
        let error = None;
        (Some(method), error)
    }

    fn trait_hir_method_reference(
        &self,
        trait_id: TraitId,
        trait_methods: &[(FuncId, TraitId, Type)],
        method_name: &str,
        location: Location,
    ) -> HirMethodReference {
        // If we find a single trait impl method, return it so we don't have to later determine the impl.
        // Exception: if the single match points at the trait's own method declaration (which happens
        // when an impl inherits the default body — the impl's slot shares the trait's `FuncId`),
        // the function meta carries the trait's `Self` type variable, not the impl's concrete type.
        // Fall through to the `TraitItemId` path so the caller can bind `Self` to the resolved
        // self type via the trait constraint.
        if trait_methods.len() == 1
            && self
                .interner
                .try_function_meta(&trait_methods[0].0)
                .is_none_or(|meta| meta.trait_id.is_none())
        {
            let (func_id, _, _) = &trait_methods[0];
            return HirMethodReference::FuncId(*func_id);
        }

        // Return a TraitMethodId with unbound generics. These will later be bound by the type-checker.
        let trait_ = self.interner.get_trait(trait_id);
        let trait_generics = trait_.get_trait_generics(location);
        let definition = trait_.find_method(method_name, self.interner).unwrap();
        let assumed = false;
        HirMethodReference::TraitItemId(HirTraitMethodReference {
            definition,
            trait_id,
            trait_generics,
            assumed,
        })
    }

    /// Assuming that we are currently elaborating a function, try to look up a method in:
    /// * the trait the function belongs to, if the object is the self-type of the method, or
    /// * in any of the traits which appear in the constraints of the function
    ///
    /// Pushes an error if the method cannot be found.
    #[tracing::instrument(level = "trace", skip_all)]
    fn lookup_method_in_trait_constraints(
        &mut self,
        object_type: &Type,
        method_name: &str,
        location: Location,
        object_location: Location,
    ) -> Option<HirMethodReference> {
        let Some(DependencyId::Function(func_id)) = self.current_item else {
            // Unexpected method outside a function.
            self.push_err(TypeCheckError::UnresolvedMethodCall {
                method_name: method_name.to_string(),
                object_type: object_type.clone(),
                location,
            });
            return None;
        };

        // The function we are elaborating, ie. where we make the method call from.
        let (func_meta_trait_id, func_trait_constraints) = self
            .with_function_meta(func_id, |meta| {
                (meta.trait_id, meta.all_trait_constraints().cloned().collect::<Vec<_>>())
            });

        // If inside a trait method, check if it's a method on `self`
        if let Some(trait_id) = func_meta_trait_id
            && Some(object_type) == self.self_type.as_ref()
        {
            let the_trait = self.interner.get_trait(trait_id);
            let constraint = the_trait.as_constraint(the_trait.name.location());
            let mut visited = BTreeSet::new();
            let mut matches = self.lookup_methods_in_trait(
                the_trait,
                method_name,
                &constraint.trait_bound,
                &mut visited,
            );
            if matches.len() == 1 {
                let method = matches.remove(0);
                let assumed = true;
                // If it is, it's an assumed trait
                // Note that here we use the `trait_id` from `TraitItemId` because looking a method on a trait
                // might return a method on a parent trait.
                return Some(HirMethodReference::TraitItemId(HirTraitMethodReference {
                    assumed,
                    ..method
                }));
            }
            if matches.len() > 1 {
                return self.handle_trait_method_lookup_matches(
                    object_type,
                    method_name,
                    location,
                    object_location,
                    matches,
                );
            }
        }

        let mut matches = Vec::new();
        let mut visited = BTreeSet::new();

        for constraint in &func_trait_constraints {
            if *object_type == constraint.typ
                && let Some(the_trait) =
                    self.interner.try_get_trait(constraint.trait_bound.trait_id)
            {
                matches.extend(self.lookup_methods_in_trait(
                    the_trait,
                    method_name,
                    &constraint.trait_bound,
                    &mut visited,
                ));
            }
        }

        self.handle_trait_method_lookup_matches(
            object_type,
            method_name,
            location,
            object_location,
            matches,
        )
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn handle_trait_method_lookup_matches(
        &mut self,
        object_type: &Type,
        method_name: &str,
        location: Location,
        object_location: Location,
        mut matches: Vec<HirTraitMethodReference>,
    ) -> Option<HirMethodReference> {
        if matches.len() == 1 {
            return Some(HirMethodReference::TraitItemId(matches.remove(0)));
        }

        if matches.len() > 1 {
            let ident = Ident::new(method_name.to_string(), location);
            let traits =
                vecmap(matches, |method| self.fully_qualified_trait_path_by_id(method.trait_id));
            self.push_err(PathResolutionError::MultipleTraitsInScope { ident, traits });
            return None;
        }

        if object_type.is_bindable() {
            self.push_err(TypeCheckError::TypeAnnotationsNeededForMethodCall {
                location: object_location,
            });
            return None;
        }

        // Check if it's `foo.bar()` where `bar` is a member of the struct `foo`.
        // In that case we tell the user that they need to write it like `(foo.bar)()`.
        if let Type::DataType(datatype, _) = object_type {
            self.define_struct_fields_if_undefined(datatype.borrow().id);
            let datatype = datatype.borrow();
            let has_field_with_function_type = datatype.fields_raw().is_some_and(|fields| {
                fields
                    .iter()
                    .any(|field| field.name.as_str() == method_name && field.typ.is_function())
            });
            if has_field_with_function_type {
                self.push_err(TypeCheckError::CannotInvokeStructFieldFunctionType {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    location,
                });
                return None;
            }
        }

        self.push_err(TypeCheckError::UnresolvedMethodCall {
            method_name: method_name.to_string(),
            object_type: object_type.clone(),
            location,
        });

        None
    }

    /// Looks up a method in the given trait and its parent traits, recursively.
    /// Multiple matches are possible if a method with the same name exists in, for example,
    /// a child and its parent.
    fn lookup_methods_in_trait(
        &self,
        the_trait: &Trait,
        method_name: &str,
        trait_bound: &ResolvedTraitBound,
        visited: &mut BTreeSet<TraitId>,
    ) -> Vec<HirTraitMethodReference> {
        // Skip if we've already visited this trait.
        if !visited.insert(the_trait.id) {
            return Vec::new();
        }

        let mut matches = Vec::new();

        if let Some(trait_method) = the_trait.find_method(method_name, self.interner) {
            let trait_generics = trait_bound.trait_generics.clone();
            let assumed = false;
            let trait_method = HirTraitMethodReference {
                definition: trait_method,
                trait_id: the_trait.id,
                trait_generics,
                assumed,
            };
            matches.push(trait_method);
        }

        // Search in the parent traits, if any.
        let parent_bounds: Vec<_> = the_trait.parent_bounds().cloned().collect();
        for parent_trait_bound in &parent_bounds {
            // Parent bound trait ids are set during trait resolution and must always resolve;
            // `get_trait` turns a violation into a clear internal error instead of silently
            // skipping the parent trait's methods.
            let the_trait = self.interner.get_trait(parent_trait_bound.trait_id);
            let parent_trait_bound =
                self.instantiate_parent_trait_bound(trait_bound, parent_trait_bound);
            matches.extend(self.lookup_methods_in_trait(
                the_trait,
                method_name,
                &parent_trait_bound,
                visited,
            ));
        }

        matches
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn type_check_call(
        &mut self,
        call: &HirCallExpression,
        func_type: Type,
        args: Vec<(Type, ExprId, Location)>,
        location: Location,
    ) -> Type {
        self.run_lint(|elaborator| {
            lints::deprecated_function(elaborator.interner, call.func).map(Into::into)
        });

        let crossing_runtime_boundary =
            self.check_call_runtime_boundary(call.func, &func_type, &args, location);

        let return_type = self.bind_function_type(func_type, args, location);

        if crossing_runtime_boundary {
            self.check_unconstrained_call_return(&return_type, location);
        }

        return_type
    }

    /// Re-runs the runtime-mode-dependent validity checks for a call against the *current*
    /// elaboration context: that `verify_proof_with_type` is not reached from an unconstrained
    /// context, and that a constrained function only reaches an unconstrained one from within an
    /// `unsafe` block (with its arguments suitably constrained).
    ///
    /// Returns whether the call crosses the constrained/unconstrained boundary, in which case the
    /// caller must also validate the return type with [`Self::check_unconstrained_call_return`]
    /// once it is known.
    ///
    /// This is shared between regular call type-checking and the revalidation of resolved
    /// expressions spliced in from a comptime `Expr::resolve`, which were originally elaborated in
    /// a different context (see [`Self::revalidate_resolved_expression`]).
    pub(super) fn check_call_runtime_boundary(
        &mut self,
        func: ExprId,
        func_type: &Type,
        args: &[(Type, ExprId, Location)],
        location: Location,
    ) -> bool {
        let is_current_func_constrained = self.in_constrained_function();
        if !is_current_func_constrained {
            // Check if we're calling verify_proof_with_type in an unconstrained context
            self.run_lint(|elaborator| {
                lints::error_if_verify_proof_with_type(elaborator.interner, func, location)
            });
        }

        let func_type_is_unconstrained =
            if let Type::Function(_args, _ret, _env, unconstrained) = func_type {
                *unconstrained
            } else {
                false
            };

        let func_is_unconstrained_call = match self.is_unconstrained_call(func, location) {
            Ok(result) => result,
            Err(error) => {
                self.push_err(error);
                false
            }
        };
        let is_unconstrained_call = func_type_is_unconstrained || func_is_unconstrained_call;
        let crossing_runtime_boundary = is_current_func_constrained && is_unconstrained_call;

        if crossing_runtime_boundary {
            match self.unsafe_block_status {
                UnsafeBlockStatus::NotInUnsafeBlock => {
                    self.push_err(TypeCheckError::Unsafe { location });
                }
                UnsafeBlockStatus::InUnsafeBlockWithoutUnconstrainedCalls => {
                    self.unsafe_block_status =
                        UnsafeBlockStatus::InUnsafeBlockWithUnconstrainedCalls;
                }
                UnsafeBlockStatus::InUnsafeBlockWithUnconstrainedCalls => (),
            }

            // Resolve any deferred struct fields reachable through the arg
            // types so the boundary validity check sees real fields instead
            // of stub `StructWithUnknownFields`. Without this, an unresolved
            // struct is misread as an enum (`get_fields` returns `None`) and
            // the check reports a spurious "mutable reference" error. This
            // matters inside recursive `elaborate_items` (from `run_attributes`)
            // where outer-pending structs haven't been drained yet.
            for (typ, _, _) in args {
                self.define_deferred_data_types_in(typ);
            }

            let errors = lints::unconstrained_function_args(args);
            self.push_errors(errors);
        }

        crossing_runtime_boundary
    }

    /// Companion to [`Self::check_call_runtime_boundary`]: validates the return type of a call that
    /// crosses the constrained/unconstrained boundary.
    pub(super) fn check_unconstrained_call_return(
        &mut self,
        return_type: &Type,
        location: Location,
    ) {
        self.define_deferred_data_types_in(return_type);
        self.run_lint(|_| {
            lints::unconstrained_function_return(return_type, location).map(Into::into)
        });
    }

    /// Check if the callee is an unconstrained function, or a variable referring to one.
    fn is_unconstrained_call(
        &self,
        expr: ExprId,
        location: Location,
    ) -> Result<bool, CompilationError> {
        // `try_function_meta` rather than `function_meta`: the call may happen while
        // the callee's meta is still mid-resolution (e.g. a function whose signature
        // transitively calls itself through a global). A dependency-cycle error has
        // already been pushed in that case; treat the call as constrained to avoid a
        // panic.
        if let Some(func_id) = self.interner.lookup_function_from_expr(&expr, location)?
            && let Some(meta) = self.interner.try_function_meta(&func_id)
        {
            Ok(meta.is_unconstrained())
        } else {
            Ok(false)
        }
    }

    /// Check if the given method type requires a mutable reference to the object type, and check
    /// if the given object type is already a mutable reference. If not, add one.
    /// This is used to automatically transform a method call: `foo.bar()` into a function
    /// call: `bar(&mut foo)`.
    ///
    /// A notable corner case of this function is where it interacts with auto-deref of `.`.
    /// If a field is being mutated e.g. `foo.bar.mutate_bar()` where `foo: &mut Foo`, the compiler
    /// will insert a dereference before bar `(*foo).bar.mutate_bar()` which would cause us to
    /// mutate a copy of bar rather than a reference to it. We must check for this corner case here
    /// and remove the implicitly added dereference operator if we find one.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn try_add_mutable_reference_to_object(
        &mut self,
        function_type: &Type,
        object_type: &mut Type,
        object: &mut ExprId,
    ) {
        let expected_object_type = match function_type {
            Type::Function(args, _, _, _) => args.first(),
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(args, _, _, _) => args.first(),
                typ => unreachable!("Unexpected type for function: {typ}"),
            },
            typ => unreachable!("Unexpected type for function: {typ}"),
        };

        if let Some(expected_object_type) = expected_object_type {
            let actual_type = object_type.follow_bindings();

            if let Type::Reference(_, mutable) = expected_object_type.follow_bindings() {
                if !matches!(actual_type, Type::Reference(..)) {
                    let location = self.interner.id_location(*object);
                    if mutable {
                        self.check_can_mutate(*object, location);
                    }

                    let new_type = Type::Reference(Box::new(actual_type), mutable);
                    *object_type = new_type.clone();

                    // First try to remove a dereference operator that may have been implicitly
                    // inserted by a field access expression `foo.bar` on a mutable reference `foo`.
                    let new_object = self.try_remove_implicit_dereference(*object);

                    // If that didn't work, then wrap the whole expression in an `&mut`
                    *object = new_object.unwrap_or_else(|| {
                        self.interner.push_expr_full(
                            HirExpression::Prefix(HirPrefixExpression::new(
                                UnaryOp::Reference { mutable },
                                *object,
                            )),
                            location,
                            new_type,
                        )
                    });
                }
            // Otherwise if the object type is a mutable reference and the method is not, insert as
            // many dereferences as needed.
            } else if matches!(actual_type, Type::Reference(..)) {
                let (new_object, new_type) = self.insert_auto_dereferences(*object, actual_type);
                *object_type = new_type;
                *object = new_object;
            }
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn type_check_function_body(&mut self, body_type: Type, meta: &FuncMeta, body_id: ExprId) {
        let (expr_location, empty_function) = self.function_info(body_id);
        let declared_return_type = meta.return_type();
        let last_expr_location = self.last_expr_location(body_id);

        if let Type::TraitAsType(trait_id, _, generics) = declared_return_type {
            self.use_unstable_feature(UnstableFeature::TraitAsType, last_expr_location);
            if self
                .interner
                .lookup_trait_implementation(
                    &body_type,
                    *trait_id,
                    &generics.ordered,
                    &generics.named,
                )
                .is_err()
            {
                self.push_err(self.new_type_mismatch_with_source_error(
                    &body_type,
                    declared_return_type,
                    Source::Return(meta.return_type.clone(), expr_location),
                    last_expr_location,
                ));
            }
        } else {
            self.unify_with_coercions(
                &body_type,
                declared_return_type,
                body_id,
                last_expr_location,
                |elaborator| {
                    let mut error = elaborator.new_type_mismatch_with_source_error(
                        &body_type,
                        declared_return_type,
                        Source::Return(meta.return_type.clone(), expr_location),
                        last_expr_location,
                    );

                    if empty_function {
                        error = error.add_context(
                        "implicitly returns `()` as its body has no tail or `return` expression",
                    );
                    }
                    CompilationError::TypeError(error)
                },
            );
        }
    }

    /// Grab a best-effort approximation of the last expression or statement in the function.
    /// Due to the typing rules of blocks, it is expected that the type of this expression/statement
    /// matches the return type of the function.
    fn last_expr_location(&self, expr: ExprId) -> Location {
        match self.interner.expression(&expr) {
            HirExpression::Block(block) if !block.statements.is_empty() => {
                let last = block.statements.last().unwrap();
                match self.interner.statement(last) {
                    HirStatement::Expression(expr) | HirStatement::Semi(expr) => {
                        self.last_expr_location(expr)
                    }
                    _ => self.interner.statement_location(*last),
                }
            }
            _ => self.interner.expr_location(&expr),
        }
    }

    fn function_info(&self, function_body_id: ExprId) -> (Location, bool) {
        let (expr_location, empty_function) =
            if let HirExpression::Block(block) = self.interner.expression(&function_body_id) {
                let last_stmt = block.statements().last();
                let mut location = self.interner.expr_location(&function_body_id);

                if let Some(last_stmt) = last_stmt
                    && let HirStatement::Expression(expr) = self.interner.statement(last_stmt)
                {
                    location = self.interner.expr_location(&expr);
                }

                (location, last_stmt.is_none())
            } else {
                (self.interner.expr_location(&function_body_id), false)
            };
        (expr_location, empty_function)
    }

    /// Seed `bindings` (the instantiation bindings) from a trait constraint so that, when the
    /// called method's type is instantiated, the trait's type variables resolve to the right
    /// types instead of being replaced by fresh, unconstrained ones.
    ///
    /// For a normal constraint like `T: Foo<u32, Bar = bool>`, this records `Foo`'s generics
    /// and associated types as the concrete arguments (`u32`, `bool`).
    ///
    /// For an `assumed` constraint the arguments are the trait's own variables, so each one is
    /// mapped to itself. That looks like a no-op but isn't: an entry tells instantiation "leave
    /// this variable alone". With no entry, instantiation mints a fresh variable for the slot
    /// and the link back to the trait's variable is lost, which breaks in three ways:
    /// - `Self` lost: the caller is forced to write a redundant type annotation.
    /// - A generic lost: it renders as `_` and unsoundly unifies with any type.
    /// - An associated type/constant lost: with a shared default-method body it gets bound to
    ///   the first impl resolved at dispatch and then leaks into the next (e.g. a second impl's
    ///   `Self::N` reads the first impl's constant).
    pub fn bind_generics_from_trait_constraint(
        &self,
        constraint: &TraitConstraint,
        assumed: bool,
        bindings: &mut TypeBindings,
    ) {
        self.bind_generics_from_trait_bound(&constraint.trait_bound, bindings);

        // Also bind associated types inherited from parent traits, e.g. a method returning
        // `Self::A` where `A` is defined on a parent trait rather than this one. Without this
        // they'd be left as unresolved `<T as Parent>::A` placeholders.
        self.bind_parent_trait_associated_types(
            &constraint.trait_bound,
            bindings,
            &mut BTreeSet::new(),
        );

        // An `assumed` constraint is one we get for free inside a trait method, where the body
        // may call other methods on `Self`. Its "arguments" are just the trait's own variables
        // (`Self`, its generics, its associated types), so the loops below map each variable to
        // itself. See the doc comment for why those self-mappings are not no-ops.
        if assumed {
            let the_trait = self.interner.get_trait(constraint.trait_bound.trait_id);

            let self_type = the_trait.self_type_typevar.clone();
            let kind = the_trait.self_type_typevar.kind();
            bindings.insert(self_type.id(), (self_type, kind, constraint.typ.clone()));

            for (param, arg) in
                the_trait.generics.iter().zip(&constraint.trait_bound.trait_generics.ordered)
            {
                bindings.insert(
                    param.type_var.id(),
                    (param.type_var.clone(), param.kind(), arg.clone()),
                );
            }

            for associated in &the_trait.associated_types {
                let Some(arg) = constraint
                    .trait_bound
                    .trait_generics
                    .named
                    .iter()
                    .find(|named| named.name.as_str() == associated.name.as_str())
                else {
                    continue;
                };
                bindings.insert(
                    associated.type_var.id(),
                    (associated.type_var.clone(), associated.kind(), arg.typ.clone()),
                );
            }
        }
    }

    /// Recursively bind the ordered generics and associated types of every parent trait reachable
    /// from `trait_bound`, instantiating each parent bound with the child's bindings as we go. This
    /// makes associated types inherited from ancestor traits resolvable, not just those defined on
    /// the trait named by `trait_bound`.
    fn bind_parent_trait_associated_types(
        &self,
        trait_bound: &ResolvedTraitBound,
        bindings: &mut TypeBindings,
        visited: &mut BTreeSet<TraitId>,
    ) {
        if !visited.insert(trait_bound.trait_id) {
            return;
        }

        // `bind_generics_from_trait_bound` below already assumes this trait id resolves (via
        // `get_trait`); use `get_trait` here too so a missing trait is a clear internal error
        // rather than a silently-empty parent-bound list.
        let parent_bounds: Vec<_> =
            self.interner.get_trait(trait_bound.trait_id).parent_bounds().cloned().collect();

        for parent_bound in &parent_bounds {
            let instantiated = self.instantiate_parent_trait_bound(trait_bound, parent_bound);
            self.bind_generics_from_trait_bound(&instantiated, bindings);
            self.bind_parent_trait_associated_types(&instantiated, bindings, visited);
        }
    }

    /// Insert the ordered generics and associated types from the trait bound.
    pub fn bind_generics_from_trait_bound(
        &self,
        trait_bound: &ResolvedTraitBound,
        bindings: &mut TypeBindings,
    ) {
        let the_trait = self.interner.get_trait(trait_bound.trait_id);

        bind_ordered_generics(&the_trait.generics, &trait_bound.trait_generics.ordered, bindings);

        let associated_types = the_trait.associated_types.clone();
        bind_named_generics(associated_types, &trait_bound.trait_generics.named, bindings);
    }

    pub fn instantiate_parent_trait_bound(
        &self,
        trait_bound: &ResolvedTraitBound,
        parent_trait_bound: &ResolvedTraitBound,
    ) -> ResolvedTraitBound {
        let mut bindings = TypeBindings::default();
        self.bind_generics_from_trait_bound(trait_bound, &mut bindings);
        ResolvedTraitBound {
            trait_generics: parent_trait_bound.trait_generics.map(|typ| typ.substitute(&bindings)),
            ..*parent_trait_bound
        }
    }

    pub(crate) fn fully_qualified_trait_path_by_id(&self, trait_id: TraitId) -> String {
        self.fully_qualified_trait_path(self.interner.get_trait(trait_id))
    }

    pub(crate) fn fully_qualified_trait_path(&self, trait_: &Trait) -> String {
        let module_def_id = ModuleDefId::TraitId(trait_.id);
        let visibility = trait_.visibility;
        let defining_module = None;
        let trait_is_visible = module_def_id_is_visible(
            module_def_id,
            self.module_id(),
            visibility,
            defining_module,
            self.interner,
            self.def_maps,
            &self.crate_graph[self.crate_id].dependencies,
        );

        if !trait_is_visible {
            let dependencies = &self.crate_graph[self.crate_id].dependencies;

            for reexport in self.interner.get_trait_reexports(trait_.id) {
                let reexport_is_visible = module_def_id_is_visible(
                    module_def_id,
                    self.module_id(),
                    reexport.visibility,
                    Some(reexport.module_id),
                    self.interner,
                    self.def_maps,
                    dependencies,
                );
                if reexport_is_visible {
                    let module_path = fully_qualified_module_path(
                        self.def_maps,
                        self.crate_graph,
                        &self.crate_id,
                        reexport.module_id,
                    );
                    return format!("{module_path}::{}", reexport.name);
                }
            }

            if let Some(reexport) = get_ancestor_module_reexport(
                module_def_id,
                visibility,
                self.module_id(),
                self.interner,
                self.def_maps,
                dependencies,
            ) {
                let module_path = fully_qualified_module_path(
                    self.def_maps,
                    self.crate_graph,
                    &self.crate_id,
                    reexport.module_id,
                );
                return format!("{module_path}::{}::{}", reexport.name, trait_.name);
            }
        }

        fully_qualified_module_path(self.def_maps, self.crate_graph, &self.crate_id, trait_.id.0)
    }
}

/// Binds the ordered [`ResolvedGeneric`]s of a trait to the ordered generics in a [`ResolvedTraitBound`].
///
/// Panics if the number of types do not match the ordered generics in the trait.
pub(super) fn bind_ordered_generics(
    params: &[ResolvedGeneric],
    args: &[Type],
    bindings: &mut TypeBindings,
) {
    assert_eq!(params.len(), args.len(), "unexpected number of ordered generics");

    for (param, arg) in params.iter().zip_eq(args) {
        bind_generic(param, arg, bindings);
    }
}

/// Binds the associated [`ResolvedGeneric`]s of a trait to the named generics in a [`ResolvedTraitBound`].
///
/// Panics if the number of types exceeds the named generics in the trait.
/// Any named parameter that does not appear in the arguments is bound to [`Type::Error`].
fn bind_named_generics(
    mut params: Vec<ResolvedGeneric>,
    args: &[NamedType],
    bindings: &mut TypeBindings,
) {
    assert!(
        args.len() <= params.len(),
        "bind_named_generics: trait bound has more named generics than associated types"
    );

    if params.is_empty() {
        return;
    }

    for arg in args {
        let i = params
            .iter()
            .position(|typ| *typ.name == arg.name.as_str())
            .unwrap_or_else(|| unreachable!("Expected to find associated type named {}", arg.name));

        let param = params.swap_remove(i);

        bind_generic(&param, &arg.typ, bindings);
    }

    for unbound_param in params {
        bind_generic(&unbound_param, &Type::Error, bindings);
    }
}

/// Binds the type variable in a [`ResolvedGeneric`], e.g. a generic parameter of a trait,
/// to a [Type], which itself can be an unbound type variable.
///
/// If the type variable itself appears in the type, then it does nothing.
fn bind_generic(param: &ResolvedGeneric, arg: &Type, bindings: &mut TypeBindings) {
    // Avoid binding t = t
    if !arg.occurs(param.type_var.id()) {
        bindings.insert(param.type_var.id(), (param.type_var.clone(), param.kind(), arg.clone()));
    }
}
