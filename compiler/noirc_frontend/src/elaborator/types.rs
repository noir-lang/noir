//! Type resolution, unification, and method resolution (for both types and traits).

use std::{borrow::Cow, rc::Rc};

use im::HashSet;
use iter_extended::vecmap;
use noirc_errors::Location;
use rustc_hash::FxHashMap as HashMap;

use crate::{
    Kind, NamedGeneric, ResolvedGeneric, Type, TypeBinding, TypeBindings, UnificationError,
    ast::{
        AsTraitPath, BinaryOpKind, GenericTypeArgs, Ident, PathKind, UnaryOp, UnresolvedType,
        UnresolvedTypeData, UnresolvedTypeExpression, WILDCARD_TYPE,
    },
    elaborator::{UnstableFeature, path_resolution::PathResolution},
    hir::{
        def_collector::dc_crate::CompilationError,
        def_map::{ModuleDefId, fully_qualified_module_path},
        resolution::{errors::ResolverError, import::PathResolutionError},
        type_check::{
            Source, TypeCheckError,
            generics::{Generic, TraitGenerics},
        },
    },
    hir_def::{
        expr::{
            HirBinaryOp, HirCallExpression, HirExpression, HirLiteral, HirMemberAccess,
            HirMethodReference, HirPrefixExpression, HirTraitMethodReference, TraitItem,
        },
        function::FuncMeta,
        stmt::HirStatement,
        traits::{NamedType, ResolvedTraitBound, Trait, TraitConstraint},
    },
    modules::{get_ancestor_module_reexport, module_def_id_is_visible},
    node_interner::{
        DependencyId, ExprId, FuncId, GlobalValue, TraitId, TraitImplKind, TraitItemId,
    },
    shared::Signedness,
    token::SecondaryAttributeKind,
};

use super::{
    Elaborator, PathResolutionTarget, UnsafeBlockStatus, lints,
    path_resolution::{PathResolutionItem, PathResolutionMode, TypedPath},
};

pub const SELF_TYPE_NAME: &str = "Self";

#[derive(Debug)]
pub(super) struct TraitPathResolution {
    pub(super) method: TraitPathResolutionMethod,
    pub(super) item: Option<PathResolutionItem>,
    pub(super) errors: Vec<PathResolutionError>,
}

#[derive(Debug)]
pub(super) enum TraitPathResolutionMethod {
    NotATraitMethod(FuncId),
    TraitItem(TraitItem),
    MultipleTraitsInScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WildcardAllowed {
    Yes,
    No(WildcardDisallowedContext),
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
    /// Resolves an [UnresolvedType] to a [Type] with [Kind::Normal] and marks it, and any generic types it contains, as _referenced_.
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

    /// Resolves an [UnresolvedType] to a [Type] with [Kind::Normal] and marks it, and any generic types it contains, as _used_.
    pub(crate) fn use_type(
        &mut self,
        typ: UnresolvedType,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        self.use_type_with_kind(typ, &Kind::Normal, wildcard_allowed)
    }

    /// Resolves an [UnresolvedType] to a [Type] and marks it, and any generic types it contains, as _used_.
    pub(crate) fn use_type_with_kind(
        &mut self,
        typ: UnresolvedType,
        kind: &Kind,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        self.resolve_type_inner(typ, kind, PathResolutionMode::MarkAsUsed, wildcard_allowed)
    }

    /// Translates an [UnresolvedType] to a [Type] with a given [Kind] and [PathResolutionMode].
    ///
    /// Pushes an error if the resolved type is invalid.
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

    /// Resolves an [UnresolvedType] to a [Type] with a given [Kind] and marks it, and any generic types it contains, as _referenced_.
    pub(crate) fn resolve_type_with_kind(
        &mut self,
        typ: UnresolvedType,
        kind: &Kind,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        self.resolve_type_inner(typ, kind, PathResolutionMode::MarkAsReferenced, wildcard_allowed)
    }

    /// Translates an [UnresolvedType] into a [Type] with a given [Kind] and [PathResolutionMode].
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
                Type::Array(Box::new(size), elem)
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

                match *env {
                    Type::Unit | Type::Tuple(_) | Type::NamedGeneric(_) => {
                        Type::Function(args, ret, env, unconstrained)
                    }
                    _ => {
                        self.push_err(ResolverError::InvalidClosureEnvironment {
                            typ: *env,
                            location: env_location,
                        });
                        Type::Error
                    }
                }
            }
            Reference(element, mutable) => {
                if !mutable {
                    self.use_unstable_feature(UnstableFeature::Ownership, location);
                }
                Type::Reference(
                    Box::new(self.resolve_type_with_kind_inner(
                        *element,
                        kind,
                        mode,
                        wildcard_allowed,
                    )),
                    mutable,
                )
            }
            Parenthesized(typ) => {
                self.resolve_type_with_kind_inner(*typ, kind, mode, wildcard_allowed)
            }
            Resolved(id) => self.interner.get_quoted_type(id).clone(),
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
    fn lookup_associated_type_on_self(&self, path: &TypedPath) -> Option<Type> {
        if path.segments.len() == 2 && path.first_name() == Some(SELF_TYPE_NAME) {
            if let Some(trait_id) = self.current_trait {
                let the_trait = self.interner.get_trait(trait_id);
                if let Some(typ) = the_trait.get_associated_type(path.last_name()) {
                    return Some(
                        typ.clone()
                            .into_named_generic(Some((SELF_TYPE_NAME, the_trait.name.as_str()))),
                    );
                }
            }

            if let Some(impl_id) = self.current_trait_impl {
                let name = path.last_name();
                if let Some(typ) = self.interner.find_associated_type_for_impl(impl_id, name) {
                    return Some(typ.clone());
                }
            }
        }
        None
    }

    /// Resolve `T::Foo` to an associated type on a generic type parameter with trait bounds.
    ///
    /// For example, in `impl<T: Baz> Foo for T { type Bar = T::Qux; }`, this resolves `T::Qux`
    /// by finding that `T` has a bound `Baz` which defines the associated type `Qux`.
    fn lookup_associated_type_on_generic(&mut self, path: &TypedPath) -> Option<Type> {
        if path.segments.len() != 2 {
            return None;
        }

        let type_name = path.segments[0].ident.as_str();
        let assoc_name = path.last_name();

        // Check if first segment is a generic parameter
        self.find_generic(type_name)?;

        // Search trait bounds for this generic to find the associated type
        let mut found_types = Vec::new();

        for constraint in &self.trait_bounds {
            if let Type::NamedGeneric(generic) = &constraint.typ {
                if generic.name.as_ref() == type_name {
                    let trait_id = constraint.trait_bound.trait_id;
                    let the_trait = self.interner.get_trait(trait_id);

                    if let Some(assoc_type) = the_trait.get_associated_type(assoc_name) {
                        found_types.push((trait_id, assoc_type.clone()));
                    }
                }
            }
        }

        match found_types.len() {
            0 => None, // Fall through to normal resolution
            1 => {
                let (trait_id, assoc_type) = found_types.remove(0);
                let the_trait = self.interner.get_trait(trait_id);
                // Return the associated type with proper naming for display
                Some(assoc_type.into_named_generic(Some((type_name, the_trait.name.as_str()))))
            }
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

    fn resolve_named_type(
        &mut self,
        path: TypedPath,
        args: GenericTypeArgs,
        mode: PathResolutionMode,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        if args.is_empty() {
            if let Some(typ) = self.lookup_generic_or_global_type(&path, mode) {
                return typ;
            }
        }

        let location = path.location;

        // Check if the path is a type variable first. We currently disallow generics on type
        // variables since we do not support higher-kinded types.
        if let Some(typ) = self.lookup_type_variable(&path, &args, wildcard_allowed) {
            self.check_comptime_type_in_runtime_code(&typ, location);
            return typ;
        }

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
            // of definition ordering, but for now we have an explicit check here so that we at
            // least issue an error that the type was not found instead of silently passing.
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

                if !self.in_contract()
                    && self
                        .interner
                        .type_attributes(&data_type.borrow().id)
                        .iter()
                        .any(|attr| matches!(attr.kind, SecondaryAttributeKind::Abi(_)))
                {
                    self.push_err(ResolverError::AbiAttributeOutsideContract {
                        location: data_type.borrow().name.location(),
                        usage_location: Some(path.location),
                    });
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
                let typ = self.instantiate_primitive_type(
                    primitive_type,
                    args,
                    location,
                    wildcard_allowed,
                );
                self.check_comptime_type_in_runtime_code(&typ, location);
                typ
            }
            Ok(PathResolutionItem::TraitAssociatedType(associated_type_id)) => {
                let associated_type = self.interner.get_trait_associated_type(associated_type_id);
                let trait_ = self.interner.get_trait(associated_type.trait_id);

                self.push_err(ResolverError::AmbiguousAssociatedType {
                    trait_name: trait_.name.to_string(),
                    associated_type_name: associated_type.name.to_string(),
                    location,
                });

                Type::Error
            }
            Ok(item) => {
                self.push_err(ResolverError::Expected {
                    expected: "type",
                    found: item.description(self.interner),
                    location,
                });

                Type::Error
            }
            Err(err) => {
                self.push_err(err);

                Type::Error
            }
        }
    }

    /// Reports an error if `typ` is a comptime-only type and we are in runtime code.
    fn check_comptime_type_in_runtime_code(&mut self, typ: &Type, location: Location) {
        if let Type::Quoted(quoted) = typ {
            use DependencyId::*;
            let in_function_or_global = matches!(self.current_item, Some(Function(_) | Global(_)));
            if in_function_or_global && !self.in_comptime_context() {
                let typ = quoted.to_string();
                self.push_err(ResolverError::ComptimeTypeInRuntimeCode { location, typ });
            }
        }
    }

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

    /// Resolves the ordered and named [GenericTypeArgs] into [Type]s and associated [NamedType]s,
    /// marking all of them as _used_.
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

    /// Resolves the ordered and named [GenericTypeArgs] into [Type]s and associated [NamedType]s.
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

    /// Matches [GenericTypeArgs::ordered_args] to the [Generic::generic_kinds] of a [Generic] type,
    /// resolving them to [Type]s with the given [PathResolutionMode]. If the type accepts named
    /// generic arguments, those are resolved as well and returned as associated [NamedType]s.
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

        let ordered_args = expected_kinds.iter().zip(args.ordered_args);
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
    /// go through a list of named [UnresolvedType]s and match them up to the named generics of the type,
    /// returning the resolved [NamedType]s and pushing errors for any unexpected, duplicate or missing entries.
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

    fn lookup_generic_or_global_type(
        &mut self,
        path: &TypedPath,
        mode: PathResolutionMode,
    ) -> Option<Type> {
        if path.segments.len() == 1 {
            let name = path.last_name();
            if let Some(generic) = self.find_generic(name) {
                let generic = generic.clone();
                return Some(generic.into_named_generic(None));
            }
        } else if let Some(typ) = self.lookup_associated_type_on_self(path) {
            if let Some(last_segment) = path.segments.last() {
                if last_segment.generics.is_some() {
                    self.push_err(ResolverError::GenericsOnAssociatedType {
                        location: last_segment.turbofish_location(),
                    });
                }
            }
            return Some(typ);
        } else if let Some(typ) = self.lookup_associated_type_on_generic(path) {
            if let Some(last_segment) = path.segments.last() {
                if last_segment.generics.is_some() {
                    self.push_err(ResolverError::GenericsOnAssociatedType {
                        location: last_segment.turbofish_location(),
                    });
                }
            }
            return Some(typ);
        }

        // If we cannot find a local generic of the same name, try to look up a global
        match self.resolve_path_inner(path.clone(), PathResolutionTarget::Value, mode) {
            Ok(PathResolution { item: PathResolutionItem::Global(id), errors }) => {
                self.push_errors(errors);

                if let Some(current_item) = self.current_item {
                    self.interner.add_global_dependency(current_item, id);
                }

                let reference_location = path.location;
                self.interner.add_global_reference(id, reference_location);
                let opt_global_let_statement = self.interner.get_global_let_statement(id);
                let kind = opt_global_let_statement
                    .as_ref()
                    .map(|let_statement| Kind::numeric(let_statement.r#type.clone()))
                    .unwrap_or(Kind::u32());

                let Some(stmt) = opt_global_let_statement else {
                    if self.elaborate_global_if_unresolved(&id) {
                        return self.lookup_generic_or_global_type(path, mode);
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

                let Some(global_value) = global_value.to_non_negative_signed_field() else {
                    let global_value = global_value.clone();
                    if global_value.is_integral() {
                        self.push_err(ResolverError::NegativeGlobalType { location, global_value });
                    } else {
                        self.push_err(ResolverError::NonIntegralGlobalType {
                            location,
                            global_value,
                        });
                    }
                    return None;
                };

                let Ok(global_value) = kind.ensure_value_fits(global_value, location) else {
                    self.push_err(ResolverError::GlobalDoesNotFitItsType {
                        location,
                        global_value,
                        kind,
                    });
                    return None;
                };

                Some(Type::Constant(global_value, kind))
            }
            _ => None,
        }
    }

    pub(super) fn convert_expression_type(
        &mut self,
        length: UnresolvedTypeExpression,
        expected_kind: &Kind,
        location: Location,
        wildcard_allowed: WildcardAllowed,
    ) -> Type {
        match length {
            UnresolvedTypeExpression::Variable(path) => {
                let mut ab = GenericTypeArgs::default();
                // Use generics from path, if they exist
                if let Some(last_segment) = path.segments.last() {
                    if let Some(generics) = &last_segment.generics {
                        ab.ordered_args = generics.clone();
                    }
                }
                let path = self.validate_path(path);
                let mode = PathResolutionMode::MarkAsReferenced;
                let mut typ = self.resolve_named_type(path, ab, mode, wildcard_allowed);
                if let Type::Alias(alias, vec) = typ {
                    typ = alias.borrow().get_type(&vec);
                }
                self.check_type_kind(typ, expected_kind, location)
            }
            UnresolvedTypeExpression::Constant(int, suffix, _span) => {
                let suffix_kind = if let Some(suffix) = suffix {
                    suffix.as_kind()
                } else {
                    let integer_or_field_var =
                        self.interner.next_type_variable_with_kind(Kind::IntegerOrField);
                    Kind::Numeric(Box::new(integer_or_field_var))
                };

                if !suffix_kind.unifies(expected_kind) {
                    self.push_err(TypeCheckError::ExpectingOtherError {
                        message: format!("convert_expression_type: {suffix_kind} does not unify with expected {expected_kind}"),
                        location,
                    });
                }

                Type::Constant(int, suffix_kind)
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
                    (Type::Constant(lhs, lhs_kind), Type::Constant(rhs, rhs_kind)) => {
                        if !lhs_kind.unifies(&rhs_kind) {
                            self.push_err(TypeCheckError::TypeKindMismatch {
                                expected_kind: lhs_kind,
                                expr_kind: rhs_kind,
                                expr_location: location,
                            });
                            return Type::Error;
                        }
                        match op.function(lhs, rhs, &lhs_kind, location) {
                            Ok(result) => Type::Constant(result, lhs_kind),
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
            UnresolvedTypeExpression::AsTraitPath(path) => {
                let mode = PathResolutionMode::MarkAsReferenced;
                let typ = self.resolve_as_trait_path(*path, mode, wildcard_allowed);
                self.check_type_kind(typ, expected_kind, location)
            }
        }
    }

    /// Checks that the type's [Kind] matches the expected kind, issuing an error if it does not.
    /// Returns `typ` unless an error occurs - in which case [Type::Error] is returned.
    pub(super) fn check_type_kind(
        &mut self,
        typ: Type,
        expected_kind: &Kind,
        location: Location,
    ) -> Type {
        if typ.has_cyclic_alias(&mut HashSet::default()) {
            self.push_err(TypeCheckError::CyclicType { typ, location });
            return Type::Error;
        }

        if self.check_kind(typ.kind(), expected_kind, location) { typ } else { Type::Error }
    }

    /// Checks that `expr_kind` matches `expected_kind`, issuing an error if it does not.
    /// Returns `true` if the kinds unify.
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

    /// Try to resolve an [AsTraitPath] as `<Self as {trait}>::{ident}` to the [Type] of the `{ident}`.
    ///
    /// If it's a different pattern then returns `None`.
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

        if !ordered.iter().all(|typ| matches!(typ, Type::NamedGeneric(_)))
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
            // The call to `lookup_generic_or_global_type` which calls `lookup_associated_type_on_self`
            // will only be made if the args are empty, and that's what we tried to ascertain before
            // by checking that none of them are bound to a concrete type.
            GenericTypeArgs::default(),
            mode,
            wildcard_allowed,
        );
        Some(typ)
    }

    fn get_associated_type_from_trait_impl(
        &mut self,
        path: AsTraitPath,
        impl_kind: TraitImplKind,
    ) -> Type {
        let associated_types = match impl_kind {
            TraitImplKind::Assumed { trait_generics, .. } => Cow::Owned(trait_generics.named),
            TraitImplKind::Normal(impl_id) => {
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

    /// This resolves `Self::some_static_method`, inside an impl block (where we don't have a concrete self_type)
    /// or inside a trait default method.
    ///
    /// Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    /// E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_static_method_by_self(&self, path: &TypedPath) -> Option<TraitPathResolution> {
        // If we are inside a trait impl, `Self` is known to be a concrete type so we don't have
        // to solve the path via trait method lookup.
        if self.current_trait_impl.is_some() {
            return None;
        }

        let trait_id = self.current_trait?;

        if path.kind == PathKind::Plain && path.segments.len() == 2 {
            let is_self_type = path.segments[0].ident.is_self_type_name();
            let method = &path.segments[1].ident;

            if is_self_type {
                let the_trait = self.interner.get_trait(trait_id);
                // Allow referring to trait constants via Self:: as well
                let definition =
                    the_trait.find_method_or_constant(method.as_str(), self.interner)?;
                let constraint = the_trait.as_constraint(path.location);
                let trait_method = TraitItem { definition, constraint, assumed: true };
                let method = TraitPathResolutionMethod::TraitItem(trait_method);
                return Some(TraitPathResolution { method, item: None, errors: Vec::new() });
            }
        }
        None
    }

    /// This resolves `TraitName::some_static_method`
    ///
    /// Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    /// E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_static_method(&mut self, path: &TypedPath) -> Option<TraitPathResolution> {
        let path_resolution = self.use_path_as_type(path.clone()).ok()?;
        let func_id = path_resolution.item.function_id()?;
        let meta = self.interner.try_function_meta(&func_id)?;
        let the_trait = self.interner.get_trait(meta.trait_id?);
        let method = the_trait.find_method(path.last_name(), self.interner)?;
        let constraint = the_trait.as_constraint(path.location);
        let trait_method = TraitItem { definition: method, constraint, assumed: false };
        let method = TraitPathResolutionMethod::TraitItem(trait_method);
        let item = Some(path_resolution.item);
        Some(TraitPathResolution { method, item, errors: path_resolution.errors })
    }

    /// This resolves a static trait method T::trait_method by iterating over the where clause
    ///
    /// Returns the trait method, trait constraint, and whether the impl is assumed from a where
    /// clause. This is always true since this helper searches where clauses for a generic constraint.
    /// E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_method_by_named_generic(
        &mut self,
        path: &TypedPath,
    ) -> Option<TraitPathResolution> {
        if path.segments.len() != 2 {
            return None;
        }

        let type_name = path.segments[0].ident.as_str();
        let method_name = path.last_name();

        let mut matches = Vec::new();

        for constraint in self.trait_bounds.clone() {
            if let Type::NamedGeneric(NamedGeneric { name, .. }) = &constraint.typ {
                // if `path` is `T::method_name`, we're looking for constraint of the form `T: SomeTrait`
                if type_name != name.as_str() {
                    continue;
                }

                let the_trait = self.interner.get_trait(constraint.trait_bound.trait_id);
                self.find_methods_or_constants_in_trait(
                    path,
                    constraint,
                    the_trait,
                    the_trait.id,
                    &mut matches,
                );
            }
        }

        if matches.len() == 1 {
            let method = matches.remove(0).0;

            if path.segments[0].generics.is_some() {
                let turbofish_location = path.segments[0].turbofish_location();
                self.push_err(PathResolutionError::TurbofishNotAllowedOnItem {
                    item: "generic parameter".to_string(),
                    location: turbofish_location,
                });
            }

            return Some(TraitPathResolution { method, item: None, errors: Vec::new() });
        }

        if matches.len() > 1 {
            let location = path.location;
            let ident = Ident::new(method_name.to_string(), location);
            let traits = vecmap(matches, |(_, trait_id)| {
                let trait_ = self.interner.get_trait(trait_id);
                self.fully_qualified_trait_path(trait_)
            });
            let errors = vec![PathResolutionError::MultipleTraitsInScope { ident, traits }];
            return Some(TraitPathResolution {
                method: TraitPathResolutionMethod::MultipleTraitsInScope,
                item: None,
                errors,
            });
        }

        None
    }

    fn find_methods_or_constants_in_trait(
        &self,
        path: &TypedPath,
        constraint: TraitConstraint,
        the_trait: &Trait,
        starting_trait_id: TraitId,
        matches: &mut Vec<(TraitPathResolutionMethod, TraitId)>,
    ) {
        if let Some(definition) = the_trait.find_method_or_constant(path.last_name(), self.interner)
        {
            let trait_item =
                TraitItem { definition, constraint: constraint.clone(), assumed: true };
            let method = TraitPathResolutionMethod::TraitItem(trait_item);
            matches.push((method, the_trait.id));
        }

        for trait_bound in &the_trait.trait_bounds {
            let parent_trait = self.interner.get_trait(trait_bound.trait_id);
            if parent_trait.id == starting_trait_id {
                // Avoid infinite recursion in case of cyclic trait bounds
                continue;
            }

            let constraint =
                TraitConstraint { typ: constraint.typ.clone(), trait_bound: trait_bound.clone() };
            self.find_methods_or_constants_in_trait(
                path,
                constraint,
                parent_trait,
                starting_trait_id,
                matches,
            );
        }
    }

    /// This resolves a method in the form `Type::method` where `method` is a trait method
    fn resolve_type_trait_method(&mut self, path: &TypedPath) -> Option<TraitPathResolution> {
        if path.segments.len() < 2 {
            return None;
        }

        let mut path = path.clone();
        let location = path.location;
        let last_segment = path.pop();
        let before_last_segment = path.last_segment();
        let turbofish = before_last_segment.turbofish();

        let path_resolution = self.use_path_as_type(path).ok()?;
        let typ = match path_resolution.item {
            PathResolutionItem::Type(type_id) => {
                let generics = self.resolve_struct_id_turbofish_generics(type_id, turbofish);
                let datatype = self.get_type(type_id);
                Type::DataType(datatype, generics)
            }
            PathResolutionItem::TypeAlias(type_alias_id) => {
                let generics =
                    self.resolve_type_alias_id_turbofish_generics(type_alias_id, turbofish);
                let type_alias = self.interner.get_type_alias(type_alias_id);
                let type_alias = type_alias.borrow();
                type_alias.get_type(&generics)
            }
            PathResolutionItem::PrimitiveType(primitive_type) => {
                let (typ, _) =
                    self.instantiate_primitive_type_with_turbofish(primitive_type, turbofish);
                typ
            }
            PathResolutionItem::Module(..)
            | PathResolutionItem::Trait(..)
            | PathResolutionItem::TraitAssociatedType(..)
            | PathResolutionItem::Global(..)
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

        let method_name = last_segment.ident.as_str();

        // If we can find a method on the type, this is definitely not a trait method
        let check_self_param = false;
        if self.interner.lookup_direct_method(&typ, method_name, check_self_param).is_some() {
            return None;
        }

        let has_self_arg = false;
        let trait_methods = self.interner.lookup_trait_methods(&typ, method_name, has_self_arg);

        if trait_methods.is_empty() {
            return None;
        }

        let (hir_method_reference, error) =
            self.get_trait_method_in_scope(&trait_methods, method_name, last_segment.location);
        let hir_method_reference = hir_method_reference?;
        match hir_method_reference {
            HirMethodReference::FuncId(func_id) => {
                // It could happen that we find a single function (one in a trait impl)
                let mut errors = path_resolution.errors;
                if let Some(error) = error {
                    errors.push(error);
                }

                let method = TraitPathResolutionMethod::NotATraitMethod(func_id);
                Some(TraitPathResolution { method, item: None, errors })
            }
            HirMethodReference::TraitItemId(HirTraitMethodReference {
                definition,
                trait_id,
                ..
            }) => {
                let trait_ = self.interner.get_trait(trait_id);
                let mut constraint = trait_.as_constraint(location);
                constraint.typ = typ.clone();

                let trait_method = TraitItem { definition, constraint, assumed: false };
                let func_id = hir_method_reference.func_id(self.interner)?;
                let item = PathResolutionItem::TypeTraitFunction(typ, trait_id, func_id);

                let mut errors = path_resolution.errors;
                if let Some(error) = error {
                    errors.push(error);
                }

                let method = TraitPathResolutionMethod::TraitItem(trait_method);
                Some(TraitPathResolution { method, item: Some(item), errors })
            }
        }
    }

    /// Try to resolve a [TypedPath] to a trait method path.
    ///
    /// Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    /// E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    pub(super) fn resolve_trait_generic_path(
        &mut self,
        path: &TypedPath,
    ) -> Option<TraitPathResolution> {
        self.resolve_trait_static_method_by_self(path)
            .or_else(|| self.resolve_trait_static_method(path))
            .or_else(|| self.resolve_trait_method_by_named_generic(path))
            .or_else(|| self.resolve_type_trait_method(path))
    }

    /// Unify two types, modifying both in the process.
    ///
    /// Pushes an error on failure.
    pub(super) fn unify(
        &mut self,
        actual: &Type,
        expected: &Type,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        if let Err(UnificationError) = actual.unify(expected) {
            self.push_err(make_error());
        }
    }

    /// Wrapper of [Type::unify_with_coercions], pushing any unification errors.
    pub(super) fn unify_with_coercions(
        &mut self,
        actual: &Type,
        expected: &Type,
        expression: ExprId,
        location: Location,
        make_error: impl FnOnce() -> CompilationError,
    ) {
        let mut errors = Vec::new();
        actual.unify_with_coercions(
            expected,
            expression,
            location,
            self.interner,
            &mut errors,
            make_error,
        );

        // When passing lambdas to unconstrained functions that don't explicitly state
        // that they expect unconstrained lambdas, ignore the coercion.
        if self.in_unconstrained_args {
            errors.retain(|err| {
                !matches!(err, CompilationError::TypeError(TypeCheckError::UnsafeFn { .. }))
            });
        }

        self.push_errors(errors);
    }

    /// Return a fresh integer or field type variable and log it
    /// in self.type_variables to default it later.
    pub(super) fn polymorphic_integer_or_field(&mut self) -> Type {
        let typ = Type::polymorphic_integer_or_field(self.interner);
        self.push_defaultable_type_variable(typ.clone());
        typ
    }

    /// Return a fresh integer type variable and log it
    /// in self.type_variables to default it later.
    pub(super) fn polymorphic_integer(&mut self) -> Type {
        let typ = Type::polymorphic_integer(self.interner);
        self.push_defaultable_type_variable(typ.clone());
        typ
    }

    /// Return a fresh integer type variable and log it
    /// in self.type_variables to default it later.
    pub(super) fn type_variable_with_kind(&mut self, type_var_kind: Kind) -> Type {
        let typ = Type::type_variable_with_kind(self.interner, type_var_kind);
        self.push_defaultable_type_variable(typ.clone());
        typ
    }

    /// Translates a (possibly Unspecified) UnresolvedType to a Type.
    /// Any UnresolvedType::Unspecified encountered are replaced with fresh type variables.
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
    pub(super) fn insert_auto_dereferences(&mut self, object: ExprId, typ: Type) -> (ExprId, Type) {
        if let Type::Reference(element, _mut) = typ.follow_bindings() {
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
        } else {
            (object, typ)
        }
    }

    /// Given a method object: `(*foo).bar` of a method call `(*foo).bar.baz()`, remove the
    /// implicitly added dereference operator if one is found.
    ///
    /// Returns Some(new_expr_id) if a dereference was removed and None otherwise.
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

        for (param, (arg, arg_expr_id, arg_location)) in fn_params.iter().zip(callsite_args) {
            self.unify_with_coercions(arg, param, *arg_expr_id, *arg_location, || {
                CompilationError::TypeError(TypeCheckError::TypeMismatch {
                    expected_typ: param.to_string(),
                    expr_typ: arg.to_string(),
                    expr_location: *arg_location,
                })
            });
        }

        fn_ret.clone()
    }

    pub(super) fn bind_function_type(
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
            Literal(HirLiteral::Integer(field)) if !field.is_negative() => {
                Some(field.absolute_value())
            }

            // TODO(https://github.com/noir-lang/noir/issues/6247):
            // handle negative literals
            _ => None,
        };

        let from_is_polymorphic = match from_follow_bindings {
            Type::Integer(..) | Type::FieldElement | Type::Bool => false,

            Type::TypeVariable(ref var) if var.is_integer() || var.is_integer_or_field() => true,
            Type::TypeVariable(_) => {
                // NOTE: in reality the expected type can also include bool, but for the compiler's simplicity
                // we only allow integer types. If a bool is in `from` it will need an explicit type annotation.
                let expected = self.polymorphic_integer_or_field();
                self.unify(from, &expected, || TypeCheckError::InvalidCast {
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

        // TODO(https://github.com/noir-lang/noir/issues/6247):
        // handle negative literals
        // when casting a polymorphic value to a specifically sized type,
        // check that it fits or throw a warning
        if let (Some(from_value), Some(to_maximum_size)) =
            (from_value_opt, to.integral_maximum_size())
        {
            if from_is_polymorphic && from_value > to_maximum_size {
                let from = from.clone();
                let to = to.clone();
                let reason = format!(
                    "casting untyped value ({from_value}) to a type with a maximum size ({to_maximum_size}) that's smaller than it"
                );
                // we warn that the 'to' type is too small for the value
                self.push_err(TypeCheckError::DownsizingCast { from, to, location, reason });
            }
        }

        match to {
            Type::Integer(sign, bits) => Type::Integer(sign, bits),
            Type::FieldElement => {
                if from_follow_bindings.is_signed() {
                    self.push_err(TypeCheckError::UnsupportedFieldCast { location });
                }

                Type::FieldElement
            }
            Type::Bool => {
                let from_is_numeric = match from_follow_bindings {
                    Type::Integer(..) | Type::FieldElement => true,
                    Type::TypeVariable(ref var) => var.is_integer() || var.is_integer_or_field(),
                    _ => false,
                };
                if from_is_numeric {
                    self.push_err(TypeCheckError::CannotCastNumericToBool {
                        typ: from_follow_bindings,
                        location,
                    });
                }

                Type::Bool
            }
            Type::Error => Type::Error,
            _ => {
                self.push_err(TypeCheckError::UnsupportedCast { location });
                Type::Error
            }
        }
    }

    /// Given a binary comparison operator and another type. This method will produce the output type
    /// and a boolean indicating whether to use the trait impl corresponding to the operator
    /// or not. A value of false indicates the caller to use a primitive operation for this
    /// operator, while a true value indicates a user-provided trait impl is required.
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
                if sign_x != sign_y {
                    return Err(TypeCheckError::IntegerSignedness {
                        sign_x: *sign_x,
                        sign_y: *sign_y,
                        location,
                    });
                }
                if bit_width_x != bit_width_y {
                    return Err(TypeCheckError::IntegerBitWidth {
                        bit_width_x: *bit_width_x,
                        bit_width_y: *bit_width_y,
                        location,
                    });
                }
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
                self.unify(lhs, rhs, || TypeCheckError::TypeMismatchWithSource {
                    expected: lhs.clone(),
                    actual: rhs.clone(),
                    location: op.location,
                    source: Source::Binary,
                });
                Ok((Bool, true))
            }
        }
    }

    /// Handles the TypeVariable case for checking binary operators.
    /// Returns true if we should use the impl for the operator instead of the primitive
    /// version of it.
    fn bind_type_variables_for_infix(
        &mut self,
        lhs_type: &Type,
        op: &HirBinaryOp,
        rhs_type: &Type,
        location: Location,
    ) -> bool {
        self.unify(lhs_type, rhs_type, || TypeCheckError::TypeMismatchWithSource {
            expected: lhs_type.clone(),
            actual: rhs_type.clone(),
            source: Source::Binary,
            location,
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
            self.unify(lhs_type, &target, || match op.kind {
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
                if sign_x != sign_y {
                    return Err(TypeCheckError::IntegerSignedness {
                        sign_x: *sign_x,
                        sign_y: *sign_y,
                        location,
                    });
                }
                if bit_width_x != bit_width_y {
                    return Err(TypeCheckError::IntegerBitWidth {
                        bit_width_x: *bit_width_x,
                        bit_width_y: *bit_width_y,
                        location,
                    });
                }
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
                self.unify(lhs, rhs, || TypeCheckError::TypeMismatchWithSource {
                    expected: lhs.clone(),
                    actual: rhs.clone(),
                    location: op.location,
                    source: Source::Binary,
                });
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
                            self.unify(rhs_type, &integer_type, || {
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

                    Bool => Ok((Bool, false)),

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
                    self.unify(rhs_type, &mutable, || TypeCheckError::TypeMismatch {
                        expr_typ: rhs_type.to_string(),
                        expected_typ: mutable.to_string(),
                        expr_location: location,
                    });
                }
                Ok((element_type, false))
            }
        }
    }

    /// Prerequisite: verify_trait_constraint of the operator's trait constraint.
    ///
    /// Although by this point the operator is expected to already have a trait impl,
    /// we still need to match the operator's type against the method's instantiated type
    /// to ensure the instantiation bindings are correct and the monomorphizer can
    /// re-apply the needed bindings.
    pub(super) fn type_check_operator_method(
        &mut self,
        expr_id: ExprId,
        trait_method_id: TraitItemId,
        object_type: &Type,
        return_type: &Type,
        location: Location,
    ) {
        let method_type = self.interner.definition_type(trait_method_id.item_id);
        let (method_type, mut bindings) = method_type.instantiate(self.interner);

        match method_type {
            Type::Function(args, ret, env, _unconstrained) => {
                assert!(
                    !args.is_empty(),
                    "type_check_operator_method ICE: expected operator method to have at least one argument type"
                );

                self.unify(&env, &Type::Unit, || TypeCheckError::TypeMismatch {
                    expected_typ: Type::Unit.to_string(),
                    expr_typ: env.to_string(),
                    expr_location: location,
                });

                let mut bindings = TypeBindings::default();
                let unifies = ret.try_unify(return_type, &mut bindings).is_ok();
                if !unifies {
                    // // TODO(https://github.com/noir-lang/noir/issues/10537): the following comment
                    // // on unifying 'object_type' with 'expected_object_type' is out of date because
                    // // attempting to unify the return type of 'method_type' with 'result_type' is
                    // // failing sometimes, e.g. the following 'panic!' message is being reached when running
                    // // 'cargo run check' in the 'noir_stdlib':
                    // // type_check_operator_method: ret: Ordering, return_type: bool, args: ['6832, '6832], object_type: T'67, definition_name: "cmp"
                    // let definition_name = &self.interner.definition(trait_method_id.item_id).name;
                    // panic!("type_check_operator_method: ret: {ret:?}, return_type: {return_type:?}, args: {args:?}, object_type: {object_type:?}, definition_name: {definition_name:?}");
                }

                // We can cheat a bit and match against only the object type here since no operator
                // overload uses other generic parameters or return types aside from the object type.
                let expected_object_type = &args[0];
                self.unify(object_type, expected_object_type, || TypeCheckError::TypeMismatch {
                    expected_typ: expected_object_type.to_string(),
                    expr_typ: object_type.to_string(),
                    expr_location: location,
                });
            }
            other => {
                unreachable!("Expected operator method to have a function type, but found {other}")
            }
        }

        // We must also remember to apply these substitutions to the object_type
        // referenced by the selected trait impl, if one has yet to be selected.
        let impl_kind = self.interner.get_selected_impl_for_expression(expr_id);
        if let Some(TraitImplKind::Assumed { object_type, trait_generics }) = impl_kind {
            let the_trait = self.interner.get_trait(trait_method_id.trait_id);
            let object_type = object_type.substitute(&bindings);
            bindings.insert(
                the_trait.self_type_typevar.id(),
                (
                    the_trait.self_type_typevar.clone(),
                    the_trait.self_type_typevar.kind(),
                    object_type.clone(),
                ),
            );

            self.interner.select_impl_for_expression(
                expr_id,
                TraitImplKind::Assumed { object_type, trait_generics },
            );
        }

        self.interner.store_instantiation_bindings(expr_id, bindings);
    }

    pub(super) fn type_check_member_access(
        &mut self,
        mut access: HirMemberAccess,
        expr_id: ExprId,
        lhs_type: Type,
        location: Location,
    ) -> Type {
        let access_lhs = &mut access.lhs;

        let dereference_lhs = |this: &mut Self, lhs_type, element| {
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
    pub(super) fn check_field_access(
        &mut self,
        lhs_type: &Type,
        field_name: &str,
        location: Location,
        dereference_lhs: Option<impl FnMut(&mut Self, Type, Type)>,
    ) -> Option<(Type, usize)> {
        let lhs_type = lhs_type.follow_bindings();

        match &lhs_type {
            Type::DataType(s, args) => {
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
                    dereference_lhs(self, lhs_type.clone(), element.as_ref().clone());
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

    fn lookup_type_or_primitive_method(
        &mut self,
        object_type: &Type,
        method_name: &str,
        location: Location,
        object_location: Location,
        check_self_param: bool,
    ) -> Option<HirMethodReference> {
        // First search in the type methods. If there is one, that's the one.
        if let Some(method_id) =
            self.interner.lookup_direct_method(object_type, method_name, check_self_param)
        {
            return Some(HirMethodReference::FuncId(method_id));
        }

        // Next lookup all matching trait methods.
        let trait_methods =
            self.interner.lookup_trait_methods(object_type, method_name, check_self_param);

        // If there's at least one matching trait method we need to see if only one is in scope.
        if !trait_methods.is_empty() {
            return self.return_trait_method_in_scope(&trait_methods, method_name, location);
        }

        // If we couldn't find any trait methods, search in
        // impls for all types `T`, e.g. `impl<T> Foo for T`
        let generic_methods =
            self.interner.lookup_generic_methods(object_type, method_name, check_self_param);
        if !generic_methods.is_empty() {
            return self.return_trait_method_in_scope(&generic_methods, method_name, location);
        }

        if let Type::DataType(datatype, _) = object_type {
            let datatype = datatype.borrow();
            let mut has_field_with_function_type = false;

            if let Some(fields) = datatype.fields_raw() {
                has_field_with_function_type = fields
                    .iter()
                    .any(|field| field.name.as_str() == method_name && field.typ.is_function());
            }

            if has_field_with_function_type {
                self.push_err(TypeCheckError::CannotInvokeStructFieldFunctionType {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    location,
                });
            } else {
                self.push_err(TypeCheckError::UnresolvedMethodCall {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    location,
                });
            }
            None
        } else {
            // It could be that this type is a composite type that is bound to a trait,
            // for example `x: (T, U) ... where (T, U): SomeTrait`
            // (so this case is a generalization of the NamedGeneric case)
            self.lookup_method_in_trait_constraints(
                object_type,
                method_name,
                location,
                object_location,
            )
        }
    }

    /// Given a list of functions and the trait they belong to, returns the one function
    /// that is in scope.
    fn return_trait_method_in_scope(
        &mut self,
        trait_methods: &[(FuncId, TraitId)],
        method_name: &str,
        location: Location,
    ) -> Option<HirMethodReference> {
        let (method, error) = self.get_trait_method_in_scope(trait_methods, method_name, location);
        if let Some(error) = error {
            self.push_err(error);
        }
        method
    }

    fn get_trait_method_in_scope(
        &mut self,
        trait_methods: &[(FuncId, TraitId)],
        method_name: &str,
        location: Location,
    ) -> (Option<HirMethodReference>, Option<PathResolutionError>) {
        let module_id = self.module_id();
        let module_data = self.get_module(module_id);

        // Only keep unique trait IDs: multiple trait methods might come from the same trait
        // but implemented with different generics (like `Convert<Field>` and `Convert<i32>`).
        let traits: HashSet<TraitId> =
            trait_methods.iter().map(|(_, trait_id)| *trait_id).collect();

        let traits_in_scope: Vec<_> = traits
            .iter()
            .filter_map(|trait_id| {
                module_data.find_trait_in_scope(*trait_id).map(|name| (*trait_id, name.clone()))
            })
            .collect();

        for (_, trait_name) in &traits_in_scope {
            self.usage_tracker.mark_as_used(module_id, trait_name);
        }

        if traits_in_scope.is_empty() {
            if traits.len() == 1 {
                // This is the backwards-compatible case where there's a single trait but it's not in scope
                let trait_id = *traits.iter().next().unwrap();
                let trait_ = self.interner.get_trait(trait_id);
                let trait_name = self.fully_qualified_trait_path(trait_);
                let method =
                    self.trait_hir_method_reference(trait_id, trait_methods, method_name, location);
                let error = PathResolutionError::TraitMethodNotInScope {
                    ident: Ident::new(method_name.into(), location),
                    trait_name,
                };
                return (Some(method), Some(error));
            } else {
                let traits = vecmap(traits, |trait_id| {
                    let trait_ = self.interner.get_trait(trait_id);
                    self.fully_qualified_trait_path(trait_)
                });
                let method_not_found = None;
                let error = PathResolutionError::UnresolvedWithPossibleTraitsToImport {
                    ident: Ident::new(method_name.into(), location),
                    traits,
                };
                return (method_not_found, Some(error));
            }
        }

        if traits_in_scope.len() > 1 {
            let traits = vecmap(traits, |trait_id| {
                let trait_ = self.interner.get_trait(trait_id);
                self.fully_qualified_trait_path(trait_)
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
        trait_methods: &[(FuncId, TraitId)],
        method_name: &str,
        location: Location,
    ) -> HirMethodReference {
        // If we find a single trait impl method, return it so we don't have to later determine the impl
        if trait_methods.len() == 1 {
            let (func_id, _) = trait_methods[0];
            return HirMethodReference::FuncId(func_id);
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
    fn lookup_method_in_trait_constraints(
        &mut self,
        object_type: &Type,
        method_name: &str,
        location: Location,
        object_location: Location,
    ) -> Option<HirMethodReference> {
        let func_id = match self.current_item {
            Some(DependencyId::Function(id)) => id,
            _ => {
                // Unexpected method outside a function.
                self.push_err(TypeCheckError::UnresolvedMethodCall {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    location,
                });
                return None;
            }
        };
        let func_meta = self.interner.function_meta(&func_id);

        // If inside a trait method, check if it's a method on `self`
        if let Some(trait_id) = func_meta.trait_id {
            if Some(object_type) == self.self_type.as_ref() {
                let the_trait = self.interner.get_trait(trait_id);
                let constraint = the_trait.as_constraint(the_trait.name.location());
                let mut matches = self.lookup_methods_in_trait(
                    the_trait,
                    method_name,
                    &constraint.trait_bound,
                    the_trait.id,
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
        }

        let mut matches = Vec::new();

        for constraint in func_meta.all_trait_constraints() {
            if *object_type == constraint.typ {
                if let Some(the_trait) =
                    self.interner.try_get_trait(constraint.trait_bound.trait_id)
                {
                    let trait_matches = self.lookup_methods_in_trait(
                        the_trait,
                        method_name,
                        &constraint.trait_bound,
                        the_trait.id,
                    );
                    matches.extend(trait_matches);
                }
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
            let traits = vecmap(matches, |method| {
                let trait_ = self.interner.get_trait(method.trait_id);
                self.fully_qualified_trait_path(trait_)
            });
            self.push_err(PathResolutionError::MultipleTraitsInScope { ident, traits });
            return None;
        }

        if object_type.is_bindable() {
            self.push_err(TypeCheckError::TypeAnnotationsNeededForMethodCall {
                location: object_location,
            });
        } else {
            self.push_err(TypeCheckError::UnresolvedMethodCall {
                method_name: method_name.to_string(),
                object_type: object_type.clone(),
                location,
            });
        }

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
        starting_trait_id: TraitId,
    ) -> Vec<HirTraitMethodReference> {
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

        // Search in the parent traits, if any
        for parent_trait_bound in &the_trait.trait_bounds {
            if let Some(the_trait) = self.interner.try_get_trait(parent_trait_bound.trait_id) {
                // Avoid looping forever in case there are cycles
                if the_trait.id == starting_trait_id {
                    continue;
                }

                let parent_trait_bound =
                    self.instantiate_parent_trait_bound(trait_bound, parent_trait_bound);
                let parent_matches = self.lookup_methods_in_trait(
                    the_trait,
                    method_name,
                    &parent_trait_bound,
                    starting_trait_id,
                );
                matches.extend(parent_matches);
            }
        }

        matches
    }

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

        let is_current_func_constrained = self.in_constrained_function();
        if !is_current_func_constrained {
            // Check if we're calling verify_proof_with_type in an unconstrained context
            self.run_lint(|elaborator| {
                lints::error_if_verify_proof_with_type(elaborator.interner, call.func)
                    .map(Into::into)
            });
        }

        let func_type_is_unconstrained =
            if let Type::Function(_args, _ret, _env, unconstrained) = &func_type {
                *unconstrained
            } else {
                false
            };

        let is_unconstrained_call =
            func_type_is_unconstrained || self.is_unconstrained_call(call.func);
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

            let errors = lints::unconstrained_function_args(&args);
            self.push_errors(errors);
        }

        let return_type = self.bind_function_type(func_type, args, location);

        if crossing_runtime_boundary {
            self.run_lint(|_| {
                lints::unconstrained_function_return(&return_type, location).map(Into::into)
            });
        }

        return_type
    }

    /// Check if the callee is an unconstrained function, or a variable referring to one.
    fn is_unconstrained_call(&self, expr: ExprId) -> bool {
        if let Some(func_id) = self.interner.lookup_function_from_expr(&expr) {
            let modifiers = self.interner.function_modifiers(&func_id);
            modifiers.is_unconstrained
        } else {
            false
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
                    self.check_can_mutate(*object, location);

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
                self.push_err(TypeCheckError::TypeMismatchWithSource {
                    expected: declared_return_type.clone(),
                    actual: body_type,
                    location: last_expr_location,
                    source: Source::Return(meta.return_type.clone(), expr_location),
                });
            }
        } else {
            self.unify_with_coercions(
                &body_type,
                declared_return_type,
                body_id,
                last_expr_location,
                || {
                    let mut error = TypeCheckError::TypeMismatchWithSource {
                        expected: declared_return_type.clone(),
                        actual: body_type.clone(),
                        location: last_expr_location,
                        source: Source::Return(meta.return_type.clone(), expr_location),
                    };

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

                if let Some(last_stmt) = last_stmt {
                    if let HirStatement::Expression(expr) = self.interner.statement(last_stmt) {
                        location = self.interner.expr_location(&expr);
                    }
                }

                (location, last_stmt.is_none())
            } else {
                (self.interner.expr_location(&function_body_id), false)
            };
        (expr_location, empty_function)
    }

    pub fn bind_generics_from_trait_constraint(
        &self,
        constraint: &TraitConstraint,
        assumed: bool,
        bindings: &mut TypeBindings,
    ) {
        self.bind_generics_from_trait_bound(&constraint.trait_bound, bindings);

        // If the trait impl is already assumed to exist we should add any type bindings for `Self`.
        // Otherwise `self` will be replaced with a fresh type variable, which will require the user
        // to specify a redundant type annotation.
        if assumed {
            let the_trait = self.interner.get_trait(constraint.trait_bound.trait_id);
            let self_type = the_trait.self_type_typevar.clone();
            let kind = the_trait.self_type_typevar.kind();
            bindings.insert(self_type.id(), (self_type, kind, constraint.typ.clone()));
        }
    }

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

pub(super) fn bind_ordered_generics(
    params: &[ResolvedGeneric],
    args: &[Type],
    bindings: &mut TypeBindings,
) {
    assert_eq!(params.len(), args.len());

    for (param, arg) in params.iter().zip(args) {
        bind_generic(param, arg, bindings);
    }
}

fn bind_named_generics(
    mut params: Vec<ResolvedGeneric>,
    args: &[NamedType],
    bindings: &mut TypeBindings,
) {
    assert!(args.len() <= params.len());

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

fn bind_generic(param: &ResolvedGeneric, arg: &Type, bindings: &mut TypeBindings) {
    // Avoid binding t = t
    if !arg.occurs(param.type_var.id()) {
        bindings.insert(param.type_var.id(), (param.type_var.clone(), param.kind(), arg.clone()));
    }
}
