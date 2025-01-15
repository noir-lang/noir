use std::{borrow::Cow, rc::Rc};

use im::HashSet;
use iter_extended::vecmap;
use noirc_errors::{Location, Span};
use rustc_hash::FxHashMap as HashMap;

use crate::{
    ast::{
        AsTraitPath, BinaryOpKind, GenericTypeArgs, Ident, IntegerBitSize, Path, PathKind,
        Signedness, UnaryOp, UnresolvedGeneric, UnresolvedGenerics, UnresolvedType,
        UnresolvedTypeData, UnresolvedTypeExpression, WILDCARD_TYPE,
    },
    hir::{
        def_collector::dc_crate::CompilationError,
        def_map::{fully_qualified_module_path, ModuleDefId},
        resolution::{errors::ResolverError, import::PathResolutionError},
        type_check::{
            generics::{Generic, TraitGenerics},
            NoMatchingImplFoundError, Source, TypeCheckError,
        },
    },
    hir_def::{
        expr::{
            HirBinaryOp, HirCallExpression, HirExpression, HirLiteral, HirMemberAccess,
            HirMethodReference, HirPrefixExpression, TraitMethod,
        },
        function::FuncMeta,
        stmt::HirStatement,
        traits::{NamedType, ResolvedTraitBound, Trait, TraitConstraint},
    },
    node_interner::{
        DependencyId, ExprId, FuncId, GlobalValue, ImplSearchErrorKind, NodeInterner, TraitId,
        TraitImplKind, TraitMethodId,
    },
    token::SecondaryAttribute,
    Generics, Kind, ResolvedGeneric, Type, TypeBinding, TypeBindings, UnificationError,
};

use super::{lints, path_resolution::PathResolutionItem, Elaborator, UnsafeBlockStatus};

pub const SELF_TYPE_NAME: &str = "Self";

pub(super) struct TraitPathResolution {
    pub(super) method: TraitMethod,
    pub(super) item: Option<PathResolutionItem>,
    pub(super) errors: Vec<PathResolutionError>,
}

impl<'context> Elaborator<'context> {
    /// Translates an UnresolvedType to a Type with a `TypeKind::Normal`
    pub(crate) fn resolve_type(&mut self, typ: UnresolvedType) -> Type {
        let span = typ.span;
        let resolved_type = self.resolve_type_inner(typ, &Kind::Normal);
        if resolved_type.is_nested_slice() {
            self.push_err(ResolverError::NestedSlices { span });
        }
        resolved_type
    }

    /// Translates an UnresolvedType into a Type and appends any
    /// freshly created TypeVariables created to new_variables.
    pub fn resolve_type_inner(&mut self, typ: UnresolvedType, kind: &Kind) -> Type {
        use crate::ast::UnresolvedTypeData::*;

        let span = typ.span;
        let (named_path_span, is_self_type_name, is_synthetic) =
            if let Named(ref named_path, _, synthetic) = typ.typ {
                (
                    Some(named_path.last_ident().span()),
                    named_path.last_ident().is_self_type_name(),
                    synthetic,
                )
            } else {
                (None, false, false)
            };

        let resolved_type = match typ.typ {
            FieldElement => Type::FieldElement,
            Array(size, elem) => {
                let elem = Box::new(self.resolve_type_inner(*elem, kind));
                let size = self.convert_expression_type(size, &Kind::u32(), span);
                Type::Array(Box::new(size), elem)
            }
            Slice(elem) => {
                let elem = Box::new(self.resolve_type_inner(*elem, kind));
                Type::Slice(elem)
            }
            Expression(expr) => self.convert_expression_type(expr, kind, span),
            Integer(sign, bits) => Type::Integer(sign, bits),
            Bool => Type::Bool,
            String(size) => {
                let resolved_size = self.convert_expression_type(size, &Kind::u32(), span);
                Type::String(Box::new(resolved_size))
            }
            FormatString(size, fields) => {
                let resolved_size = self.convert_expression_type(size, &Kind::u32(), span);
                let fields = self.resolve_type_inner(*fields, kind);
                Type::FmtString(Box::new(resolved_size), Box::new(fields))
            }
            Quoted(quoted) => {
                let in_function = matches!(self.current_item, Some(DependencyId::Function(_)));
                if in_function && !self.in_comptime_context() {
                    let span = typ.span;
                    let typ = quoted.to_string();
                    self.push_err(ResolverError::ComptimeTypeInRuntimeCode { span, typ });
                }
                Type::Quoted(quoted)
            }
            Unit => Type::Unit,
            Unspecified => {
                let span = typ.span;
                self.push_err(TypeCheckError::UnspecifiedType { span });
                Type::Error
            }
            Error => Type::Error,
            Named(path, args, _) => self.resolve_named_type(path, args),
            TraitAsType(path, args) => self.resolve_trait_as_type(path, args),

            Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| self.resolve_type_inner(field, kind)))
            }
            Function(args, ret, env, unconstrained) => {
                let args = vecmap(args, |arg| self.resolve_type_inner(arg, kind));
                let ret = Box::new(self.resolve_type_inner(*ret, kind));
                let env_span = env.span;

                let env = Box::new(self.resolve_type_inner(*env, kind));

                match *env {
                    Type::Unit | Type::Tuple(_) | Type::NamedGeneric(_, _) => {
                        Type::Function(args, ret, env, unconstrained)
                    }
                    _ => {
                        self.push_err(ResolverError::InvalidClosureEnvironment {
                            typ: *env,
                            span: env_span,
                        });
                        Type::Error
                    }
                }
            }
            MutableReference(element) => {
                Type::MutableReference(Box::new(self.resolve_type_inner(*element, kind)))
            }
            Parenthesized(typ) => self.resolve_type_inner(*typ, kind),
            Resolved(id) => self.interner.get_quoted_type(id).clone(),
            AsTraitPath(path) => self.resolve_as_trait_path(*path),
            Interned(id) => {
                let typ = self.interner.get_unresolved_type_data(id).clone();
                return self.resolve_type_inner(UnresolvedType { typ, span }, kind);
            }
        };

        let location = Location::new(named_path_span.unwrap_or(typ.span), self.file);
        match resolved_type {
            Type::Struct(ref struct_type, _) => {
                // Record the location of the type reference
                self.interner.push_type_ref_location(resolved_type.clone(), location);
                if !is_synthetic {
                    self.interner.add_struct_reference(
                        struct_type.borrow().id,
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

        if !kind.unifies(&resolved_type.kind()) {
            let expected_typ_err = CompilationError::TypeError(TypeCheckError::TypeKindMismatch {
                expected_kind: kind.to_string(),
                expr_kind: resolved_type.kind().to_string(),
                expr_span: span,
            });
            self.errors.push((expected_typ_err, self.file));
            return Type::Error;
        }

        resolved_type
    }

    pub fn find_generic(&self, target_name: &str) -> Option<&ResolvedGeneric> {
        self.generics.iter().find(|generic| generic.name.as_ref() == target_name)
    }

    // Resolve Self::Foo to an associated type on the current trait or trait impl
    fn lookup_associated_type_on_self(&self, path: &Path) -> Option<Type> {
        if path.segments.len() == 2 && path.first_name() == Some(SELF_TYPE_NAME) {
            if let Some(trait_id) = self.current_trait {
                let the_trait = self.interner.get_trait(trait_id);
                if let Some(typ) = the_trait.get_associated_type(path.last_name()) {
                    return Some(typ.clone().as_named_generic());
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

    fn resolve_named_type(&mut self, path: Path, args: GenericTypeArgs) -> Type {
        if args.is_empty() {
            if let Some(typ) = self.lookup_generic_or_global_type(&path) {
                return typ;
            }
        }

        // Check if the path is a type variable first. We currently disallow generics on type
        // variables since we do not support higher-kinded types.
        if path.segments.len() == 1 {
            let name = path.last_name();

            if name == SELF_TYPE_NAME {
                if let Some(self_type) = self.self_type.clone() {
                    if !args.is_empty() {
                        self.push_err(ResolverError::GenericsOnSelfType { span: path.span() });
                    }
                    return self_type;
                }
            } else if name == WILDCARD_TYPE {
                return self.interner.next_type_variable_with_kind(Kind::Any);
            }
        } else if let Some(typ) = self.lookup_associated_type_on_self(&path) {
            if !args.is_empty() {
                self.push_err(ResolverError::GenericsOnAssociatedType { span: path.span() });
            }
            return typ;
        }

        let span = path.span();

        if let Some(type_alias) = self.lookup_type_alias(path.clone()) {
            let id = type_alias.borrow().id;
            let (args, _) = self.resolve_type_args(args, id, path.span());

            if let Some(item) = self.current_item {
                self.interner.add_type_alias_dependency(item, id);
            }

            // Collecting Type Alias references [Location]s to be used by LSP in order
            // to resolve the definition of the type alias
            self.interner.add_type_alias_ref(id, Location::new(span, self.file));

            // Because there is no ordering to when type aliases (and other globals) are resolved,
            // it is possible for one to refer to an Error type and issue no error if it is set
            // equal to another type alias. Fixing this fully requires an analysis to create a DFG
            // of definition ordering, but for now we have an explicit check here so that we at
            // least issue an error that the type was not found instead of silently passing.
            return Type::Alias(type_alias, args);
        }

        match self.lookup_struct_or_error(path) {
            Some(struct_type) => {
                if self.resolving_ids.contains(&struct_type.borrow().id) {
                    self.push_err(ResolverError::SelfReferentialStruct {
                        span: struct_type.borrow().name.span(),
                    });

                    return Type::Error;
                }

                if !self.in_contract()
                    && self
                        .interner
                        .struct_attributes(&struct_type.borrow().id)
                        .iter()
                        .any(|attr| matches!(attr, SecondaryAttribute::Abi(_)))
                {
                    self.push_err(ResolverError::AbiAttributeOutsideContract {
                        span: struct_type.borrow().name.span(),
                    });
                }

                let (args, _) = self.resolve_type_args(args, struct_type.borrow(), span);

                if let Some(current_item) = self.current_item {
                    let dependency_id = struct_type.borrow().id;
                    self.interner.add_type_dependency(current_item, dependency_id);
                }

                Type::Struct(struct_type, args)
            }
            None => Type::Error,
        }
    }

    fn resolve_trait_as_type(&mut self, path: Path, args: GenericTypeArgs) -> Type {
        // Fetch information needed from the trait as the closure for resolving all the `args`
        // requires exclusive access to `self`
        let span = path.span;
        let trait_as_type_info = self.lookup_trait_or_error(path).map(|t| t.id);

        if let Some(id) = trait_as_type_info {
            let (ordered, named) = self.resolve_type_args(args, id, span);
            let name = self.interner.get_trait(id).name.to_string();
            let generics = TraitGenerics { ordered, named };
            Type::TraitAsType(id, Rc::new(name), generics)
        } else {
            Type::Error
        }
    }

    /// Identical to `resolve_type_args` but does not allow
    /// associated types to be elided since trait impls must specify them.
    pub(super) fn resolve_trait_args_from_trait_impl(
        &mut self,
        args: GenericTypeArgs,
        item: TraitId,
        span: Span,
    ) -> (Vec<Type>, Vec<NamedType>) {
        self.resolve_type_args_inner(args, item, span, false)
    }

    pub(super) fn resolve_type_args(
        &mut self,
        args: GenericTypeArgs,
        item: impl Generic,
        span: Span,
    ) -> (Vec<Type>, Vec<NamedType>) {
        self.resolve_type_args_inner(args, item, span, true)
    }

    pub(super) fn resolve_type_args_inner(
        &mut self,
        mut args: GenericTypeArgs,
        item: impl Generic,
        span: Span,
        allow_implicit_named_args: bool,
    ) -> (Vec<Type>, Vec<NamedType>) {
        let expected_kinds = item.generics(self.interner);

        if args.ordered_args.len() != expected_kinds.len() {
            self.push_err(TypeCheckError::GenericCountMismatch {
                item: item.item_name(self.interner),
                expected: expected_kinds.len(),
                found: args.ordered_args.len(),
                span,
            });
            let error_type = UnresolvedTypeData::Error.with_span(span);
            args.ordered_args.resize(expected_kinds.len(), error_type);
        }

        let ordered_args = expected_kinds.iter().zip(args.ordered_args);
        let ordered =
            vecmap(ordered_args, |(generic, typ)| self.resolve_type_inner(typ, &generic.kind()));

        let mut associated = Vec::new();

        if item.accepts_named_type_args() {
            associated = self.resolve_associated_type_args(
                args.named_args,
                item,
                span,
                allow_implicit_named_args,
            );
        } else if !args.named_args.is_empty() {
            let item_kind = item.item_kind();
            self.push_err(ResolverError::NamedTypeArgs { span, item_kind });
        }

        (ordered, associated)
    }

    fn resolve_associated_type_args(
        &mut self,
        args: Vec<(Ident, UnresolvedType)>,
        item: impl Generic,
        span: Span,
        allow_implicit_named_args: bool,
    ) -> Vec<NamedType> {
        let mut seen_args = HashMap::default();
        let mut required_args = item.named_generics(self.interner);
        let mut resolved = Vec::with_capacity(required_args.len());

        // Go through each argument to check if it is in our required_args list.
        // If it is remove it from the list, otherwise issue an error.
        for (name, typ) in args {
            let index =
                required_args.iter().position(|item| item.name.as_ref() == &name.0.contents);

            let Some(index) = index else {
                if let Some(prev_span) = seen_args.get(&name.0.contents).copied() {
                    self.push_err(TypeCheckError::DuplicateNamedTypeArg { name, prev_span });
                } else {
                    let item = item.item_name(self.interner);
                    self.push_err(TypeCheckError::NoSuchNamedTypeArg { name, item });
                }
                continue;
            };

            // Remove the argument from the required list so we remember that we already have it
            let expected = required_args.remove(index);
            seen_args.insert(name.0.contents.clone(), name.span());

            let typ = self.resolve_type_inner(typ, &expected.kind());
            resolved.push(NamedType { name, typ });
        }

        // Anything that hasn't been removed yet is missing.
        // Fill it in to avoid a panic if we allow named args to be elided, otherwise error.
        for generic in required_args {
            let name = generic.name.clone();

            if allow_implicit_named_args {
                let name = Ident::new(name.as_ref().clone(), span);
                let typ = self.interner.next_type_variable();
                resolved.push(NamedType { name, typ });
            } else {
                let item = item.item_name(self.interner);
                self.push_err(TypeCheckError::MissingNamedTypeArg { item, span, name });
            }
        }

        resolved
    }

    pub fn lookup_generic_or_global_type(&mut self, path: &Path) -> Option<Type> {
        if path.segments.len() == 1 {
            let name = path.last_name();
            if let Some(generic) = self.find_generic(name) {
                let generic = generic.clone();
                return Some(Type::NamedGeneric(generic.type_var, generic.name));
            }
        } else if let Some(typ) = self.lookup_associated_type_on_self(path) {
            return Some(typ);
        }

        // If we cannot find a local generic of the same name, try to look up a global
        match self.resolve_path_or_error(path.clone()) {
            Ok(PathResolutionItem::Global(id)) => {
                if let Some(current_item) = self.current_item {
                    self.interner.add_global_dependency(current_item, id);
                }

                let reference_location = Location::new(path.span(), self.file);
                self.interner.add_global_reference(id, reference_location);
                let kind = self
                    .interner
                    .get_global_let_statement(id)
                    .map(|let_statement| Kind::numeric(let_statement.r#type))
                    .unwrap_or(Kind::u32());

                let Some(stmt) = self.interner.get_global_let_statement(id) else {
                    if self.elaborate_global_if_unresolved(&id) {
                        return self.lookup_generic_or_global_type(path);
                    } else {
                        let path = path.clone();
                        self.push_err(ResolverError::NoSuchNumericTypeVariable { path });
                        return None;
                    }
                };

                let rhs = stmt.expression;
                let span = self.interner.expr_span(&rhs);

                let GlobalValue::Resolved(global_value) = &self.interner.get_global(id).value
                else {
                    self.push_err(ResolverError::UnevaluatedGlobalType { span });
                    return None;
                };

                let Some(global_value) = global_value.to_field_element() else {
                    let global_value = global_value.clone();
                    if global_value.is_integral() {
                        self.push_err(ResolverError::NegativeGlobalType { span, global_value });
                    } else {
                        self.push_err(ResolverError::NonIntegralGlobalType { span, global_value });
                    }
                    return None;
                };

                let Ok(global_value) = kind.ensure_value_fits(global_value, span) else {
                    self.push_err(ResolverError::GlobalLargerThanKind { span, global_value, kind });
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
        span: Span,
    ) -> Type {
        match length {
            UnresolvedTypeExpression::Variable(path) => {
                let typ = self.resolve_named_type(path, GenericTypeArgs::default());
                self.check_kind(typ, expected_kind, span)
            }
            UnresolvedTypeExpression::Constant(int, _span) => {
                Type::Constant(int, expected_kind.clone())
            }
            UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, span) => {
                let (lhs_span, rhs_span) = (lhs.span(), rhs.span());
                let lhs = self.convert_expression_type(*lhs, expected_kind, lhs_span);
                let rhs = self.convert_expression_type(*rhs, expected_kind, rhs_span);

                match (lhs, rhs) {
                    (Type::Constant(lhs, lhs_kind), Type::Constant(rhs, rhs_kind)) => {
                        if !lhs_kind.unifies(&rhs_kind) {
                            self.push_err(TypeCheckError::TypeKindMismatch {
                                expected_kind: lhs_kind.to_string(),
                                expr_kind: rhs_kind.to_string(),
                                expr_span: span,
                            });
                            return Type::Error;
                        }
                        match op.function(lhs, rhs, &lhs_kind, span) {
                            Ok(result) => Type::Constant(result, lhs_kind),
                            Err(err) => {
                                let err = Box::new(err);
                                self.push_err(ResolverError::BinaryOpError {
                                    lhs,
                                    op,
                                    rhs,
                                    err,
                                    span,
                                });
                                Type::Error
                            }
                        }
                    }
                    (lhs, rhs) => {
                        let infix = Type::InfixExpr(Box::new(lhs), op, Box::new(rhs));
                        Type::CheckedCast { from: Box::new(infix.clone()), to: Box::new(infix) }
                            .canonicalize()
                    }
                }
            }
            UnresolvedTypeExpression::AsTraitPath(path) => {
                let typ = self.resolve_as_trait_path(*path);
                self.check_kind(typ, expected_kind, span)
            }
        }
    }

    fn check_kind(&mut self, typ: Type, expected_kind: &Kind, span: Span) -> Type {
        if !typ.kind().unifies(expected_kind) {
            self.push_err(TypeCheckError::TypeKindMismatch {
                expected_kind: expected_kind.to_string(),
                expr_kind: typ.kind().to_string(),
                expr_span: span,
            });
            return Type::Error;
        }
        typ
    }

    fn resolve_as_trait_path(&mut self, path: AsTraitPath) -> Type {
        let span = path.trait_path.span;
        let Some(trait_id) = self.resolve_trait_by_path(path.trait_path.clone()) else {
            // Error should already be pushed in the None case
            return Type::Error;
        };

        let (ordered, named) = self.resolve_type_args(path.trait_generics.clone(), trait_id, span);
        let object_type = self.resolve_type(path.typ.clone());

        match self.interner.lookup_trait_implementation(&object_type, trait_id, &ordered, &named) {
            Ok(impl_kind) => self.get_associated_type_from_trait_impl(path, impl_kind),
            Err(constraints) => {
                self.push_trait_constraint_error(&object_type, constraints, span);
                Type::Error
            }
        }
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

    // this resolves Self::some_static_method, inside an impl block (where we don't have a concrete self_type)
    // or inside a trait default method.
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_static_method_by_self(&mut self, path: &Path) -> Option<TraitPathResolution> {
        let trait_id = if let Some(current_trait) = self.current_trait {
            current_trait
        } else {
            let trait_impl = self.current_trait_impl?;
            self.interner.try_get_trait_implementation(trait_impl)?.borrow().trait_id
        };

        if path.kind == PathKind::Plain && path.segments.len() == 2 {
            let name = &path.segments[0].ident.0.contents;
            let method = &path.segments[1].ident;

            if name == SELF_TYPE_NAME {
                let the_trait = self.interner.get_trait(trait_id);
                let method = the_trait.find_method(method.0.contents.as_str())?;
                let constraint = the_trait.as_constraint(path.span);
                return Some(TraitPathResolution {
                    method: TraitMethod { method_id: method, constraint, assumed: true },
                    item: None,
                    errors: Vec::new(),
                });
            }
        }
        None
    }

    // this resolves TraitName::some_static_method
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_static_method(&mut self, path: &Path) -> Option<TraitPathResolution> {
        let path_resolution = self.resolve_path(path.clone()).ok()?;
        let func_id = path_resolution.item.function_id()?;
        let meta = self.interner.try_function_meta(&func_id)?;
        let the_trait = self.interner.get_trait(meta.trait_id?);
        let method = the_trait.find_method(path.last_name())?;
        let constraint = the_trait.as_constraint(path.span);
        Some(TraitPathResolution {
            method: TraitMethod { method_id: method, constraint, assumed: false },
            item: Some(path_resolution.item),
            errors: path_resolution.errors,
        })
    }

    // This resolves a static trait method T::trait_method by iterating over the where clause
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed from a where
    // clause. This is always true since this helper searches where clauses for a generic constraint.
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    fn resolve_trait_method_by_named_generic(
        &mut self,
        path: &Path,
    ) -> Option<TraitPathResolution> {
        if path.segments.len() != 2 {
            return None;
        }

        for constraint in self.trait_bounds.clone() {
            if let Type::NamedGeneric(_, name) = &constraint.typ {
                // if `path` is `T::method_name`, we're looking for constraint of the form `T: SomeTrait`
                if path.segments[0].ident.0.contents != name.as_str() {
                    continue;
                }

                let the_trait = self.interner.get_trait(constraint.trait_bound.trait_id);
                if let Some(method) = the_trait.find_method(path.last_name()) {
                    return Some(TraitPathResolution {
                        method: TraitMethod { method_id: method, constraint, assumed: true },
                        item: None,
                        errors: Vec::new(),
                    });
                }
            }
        }
        None
    }

    // Try to resolve the given trait method path.
    //
    // Returns the trait method, trait constraint, and whether the impl is assumed to exist by a where clause or not
    // E.g. `t.method()` with `where T: Foo<Bar>` in scope will return `(Foo::method, T, vec![Bar])`
    pub(super) fn resolve_trait_generic_path(
        &mut self,
        path: &Path,
    ) -> Option<TraitPathResolution> {
        self.resolve_trait_static_method_by_self(path)
            .or_else(|| self.resolve_trait_static_method(path))
            .or_else(|| self.resolve_trait_method_by_named_generic(path))
    }

    pub(super) fn unify(
        &mut self,
        actual: &Type,
        expected: &Type,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        if let Err(UnificationError) = actual.unify(expected) {
            self.errors.push((make_error().into(), self.file));
        }
    }

    /// Do not apply type bindings even after a successful unification.
    /// This function is used by the interpreter for some comptime code
    /// which can change types e.g. on each iteration of a for loop.
    pub fn unify_without_applying_bindings(
        &mut self,
        actual: &Type,
        expected: &Type,
        file: fm::FileId,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let mut bindings = TypeBindings::new();
        if actual.try_unify(expected, &mut bindings).is_err() {
            self.errors.push((make_error().into(), file));
        }
    }

    /// Wrapper of Type::unify_with_coercions using self.errors
    pub(super) fn unify_with_coercions(
        &mut self,
        actual: &Type,
        expected: &Type,
        expression: ExprId,
        span: Span,
        make_error: impl FnOnce() -> TypeCheckError,
    ) {
        let mut errors = Vec::new();
        actual.unify_with_coercions(
            expected,
            expression,
            span,
            self.interner,
            &mut errors,
            make_error,
        );
        self.errors.extend(errors.into_iter().map(|error| (error.into(), self.file)));
    }

    /// Return a fresh integer or field type variable and log it
    /// in self.type_variables to default it later.
    pub(super) fn polymorphic_integer_or_field(&mut self) -> Type {
        let typ = Type::polymorphic_integer_or_field(self.interner);
        self.push_type_variable(typ.clone());
        typ
    }

    /// Return a fresh integer type variable and log it
    /// in self.type_variables to default it later.
    pub(super) fn polymorphic_integer(&mut self) -> Type {
        let typ = Type::polymorphic_integer(self.interner);
        self.push_type_variable(typ.clone());
        typ
    }

    /// Return a fresh integer type variable and log it
    /// in self.type_variables to default it later.
    pub(super) fn type_variable_with_kind(&mut self, type_var_kind: Kind) -> Type {
        let typ = Type::type_variable_with_kind(self.interner, type_var_kind);
        self.push_type_variable(typ.clone());
        typ
    }

    /// Translates a (possibly Unspecified) UnresolvedType to a Type.
    /// Any UnresolvedType::Unspecified encountered are replaced with fresh type variables.
    pub(super) fn resolve_inferred_type(&mut self, typ: UnresolvedType) -> Type {
        match &typ.typ {
            UnresolvedTypeData::Unspecified => {
                self.interner.next_type_variable_with_kind(Kind::Any)
            }
            _ => self.resolve_type(typ),
        }
    }

    /// Insert as many dereference operations as necessary to automatically dereference a method
    /// call object to its base value type T.
    pub(super) fn insert_auto_dereferences(&mut self, object: ExprId, typ: Type) -> (ExprId, Type) {
        if let Type::MutableReference(element) = typ.follow_bindings() {
            let location = self.interner.id_location(object);

            let object = self.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                operator: UnaryOp::Dereference { implicitly_added: true },
                rhs: object,
                trait_method_id: None,
            }));
            self.interner.push_expr_type(object, element.as_ref().clone());
            self.interner.push_expr_location(object, location.span, location.file);

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
        callsite_args: &[(Type, ExprId, Span)],
        span: Span,
    ) -> Type {
        if fn_params.len() != callsite_args.len() {
            self.push_err(TypeCheckError::ParameterCountMismatch {
                expected: fn_params.len(),
                found: callsite_args.len(),
                span,
            });
            return Type::Error;
        }

        for (param, (arg, arg_expr_id, arg_span)) in fn_params.iter().zip(callsite_args) {
            self.unify_with_coercions(arg, param, *arg_expr_id, *arg_span, || {
                TypeCheckError::TypeMismatch {
                    expected_typ: param.to_string(),
                    expr_typ: arg.to_string(),
                    expr_span: *arg_span,
                }
            });
        }

        fn_ret.clone()
    }

    pub(super) fn bind_function_type(
        &mut self,
        function: Type,
        args: Vec<(Type, ExprId, Span)>,
        span: Span,
    ) -> Type {
        // Could do a single unification for the entire function type, but matching beforehand
        // lets us issue a more precise error on the individual argument that fails to type check.
        match function {
            Type::TypeVariable(binding) if binding.kind() == Kind::Normal => {
                if let TypeBinding::Bound(typ) = &*binding.borrow() {
                    return self.bind_function_type(typ.clone(), args, span);
                }

                let ret = self.interner.next_type_variable();
                let args = vecmap(args, |(arg, _, _)| arg);
                let env_type = self.interner.next_type_variable();
                let expected =
                    Type::Function(args, Box::new(ret.clone()), Box::new(env_type), false);

                let expected_kind = expected.kind();
                if let Err(error) = binding.try_bind(expected, &expected_kind, span) {
                    self.push_err(error);
                }
                ret
            }
            // The closure env is ignored on purpose: call arguments never place
            // constraints on closure environments.
            Type::Function(parameters, ret, _env, _unconstrained) => {
                self.bind_function_type_impl(&parameters, &ret, &args, span)
            }
            Type::Error => Type::Error,
            found => {
                self.push_err(TypeCheckError::ExpectedFunction { found, span });
                Type::Error
            }
        }
    }

    pub(super) fn check_cast(
        &mut self,
        from_expr_id: &ExprId,
        from: &Type,
        to: &Type,
        span: Span,
    ) -> Type {
        let from_follow_bindings = from.follow_bindings();

        let from_value_opt = match self.interner.expression(from_expr_id) {
            HirExpression::Literal(HirLiteral::Integer(int, false)) => Some(int),

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
                    span,
                    reason: "casting from a non-integral type is unsupported".into(),
                });
                true
            }
            Type::Error => return Type::Error,
            from => {
                let reason = "casting from this type is unsupported".into();
                self.push_err(TypeCheckError::InvalidCast { from, span, reason });
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
                let reason = format!("casting untyped value ({from_value}) to a type with a maximum size ({to_maximum_size}) that's smaller than it");
                // we warn that the 'to' type is too small for the value
                self.push_err(TypeCheckError::DownsizingCast { from, to, span, reason });
            }
        }

        match to {
            Type::Integer(sign, bits) => Type::Integer(*sign, *bits),
            Type::FieldElement => Type::FieldElement,
            Type::Bool => Type::Bool,
            Type::Error => Type::Error,
            _ => {
                self.push_err(TypeCheckError::UnsupportedCast { span });
                Type::Error
            }
        }
    }

    // Given a binary comparison operator and another type. This method will produce the output type
    // and a boolean indicating whether to use the trait impl corresponding to the operator
    // or not. A value of false indicates the caller to use a primitive operation for this
    // operator, while a true value indicates a user-provided trait impl is required.
    fn comparator_operand_type_rules(
        &mut self,
        lhs_type: &Type,
        rhs_type: &Type,
        op: &HirBinaryOp,
        span: Span,
    ) -> Result<(Type, bool), TypeCheckError> {
        use Type::*;

        match (lhs_type, rhs_type) {
            // Avoid reporting errors multiple times
            (Error, _) | (_, Error) => Ok((Bool, false)),
            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                self.comparator_operand_type_rules(&alias, other, op, span)
            }

            // Matches on TypeVariable must be first to follow any type
            // bindings.
            (TypeVariable(var), other) | (other, TypeVariable(var)) => {
                if let TypeBinding::Bound(ref binding) = &*var.borrow() {
                    return self.comparator_operand_type_rules(other, binding, op, span);
                }

                let use_impl = self.bind_type_variables_for_infix(lhs_type, op, rhs_type, span);
                Ok((Bool, use_impl))
            }
            (Integer(sign_x, bit_width_x), Integer(sign_y, bit_width_y)) => {
                if sign_x != sign_y {
                    return Err(TypeCheckError::IntegerSignedness {
                        sign_x: *sign_x,
                        sign_y: *sign_y,
                        span,
                    });
                }
                if bit_width_x != bit_width_y {
                    return Err(TypeCheckError::IntegerBitWidth {
                        bit_width_x: *bit_width_x,
                        bit_width_y: *bit_width_y,
                        span,
                    });
                }
                Ok((Bool, false))
            }
            (FieldElement, FieldElement) => {
                if op.kind.is_valid_for_field_type() {
                    Ok((Bool, false))
                } else {
                    Err(TypeCheckError::FieldComparison { span })
                }
            }

            // <= and friends are technically valid for booleans, just not very useful
            (Bool, Bool) => Ok((Bool, false)),

            (lhs, rhs) => {
                self.unify(lhs, rhs, || TypeCheckError::TypeMismatchWithSource {
                    expected: lhs.clone(),
                    actual: rhs.clone(),
                    span: op.location.span,
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
        span: Span,
    ) -> bool {
        self.unify(lhs_type, rhs_type, || TypeCheckError::TypeMismatchWithSource {
            expected: lhs_type.clone(),
            actual: rhs_type.clone(),
            source: Source::Binary,
            span,
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
                Less | LessEqual | Greater | GreaterEqual => FieldComparison { span },
                And | Or | Xor | ShiftRight | ShiftLeft => FieldBitwiseOp { span },
                Modulo => FieldModulo { span },
                other => unreachable!("Operator {other:?} should be valid for Field"),
            });
        }

        use_impl
    }

    // Given a binary operator and another type. This method will produce the output type
    // and a boolean indicating whether to use the trait impl corresponding to the operator
    // or not. A value of false indicates the caller to use a primitive operation for this
    // operator, while a true value indicates a user-provided trait impl is required.
    pub(super) fn infix_operand_type_rules(
        &mut self,
        lhs_type: &Type,
        op: &HirBinaryOp,
        rhs_type: &Type,
        span: Span,
    ) -> Result<(Type, bool), TypeCheckError> {
        if op.kind.is_comparator() {
            return self.comparator_operand_type_rules(lhs_type, rhs_type, op, span);
        }

        use Type::*;
        match (lhs_type, rhs_type) {
            // An error type on either side will always return an error
            (Error, _) | (_, Error) => Ok((Error, false)),
            (Alias(alias, args), other) | (other, Alias(alias, args)) => {
                let alias = alias.borrow().get_type(args);
                self.infix_operand_type_rules(&alias, op, other, span)
            }

            // Matches on TypeVariable must be first so that we follow any type
            // bindings.
            (TypeVariable(int), other) | (other, TypeVariable(int)) => {
                if op.kind == BinaryOpKind::ShiftLeft || op.kind == BinaryOpKind::ShiftRight {
                    self.unify(
                        rhs_type,
                        &Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight),
                        || TypeCheckError::InvalidShiftSize { span },
                    );
                    let use_impl = if lhs_type.is_numeric_value() {
                        let integer_type = self.polymorphic_integer();
                        self.bind_type_variables_for_infix(lhs_type, op, &integer_type, span)
                    } else {
                        true
                    };
                    return Ok((lhs_type.clone(), use_impl));
                }
                if let TypeBinding::Bound(ref binding) = &*int.borrow() {
                    return self.infix_operand_type_rules(binding, op, other, span);
                }
                let use_impl = self.bind_type_variables_for_infix(lhs_type, op, rhs_type, span);
                Ok((other.clone(), use_impl))
            }
            (Integer(sign_x, bit_width_x), Integer(sign_y, bit_width_y)) => {
                if op.kind == BinaryOpKind::ShiftLeft || op.kind == BinaryOpKind::ShiftRight {
                    if *sign_y != Signedness::Unsigned || *bit_width_y != IntegerBitSize::Eight {
                        return Err(TypeCheckError::InvalidShiftSize { span });
                    }
                    return Ok((Integer(*sign_x, *bit_width_x), false));
                }
                if sign_x != sign_y {
                    return Err(TypeCheckError::IntegerSignedness {
                        sign_x: *sign_x,
                        sign_y: *sign_y,
                        span,
                    });
                }
                if bit_width_x != bit_width_y {
                    return Err(TypeCheckError::IntegerBitWidth {
                        bit_width_x: *bit_width_x,
                        bit_width_y: *bit_width_y,
                        span,
                    });
                }
                Ok((Integer(*sign_x, *bit_width_x), false))
            }
            // The result of two Fields is always a witness
            (FieldElement, FieldElement) => {
                if !op.kind.is_valid_for_field_type() {
                    if op.kind == BinaryOpKind::Modulo {
                        return Err(TypeCheckError::FieldModulo { span });
                    } else {
                        return Err(TypeCheckError::FieldBitwiseOp { span });
                    }
                }
                Ok((FieldElement, false))
            }

            (Bool, Bool) => Ok((Bool, false)),

            (lhs, rhs) => {
                if op.kind == BinaryOpKind::ShiftLeft || op.kind == BinaryOpKind::ShiftRight {
                    if rhs == &Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight) {
                        return Ok((lhs.clone(), true));
                    }
                    return Err(TypeCheckError::InvalidShiftSize { span });
                }
                self.unify(lhs, rhs, || TypeCheckError::TypeMismatchWithSource {
                    expected: lhs.clone(),
                    actual: rhs.clone(),
                    span: op.location.span,
                    source: Source::Binary,
                });
                Ok((lhs.clone(), true))
            }
        }
    }

    // Given a unary operator and a type, this method will produce the output type
    // and a boolean indicating whether to use the trait impl corresponding to the operator
    // or not. A value of false indicates the caller to use a primitive operation for this
    // operator, while a true value indicates a user-provided trait impl is required.
    pub(super) fn prefix_operand_type_rules(
        &mut self,
        op: &UnaryOp,
        rhs_type: &Type,
        span: Span,
    ) -> Result<(Type, bool), TypeCheckError> {
        use Type::*;

        match op {
            crate::ast::UnaryOp::Minus | crate::ast::UnaryOp::Not => {
                match rhs_type {
                    // An error type will always return an error
                    Error => Ok((Error, false)),
                    Alias(alias, args) => {
                        let alias = alias.borrow().get_type(args);
                        self.prefix_operand_type_rules(op, &alias, span)
                    }

                    // Matches on TypeVariable must be first so that we follow any type
                    // bindings.
                    TypeVariable(int) => {
                        if let TypeBinding::Bound(ref binding) = &*int.borrow() {
                            return self.prefix_operand_type_rules(op, binding, span);
                        }

                        // The `!` prefix operator is not valid for Field, so if this is a numeric
                        // type we constrain it to just (non-Field) integer types.
                        if matches!(op, crate::ast::UnaryOp::Not) && rhs_type.is_numeric_value() {
                            let integer_type = Type::polymorphic_integer(self.interner);
                            self.unify(rhs_type, &integer_type, || {
                                TypeCheckError::InvalidUnaryOp { kind: rhs_type.to_string(), span }
                            });
                        }

                        Ok((rhs_type.clone(), !rhs_type.is_numeric_value()))
                    }
                    Integer(sign_x, bit_width_x) => {
                        if *op == UnaryOp::Minus && *sign_x == Signedness::Unsigned {
                            return Err(TypeCheckError::InvalidUnaryOp {
                                kind: rhs_type.to_string(),
                                span,
                            });
                        }
                        Ok((Integer(*sign_x, *bit_width_x), false))
                    }
                    // The result of a Field is always a witness
                    FieldElement => {
                        if *op == UnaryOp::Not {
                            return Err(TypeCheckError::FieldNot { span });
                        }
                        Ok((FieldElement, false))
                    }

                    Bool => Ok((Bool, false)),

                    _ => Ok((rhs_type.clone(), true)),
                }
            }
            crate::ast::UnaryOp::MutableReference => {
                Ok((Type::MutableReference(Box::new(rhs_type.follow_bindings())), false))
            }
            crate::ast::UnaryOp::Dereference { implicitly_added: _ } => {
                let element_type = self.interner.next_type_variable();
                let expected = Type::MutableReference(Box::new(element_type.clone()));
                self.unify(rhs_type, &expected, || TypeCheckError::TypeMismatch {
                    expr_typ: rhs_type.to_string(),
                    expected_typ: expected.to_string(),
                    expr_span: span,
                });
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
        trait_method_id: TraitMethodId,
        object_type: &Type,
        span: Span,
    ) {
        let the_trait = self.interner.get_trait(trait_method_id.trait_id);

        let method = &the_trait.methods[trait_method_id.method_index];
        let (method_type, mut bindings) = method.typ.clone().instantiate(self.interner);

        match method_type {
            Type::Function(args, _, _, _) => {
                // We can cheat a bit and match against only the object type here since no operator
                // overload uses other generic parameters or return types aside from the object type.
                let expected_object_type = &args[0];
                self.unify(object_type, expected_object_type, || TypeCheckError::TypeMismatch {
                    expected_typ: expected_object_type.to_string(),
                    expr_typ: object_type.to_string(),
                    expr_span: span,
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
        span: Span,
    ) -> Type {
        let access_lhs = &mut access.lhs;

        let dereference_lhs = |this: &mut Self, lhs_type, element| {
            let old_lhs = *access_lhs;
            *access_lhs = this.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                operator: crate::ast::UnaryOp::Dereference { implicitly_added: true },
                rhs: old_lhs,
                trait_method_id: None,
            }));
            this.interner.push_expr_type(old_lhs, lhs_type);
            this.interner.push_expr_type(*access_lhs, element);

            let old_location = this.interner.id_location(old_lhs);
            this.interner.push_expr_location(*access_lhs, span, old_location.file);
        };

        // If this access is just a field offset, we want to avoid dereferencing
        let dereference_lhs = (!access.is_offset).then_some(dereference_lhs);

        match self.check_field_access(&lhs_type, &access.rhs.0.contents, span, dereference_lhs) {
            Some((element_type, index)) => {
                self.interner.set_field_index(expr_id, index);
                // We must update `access` in case we added any dereferences to it
                self.interner.replace_expr(&expr_id, HirExpression::MemberAccess(access));
                element_type
            }
            None => Type::Error,
        }
    }

    pub(crate) fn lookup_method(
        &mut self,
        object_type: &Type,
        method_name: &str,
        span: Span,
        has_self_arg: bool,
    ) -> Option<HirMethodReference> {
        match object_type.follow_bindings() {
            // TODO: We should allow method calls on `impl Trait`s eventually.
            //       For now it is fine since they are only allowed on return types.
            Type::TraitAsType(..) => {
                self.push_err(TypeCheckError::UnresolvedMethodCall {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    span,
                });
                None
            }
            Type::NamedGeneric(_, _) => {
                self.lookup_method_in_trait_constraints(object_type, method_name, span)
            }
            // Mutable references to another type should resolve to methods of their element type.
            // This may be a struct or a primitive type.
            Type::MutableReference(element) => {
                self.lookup_method(&element, method_name, span, has_self_arg)
            }

            // If we fail to resolve the object to a struct type, we have no way of type
            // checking its arguments as we can't even resolve the name of the function
            Type::Error => None,

            // The type variable must be unbound at this point since follow_bindings was called
            Type::TypeVariable(var) if var.kind() == Kind::Normal => {
                self.push_err(TypeCheckError::TypeAnnotationsNeededForMethodCall { span });
                None
            }

            other => {
                self.lookup_struct_or_primitive_method(&other, method_name, span, has_self_arg)
            }
        }
    }

    fn lookup_struct_or_primitive_method(
        &mut self,
        object_type: &Type,
        method_name: &str,
        span: Span,
        has_self_arg: bool,
    ) -> Option<HirMethodReference> {
        // First search in the type methods. If there is one, that's the one.
        if let Some(method_id) =
            self.interner.lookup_direct_method(object_type, method_name, has_self_arg)
        {
            return Some(HirMethodReference::FuncId(method_id));
        }

        // Next lookup all matching trait methods.
        let trait_methods =
            self.interner.lookup_trait_methods(object_type, method_name, has_self_arg);

        // If there's at least one matching trait method we need to see if only one is in scope.
        if !trait_methods.is_empty() {
            return self.return_trait_method_in_scope(&trait_methods, method_name, span);
        }

        // If we couldn't find any trait methods, search in
        // impls for all types `T`, e.g. `impl<T> Foo for T`
        let generic_methods =
            self.interner.lookup_generic_methods(object_type, method_name, has_self_arg);
        if !generic_methods.is_empty() {
            return self.return_trait_method_in_scope(&generic_methods, method_name, span);
        }

        if let Type::Struct(struct_type, _) = object_type {
            let has_field_with_function_type = struct_type
                .borrow()
                .get_fields_as_written()
                .into_iter()
                .any(|field| field.name.0.contents == method_name && field.typ.is_function());
            if has_field_with_function_type {
                self.push_err(TypeCheckError::CannotInvokeStructFieldFunctionType {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    span,
                });
            } else {
                self.push_err(TypeCheckError::UnresolvedMethodCall {
                    method_name: method_name.to_string(),
                    object_type: object_type.clone(),
                    span,
                });
            }
            None
        } else {
            // It could be that this type is a composite type that is bound to a trait,
            // for example `x: (T, U) ... where (T, U): SomeTrait`
            // (so this case is a generalization of the NamedGeneric case)
            self.lookup_method_in_trait_constraints(object_type, method_name, span)
        }
    }

    /// Given a list of functions and the trait they belong to, returns the one function
    /// that is in scope.
    fn return_trait_method_in_scope(
        &mut self,
        trait_methods: &[(FuncId, TraitId)],
        method_name: &str,
        span: Span,
    ) -> Option<HirMethodReference> {
        let module_id = self.module_id();
        let module_data = self.get_module(module_id);

        // Only keep unique trait IDs: multiple trait methods might come from the same trait
        // but implemented with different generics (like `Convert<Field>` and `Convert<i32>`).
        let traits: HashSet<TraitId> =
            trait_methods.iter().map(|(_, trait_id)| *trait_id).collect();

        let traits_in_scope: Vec<_> = traits
            .iter()
            .filter_map(|trait_id| {
                let trait_ = self.interner.get_trait(*trait_id);
                let trait_name = &trait_.name;
                let map = module_data.scope().types().get(trait_name)?;
                let imported_item = map.get(&None)?;
                if imported_item.0 == ModuleDefId::TraitId(*trait_id) {
                    Some((*trait_id, trait_name))
                } else {
                    None
                }
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

                self.push_err(PathResolutionError::TraitMethodNotInScope {
                    ident: Ident::new(method_name.into(), span),
                    trait_name,
                });

                return Some(self.trait_hir_method_reference(
                    trait_id,
                    trait_methods,
                    method_name,
                    span,
                ));
            } else {
                let traits = vecmap(traits, |trait_id| {
                    let trait_ = self.interner.get_trait(trait_id);
                    self.fully_qualified_trait_path(trait_)
                });
                self.push_err(PathResolutionError::UnresolvedWithPossibleTraitsToImport {
                    ident: Ident::new(method_name.into(), span),
                    traits,
                });
                return None;
            }
        }

        if traits_in_scope.len() > 1 {
            let traits = vecmap(traits, |trait_id| {
                let trait_ = self.interner.get_trait(trait_id);
                self.fully_qualified_trait_path(trait_)
            });
            self.push_err(PathResolutionError::MultipleTraitsInScope {
                ident: Ident::new(method_name.into(), span),
                traits,
            });
            return None;
        }

        let trait_id = traits_in_scope[0].0;
        Some(self.trait_hir_method_reference(trait_id, trait_methods, method_name, span))
    }

    fn trait_hir_method_reference(
        &self,
        trait_id: TraitId,
        trait_methods: &[(FuncId, TraitId)],
        method_name: &str,
        span: Span,
    ) -> HirMethodReference {
        // If we find a single trait impl method, return it so we don't have to later determine the impl
        if trait_methods.len() == 1 {
            let (func_id, _) = trait_methods[0];
            return HirMethodReference::FuncId(func_id);
        }

        // Return a TraitMethodId with unbound generics. These will later be bound by the type-checker.
        let trait_ = self.interner.get_trait(trait_id);
        let generics = trait_.as_constraint(span).trait_bound.trait_generics;
        let trait_method_id = trait_.find_method(method_name).unwrap();
        HirMethodReference::TraitMethodId(trait_method_id, generics, false)
    }

    fn lookup_method_in_trait_constraints(
        &mut self,
        object_type: &Type,
        method_name: &str,
        span: Span,
    ) -> Option<HirMethodReference> {
        let func_id = match self.current_item {
            Some(DependencyId::Function(id)) => id,
            _ => panic!("unexpected method outside a function: {method_name}"),
        };
        let func_meta = self.interner.function_meta(&func_id);

        // If inside a trait method, check if it's a method on `self`
        if let Some(trait_id) = func_meta.trait_id {
            if Some(object_type) == self.self_type.as_ref() {
                let the_trait = self.interner.get_trait(trait_id);
                let constraint = the_trait.as_constraint(the_trait.name.span());
                if let Some(HirMethodReference::TraitMethodId(method_id, generics, _)) = self
                    .lookup_method_in_trait(
                        the_trait,
                        method_name,
                        &constraint.trait_bound,
                        the_trait.id,
                    )
                {
                    // If it is, it's an assumed trait
                    return Some(HirMethodReference::TraitMethodId(method_id, generics, true));
                }
            }
        }

        for constraint in &func_meta.trait_constraints {
            if *object_type == constraint.typ {
                if let Some(the_trait) =
                    self.interner.try_get_trait(constraint.trait_bound.trait_id)
                {
                    if let Some(method) = self.lookup_method_in_trait(
                        the_trait,
                        method_name,
                        &constraint.trait_bound,
                        the_trait.id,
                    ) {
                        return Some(method);
                    }
                }
            }
        }

        self.push_err(TypeCheckError::UnresolvedMethodCall {
            method_name: method_name.to_string(),
            object_type: object_type.clone(),
            span,
        });

        None
    }

    fn lookup_method_in_trait(
        &self,
        the_trait: &Trait,
        method_name: &str,
        trait_bound: &ResolvedTraitBound,
        starting_trait_id: TraitId,
    ) -> Option<HirMethodReference> {
        if let Some(trait_method) = the_trait.find_method(method_name) {
            return Some(HirMethodReference::TraitMethodId(
                trait_method,
                trait_bound.trait_generics.clone(),
                false,
            ));
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
                if let Some(method) = self.lookup_method_in_trait(
                    the_trait,
                    method_name,
                    &parent_trait_bound,
                    starting_trait_id,
                ) {
                    return Some(method);
                }
            }
        }

        None
    }

    pub(super) fn type_check_call(
        &mut self,
        call: &HirCallExpression,
        func_type: Type,
        args: Vec<(Type, ExprId, Span)>,
        span: Span,
    ) -> Type {
        self.run_lint(|elaborator| {
            lints::deprecated_function(elaborator.interner, call.func).map(Into::into)
        });

        let is_current_func_constrained = self.in_constrained_function();

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
                    self.push_err(TypeCheckError::Unsafe { span });
                }
                UnsafeBlockStatus::InUnsafeBlockWithoutUnconstrainedCalls => {
                    self.unsafe_block_status = UnsafeBlockStatus::InUnsafeBlockWithConstrainedCalls;
                }
                UnsafeBlockStatus::InUnsafeBlockWithConstrainedCalls => (),
            }

            if let Some(called_func_id) = self.interner.lookup_function_from_expr(&call.func) {
                self.run_lint(|elaborator| {
                    lints::oracle_called_from_constrained_function(
                        elaborator.interner,
                        &called_func_id,
                        is_current_func_constrained,
                        span,
                    )
                    .map(Into::into)
                });
            }

            let errors = lints::unconstrained_function_args(&args);
            for error in errors {
                self.push_err(error);
            }
        }

        let return_type = self.bind_function_type(func_type, args, span);

        if crossing_runtime_boundary {
            self.run_lint(|_| {
                lints::unconstrained_function_return(&return_type, span).map(Into::into)
            });
        }

        return_type
    }

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

            if matches!(expected_object_type.follow_bindings(), Type::MutableReference(_)) {
                if !matches!(actual_type, Type::MutableReference(_)) {
                    if let Err(error) = verify_mutable_reference(self.interner, *object) {
                        self.push_err(TypeCheckError::ResolverError(error));
                    }

                    let new_type = Type::MutableReference(Box::new(actual_type));
                    *object_type = new_type.clone();

                    // First try to remove a dereference operator that may have been implicitly
                    // inserted by a field access expression `foo.bar` on a mutable reference `foo`.
                    let new_object = self.try_remove_implicit_dereference(*object);

                    // If that didn't work, then wrap the whole expression in an `&mut`
                    *object = new_object.unwrap_or_else(|| {
                        let location = self.interner.id_location(*object);

                        let new_object =
                            self.interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                                operator: UnaryOp::MutableReference,
                                rhs: *object,
                                trait_method_id: None,
                            }));
                        self.interner.push_expr_type(new_object, new_type);
                        self.interner.push_expr_location(new_object, location.span, location.file);
                        new_object
                    });
                }
            // Otherwise if the object type is a mutable reference and the method is not, insert as
            // many dereferences as needed.
            } else if matches!(actual_type, Type::MutableReference(_)) {
                let (new_object, new_type) = self.insert_auto_dereferences(*object, actual_type);
                *object_type = new_type;
                *object = new_object;
            }
        }
    }

    pub fn type_check_function_body(&mut self, body_type: Type, meta: &FuncMeta, body_id: ExprId) {
        let (expr_span, empty_function) = self.function_info(body_id);
        let declared_return_type = meta.return_type();

        let func_span = self.interner.expr_span(&body_id); // XXX: We could be more specific and return the span of the last stmt, however stmts do not have spans yet
        if let Type::TraitAsType(trait_id, _, generics) = declared_return_type {
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
                    span: func_span,
                    source: Source::Return(meta.return_type.clone(), expr_span),
                });
            }
        } else {
            self.unify_with_coercions(&body_type, declared_return_type, body_id, func_span, || {
                let mut error = TypeCheckError::TypeMismatchWithSource {
                    expected: declared_return_type.clone(),
                    actual: body_type.clone(),
                    span: func_span,
                    source: Source::Return(meta.return_type.clone(), expr_span),
                };

                if empty_function {
                    error = error.add_context(
                        "implicitly returns `()` as its body has no tail or `return` expression",
                    );
                }
                error
            });
        }
    }

    fn function_info(&self, function_body_id: ExprId) -> (noirc_errors::Span, bool) {
        let (expr_span, empty_function) =
            if let HirExpression::Block(block) = self.interner.expression(&function_body_id) {
                let last_stmt = block.statements().last();
                let mut span = self.interner.expr_span(&function_body_id);

                if let Some(last_stmt) = last_stmt {
                    if let HirStatement::Expression(expr) = self.interner.statement(last_stmt) {
                        span = self.interner.expr_span(&expr);
                    }
                }

                (span, last_stmt.is_none())
            } else {
                (self.interner.expr_span(&function_body_id), false)
            };
        (expr_span, empty_function)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn verify_trait_constraint(
        &mut self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
        associated_types: &[NamedType],
        function_ident_id: ExprId,
        select_impl: bool,
        span: Span,
    ) {
        match self.interner.lookup_trait_implementation(
            object_type,
            trait_id,
            trait_generics,
            associated_types,
        ) {
            Ok(impl_kind) => {
                if select_impl {
                    self.interner.select_impl_for_expression(function_ident_id, impl_kind);
                }
            }
            Err(error) => self.push_trait_constraint_error(object_type, error, span),
        }
    }

    fn push_trait_constraint_error(
        &mut self,
        object_type: &Type,
        error: ImplSearchErrorKind,
        span: Span,
    ) {
        match error {
            ImplSearchErrorKind::TypeAnnotationsNeededOnObjectType => {
                self.push_err(TypeCheckError::TypeAnnotationsNeededForMethodCall { span });
            }
            ImplSearchErrorKind::Nested(constraints) => {
                if let Some(error) = NoMatchingImplFoundError::new(self.interner, constraints, span)
                {
                    self.push_err(TypeCheckError::NoMatchingImplFound(error));
                }
            }
            ImplSearchErrorKind::MultipleMatching(candidates) => {
                let object_type = object_type.clone();
                self.push_err(TypeCheckError::MultipleMatchingImpls {
                    object_type,
                    span,
                    candidates,
                });
            }
        }
    }

    pub fn add_existing_generics(
        &mut self,
        unresolved_generics: &UnresolvedGenerics,
        generics: &Generics,
    ) {
        assert_eq!(unresolved_generics.len(), generics.len());

        for (unresolved_generic, generic) in unresolved_generics.iter().zip(generics) {
            self.add_existing_generic(unresolved_generic, unresolved_generic.span(), generic);
        }
    }

    pub fn add_existing_generic(
        &mut self,
        unresolved_generic: &UnresolvedGeneric,
        span: Span,
        resolved_generic: &ResolvedGeneric,
    ) {
        let name = &unresolved_generic.ident().0.contents;

        if let Some(generic) = self.find_generic(name) {
            self.push_err(ResolverError::DuplicateDefinition {
                name: name.clone(),
                first_span: generic.span,
                second_span: span,
            });
        } else {
            self.generics.push(resolved_generic.clone());
        }
    }

    /// Push a type variable into the current FunctionContext to be defaulted if needed
    /// at the end of the earlier of either the current function or the current comptime scope.
    fn push_type_variable(&mut self, typ: Type) {
        let context = self.function_context.last_mut();
        let context = context.expect("The function_context stack should always be non-empty");
        context.type_variables.push(typ);
    }

    /// Push a trait constraint into the current FunctionContext to be solved if needed
    /// at the end of the earlier of either the current function or the current comptime scope.
    pub fn push_trait_constraint(
        &mut self,
        constraint: TraitConstraint,
        expr_id: ExprId,
        select_impl: bool,
    ) {
        let context = self.function_context.last_mut();
        let context = context.expect("The function_context stack should always be non-empty");
        context.trait_constraints.push((constraint, expr_id, select_impl));
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
        let mut bindings = TypeBindings::new();
        self.bind_generics_from_trait_bound(trait_bound, &mut bindings);
        ResolvedTraitBound {
            trait_generics: parent_trait_bound.trait_generics.map(|typ| typ.substitute(&bindings)),
            ..*parent_trait_bound
        }
    }

    pub(crate) fn fully_qualified_trait_path(&self, trait_: &Trait) -> String {
        fully_qualified_module_path(self.def_maps, self.crate_graph, &trait_.crate_id, trait_.id.0)
    }
}

pub(crate) fn bind_ordered_generics(
    params: &[ResolvedGeneric],
    args: &[Type],
    bindings: &mut TypeBindings,
) {
    assert_eq!(params.len(), args.len());

    for (param, arg) in params.iter().zip(args) {
        bind_generic(param, arg, bindings);
    }
}

pub(crate) fn bind_named_generics(
    mut params: Vec<ResolvedGeneric>,
    args: &[NamedType],
    bindings: &mut TypeBindings,
) {
    assert!(args.len() <= params.len());

    for arg in args {
        let i = params
            .iter()
            .position(|typ| *typ.name == arg.name.0.contents)
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

/// Gives an error if a user tries to create a mutable reference
/// to an immutable variable.
fn verify_mutable_reference(interner: &NodeInterner, rhs: ExprId) -> Result<(), ResolverError> {
    match interner.expression(&rhs) {
        HirExpression::MemberAccess(member_access) => {
            verify_mutable_reference(interner, member_access.lhs)
        }
        HirExpression::Index(_) => {
            let span = interner.expr_span(&rhs);
            Err(ResolverError::MutableReferenceToArrayElement { span })
        }
        HirExpression::Ident(ident, _) => {
            if let Some(definition) = interner.try_definition(ident.id) {
                if !definition.mutable {
                    return Err(ResolverError::MutableReferenceToImmutableVariable {
                        span: interner.expr_span(&rhs),
                        variable: definition.name.clone(),
                    });
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
