use super::Elaborator;
use crate::elaborator::types::TraitPathResolutionMethod;
use crate::elaborator::TypedPath;
use crate::hir_def::expr::{ HirIdent, HirExpression, TraitItem, ImplKind, HirMethodReference };
use crate::{Type, TypeBindings};
use crate::ast::{ Expression, ExpressionKind, Path, TypePath, UnresolvedTypeExpression };
use crate::node_interner::{ DefinitionInfo, DefinitionKind, ExprId };
use noirc_errors::Location;
use crate::elaborator::{ PathResolutionItem, Ident, types::SELF_TYPE_NAME, GenericTypeArgs };
use iter_extended::vecmap;
use crate::TypeAlias;

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
            // A type alias to a numeric generics is considered like a variable
            // but it is not a real variable so it does not resolve to a valid Identifier
            // In order to handle this, we retrieve the numeric generics expression that the type aliases to
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

        // Resolve any generics if we the variable we have resolved is a function
        // and if the turbofish operator was used.
        let generics = if let Some(DefinitionKind::Function(func_id)) = &definition_kind {
            self.resolve_function_turbofish_generics(func_id, resolved_turbofish, location)
        } else {
            None
        };

        if let Some(DefinitionKind::Function(func_id)) = &definition_kind {
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
        }

        let id = self.interner.push_expr(HirExpression::Ident(expr.clone(), generics.clone()));

        self.interner.push_expr_location(id, location);
        // TODO: set this to `true`. See https://github.com/noir-lang/noir/issues/8687
        let push_required_type_variables = self.current_trait.is_none();
        let typ = self.type_check_variable_with_bindings(
            expr,
            id,
            generics,
            bindings,
            push_required_type_variables,
        );
        self.interner.push_expr_type(id, typ.clone());

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
    /// In the first case we elaborate this as if it were a TypePath
    /// (for example, if `Self` is `u32` then we consider this the same as `u32::method_name`).
    /// A regular path lookup won't work here for the same reason `TypePath` exists.
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
            let id = self.interner.push_expr(hir_expr);
            self.interner.push_expr_location(id, location);
            self.interner.push_expr_type(id, numeric_type.clone());
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

    fn resolve_variable(&mut self, path: TypedPath) -> (HirIdent, Option<PathResolutionItem>) {
        if let Some(trait_path_resolution) = self.resolve_trait_generic_path(&path) {
            for error in trait_path_resolution.errors {
                self.push_err(error);
            }

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
                let typ = self.instantiate_primitive_type_with_turbofish(primitive_type, turbofish);
                let generics = match typ {
                    Type::String(length) => {
                        vec![*length]
                    }
                    Type::FmtString(length, element) => {
                        vec![*length, *element]
                    }
                    _ => Vec::new(),
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

    pub(super) fn elaborate_type_path(&mut self, path: TypePath) -> (ExprId, Type) {
        let typ_location = path.typ.location;
        let turbofish = path.turbofish;
        let wildcard_allowed = true;
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
            HirMethodReference::TraitItemId(definition, trait_id, generics, _) => {
                let mut constraint =
                    self.interner.get_trait(trait_id).as_constraint(ident_location);
                constraint.trait_bound.trait_generics = generics;
                ImplKind::TraitItem(TraitItem { definition, constraint, assumed: false })
            }
        };

        let ident = HirIdent { location: ident_location, id, impl_kind };
        let id = self.interner.push_expr(HirExpression::Ident(ident.clone(), generics.clone()));
        self.interner.push_expr_location(id, ident_location);

        let typ = self.type_check_variable(ident, id, generics);
        self.interner.push_expr_type(id, typ.clone());

        (id, typ)
    }
}

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
