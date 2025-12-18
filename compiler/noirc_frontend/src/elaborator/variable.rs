//! Everything to do with elaboration of variables.
//! Notably, variables may require trait constraints to be solved later on.

use super::Elaborator;
use crate::TypeAlias;
use crate::ast::{
    Expression, ExpressionKind, GenericTypeArgs, Ident, Path, TypePath, UnresolvedTypeExpression,
};
use crate::elaborator::TypedPath;
use crate::elaborator::function_context::BindableTypeVariableKind;
use crate::elaborator::path_resolution::PathResolutionItem;
use crate::elaborator::types::{SELF_TYPE_NAME, TraitPathResolutionMethod, WildcardAllowed};
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::hir::type_check::TypeCheckError;
use crate::hir_def::expr::{
    HirExpression, HirIdent, HirMethodReference, HirTraitMethodReference, ImplKind, TraitItem,
};
use crate::node_interner::pusher::{HasLocation, PushedExpr};
use crate::node_interner::{DefinitionId, DefinitionInfo, DefinitionKind, ExprId, TraitImplKind};
use crate::{Kind, Type, TypeBindings};
use iter_extended::vecmap;
use noirc_errors::Location;

impl Elaborator<'_> {
    pub(super) fn elaborate_variable(&mut self, variable: Path) -> (ExprId, Type) {
        let variable = self.validate_path(variable);
        if let Some((expr_id, typ)) =
            self.elaborate_variable_as_self_method_or_associated_constant(&variable)
        {
            return (expr_id, typ);
        }

        let resolved_turbofish = variable.segments.last().unwrap().generics.clone();

        let location = variable.location;
        let (expr, item) = self.resolve_variable(variable);
        let definition_id = expr.id;

        if let Some(PathResolutionItem::TypeAlias(alias)) = item {
            // A type alias to a numeric generics is considered like a variable,
            // but it is not a real variable so it does not resolve to a valid Identifier.
            // In order to handle this, we retrieve the numeric generics expression that the type aliases to.
            let type_alias = self.interner.get_type_alias(alias);
            if let Some(expr) = &type_alias.borrow().numeric_expr {
                let expr = UnresolvedTypeExpression::to_expression_kind(expr);
                let expr = Expression::new(expr, type_alias.borrow().location);
                return self.elaborate_expression(expr);
            }
        }

        let (type_generics, self_generic) = if let Some(item) = item {
            self.resolve_item_turbofish_and_self_type(item)
        } else {
            (Vec::new(), None)
        };

        let definition = self.interner.try_definition(definition_id);
        let is_comptime_local = !self.in_comptime_context()
            && definition.is_some_and(DefinitionInfo::is_comptime_local);
        let definition_kind = definition.as_ref().map(|definition| definition.kind.clone());

        let mut bindings = TypeBindings::default();
        let generics = if let Some(DefinitionKind::Function(func_id)) = &definition_kind {
            // If there's a self type, bind it to the self type generic
            if let Some(self_generic) = self_generic {
                let func_generics = &self.interner.function_meta(func_id).all_generics;
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
                // `all_generics` will always have the enclosing type generics first, so we need to bind those
                let func_generics = &self.interner.function_meta(func_id).all_generics;
                for (type_generic, func_generic) in type_generics.into_iter().zip(func_generics) {
                    let type_var = &func_generic.type_var;
                    bindings
                        .insert(type_var.id(), (type_var.clone(), type_var.kind(), type_generic));
                }
            }

            // Resolve any generics if the variable we have resolved is a function
            // and if the turbofish operator was used.
            self.resolve_function_turbofish_generics(func_id, resolved_turbofish, location)
        } else {
            None
        };

        let id = self.intern_expr(HirExpression::Ident(expr.clone(), generics.clone()), location);

        // TODO: set this to `true`. See https://github.com/noir-lang/noir/issues/8687
        let push_required_type_variables = self.current_trait.is_none();
        let typ = self.type_check_variable_with_bindings(
            expr,
            &id,
            generics,
            bindings,
            push_required_type_variables,
        );
        let id = self.intern_expr_type(id, typ.clone());

        // If this variable it a comptime local variable, use its current value as the final expression
        if is_comptime_local {
            let mut interpreter = self.setup_interpreter();
            let value = interpreter.evaluate(id);
            // If the value is an error it means the variable already had an error, so don't report it here again
            // (the error will make no sense, it will say that a non-comptime variable was referenced at runtime
            // but that's not true)
            if value.is_ok() {
                let (id, typ) = self.inline_comptime_value(value, location);
                self.debug_comptime(location, |interner| id.to_display_ast(interner).kind);
                (id, typ)
            } else {
                (id, typ)
            }
        } else {
            (id, typ)
        }
    }

    /// Checks whether `variable` is `Self::method_name` or `Self::AssociatedConstant` when we are inside a trait impl and `Self`
    /// resolves to a primitive type.
    ///
    /// In the first case we elaborate this as if it were a [TypePath]
    /// (for example, if `Self` is `u32` then we consider this the same as `u32::method_name`).
    /// A regular path lookup won't work here for the same reason [TypePath] exists.
    ///
    /// In the second case we solve the associated constant by looking up its value, later
    /// turning it into a literal.
    fn elaborate_variable_as_self_method_or_associated_constant(
        &mut self,
        variable: &TypedPath,
    ) -> Option<(ExprId, Type)> {
        if !(variable.segments.len() == 2 && variable.segments[0].ident.is_self_type_name()) {
            return None;
        }

        let location = variable.location;
        let name = variable.segments[1].ident.as_str();
        let self_type = self.self_type.as_ref()?;
        let trait_impl_id = &self.current_trait_impl?;

        // Check the `Self::AssociatedConstant` case when inside a trait impl
        if let Some((definition_id, numeric_type)) =
            self.interner.get_trait_impl_associated_constant(*trait_impl_id, name).cloned()
        {
            let hir_ident = HirIdent::non_trait_method(definition_id, location);
            let hir_expr = HirExpression::Ident(hir_ident, None);
            let id = self.interner.push_expr_full(hir_expr, location, numeric_type.clone());
            return Some((id, numeric_type));
        }

        // Check the `Self::method_name` case when `Self` is a primitive type
        if matches!(self.self_type, Some(Type::DataType(..))) {
            return None;
        }

        let ident = variable.segments[1].ident.clone();
        let typ_location = variable.segments[0].location;
        Some(self.elaborate_type_path_impl(self_type.clone(), ident, None, typ_location))
    }

    /// Resolve a [TypedPath] to a [HirIdent] of either some trait method, or a local or global variable.
    fn resolve_variable(&mut self, path: TypedPath) -> (HirIdent, Option<PathResolutionItem>) {
        if let Some(trait_path_resolution) = self.resolve_trait_generic_path(&path) {
            self.push_errors(trait_path_resolution.errors);

            return match trait_path_resolution.method {
                TraitPathResolutionMethod::NotATraitMethod(func_id) => (
                    HirIdent {
                        location: path.location,
                        id: self.interner.function_definition_id(func_id),
                        impl_kind: ImplKind::NotATraitMethod,
                    },
                    trait_path_resolution.item,
                ),

                TraitPathResolutionMethod::TraitItem(item) => (
                    HirIdent {
                        location: path.location,
                        id: item.definition,
                        impl_kind: ImplKind::TraitItem(item),
                    },
                    trait_path_resolution.item,
                ),

                TraitPathResolutionMethod::MultipleTraitsInScope => {
                    // An error has already been pushed, return a dummy identifier
                    let hir_ident = HirIdent {
                        location: path.location,
                        id: DefinitionId::dummy_id(),
                        impl_kind: ImplKind::NotATraitMethod,
                    };
                    (hir_ident, trait_path_resolution.item)
                }
            };
        }

        // If the Path is being used as an Expression, then it is referring to a global from a separate module
        // Otherwise, then it is referring to an Identifier
        // This lookup allows support of such statements: let x = foo::bar::SOME_GLOBAL + 10;
        // If the expression is a singular indent, we search the resolver's current scope as normal.
        let location = path.location;
        let ((hir_ident, var_scope_index), item) = self.get_ident_from_path(path);

        self.handle_hir_ident(&hir_ident, var_scope_index, location);

        (hir_ident, item)
    }

    /// Solve any generics that are part of the path before the function, for example:
    ///
    /// ```noir
    /// foo::Bar::<i32>::baz
    /// ```
    /// Solve `<i32>` above
    fn resolve_item_turbofish_and_self_type(
        &mut self,
        item: PathResolutionItem,
    ) -> (Vec<Type>, Option<Type>) {
        match item {
            PathResolutionItem::Method(struct_id, Some(generics), _func_id) => {
                let generics = self.resolve_struct_id_turbofish_generics(struct_id, Some(generics));
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
                let generics =
                    self.resolve_type_alias_id_turbofish_generics(type_alias_id, generics);

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
                );
                (generics, None)
            }
            PathResolutionItem::TypeTraitFunction(self_type, _trait_id, _func_id) => {
                (Vec::new(), Some(self_type))
            }
            PathResolutionItem::PrimitiveFunction(primitive_type, turbofish, _func_id) => {
                let (typ, has_generics) =
                    self.instantiate_primitive_type_with_turbofish(primitive_type, turbofish);
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
            | PathResolutionItem::ModuleFunction(..) => (Vec::new(), None),
        }
    }

    /// Elaborates a type path used in an expression, e.g. `Type::method::<Args>`
    pub(super) fn elaborate_type_path(&mut self, path: TypePath) -> (ExprId, Type) {
        let typ_location = path.typ.location;
        let turbofish = path.turbofish;
        let wildcard_allowed = WildcardAllowed::Yes;
        let typ = self.use_type(path.typ, wildcard_allowed);
        self.elaborate_type_path_impl(typ, path.item, turbofish, typ_location)
    }

    fn elaborate_type_path_impl(
        &mut self,
        typ: Type,
        ident: Ident,
        turbofish: Option<GenericTypeArgs>,
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

        let func_id = method
            .func_id(self.interner)
            .expect("Expected trait function to be a DefinitionKind::Function");

        let generics =
            turbofish.map(|turbofish| self.use_type_args(turbofish, func_id, ident_location).0);

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

        let typ = self.type_check_variable(ident, &id, generics);
        let id = self.intern_expr_type(id, typ.clone());

        (id, typ)
    }

    /// Given an [HirIdent], look up its definition, and:
    /// * mark it as referenced at the ident [Location] (LSP mode only)
    /// * mark the item currently being elaborated as a dependency of it
    /// * elaborate a global definition, if needed
    /// * add local identifiers to lambda captures
    pub(crate) fn handle_hir_ident(
        &mut self,
        hir_ident: &HirIdent,
        var_scope_index: usize,
        location: Location,
    ) {
        if hir_ident.id == DefinitionId::dummy_id() {
            return;
        }

        match self.interner.definition(hir_ident.id).kind {
            DefinitionKind::Function(func_id) => {
                if let Some(current_item) = self.current_item {
                    self.interner.add_function_dependency(current_item, func_id);
                }

                self.interner.add_function_reference(func_id, hir_ident.location);
            }
            DefinitionKind::Global(global_id) => {
                self.elaborate_global_if_unresolved(&global_id);
                if let Some(current_item) = self.current_item {
                    self.interner.add_global_dependency(current_item, global_id);
                }

                self.interner.add_global_reference(global_id, hir_ident.location);
            }
            DefinitionKind::NumericGeneric(_, ref numeric_typ) => {
                // Initialize numeric generics to a polymorphic integer type in case
                // they're used in expressions. We must do this here since type_check_variable
                // does not check definition kinds and otherwise expects parameters to
                // already be typed.
                if self.interner.definition_type(hir_ident.id) == Type::Error {
                    let type_var_kind = Kind::Numeric(numeric_typ.clone());
                    let typ = self.type_variable_with_kind(type_var_kind);
                    self.interner.push_definition_type(hir_ident.id, typ);
                }
            }
            DefinitionKind::Local(_) => {
                // only local variables can be captured by closures.
                self.resolve_local_variable(hir_ident.clone(), var_scope_index);

                self.interner.add_local_reference(hir_ident.id, location);
            }
            DefinitionKind::AssociatedConstant(..) => {
                // Nothing to do here
            }
        }
    }

    /// Starting with empty bindings, perform the type checking of an interned expression
    /// and a corresponding identifier, returning the instantiated [Type].
    pub(crate) fn type_check_variable(
        &mut self,
        ident: HirIdent,
        expr_id: &PushedExpr<HasLocation>,
        generics: Option<Vec<Type>>,
    ) -> Type {
        let bindings = TypeBindings::default();
        // TODO: set this to `true`. See https://github.com/noir-lang/noir/issues/8687
        let push_required_type_variables = self.current_trait.is_none();
        self.type_check_variable_with_bindings(
            ident,
            expr_id,
            generics,
            bindings,
            push_required_type_variables,
        )
    }

    /// Perform the type checking of an interned expression and a corresponding identifier,
    /// returning the instantiated [Type].
    ///
    /// If `push_required_type_variables`, the bindings are added to the function context,
    /// to be checked before it's finished.
    pub(crate) fn type_check_variable_with_bindings(
        &mut self,
        ident: HirIdent,
        expr_id: &PushedExpr<HasLocation>,
        generics: Option<Vec<Type>>,
        mut bindings: TypeBindings,
        push_required_type_variables: bool,
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

        // An identifiers type may be forall-quantified in the case of generic functions.
        // E.g. `fn foo<T>(t: T, field: Field) -> T` has type `forall T. fn(T, Field) -> T`.
        // We must instantiate identifiers at every call site to replace this T with a new type
        // variable to handle generic functions.
        let t = self.interner.id_type_substitute_trait_as_type(ident.id);

        let definition = self.interner.try_definition(ident.id);
        let function_generic_count = definition.map_or(0, |definition| match &definition.kind {
            DefinitionKind::Function(function) => {
                self.interner.function_modifiers(function).generic_count
            }
            _ => 0,
        });

        let location = self.interner.expr_location(expr_id);

        // This instantiates a trait's generics as well which need to be set
        // when the constraint below is later solved for when the function is
        // finished. How to link the two?
        let (typ, bindings) =
            self.instantiate(t, bindings, generics, function_generic_count, location);

        if let ImplKind::TraitItem(mut method) = ident.impl_kind {
            method.constraint.apply_bindings(&bindings);
            if method.assumed {
                let trait_generics = method.constraint.trait_bound.trait_generics.clone();
                let object_type = method.constraint.typ;
                let trait_impl = TraitImplKind::Assumed { object_type, trait_generics };
                self.interner.select_impl_for_expression(**expr_id, trait_impl);
            } else {
                self.push_trait_constraint(
                    method.constraint,
                    **expr_id,
                    true, // this constraint should lead to choosing a trait impl method
                );
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
        if let Some(definition) = self.interner.try_definition(ident.id) {
            if let DefinitionKind::Function(function) = definition.kind {
                let function = self.interner.function_meta(&function);
                for mut constraint in function.all_trait_constraints().cloned().collect::<Vec<_>>()
                {
                    constraint.apply_bindings(&bindings);

                    self.push_trait_constraint(
                        constraint, **expr_id,
                        false, // This constraint shouldn't lead to choosing a trait impl method
                    );
                }
            }
        }

        if push_required_type_variables {
            for (type_variable, _kind, typ) in bindings.values() {
                self.push_required_type_variable(
                    type_variable.id(),
                    typ.clone(),
                    BindableTypeVariableKind::Ident(ident.id),
                    ident.location,
                );
            }
        }

        self.interner.store_instantiation_bindings(**expr_id, bindings);
        typ
    }

    /// Instantiate a [Type] with the given [TypeBindings], returning the bindings potentially
    /// extended from any turbofish generics.
    ///
    /// If there are turbofish generics and their number matches the expectations of the function,
    /// those are used as well, otherwise they are ignored and an error is pushed.
    fn instantiate(
        &mut self,
        typ: Type,
        bindings: TypeBindings,
        turbofish_generics: Option<Vec<Type>>,
        function_generic_count: usize,
        location: Location,
    ) -> (Type, TypeBindings) {
        match turbofish_generics {
            Some(turbofish_generics) => {
                if turbofish_generics.len() != function_generic_count {
                    let type_check_err = TypeCheckError::IncorrectTurbofishGenericCount {
                        expected_count: function_generic_count,
                        actual_count: turbofish_generics.len(),
                        location,
                    };
                    self.push_err(CompilationError::TypeError(type_check_err));
                    typ.instantiate_with_bindings(bindings, self.interner)
                } else {
                    // Fetch the count of any implicit generics on the function, such as
                    // for a method within a generic impl.
                    let implicit_generic_count = match &typ {
                        Type::Forall(generics, _) => generics.len() - function_generic_count,
                        _ => 0,
                    };
                    typ.instantiate_with_bindings_and_turbofish(
                        bindings,
                        turbofish_generics,
                        self.interner,
                        implicit_generic_count,
                    )
                }
            }
            None => typ.instantiate_with_bindings(bindings, self.interner),
        }
    }
}

/// Bind the generics of the [Type] aliased by the [TypeAlias] to a vector of generic arguments,
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
        _ => panic!("Expected type alias to point to struct or alias"),
    }
}
