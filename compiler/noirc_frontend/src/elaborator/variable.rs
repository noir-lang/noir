//! Everything to do with elaboration of variables.
//! Notably, variables may require trait constraints to be solved later on.

use itertools::Itertools;

use super::Elaborator;
use crate::TypeAlias;
use crate::ast::{
    Expression, ExpressionKind, GenericTypeArgs, Ident, Path, PathKind, TypePath,
    UnresolvedTypeExpression,
};
use crate::elaborator::TypedPath;
use crate::elaborator::function_context::BindableTypeVariableKind;
use crate::elaborator::path_resolution::{
    PathResolution, PathResolutionItem, Turbofish, TypedPathSegment,
};
use crate::elaborator::patterns::{PathValue, Variable};
use crate::elaborator::types::{SELF_TYPE_NAME, WildcardAllowed};
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::hir::def_map::ModuleId;
use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::import::PathResolutionError;
use crate::hir::type_check::TypeCheckError;
use crate::hir_def::expr::{
    HirExpression, HirIdent, HirMethodReference, HirTraitMethodReference, ImplKind, TraitItem,
};
use crate::hir_def::traits::TraitConstraint;
use crate::node_interner::pusher::{HasLocation, PushedExpr};
use crate::node_interner::{
    DefinitionId, DefinitionInfo, DefinitionKind, ExprId, TraitId, TraitImplId, TraitImplKind,
    TypeAliasId,
};
use crate::{Kind, Type, TypeBindings, TypeVariable, TypeVariableId};
use iter_extended::{btree_map, vecmap};
use noirc_errors::{Located, Location};

/// The result of [`Elaborator::resolve_variable`]: what a path used as an expression resolves
/// to. See [`Elaborator::resolve_variable`] for the full, ordered list of the path forms each
/// variant covers.
#[allow(clippy::large_enum_variant)]
pub(crate) enum VariableResolution {
    /// The path was elaborated straight to an expression. This happens for the `Self::*` forms
    /// handled inside a trait impl: an associated constant, an associated-type method, or a
    /// method when `Self` is a primitive type.
    Expression(ExprId, Type),
    /// The path resolved to a local variable, a definition (global, function, enum-variant
    /// global), or a trait item (method or associated constant).
    Ident(HirIdent, Option<PathResolutionItem>),
    /// The path resolved to a type alias that is numeric, infinitely recursive or one that errored.
    TypeAlias(TypeAliasId),
}

/// A path's prefix resolved into [its kind](PathPrefixKind) plus the "rest" — the last segment
/// (the item accessed on the prefix) and the prefix's own turbofish. Produced by
/// [`Elaborator::resolve_path_prefix`]; [`Elaborator::resolve_prefixed_variable`] resolves the last
/// segment against the kind without re-deriving anything from the original path.
#[derive(Debug)]
struct ResolvedPrefix {
    /// The path's last segment: the item being accessed on the prefix.
    last_segment: TypedPathSegment,
    /// Turbofish on the prefix's own last segment (e.g. the `<T>` in `Foo::<T>::bar`).
    turbofish: Option<Turbofish>,
    kind: PathPrefixKind,
}

/// What the prefix of a path (every segment but the last) is, when the path is used as an
/// expression. This describes *what the prefix is*, not the already-resolved item; the shared
/// "rest" (last segment, turbofish) lives on [`ResolvedPrefix`].
#[derive(Debug)]
enum PathPrefixKind {
    /// `Self` inside a trait *impl*, where `Self` is the impl's concrete type (carried here, along
    /// with the impl) since a trait impl always has both. The last segment is an associated-type
    /// method, an associated constant, a primitive-`Self` method, or a plain method on the self
    /// type ([`Elaborator::resolve_self_in_trait_impl`]).
    SelfInTraitImpl { self_type: Type, trait_impl_id: TraitImplId },
    /// `Self` inside a trait *definition* (carried here): an assumed constraint on the current
    /// trait (or a supertrait reached through it) ([`Elaborator::resolve_self_in_trait`]).
    SelfInTrait { trait_id: TraitId },
    /// `Self` as a plain concrete type (an inherent impl, or any other context where `Self` names
    /// a data type): the last segment resolves like `Type::method`
    /// ([`Elaborator::resolve_self_as_concrete_type`]).
    SelfInImpl,
    /// A generic parameter with `: Trait` bound(s) in scope (e.g. `T` in `T::method`); the carried
    /// constraints are the matched bounds. The last segment is a method or associated constant
    /// reached through one of them.
    BoundedGeneric(Vec<TraitConstraint>),
    /// A trait (e.g. `Trait` in `Trait::method` / `Trait::CONST`). The last segment is a trait
    /// static method or an associated constant.
    Trait { trait_id: TraitId, resolution: PathResolution },
    /// A concrete type, type alias, or primitive type (e.g. `Type` in `Type::method`). The last
    /// segment is an inherent or qualified trait method.
    Type { resolution: PathResolution },
    /// A module (e.g. `foo::bar` in `foo::bar::GLOBAL`). The last segment is an ordinary value
    /// item, resolved as a value directly in `module_id`. `errors` are the prefix's own resolution
    /// errors (e.g. an intermediate segment's visibility), reported when the last segment resolves.
    Module { module_id: ModuleId, errors: Vec<PathResolutionError> },
    /// The prefix is `Self` but there is no self type in scope (e.g. in a free function), so `Self`
    /// names nothing.
    SelfNotInScope,
    /// The prefix is not something that can carry an associated item: it is not a type, trait, or
    /// module (it is a value, or it failed to resolve), so the path names nothing. The carried error
    /// (either the prefix's own resolution failure, or that it is a value rather than a namespace) is
    /// reported as-is.
    InvalidPrefix { error: PathResolutionError },
}

impl Elaborator<'_> {
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn elaborate_variable(&mut self, variable: Path) -> (ExprId, Type) {
        let (id, typ, is_comptime_local, location) = self.elaborate_variable_inner(variable);

        // Only check has_errors when we need to call the interpreter
        // If this variable is a comptime local variable, use its current value as the final expression
        if is_comptime_local {
            let mut interpreter = self.setup_interpreter();
            let value = interpreter.evaluate(id);
            // If the value is an error it means the variable already had an error, so don't report it here again
            // (the error will make no sense, it will say that a non-comptime variable was referenced at runtime
            // but that's not true)
            if value.is_ok() {
                let from_macro_call = false;
                let (id, typ) = self.inline_comptime_value(value, location, from_macro_call);
                self.debug_comptime(location, |interner, _| id.to_display_ast(interner).kind);
                (id, typ)
            } else {
                (id, typ)
            }
        } else {
            (id, typ)
        }
    }

    /// Helper function containing the elaboration logic for a variable.
    /// Returns the expression ID, type, whether it's a comptime local, and location.
    #[tracing::instrument(level = "trace", skip_all)]
    fn elaborate_variable_inner(&mut self, variable: Path) -> (ExprId, Type, bool, Location) {
        let variable = self.validate_path(variable);

        let resolved_turbofish = variable.segments.last().unwrap().generics.clone();

        // A turbofish on the segment *before* the last one (e.g. `Foo::<u32>::Spam`) provides
        // type generics for the type the last segment is resolved within. This is needed for
        // fieldless enum variants, which resolve to a global rather than a function.
        let type_segment_turbofish = (variable.segments.len() >= 2)
            .then(|| variable.segments[variable.segments.len() - 2].generics.clone())
            .flatten();

        let location = variable.location;
        let variable_resolution = self.resolve_variable(variable);

        let (hir_ident, item) = match variable_resolution {
            Some(VariableResolution::Expression(id, typ)) => return (id, typ, false, location),
            Some(VariableResolution::TypeAlias(type_alias_id)) => {
                // A type alias to a numeric generics is considered like a variable,
                // but it is not a real variable so it does not resolve to a valid Identifier.
                // In order to handle this, we retrieve the numeric generics expression that the type aliases to.
                let type_alias = self.interner.get_type_alias(type_alias_id);
                let alias_module_id = type_alias.borrow().module_id;
                if let Some(type_alias_expr) = &type_alias.borrow().numeric_expr {
                    // Extract the declared numeric type from the type alias's kind.
                    let declared_type = match type_alias.borrow().typ.kind() {
                        Kind::Numeric(declared_type) => declared_type,
                        _ => Box::new(Type::Error),
                    };
                    let declared_type = *declared_type;
                    let var_expr = UnresolvedTypeExpression::to_expression_kind(type_alias_expr);

                    // The expression we create for this particular instantiation of the numeric type alias
                    // must have the same location as the path that refers to it.
                    let var_expr = Expression::new(var_expr, location);

                    // Resolve turbofish generics for the type alias.
                    // `resolved_turbofish` contains already-resolved types from
                    // validate_path, so we use resolve_alias_turbofish_generics
                    // directly which accepts resolved types.
                    let alias_generics = &type_alias.borrow().generics;
                    let alias_generic_types = vecmap(alias_generics, |generic| {
                        self.interner.next_type_variable_with_kind(generic.kind())
                    });
                    let mut errors = Vec::new();
                    let type_alias_ref = type_alias.borrow();
                    let resolved_generics = self.resolve_alias_turbofish_generics(
                        &type_alias_ref,
                        alias_generic_types,
                        resolved_turbofish,
                        location,
                        &mut errors,
                    );
                    self.push_errors(errors);

                    // Introduce alias generics into scope so the numeric expression
                    // resolves them correctly (not to globals or other variables
                    // that happen to share the same name). Bind each generic's type
                    // variable to the turbofish-resolved type.
                    self.push_scope();
                    for (generic, resolved_type) in
                        alias_generics.iter().zip_eq(resolved_generics.iter())
                    {
                        if let Kind::Numeric(numeric_type) = &generic.kind() {
                            let id = self.interner.next_type_variable_id();
                            let type_var = TypeVariable::unbound(id, generic.kind());
                            type_var.bind(resolved_type.clone());
                            let definition =
                                DefinitionKind::NumericGeneric(type_var, numeric_type.clone());
                            let ident = Ident::new(generic.name.to_string(), generic.location);
                            let hir_ident = self.add_variable_decl(
                                ident, false, // mutable
                                true,  // allow_shadowing
                                false, // warn_if_unused
                                false, // warn_if_not_mutated
                                definition,
                            );
                            self.interner.push_definition_type(hir_ident.id, *numeric_type.clone());
                        }
                    }

                    // The alias's numeric expression has already been kind-checked at
                    // alias-definition time (see `convert_expression_type`), which is
                    // where any "value does not fit" diagnostic is emitted. Drop any
                    // literals queued for the function-context fit check during
                    // re-elaboration so the same overflow is not reported twice.
                    let literals_before = self.integer_literal_expr_ids_len();
                    // Re-elaborate the alias body in the alias's defining module
                    // so unqualified names resolve against the alias's scope,
                    // not the caller's. Mirrors `define_type_alias` in mod.rs.
                    let previous_module = self.replace_module(alias_module_id);
                    let (id, typ) = self.elaborate_expression(var_expr);
                    self.restore_module(previous_module);
                    self.truncate_integer_literal_expr_ids(literals_before);
                    self.pop_scope();

                    // Unify the expression's type with the declared type from the type alias
                    // to ensure proper type checking.
                    self.unify_or_type_mismatch(&typ, &declared_type, type_alias_expr.location());

                    return (id, declared_type, false, location);
                }
                (None, None)
            }
            Some(VariableResolution::Ident(ident, item)) => (Some(ident), item),
            None => (None, None),
        };

        let definition_id = hir_ident.as_ref().map(|ident| ident.id);

        let (type_generics, self_generic) = if let Some(item) = item {
            self.resolve_item_turbofish_and_self_type(item)
        } else {
            (Vec::new(), None)
        };

        let definition = definition_id.map(|id| self.interner.definition(id));
        let is_comptime_local = !self.in_comptime_context()
            && definition.is_some_and(DefinitionInfo::is_comptime_local);
        let definition_kind = definition.as_ref().map(|definition| definition.kind.clone());

        let mut bindings = TypeBindings::default();
        let generics = if let Some(DefinitionKind::Function(func_id)) = &definition_kind {
            self.usage_tracker.mark_impl_function_as_used(func_id);

            // If there's a self type, bind it to the self type generic
            if let Some(self_generic) = self_generic {
                let func_generics = &self.function_meta(*func_id).all_generics;
                let self_resolved_generic =
                    func_generics.iter().find(|generic| generic.name.as_str() == SELF_TYPE_NAME);
                if let Some(self_resolved_generic) = self_resolved_generic {
                    let type_var = &self_resolved_generic.type_var;
                    bindings
                        .insert(type_var.id(), (type_var.clone(), type_var.kind(), self_generic));
                }
            }

            // If this is a function call on a type that has generics, we need to bind those generic types.
            if !type_generics.is_empty() {
                // We must only bind the type-level portion here; method generics are handled
                // separately by the method turbofish.
                let func_meta = self.function_meta(*func_id);
                let impl_generics = vecmap(func_meta.impl_generics(), |g| g.type_var.clone());
                let self_type =
                    func_meta.self_type.as_ref().map(|t| t.follow_bindings_shallow().into_owned());

                // An enum variant constructor (e.g. `Foo::<u32>::Eggs`) has no `impl_generics`
                // and no `self_type`; the enum's generics are its `direct_generics`. The
                // type-segment turbofish provides types for exactly those generics, so bind
                // them directly. `type_generics` was resolved against the enum's generics, so
                // its length matches `direct_generics`.
                if func_meta.enum_variant_index.is_some() {
                    let direct_generics =
                        vecmap(&func_meta.direct_generics, |g| g.type_var.clone());
                    for (type_generic, type_var) in
                        type_generics.into_iter().zip_eq(direct_generics)
                    {
                        bindings.insert(
                            type_var.id(),
                            (type_var.clone(), type_var.kind(), type_generic),
                        );
                    }
                }
                // For partially concrete impls (e.g. `impl<B> S<u32, B>`), the number of
                // impl generics differs from the number of struct generics. The turbofish
                // `S::<u32, bool>` provides type_generics aligned with the struct's params
                // [A, B], not the impl's generics [B]. Replace each impl generic in
                // `self_type`'s args with a fresh type variable and unify those with the
                // turbofish-provided type generics. The fresh type variables get bound by
                // unification, and we record those bindings for the impl generics.
                else if let Some(Type::DataType(_, self_type_args)) = self_type {
                    assert_eq!(
                        type_generics.len(),
                        self_type_args.len(),
                        "ICE: turbofish type_generics count ({}) should match self_type_args count ({})",
                        type_generics.len(),
                        self_type_args.len(),
                    );
                    let impl_replacements: TypeBindings = impl_generics
                        .iter()
                        .map(|type_var| {
                            let kind = type_var.kind();
                            let fresh = self.interner.next_type_variable_with_kind(kind.clone());
                            (type_var.id(), (type_var.clone(), kind, fresh))
                        })
                        .collect();
                    for (type_generic, self_type_arg) in
                        type_generics.into_iter().zip_eq(self_type_args)
                    {
                        let substituted = self_type_arg.substitute(&impl_replacements);
                        self.unify_or_type_mismatch(&type_generic, &substituted, location);
                    }
                    bindings.extend(impl_replacements);
                } else if type_generics.len() <= impl_generics.len() {
                    // For trait function paths, impl_generics may include Self and associated
                    // type generics after the trait's declared generics. The turbofish only
                    // provides types for the declared generics, which are always the first
                    // elements. Slice to match.
                    let impl_generics = &impl_generics[..type_generics.len()];
                    for (type_generic, type_var) in type_generics.into_iter().zip_eq(impl_generics)
                    {
                        bindings.insert(
                            type_var.id(),
                            (type_var.clone(), type_var.kind(), type_generic),
                        );
                    }
                } else {
                    unreachable!(
                        "type_generics.len() ({}) > impl_generics.len() ({}): \
                        turbofish resolution should normalize the count",
                        type_generics.len(),
                        impl_generics.len()
                    );
                }
            }

            // Resolve any generics if the variable we have resolved is a function
            // and if the turbofish operator was used.
            self.resolve_function_turbofish_generics(func_id, resolved_turbofish, location)
        } else {
            // A fieldless enum variant resolves to a global. A turbofish on the variant path
            // binds the enum's generics, which global path resolution does not carry on its
            // own. The turbofish may be on the type segment (`Foo::<u32>::Spam`) or the
            // variant segment (`Foo::Spam::<u32>`); both denote the same enum generics.
            let is_enum_variant_global =
                if let Some(DefinitionKind::Global(global_id)) = &definition_kind {
                    self.interner.is_enum_variant_global(*global_id)
                } else {
                    false
                };

            if is_enum_variant_global {
                if let Some(turbofish) = resolved_turbofish.or(type_segment_turbofish) {
                    self.bind_enum_variant_global_turbofish(
                        definition_id.unwrap(),
                        &turbofish,
                        location,
                        &mut bindings,
                    );
                }
            } else if let Some(unused_resolved_turbofish) = resolved_turbofish {
                let message = format!(
                    "elaborate_variable_inner: unused resolved_turbofish: {unused_resolved_turbofish:?}"
                );
                self.push_err(TypeCheckError::expecting_other_error(message, location));
            }

            None
        };

        let (id, typ) = if let Some(hir_ident) = hir_ident {
            let id = self
                .intern_expr(HirExpression::Ident(hir_ident.clone(), generics.clone()), location);

            let typ = self.type_check_variable_with_bindings(hir_ident, &id, generics, bindings);
            let id = self.intern_expr_type(id, typ.clone());
            (id, typ)
        } else {
            let expr = HirExpression::Error;
            let id = self.intern_expr(expr, location);
            let typ = Type::Error;
            let id = self.intern_expr_type(id, typ.clone());
            (id, typ)
        };

        (id, typ, is_comptime_local, location)
    }

    /// Bind the type-segment turbofish of a fieldless enum variant path (e.g. `Foo::<u32>::Spam`)
    /// to the variant global's `Forall` generics (the enum's generics), validating the count the
    /// same way [`Self::resolve_item_turbofish_generics`] does. A non-generic enum has no `Forall`,
    /// so a turbofish on it produces a count-mismatch error.
    fn bind_enum_variant_global_turbofish(
        &mut self,
        definition_id: DefinitionId,
        turbofish: &[Located<Type>],
        location: Location,
        bindings: &mut TypeBindings,
    ) {
        let global_type = self.interner.definition_type(definition_id);
        let (typevars, enum_name) = match &global_type {
            Type::Forall(typevars, body) => (typevars.clone(), data_type_name(body)),
            other => (Vec::new(), data_type_name(other)),
        };

        let mut turbofish = turbofish.to_vec();
        if turbofish.len() != typevars.len() {
            self.push_err(TypeCheckError::GenericCountMismatch {
                item: format!("enum `{}`", enum_name.unwrap_or_default()),
                expected: typevars.len(),
                found: turbofish.len(),
                location,
            });
            // Pad/truncate to the expected length so every generic is still determined,
            // matching `resolve_function_turbofish_generics` and avoiding a cascade of
            // "type annotation needed" errors.
            turbofish.resize(typevars.len(), Located::from(location, Type::Error));
        }

        for (located_type, type_var) in turbofish.into_iter().zip(&typevars) {
            let type_location = located_type.location();
            let typ = self.check_type_kind(located_type.contents, &type_var.kind(), type_location);
            bindings.insert(type_var.id(), (type_var.clone(), type_var.kind(), typ));
        }
    }

    /// Resolves the `Self::…` forms that only make sense inside a trait impl, where `Self` is a
    /// concrete type with associated items. The segments *after* `Self` (the "rest") select the
    /// form:
    ///
    /// - `[AssociatedType, method]` — resolve the associated type, then elaborate the method on it.
    /// - `[item]` — an associated constant (looked up for its value, later a literal), or, when
    ///   `Self` is a primitive type, a method elaborated as if it were a [TypePath]
    ///   (`u32::method_name`); a regular path lookup won't work, for the same reason [TypePath]
    ///   exists.
    ///
    /// Returns `None` for any other shape (including a data-type `Self`, handled as a plain type
    /// prefix), so the caller falls back to resolving `Self` as a type.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_variable_as_self_method_or_associated_constant(
        &mut self,
        variable: &TypedPath,
        self_type: Type,
        trait_impl_id: TraitImplId,
    ) -> Option<(ExprId, Type)> {
        match &variable.segments[1..] {
            [associated_type, method] => {
                let associated_type = self
                    .interner
                    .find_associated_type_for_impl(trait_impl_id, associated_type.ident.as_str())
                    .cloned()?;
                // Extract already-resolved turbofish generics from the method segment.
                let resolved_generics = method.generics.as_ref().map(|generics| {
                    generics.iter().map(|located| located.contents.clone()).collect()
                });
                Some(self.elaborate_type_path_impl_with_resolved_generics(
                    associated_type,
                    &method.ident,
                    resolved_generics,
                    variable.segments[1].location,
                ))
            }
            [item] => self.elaborate_self_associated_constant_or_primitive_method(
                &self_type,
                trait_impl_id,
                item,
                variable.location,
                variable.segments[0].location,
            ),
            _ => None,
        }
    }

    /// The `Self::item` (single segment after `Self`) case of
    /// [`Self::resolve_variable_as_self_method_or_associated_constant`]: an associated constant
    /// (from the impl, or from the trait when the impl is missing it), or a method when `Self` is a
    /// primitive type. A data-type `Self` returns `None` so it is resolved as a plain type prefix.
    fn elaborate_self_associated_constant_or_primitive_method(
        &mut self,
        self_type: &Type,
        trait_impl_id: TraitImplId,
        item: &TypedPathSegment,
        location: Location,
        self_location: Location,
    ) -> Option<(ExprId, Type)> {
        let name = item.ident.as_str();

        // The associated constant declared on the impl.
        if let Some((definition_id, numeric_type)) =
            self.interner.get_trait_impl_associated_constant(trait_impl_id, name).cloned()
        {
            return Some(self.intern_associated_constant(definition_id, numeric_type, location));
        }

        // The constant declared on the trait, even if the impl is missing it. This prevents a
        // spurious "Could not resolve" inside trait methods; the "missing associated constant"
        // error is reported elsewhere.
        if let Some(trait_impl) = self.interner.try_get_trait_implementation(trait_impl_id) {
            let trait_id = trait_impl.borrow().trait_id;
            let trait_ = self.interner.get_trait(trait_id);
            if let Some(definition_id) = trait_.associated_constant_ids.get(name).copied() {
                let numeric_type = self.interner.definition_type(definition_id);
                return Some(self.intern_associated_constant(
                    definition_id,
                    numeric_type,
                    location,
                ));
            }
        }

        // A data-type `Self::method` is resolved as a plain type prefix, not here.
        if matches!(self_type, Type::DataType(..)) {
            return None;
        }

        Some(self.elaborate_type_path_impl(self_type.clone(), &item.ident, None, self_location))
    }

    /// Intern an identifier expression referring to an associated constant of the given type.
    fn intern_associated_constant(
        &mut self,
        definition_id: DefinitionId,
        numeric_type: Type,
        location: Location,
    ) -> (ExprId, Type) {
        let hir_ident = HirIdent::non_trait_method(definition_id, location);
        let hir_expr = HirExpression::Ident(hir_ident, None);
        let id = self.interner.push_expr_full(hir_expr, location, numeric_type.clone());
        (id, numeric_type)
    }

    /// Resolve a [`TypedPath`] used as an expression to the item it names.
    ///
    /// A path with a prefix (more than one segment) names an item accessed *through* that prefix,
    /// fully handled by [`Self::resolve_prefixed_variable`]. A single-segment path resolves in the
    /// current scope (a local variable, or a value item — global, function, enum-variant global,
    /// numeric type alias) via [`Self::resolve_unprefixed_variable`].
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_variable(&mut self, path: TypedPath) -> Option<VariableResolution> {
        if path.segments.len() > 1 {
            self.resolve_prefixed_variable(path)
        } else {
            self.resolve_unprefixed_variable(path)
        }
    }

    /// Resolve a [`TypedPath`] that has a prefix (more than one segment) to the item it names —
    /// fully: it always resolves the path, reports an error, or falls back to a value lookup, so
    /// the caller never needs a further fallback. [`Self::resolve_path_prefix`] classifies the
    /// prefix once; the last segment is resolved against that classification directly in the
    /// already-resolved prefix (a module value via [`Self::resolve_value_in_module`], an enum
    /// variant or associated constant on a type via [`Self::resolve_value_in_type`]).
    ///
    /// Returns `None` only when an error has already been reported (an ambiguous trait method, or
    /// an unresolved name), so the caller should produce an error expression rather than retry.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_prefixed_variable(&mut self, path: TypedPath) -> Option<VariableResolution> {
        let ResolvedPrefix { last_segment, turbofish, kind } = self.resolve_path_prefix(&path);

        match kind {
            // `Self` is contextual; each context resolves the last segment its own way.
            PathPrefixKind::SelfInTraitImpl { self_type, trait_impl_id } => self
                .resolve_self_in_trait_impl(
                    path,
                    self_type,
                    trait_impl_id,
                    last_segment,
                    turbofish,
                ),
            PathPrefixKind::SelfInTrait { trait_id } => {
                self.resolve_self_in_trait(path, trait_id, last_segment, turbofish)
            }
            // `Self` is a concrete type here (classification guarantees `self_type` exists), so it
            // resolves exactly like `Type::method`.
            PathPrefixKind::SelfInImpl => {
                self.resolve_self_as_concrete_type(path, last_segment, turbofish)
            }
            PathPrefixKind::BoundedGeneric(bounds) => {
                self.resolve_bounded_generic_item(bounds, &last_segment, turbofish, path.location)
            }
            PathPrefixKind::Type { resolution } => {
                let is_self_prefix = false;
                self.resolve_method_on_type_prefix(
                    last_segment,
                    turbofish,
                    is_self_prefix,
                    resolution,
                    path.location,
                )
            }
            // A trait prefix: the last segment is either a trait static method (`Trait::method`)
            // or an associated constant (`Trait::CONST`).
            PathPrefixKind::Trait { trait_id, resolution } => self.resolve_trait_item_on_prefix(
                trait_id,
                turbofish,
                &last_segment,
                resolution,
                path.location,
            ),
            // A module prefix: the last segment is an ordinary value item, resolved as a value
            // directly in the already-resolved module.
            PathPrefixKind::Module { module_id, errors } => {
                self.push_errors(errors);
                self.resolve_value_in_module(module_id, last_segment)
            }
            // No usable prefix: the path names nothing, and a value lookup would only rediscover
            // the resolution failure already carried here, so report it directly.
            PathPrefixKind::InvalidPrefix { error } => {
                self.push_err(error);
                None
            }
            // `Self` with no self type in scope: report it directly.
            PathPrefixKind::SelfNotInScope => {
                self.push_err(PathResolutionError::Unresolved(path.segments[0].ident.clone()));
                None
            }
        }
    }

    /// Classify the prefix (every segment but the last) of a path used as an expression. This does
    /// not resolve the last segment — it only answers "what is the prefix?", which the caller uses
    /// to decide how to resolve the last segment. The caller only invokes this for a path that has
    /// a prefix (more than one segment).
    ///
    /// The classification order is significant: it decides which interpretation wins when a path
    /// is ambiguous, and mirrors the historical probe order. `Self` and a bounded generic are
    /// recognized from context (`Self` is contextual; a generic is not in the module namespace)
    /// before the prefix is resolved as a type to tell trait-, type-, and module-prefixed forms
    /// apart.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_path_prefix(&mut self, path: &TypedPath) -> ResolvedPrefix {
        // The caller (`resolve_variable`) only takes this path for a multi-segment path, so popping
        // the last segment is always valid.
        debug_assert!(path.segments.len() >= 2);

        let mut prefix = path.clone();
        let last_segment = prefix.pop();
        let turbofish = prefix.last_segment().turbofish();

        // `Self` and a generic parameter are only meaningful as a plain path: `crate::Self` /
        // `super::T` name an item in another module, not the contextual `Self` or an in-scope
        // generic, so they must not be classified as such.
        let kind = if path.kind == PathKind::Plain && path.segments[0].ident.is_self_type_name() {
            // `Self` is contextual; which kind it is depends only on where we are (a trait impl, a
            // trait definition, somewhere `Self` is a plain type, or nowhere at all), so it is
            // classified here and the matching arm resolves the last segment.
            if let Some(trait_impl_id) = self.current_trait_impl {
                let self_type =
                    self.self_type.clone().expect("a trait impl always has a self type");
                PathPrefixKind::SelfInTraitImpl { self_type, trait_impl_id }
            } else if let Some(trait_id) = self.current_trait {
                PathPrefixKind::SelfInTrait { trait_id }
            } else if self.self_type.is_some() {
                PathPrefixKind::SelfInImpl
            } else {
                PathPrefixKind::SelfNotInScope
            }
        } else if path.kind == PathKind::Plain
            && let Some(bounds) = self.matching_generic_bounds(path)
        {
            // A generic parameter (e.g. `T`) with a `T: Trait` bound in scope. A generic is not in
            // the module namespace, so this is recognized from the in-scope bounds, not resolution.
            PathPrefixKind::BoundedGeneric(bounds)
        } else {
            // Otherwise the prefix is resolved as a type to distinguish trait/type/module.
            let prefix_last_ident = prefix.last_segment().ident;
            match self.use_path_as_type(prefix) {
                Ok(resolution) => match &resolution.item {
                    PathResolutionItem::Trait(trait_id) => {
                        PathPrefixKind::Trait { trait_id: *trait_id, resolution }
                    }
                    PathResolutionItem::Type(..)
                    | PathResolutionItem::TypeAlias(..)
                    | PathResolutionItem::PrimitiveType(..) => PathPrefixKind::Type { resolution },
                    PathResolutionItem::Module(module_id) => PathPrefixKind::Module {
                        module_id: *module_id,
                        errors: resolution.errors.clone(),
                    },
                    // Resolving a type path falls back to the value namespace, so the prefix's last
                    // segment can also resolve to a value item; that (and an associated type) can't
                    // carry the last segment of the path as an associated item, so the path names
                    // nothing. Listed explicitly (rather than `_`) so a new `PathResolutionItem`
                    // must be classified here.
                    PathResolutionItem::TraitAssociatedType(..)
                    | PathResolutionItem::Global(..)
                    | PathResolutionItem::EnumVariant(..)
                    | PathResolutionItem::ModuleFunction(..)
                    | PathResolutionItem::Method(..)
                    | PathResolutionItem::SelfMethod(..)
                    | PathResolutionItem::TypeAliasFunction(..)
                    | PathResolutionItem::TraitFunction(..)
                    | PathResolutionItem::TypeTraitFunction(..)
                    | PathResolutionItem::PrimitiveFunction(..)
                    | PathResolutionItem::TraitConstant(..) => PathPrefixKind::InvalidPrefix {
                        error: PathResolutionError::Unresolved(prefix_last_ident),
                    },
                },
                Err(error) => PathPrefixKind::InvalidPrefix { error },
            }
        };

        ResolvedPrefix { last_segment, turbofish, kind }
    }

    /// If the path's first segment names a generic parameter with `: Trait` bound(s) in scope,
    /// return those matching bounds (so `Head::item` can be resolved through them). Only meaningful
    /// for a two-segment path; a generic is not in the module namespace, so this scans the in-scope
    /// bounds rather than resolving.
    pub(super) fn matching_generic_bounds(&self, path: &TypedPath) -> Option<Vec<TraitConstraint>> {
        if path.segments.len() != 2 {
            return None;
        }
        let head = path.segments[0].ident.as_str();
        let bounds: Vec<_> = self
            .trait_bounds
            .iter()
            .filter(|constraint| {
                matches!(&constraint.typ, Type::NamedGeneric(generic) if generic.name.as_str() == head)
            })
            .cloned()
            .collect();
        (!bounds.is_empty()).then_some(bounds)
    }

    /// Resolve an unprefixed (single-segment) path to a local variable, or to a value item it
    /// names (global, function, enum-variant global, numeric type alias). The counterpart to
    /// [`Self::resolve_prefixed_variable`], which resolves a prefixed path's last segment directly
    /// in the already-resolved prefix ([`Self::resolve_value_in_module`] /
    /// [`Self::resolve_value_in_type`]) and never looks for a local variable.
    fn resolve_unprefixed_variable(&mut self, path: TypedPath) -> Option<VariableResolution> {
        // The location of variables or definitions we register (for LSP) must be that of the
        // path's last segment, as intermediate segments solve to other definitions.
        let location = path.last_ident().location();

        // If the Path is being used as an Expression, then it is referring to a global from a separate module
        // Otherwise, then it is referring to an Identifier
        // This lookup allows support of such statements: let x = foo::bar::SOME_GLOBAL + 10;
        // If the expression is a singular indent, we search the resolver's current scope as normal.
        let ident_from_path = self.resolve_path_as_value(path)?;
        Some(self.variable_resolution_from_path_value(ident_from_path, location))
    }

    /// Resolve a module-prefixed path's last segment as a value item, looked up directly in the
    /// already-resolved prefix `module_id`, avoiding re-resolving the whole path now that the prefix
    /// is known to be a module. The module counterpart of [`Self::resolve_value_in_type`].
    fn resolve_value_in_module(
        &mut self,
        module_id: ModuleId,
        last_segment: TypedPathSegment,
    ) -> Option<VariableResolution> {
        let location = last_segment.ident.location();
        let ident = self.lookup_path_as_value_in_module(last_segment, module_id);
        self.variable_resolution_from_value_item(ident, location)
    }

    /// Resolve a type-prefixed path's last segment as a value member (an enum variant or associated
    /// constant) of the already-resolved type `typ`. The type-prefix counterpart of
    /// [`Self::resolve_value_in_module`], avoiding re-resolving the whole path.
    pub(super) fn resolve_value_in_type(
        &mut self,
        last_segment: &TypedPathSegment,
        typ: &Type,
        turbofish: Option<Turbofish>,
    ) -> Option<VariableResolution> {
        let location = last_segment.ident.location();
        let ident = self.lookup_path_as_value_in_type(last_segment, typ, turbofish);
        self.variable_resolution_from_value_item(ident, location)
    }

    /// Finish resolving a value item: build the [`VariableResolution`] it denotes, or report the
    /// error if it could not be resolved as a value.
    fn variable_resolution_from_value_item(
        &mut self,
        ident: Result<PathValue, ResolverError>,
        location: Location,
    ) -> Option<VariableResolution> {
        match ident {
            Ok(ident_from_path) => {
                Some(self.variable_resolution_from_path_value(ident_from_path, location))
            }
            Err(error) => {
                self.push_err(error);
                None
            }
        }
    }

    /// Build the [`VariableResolution`] an already-resolved [`PathValue`] denotes, registering
    /// the reference (for LSP) along the way.
    fn variable_resolution_from_path_value(
        &mut self,
        ident_from_path: PathValue,
        location: Location,
    ) -> VariableResolution {
        match ident_from_path {
            PathValue::Variable(variable) => {
                self.handle_local_variable(&variable);
                let hir_ident = HirIdent::non_trait_method(variable.ident.id, location);
                VariableResolution::Ident(hir_ident, None)
            }
            PathValue::Definition { id, item } => {
                self.handle_definition_id(id, location);
                let hir_ident = HirIdent::non_trait_method(id, location);
                VariableResolution::Ident(hir_ident, Some(item))
            }
            PathValue::TypeAlias(type_alias_id) => VariableResolution::TypeAlias(type_alias_id),
        }
    }

    /// Solve any generics that are part of the path before the function, for example:
    ///
    /// ```noir
    /// foo::Bar::<i32>::baz
    /// ```
    /// Solve `<i32>` above
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_item_turbofish_and_self_type(
        &mut self,
        item: PathResolutionItem,
    ) -> (Vec<Type>, Option<Type>) {
        let mut errors = Vec::new();
        let result = match item {
            PathResolutionItem::Method(struct_id, Some(generics), _func_id) => {
                let generics = self.resolve_struct_id_turbofish_generics(
                    struct_id,
                    Some(generics),
                    &mut errors,
                );
                (generics, None)
            }
            PathResolutionItem::SelfMethod(_) => {
                let generics = if let Some(Type::DataType(_, generics)) = &self.self_type {
                    generics.clone()
                } else {
                    Vec::new()
                };
                (generics, None)
            }
            PathResolutionItem::TypeAliasFunction(type_alias_id, generics, _func_id) => {
                let type_alias = self.interner.get_type_alias(type_alias_id);
                let type_alias = type_alias.borrow();
                let generics = self.resolve_type_alias_id_turbofish_generics(
                    type_alias_id,
                    generics,
                    &mut errors,
                );

                // Now instantiate the underlying struct or alias with those generics, the struct might
                // have more generics than those in the alias, like in this example:
                //
                // type Alias<T> = Struct<T, i32>;
                let generics = get_type_alias_generics(&type_alias, &generics);
                (generics, None)
            }
            PathResolutionItem::TraitFunction(trait_id, Some(generics), _func_id) => {
                let trait_ = self.interner.get_trait(trait_id);
                let kinds = vecmap(&trait_.generics, |generic| generic.kind());
                let trait_generics =
                    vecmap(&kinds, |kind| self.interner.next_type_variable_with_kind(kind.clone()));

                let generics = self.resolve_trait_turbofish_generics(
                    &trait_.name.to_string(),
                    kinds,
                    trait_generics,
                    Some(generics.generics),
                    generics.location,
                    &mut errors,
                );
                (generics, None)
            }
            PathResolutionItem::TypeTraitFunction(self_type, _trait_id, _func_id) => {
                (Vec::new(), Some(self_type))
            }
            PathResolutionItem::PrimitiveFunction(primitive_type, turbofish, _func_id) => {
                let (typ, has_generics) = self.instantiate_primitive_type_with_turbofish(
                    primitive_type,
                    turbofish,
                    &mut errors,
                );
                let generics = if has_generics {
                    match typ {
                        Type::String(length) => vec![*length],
                        Type::FmtString(length, element) => vec![*length, *element],
                        _ => {
                            unreachable!("ICE: Primitive type has been specified to have generics")
                        }
                    }
                } else {
                    Vec::new()
                };
                (generics, None)
            }
            PathResolutionItem::Method(_, None, _)
            | PathResolutionItem::TraitFunction(_, None, _)
            | PathResolutionItem::Module(..)
            | PathResolutionItem::Type(..)
            | PathResolutionItem::TypeAlias(..)
            | PathResolutionItem::PrimitiveType(..)
            | PathResolutionItem::Trait(..)
            | PathResolutionItem::TraitAssociatedType(..)
            | PathResolutionItem::Global(..)
            | PathResolutionItem::EnumVariant(..)
            | PathResolutionItem::ModuleFunction(..)
            | PathResolutionItem::TraitConstant(..) => (Vec::new(), None),
        };
        self.push_errors(errors);
        result
    }

    /// Elaborates a type path used in an expression, e.g. `Type::method::<Args>`
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn elaborate_type_path(&mut self, path: TypePath) -> (ExprId, Type) {
        let typ_location = path.typ.location;
        let turbofish = path.turbofish;
        let wildcard_allowed = WildcardAllowed::Yes;
        let typ = self.use_type(path.typ, wildcard_allowed);
        self.elaborate_type_path_impl(typ, &path.item, turbofish, typ_location)
    }

    /// Variant of [`Self::elaborate_type_path_impl_inner`] that accepts unresolved generics.
    #[tracing::instrument(level = "trace", skip_all)]
    fn elaborate_type_path_impl(
        &mut self,
        typ: Type,
        ident: &Ident,
        turbofish: Option<GenericTypeArgs>,
        typ_location: Location,
    ) -> (ExprId, Type) {
        let ident_location = ident.location();
        let check_self_param = false;

        self.interner.push_type_ref_location(&typ, typ_location);

        // A type member may be an enum-variant constructor or a trait associated constant (e.g.
        // `<Foo>::N`, `<E>::A`), which live in the value namespace rather than among the type's
        // methods; resolve those exactly as a plain `Foo::N` / `E::A` path does. Methods and
        // associated functions aren't found here, so they fall through to method lookup below.
        let segment = TypedPathSegment::without_generics(ident.clone(), ident_location);
        if let Ok(path_value) = self.lookup_path_as_value_in_type(&segment, &typ, None) {
            let resolution = self.variable_resolution_from_path_value(path_value, ident_location);
            return self.type_path_value_expr(resolution, &typ, ident_location);
        }

        let Some(method) = self.lookup_method(
            &typ,
            ident.as_str(),
            ident_location,
            typ_location,
            check_self_param,
        ) else {
            let error = Expression::new(ExpressionKind::Error, ident_location);
            return self.elaborate_expression(error);
        };

        let func_id = method
            .func_id(self.interner)
            .expect("Expected trait function to be a DefinitionKind::Function");

        let generics =
            turbofish.map(|turbofish| self.use_type_args(turbofish, func_id, ident_location).0);

        self.elaborate_type_path_impl_inner(&typ, typ_location, ident_location, method, generics)
    }

    /// Build the expression for a type-path member (`<Type>::member`) that resolved to a value — an
    /// enum-variant constructor or an associated constant. An enum-variant constructor is generic
    /// over its enum's generics, so those are bound from the receiver type `typ` (`<E<bool>>::A` has
    /// type `E<bool>`, not an unbound `E<_>`), exactly as the segment turbofish binds them for a
    /// plain `E::<bool>::A` path.
    fn type_path_value_expr(
        &mut self,
        resolution: VariableResolution,
        typ: &Type,
        location: Location,
    ) -> (ExprId, Type) {
        // A value member of a type is always an enum-variant constructor or an associated
        // constant, both of which resolve to an ident (never a local variable or a numeric type
        // alias).
        let VariableResolution::Ident(hir_ident, item) = resolution else {
            unreachable!("a type's value member always resolves to an ident");
        };

        let mut bindings = TypeBindings::default();
        if matches!(item, Some(PathResolutionItem::EnumVariant(_)))
            && let Type::DataType(_, generics) = typ
        {
            let turbofish = vecmap(generics, |generic| Located::from(location, generic.clone()));
            self.bind_enum_variant_global_turbofish(
                hir_ident.id,
                &turbofish,
                location,
                &mut bindings,
            );
        }
        let id = self.intern_expr(HirExpression::Ident(hir_ident.clone(), None), location);
        let typ = self.type_check_variable_with_bindings(hir_ident, &id, None, bindings);
        let id = self.intern_expr_type(id, typ.clone());
        (id, typ)
    }

    /// Variant of [`Self::elaborate_type_path_impl_inner`] that accepts already resolved generics.
    /// Used when the turbofish generics have already been resolved.
    #[tracing::instrument(level = "trace", skip_all)]
    fn elaborate_type_path_impl_with_resolved_generics(
        &mut self,
        typ: Type,
        ident: &Ident,
        resolved_generics: Option<Vec<Type>>,
        typ_location: Location,
    ) -> (ExprId, Type) {
        let ident_location = ident.location();
        let check_self_param = false;

        self.interner.push_type_ref_location(&typ, typ_location);

        let Some(method) = self.lookup_method(
            &typ,
            ident.as_str(),
            ident_location,
            typ_location,
            check_self_param,
        ) else {
            let error = Expression::new(ExpressionKind::Error, ident_location);
            return self.elaborate_expression(error);
        };

        self.elaborate_type_path_impl_inner(
            &typ,
            typ_location,
            ident_location,
            method,
            resolved_generics,
        )
    }

    /// Common implementation for type path impl variants.
    #[tracing::instrument(level = "trace", skip_all)]
    fn elaborate_type_path_impl_inner(
        &mut self,
        typ: &Type,
        _typ_location: Location,
        ident_location: Location,
        method: HirMethodReference,
        generics: Option<Vec<Type>>,
    ) -> (ExprId, Type) {
        let func_id = method
            .func_id(self.interner)
            .expect("Expected trait function to be a DefinitionKind::Function");

        let id = self.interner.function_definition_id(func_id);

        let impl_kind = match method {
            HirMethodReference::FuncId(_) => ImplKind::NotATraitMethod,
            HirMethodReference::TraitItemId(HirTraitMethodReference {
                definition,
                trait_id,
                trait_generics,
                assumed: _,
            }) => {
                let mut constraint =
                    self.interner.get_trait(trait_id).as_constraint(ident_location);
                constraint.trait_bound.trait_generics = trait_generics;
                ImplKind::TraitItem(TraitItem { definition, constraint, assumed: false })
            }
        };

        let ident = HirIdent { location: ident_location, id, impl_kind };
        let id =
            self.intern_expr(HirExpression::Ident(ident.clone(), generics.clone()), ident_location);

        // If the method has a self type (it's an impl or trait impl), bind `typ` to the instantiated self type.
        let (self_type_generics_count, function_typ, function_self_type) = self
            .with_function_meta(func_id, |meta| {
                (meta.impl_generics_count(), meta.typ.clone(), meta.self_type.clone())
            });
        let bindings = if self_type_generics_count > 0 {
            if let Type::Forall(type_vars, _) = &function_typ
                && let Some(self_type) = &function_self_type
            {
                // Only instantiate type vars corresponding to the `self` type, not to function direct generics
                let type_vars =
                    type_vars.iter().take(self_type_generics_count).cloned().collect::<Vec<_>>();
                let (self_type, instantiation_bindings) =
                    self_type.substitute_type_vars_with_fresh_type_vars(&type_vars, self.interner);
                let _ = typ.unify(&self_type);
                instantiation_bindings
            } else {
                TypeBindings::default()
            }
        } else {
            TypeBindings::default()
        };

        let typ = self.type_check_variable_with_bindings(ident, &id, generics, bindings);
        let id = self.intern_expr_type(id, typ.clone());

        (id, typ)
    }

    /// Given an [`HirIdent`], look up its definition, and:
    /// * mark it as referenced at the ident [Location] (LSP mode only)
    /// * mark the item currently being elaborated as a dependency of it
    /// * elaborate a global definition, if needed
    /// * add local identifiers to lambda captures
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn handle_definition_id(&mut self, definition_id: DefinitionId, location: Location) {
        match self.interner.definition(definition_id).kind {
            DefinitionKind::Function(func_id) => {
                if let Some(current_item) = self.current_item {
                    self.interner.add_function_dependency(current_item, func_id);
                }

                self.interner.add_function_reference(func_id, location);
            }
            DefinitionKind::Global(global_id) => {
                self.elaborate_global_if_unresolved(&global_id);
                if let Some(current_item) = self.current_item {
                    self.interner.add_global_dependency(current_item, global_id);
                }

                let global = self.interner.get_global_definition(global_id);
                if global.comptime && !self.in_comptime_context() {
                    self.push_err(ResolverError::ComptimeGlobalInNonComptimeCode {
                        location,
                        name: global.name.clone(),
                    });
                }

                self.interner.add_global_reference(global_id, location);
            }
            DefinitionKind::NumericGeneric(_, ref numeric_typ) => {
                // Initialize numeric generics to a polymorphic integer type in case
                // they're used in expressions. We must do this here since type_check_variable
                // does not check definition kinds and otherwise expects parameters to
                // already be typed.
                if self.interner.definition_type(definition_id) == Type::Error {
                    let type_var_kind = Kind::Numeric(numeric_typ.clone());
                    let typ = self.type_variable_with_kind(type_var_kind);
                    self.interner.push_definition_type(definition_id, typ);
                }
            }
            DefinitionKind::Local(_) => {
                // Handled separately in `handle_local_variable`
            }
            DefinitionKind::AssociatedConstant(..) => {
                // Nothing to do here
            }
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn handle_local_variable(&mut self, variable: &Variable) {
        self.check_if_variable_is_captured_by_closure(variable);
        let hir_ident = &variable.ident;
        self.interner.add_local_reference(hir_ident.id, hir_ident.location);
    }

    /// Starting with empty bindings, perform the type checking of an interned expression
    /// and a corresponding identifier, returning the instantiated [Type].
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn type_check_variable(
        &mut self,
        ident: HirIdent,
        expr_id: &PushedExpr<HasLocation>,
        generics: Option<Vec<Type>>,
    ) -> Type {
        let bindings = TypeBindings::default();
        self.type_check_variable_with_bindings(ident, expr_id, generics, bindings)
    }

    /// Perform the type checking of an interned expression and a corresponding identifier,
    /// returning the instantiated [Type].
    ///
    /// The instantiation bindings are pushed as required type variables on the current
    /// function context, to be checked at end-of-function. `push_required_type_variable`
    /// already skips this in a comptime context, where unbound generics on quoted typed
    /// expressions are expected.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn type_check_variable_with_bindings(
        &mut self,
        ident: HirIdent,
        expr_id: &PushedExpr<HasLocation>,
        generics: Option<Vec<Type>>,
        mut bindings: TypeBindings,
    ) -> Type {
        // Add type bindings from any constraints that were used.
        // We need to do this first since otherwise instantiating the type below
        // will replace each trait generic with a fresh type variable, rather than
        // the type used in the trait constraint (if it exists). See #4088.
        if let ImplKind::TraitItem(method) = &ident.impl_kind {
            self.bind_generics_from_trait_constraint(
                &method.constraint,
                method.assumed,
                &mut bindings,
            );
        }

        // If a global variable hasn't been defined yet, then we are most likely dealing with a self-dependency-cycle.
        let definition = self.interner.definition(ident.id);
        // Some associated constants also have Global as Kind, and they are not defined when look them up here; want to restrict to global `let` statements.
        if self.in_comptime_context()
            && definition.kind.is_global()
            && self.interner.try_definition_type(ident.id).is_none()
        {
            self.push_err(ResolverError::DependencyCycle {
                location: ident.location,
                item: definition.name.clone(),
                cycle: "the variable definition type hasn't been resolved yet".to_string(),
            });
            return Type::Error;
        }

        let func_id = match definition.kind {
            DefinitionKind::Function(func_id) => Some(func_id),
            _ => None,
        };

        // If the variable is a function whose meta hasn't been resolved yet, resolve
        // it now. This handles forward references — a global's RHS may name a
        // function whose meta would otherwise only be drained at end-of-elaboration.
        if let Some(func_id) = func_id {
            let item_name = definition.name.clone();
            self.define_function_meta_if_undefined(func_id);

            // If lazy resolution leaves the meta still unset, the function is currently
            // mid-resolution and we have a dependency cycle (e.g. `global F = f();`
            // combined with `fn f(_: [u8; F]) {}`).
            if self.interner.try_function_meta(&func_id).is_none() {
                self.push_err(ResolverError::DependencyCycle {
                    location: ident.location,
                    item: item_name,
                    cycle: "the function signature hasn't been resolved yet".to_string(),
                });
                return Type::Error;
            }
        }

        // An identifiers type may be forall-quantified in the case of generic functions.
        // E.g. `fn foo<T>(t: T, field: Field) -> T` has type `forall T. fn(T, Field) -> T`.
        // We must instantiate identifiers at every call site to replace this T with a new type
        // variable to handle generic functions.
        let t = self.type_substitute_trait_as_type(&ident);

        let direct_generic_ids = match func_id {
            Some(function) => vecmap(&self.function_meta(function).direct_generics, |generic| {
                generic.type_var.id()
            }),
            None => Vec::new(),
        };

        let location = self.interner.expr_location(expr_id);

        // This instantiates a trait's generics as well which need to be set
        // when the constraint below is later solved for when the function is
        // finished. How to link the two?
        let (typ, bindings) =
            self.instantiate(t, bindings, generics, &direct_generic_ids, location);

        if let ImplKind::TraitItem(mut method) = ident.impl_kind {
            method.constraint.apply_bindings(&bindings);
            if method.assumed {
                let trait_generics = method.constraint.trait_bound.trait_generics.clone();
                let object_type = method.constraint.typ;
                let trait_impl = TraitImplKind::Assumed { object_type, trait_generics };
                self.interner.select_impl_for_expression(**expr_id, trait_impl);
            } else {
                // this constraint should lead to choosing a trait impl method
                self.push_trait_constraint(method.constraint, **expr_id, true);
            }
        }

        // Push any trait constraints required by this definition to the context
        // to be checked later when the type of this variable is further constrained.
        //
        // This must be done before the above trait constraint in case the above one further
        // restricts types.
        //
        // For example, in this code:
        //
        // ```noir
        // trait One {}
        //
        // trait Two<O: One> {
        //     fn new() -> Self;
        // }
        //
        // fn foo<X: One, T: Two<X>>() {
        //     let _: T = Two::new();
        // }
        // ```
        //
        // when type-checking `Two::new` we'll have a return type `'2` which is constrained by `'2: Two<'1>`.
        // Then the definition for `new` has a constraint on it, `O: One`, which translates to `'1: One`.
        //
        // Because of the explicit type in the `let`, `'2` will be unified with `T`.
        // Then we must first verify the constraint `'2: Two<'1>`, which is now `T: Two<'1>`, to find
        // that the implementation is the assumed one `T: Two<X>` so that `'1` is bound to `X`.
        // Then we can successfully verify the constraint `'1: One` which now became `X: One` which holds
        // because of the assumed constraint.
        //
        // If we try to find a trait implementation for `'1` before finding one for `'2` we'll never find it.
        if let Some(function) = func_id {
            let function = self.function_meta(function);
            for mut constraint in function.all_trait_constraints().cloned().collect::<Vec<_>>() {
                constraint.apply_bindings(&bindings);

                // This constraint shouldn't lead to choosing a trait impl method
                self.push_trait_constraint(constraint, **expr_id, false);
            }
        }

        // Record required type variables in a predictable order to avoid nondeterminism in error messages.
        let required_type_variables = btree_map(bindings.values(), |(type_variable, _, typ)| {
            (type_variable.id(), typ.clone())
        });

        for (type_variable_id, typ) in required_type_variables {
            self.push_required_type_variable(
                type_variable_id,
                typ,
                BindableTypeVariableKind::Ident(ident.id),
                ident.location,
            );
        }

        self.interner.store_instantiation_bindings(**expr_id, bindings);
        typ
    }

    /// If the type of the [`HirIdent`] is a function that returns an `impl Trait`,
    /// then it might need elaboration before it can be substituted to a [Type].
    /// Try to elaborate it now.
    ///
    /// Returns a type error if the callee cannot be resolved on a second try,
    /// which indicates a dependency cycle.
    #[tracing::instrument(level = "trace", skip_all)]
    fn type_substitute_trait_as_type(&mut self, ident: &HirIdent) -> Type {
        let func_id = match self.interner.id_type_substitute_trait_as_type(ident.id) {
            Ok(typ) => return typ,
            Err(func_id) => func_id,
        };

        // Try to elaborate, so we get an expression for the body.
        self.elaborate_function(func_id);

        // Now try again. If it's still not working, give up.
        match self.interner.id_type_substitute_trait_as_type(ident.id) {
            Ok(typ) => typ,
            Err(_) => {
                let def = self.interner.definition(ident.id);
                self.push_err(ResolverError::DependencyCycle {
                    location: ident.location,
                    item: def.name.clone(),
                    cycle: "'impl Trait' could not be resolved to the type of the function body"
                        .to_string(),
                });
                Type::Error
            }
        }
    }

    /// Instantiate a [Type] with the given [`TypeBindings`], returning the bindings potentially
    /// extended from any turbofish generics.
    ///
    /// If there are turbofish generics and their number matches the expectations of the function,
    /// those are used as well, otherwise they are ignored and an error is pushed.
    #[tracing::instrument(level = "trace", skip_all)]
    fn instantiate(
        &mut self,
        typ: Type,
        bindings: TypeBindings,
        turbofish_generics: Option<Vec<Type>>,
        direct_generic_ids: &[TypeVariableId],
        location: Location,
    ) -> (Type, TypeBindings) {
        match turbofish_generics {
            Some(turbofish_generics) => {
                let function_generic_count = direct_generic_ids.len();
                let forall_generic_count =
                    if let Type::Forall(generics, _) = &typ { generics.len() } else { 0 };

                if turbofish_generics.len() != function_generic_count {
                    let type_check_err = TypeCheckError::IncorrectTurbofishGenericCount {
                        expected_count: function_generic_count,
                        actual_count: turbofish_generics.len(),
                        location,
                    };
                    self.push_err(CompilationError::TypeError(type_check_err));
                    typ.instantiate_with_bindings(bindings, self.interner)
                } else if forall_generic_count < function_generic_count {
                    // The next branch asserts that the number of turbofish-bound typevars matches
                    // `direct_generic_ids`, but if the forall has fewer generics than the function
                    // (e.g. when the function's generics have duplicates that were filtered out)
                    // this can never hold. A duplicate error has already been reported elsewhere,
                    // so bail out silently.
                    self.push_err(TypeCheckError::expecting_other_error(
                        "forall has fewer generics than function",
                        location,
                    ));
                    (Type::Error, bindings)
                } else {
                    typ.instantiate_with_bindings_and_turbofish(
                        bindings,
                        turbofish_generics,
                        self.interner,
                        direct_generic_ids,
                    )
                }
            }
            None => typ.instantiate_with_bindings(bindings, self.interner),
        }
    }
}

/// Returns the name of the data type a [Type] resolves to, looking through a `Forall` quantifier.
fn data_type_name(typ: &Type) -> Option<String> {
    match typ {
        Type::Forall(_, body) => data_type_name(body),
        Type::DataType(datatype, _) => Some(datatype.borrow().name.to_string()),
        _ => None,
    }
}

/// Bind the generics of the [Type] aliased by the [`TypeAlias`] to a list of generic arguments,
/// recursively expanding the generics aliased aliases, finally returning the generics of the
/// innermost aliased struct.
///
/// Panics if it encounters a type other than alias or struct.
fn get_type_alias_generics(type_alias: &TypeAlias, generics: &[Type]) -> Vec<Type> {
    let typ = type_alias.get_type(generics);
    match typ {
        Type::DataType(_, generics) => generics,
        Type::Alias(type_alias, generics) => {
            get_type_alias_generics(&type_alias.borrow(), &generics)
        }
        // Primitive types have no generics
        _ => Vec::new(),
    }
}
