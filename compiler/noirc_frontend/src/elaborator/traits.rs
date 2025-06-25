use std::{collections::BTreeMap, rc::Rc};

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    NamedGeneric, ResolvedGeneric, Type, TypeBindings,
    ast::{
        BlockExpression, FunctionDefinition, FunctionKind, FunctionReturnType, Ident,
        ItemVisibility, NoirFunction, TraitBound, TraitItem, UnresolvedGeneric, UnresolvedGenerics,
        UnresolvedTraitConstraint, UnresolvedType,
    },
    elaborator::FuncMetaDefinition,
    hir::{def_collector::dc_crate::UnresolvedTrait, type_check::TypeCheckError},
    hir_def::{
        function::Parameters,
        traits::{ResolvedTraitBound, TraitFunction},
    },
    node_interner::{DependencyId, FuncId, NodeInterner, ReferenceId, TraitId},
};

use super::Elaborator;

impl Elaborator<'_> {
    pub fn collect_traits(&mut self, traits: &mut BTreeMap<TraitId, UnresolvedTrait>) {
        for (trait_id, unresolved_trait) in traits {
            self.local_module = unresolved_trait.module_id.local_id;

            self.recover_generics(|this| {
                this.current_trait = Some(*trait_id);

                let the_trait = this.interner.get_trait(*trait_id);
                let self_typevar = the_trait.self_type_typevar.clone();
                let self_type = Type::TypeVariable(self_typevar.clone());
                this.self_type = Some(self_type.clone());

                let resolved_generics = this.interner.get_trait(*trait_id).generics.clone();
                this.add_existing_generics(
                    &unresolved_trait.trait_def.generics,
                    &resolved_generics,
                );

                let new_generics =
                    this.desugar_trait_constraints(&mut unresolved_trait.trait_def.where_clause);
                let new_generics = vecmap(new_generics, |(generic, _bounds)| {
                    // TODO: use `_bounds` variable above
                    // See https://github.com/noir-lang/noir/issues/8601
                    generic
                });
                this.generics.extend(new_generics);

                let where_clause =
                    this.resolve_trait_constraints(&unresolved_trait.trait_def.where_clause);
                this.remove_trait_constraints_from_scope(where_clause.iter());

                let mut associated_type_bounds = rustc_hash::FxHashMap::default();
                for item in &unresolved_trait.trait_def.items {
                    if let TraitItem::Type { name, bounds } = &item.item {
                        let resolved_bounds = this.resolve_trait_bounds(bounds);
                        associated_type_bounds.insert(name.to_string(), resolved_bounds);
                    }
                }

                // Each associated type in this trait is also an implicit generic
                for associated_type in &this.interner.get_trait(*trait_id).associated_types {
                    this.generics.push(associated_type.clone());
                }

                let resolved_trait_bounds =
                    this.resolve_trait_bounds(&unresolved_trait.trait_def.bounds);
                for bound in &resolved_trait_bounds {
                    this.interner
                        .add_trait_dependency(DependencyId::Trait(bound.trait_id), *trait_id);
                }

                this.interner.update_trait(*trait_id, |trait_def| {
                    trait_def.set_trait_bounds(resolved_trait_bounds);
                    trait_def.set_where_clause(where_clause);
                    trait_def.set_visibility(unresolved_trait.trait_def.visibility);
                    trait_def.set_associated_type_bounds(associated_type_bounds);
                    trait_def.set_all_generics(this.generics.clone());
                });
            });
        }

        self.self_type = None;
        self.current_trait = None;
    }

    pub fn collect_trait_methods(&mut self, traits: &mut BTreeMap<TraitId, UnresolvedTrait>) {
        for (trait_id, unresolved_trait) in traits {
            self.local_module = unresolved_trait.module_id.local_id;

            self.recover_generics(|this| {
                this.current_trait = Some(*trait_id);

                let the_trait = this.interner.get_trait(*trait_id);
                let self_typevar = the_trait.self_type_typevar.clone();
                let self_type = Type::TypeVariable(self_typevar.clone());
                this.self_type = Some(self_type.clone());

                this.generics = the_trait.all_generics.clone();

                let methods = this.resolve_trait_methods(*trait_id, unresolved_trait);

                this.interner.update_trait(*trait_id, |trait_def| {
                    trait_def.set_methods(methods);
                });
            });

            // This check needs to be after the trait's methods are set since
            // the interner may set `interner.ordering_type` based on the result type
            // of the Cmp trait, if this is it.
            if self.crate_id.is_stdlib() {
                self.interner.try_add_infix_operator_trait(*trait_id);
                self.interner.try_add_prefix_operator_trait(*trait_id);
            }
        }

        self.current_trait = None;
    }

    fn resolve_trait_bounds(&mut self, bounds: &[TraitBound]) -> Vec<ResolvedTraitBound> {
        bounds.iter().filter_map(|bound| self.resolve_trait_bound(bound)).collect()
    }

    fn resolve_trait_methods(
        &mut self,
        trait_id: TraitId,
        unresolved_trait: &UnresolvedTrait,
    ) -> Vec<TraitFunction> {
        self.local_module = unresolved_trait.module_id.local_id;

        let mut functions = vec![];

        for item in &unresolved_trait.trait_def.items {
            if let TraitItem::Function {
                name,
                generics,
                parameters,
                return_type,
                where_clause,
                body,
                is_unconstrained,
                visibility: _,
                is_comptime: _,
            } = &item.item
            {
                self.recover_generics(|this| {
                    let the_trait = this.interner.get_trait(trait_id);
                    let self_typevar = the_trait.self_type_typevar.clone();
                    let name_location = the_trait.name.location();

                    this.add_existing_generic(
                        &UnresolvedGeneric::Variable(Ident::from("Self"), Vec::new()),
                        name_location,
                        &ResolvedGeneric {
                            name: Rc::new("Self".to_owned()),
                            type_var: self_typevar,
                            location: name_location,
                        },
                    );

                    let func_id = unresolved_trait.method_ids[name.as_str()];
                    let mut where_clause = where_clause.to_vec();

                    // Attach any trait constraints on the trait to the function,
                    where_clause.extend(unresolved_trait.trait_def.where_clause.clone());

                    this.resolve_trait_function(
                        trait_id,
                        name,
                        *is_unconstrained,
                        generics,
                        parameters,
                        return_type,
                        where_clause,
                        body,
                        unresolved_trait.trait_def.visibility,
                        func_id,
                    );

                    if !item.doc_comments.is_empty() {
                        let id = ReferenceId::Function(func_id);
                        this.interner.set_doc_comments(id, item.doc_comments.clone());
                    }

                    let func_meta = this.interner.function_meta(&func_id);

                    let arguments = vecmap(&func_meta.parameters.0, |(_, typ, _)| typ.clone());
                    let return_type = func_meta.return_type().clone();

                    let generics =
                        vecmap(&this.generics.clone(), |generic| generic.type_var.clone());

                    let default_impl_list: Vec<_> = unresolved_trait
                        .fns_with_default_impl
                        .functions
                        .iter()
                        .filter(|(_, _, q)| q.name() == name.as_str())
                        .collect();

                    let default_impl = if default_impl_list.len() == 1 {
                        Some(Box::new(default_impl_list[0].2.clone()))
                    } else {
                        None
                    };

                    let no_environment = Box::new(Type::Unit);
                    let function_type = Type::Function(
                        arguments,
                        Box::new(return_type),
                        no_environment,
                        *is_unconstrained,
                    );

                    functions.push(TraitFunction {
                        name: name.clone(),
                        typ: Type::Forall(generics, Box::new(function_type)),
                        location: Location::new(name.span(), unresolved_trait.file_id),
                        default_impl,
                        default_impl_module_id: unresolved_trait.module_id,
                        trait_constraints: func_meta.trait_constraints.clone(),
                        direct_generics: func_meta.direct_generics.clone(),
                    });
                });
            }
        }
        functions
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resolve_trait_function(
        &mut self,
        trait_id: TraitId,
        name: &Ident,
        is_unconstrained: bool,
        generics: &UnresolvedGenerics,
        parameters: &[(Ident, UnresolvedType)],
        return_type: &FunctionReturnType,
        where_clause: Vec<UnresolvedTraitConstraint>,
        body: &Option<BlockExpression>,
        trait_visibility: ItemVisibility,
        func_id: FuncId,
    ) {
        let old_generic_count = self.generics.len();

        self.scopes.start_function();

        let has_body = body.is_some();

        let body = match body {
            Some(body) => body.clone(),
            None => BlockExpression { statements: Vec::new() },
        };
        let kind =
            if has_body { FunctionKind::Normal } else { FunctionKind::TraitFunctionWithoutBody };
        let mut def = FunctionDefinition::normal(
            name,
            is_unconstrained,
            generics,
            parameters,
            body,
            where_clause,
            return_type,
        );

        // Trait functions always have the same visibility as the trait they are in
        def.visibility = trait_visibility;

        let mut function = NoirFunction { kind, def };
        self.define_function_meta(
            &mut function,
            FuncMetaDefinition {
                func_id,
                trait_id: Some(trait_id),
                extra_trait_constraints: &[],
                path_resolution_module: None,
            },
        );

        // Here we elaborate functions without a body, mainly to check the arguments and return types.
        // Later on we'll elaborate functions with a body by fully type-checking them.
        if !has_body {
            self.elaborate_function(func_id);
        }

        let _ = self.scopes.end_function();
        // Don't check the scope tree for unused variables, they can't be used in a declaration anyway.
        self.generics.truncate(old_generic_count);
    }
}

/// Checks that the type of a function in a trait impl matches the type
/// of the corresponding function declaration in the trait itself.
///
/// To do this, given a trait such as:
/// `trait Foo<A> { fn foo<B>(...); }`
///
/// And an impl such as:
/// `impl<C> Foo<D> for Bar<E> { fn foo<F>(...); } `
///
/// We have to substitute:
/// - `Self` for `Bar<E>`
/// - `A` for `D`
/// - `B` for `F`
///
/// Before we can type check. Finally, we must also check that the unification
/// result does not introduce any new bindings. This can happen if the impl
/// function's type is more general than that of the trait function. E.g.
/// `fn baz<A, B>(a: A, b: B)` when the impl required `fn baz<A>(a: A, b: A)`.
///
/// This does not type check the body of the impl function.
pub(crate) fn check_trait_impl_method_matches_declaration(
    interner: &mut NodeInterner,
    function: FuncId,
    noir_function: &NoirFunction,
) -> Vec<TypeCheckError> {
    let meta = interner.function_meta(&function);
    let method_name = interner.function_name(&function);
    let mut errors = Vec::new();

    let definition_type = meta.typ.as_monotype();

    let impl_id =
        meta.trait_impl.expect("Trait impl function should have a corresponding trait impl");

    // If the trait implementation is not defined in the interner then there was a previous
    // error in resolving the trait path and there is likely no trait for this impl.
    let Some(impl_) = interner.try_get_trait_implementation(impl_id) else {
        return errors;
    };

    let impl_ = impl_.borrow();
    let trait_info = interner.get_trait(impl_.trait_id);

    if trait_info.generics.len() != impl_.trait_generics.len() {
        let expected = trait_info.generics.len();
        let found = impl_.trait_generics.len();
        let location = impl_.ident.location();
        let item = trait_info.name.to_string();
        errors.push(TypeCheckError::GenericCountMismatch { item, expected, found, location });
    }

    // Substitute each generic on the trait with the corresponding generic on the impl
    let mut bindings = interner.trait_to_impl_bindings(
        impl_.trait_id,
        impl_id,
        &impl_.trait_generics,
        impl_.typ.clone(),
    );

    // If this is None, the trait does not have the corresponding function.
    // This error should have been caught in name resolution already so we don't
    // issue an error for it here.
    if let Some(trait_fn_id) = trait_info.method_ids.get(method_name) {
        let trait_fn_meta = interner.function_meta(trait_fn_id);

        if trait_fn_meta.direct_generics.len() != meta.direct_generics.len() {
            let expected = trait_fn_meta.direct_generics.len();
            let found = meta.direct_generics.len();
            let location = meta.name.location;
            let item = method_name.to_string();
            errors.push(TypeCheckError::GenericCountMismatch { item, expected, found, location });
        }

        // Substitute each generic on the trait function with the corresponding generic on the impl function
        for (
            ResolvedGeneric { type_var: trait_fn_generic, .. },
            ResolvedGeneric { name, type_var: impl_fn_generic, .. },
        ) in trait_fn_meta.direct_generics.iter().zip(&meta.direct_generics)
        {
            let trait_fn_kind = trait_fn_generic.kind();
            let arg = Type::NamedGeneric(NamedGeneric {
                type_var: impl_fn_generic.clone(),
                name: name.clone(),
                implicit: false,
            });
            bindings.insert(trait_fn_generic.id(), (trait_fn_generic.clone(), trait_fn_kind, arg));
        }

        let (declaration_type, _) = trait_fn_meta.typ.instantiate_with_bindings(bindings, interner);

        check_function_type_matches_expected_type(
            &declaration_type,
            definition_type,
            method_name,
            &meta.parameters,
            &meta.return_type,
            noir_function,
            meta.name.location,
            trait_info.name.as_str(),
            &mut errors,
        );
    }

    errors
}

#[allow(clippy::too_many_arguments)]
fn check_function_type_matches_expected_type(
    expected: &Type,
    actual: &Type,
    method_name: &str,
    actual_parameters: &Parameters,
    actual_return_type: &FunctionReturnType,
    noir_function: &NoirFunction,
    location: Location,
    trait_name: &str,
    errors: &mut Vec<TypeCheckError>,
) {
    let mut bindings = TypeBindings::default();
    if let (
        Type::Function(params_a, ret_a, env_a, unconstrained_a),
        Type::Function(params_b, ret_b, env_b, unconstrained_b),
    ) = (expected, actual)
    {
        // Shouldn't need to unify envs, they should always be equal since they're both free functions
        debug_assert_eq!(env_a, env_b, "envs should match as they're both free functions");

        if unconstrained_a != unconstrained_b {
            errors.push(TypeCheckError::UnconstrainedMismatch {
                item: method_name.to_string(),
                expected: *unconstrained_a,
                location,
            });
        }

        if params_a.len() == params_b.len() {
            for (i, (a, b)) in params_a.iter().zip(params_b.iter()).enumerate() {
                if a.try_unify(b, &mut bindings).is_err() {
                    let parameter_location = noir_function.def.parameters.get(i);
                    let parameter_location = parameter_location.map(|param| param.typ.location);
                    let parameter_location =
                        parameter_location.unwrap_or_else(|| actual_parameters.0[i].0.location());

                    errors.push(TypeCheckError::TraitMethodParameterTypeMismatch {
                        method_name: method_name.to_string(),
                        expected_typ: a.to_string(),
                        actual_typ: b.to_string(),
                        parameter_location,
                        parameter_index: i + 1,
                    });
                }
            }

            if ret_b.try_unify(ret_a, &mut bindings).is_err() {
                errors.push(TypeCheckError::TypeMismatch {
                    expected_typ: ret_a.to_string(),
                    expr_typ: ret_b.to_string(),
                    expr_location: actual_return_type.location(),
                });
            }
        } else {
            errors.push(TypeCheckError::MismatchTraitImplNumParameters {
                actual_num_parameters: params_b.len(),
                expected_num_parameters: params_a.len(),
                trait_name: trait_name.to_string(),
                method_name: method_name.to_string(),
                location,
            });
        }
    }

    // If result bindings is not empty, a type variable was bound which means the two
    // signatures were not a perfect match. Note that this relies on us already binding
    // all the expected generics to each other prior to this check.
    if !bindings.is_empty() {
        let expected_typ = expected.to_string();
        let expr_typ = actual.to_string();
        errors.push(TypeCheckError::TypeMismatch {
            expected_typ,
            expr_typ,
            expr_location: location,
        });
    }
}
