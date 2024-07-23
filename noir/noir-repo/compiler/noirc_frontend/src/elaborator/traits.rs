use std::{collections::BTreeMap, rc::Rc};

use iter_extended::vecmap;
use noirc_errors::{Location, Span};

use crate::{
    ast::{
        FunctionKind, TraitItem, UnresolvedGeneric, UnresolvedGenerics, UnresolvedTraitConstraint,
    },
    hir::{
        def_collector::dc_crate::{
            CollectedItems, CompilationError, UnresolvedTrait, UnresolvedTraitImpl,
        },
        type_check::TypeCheckError,
    },
    hir_def::{
        function::Parameters,
        traits::{TraitConstant, TraitFunction, TraitType},
    },
    macros_api::{
        BlockExpression, FunctionDefinition, FunctionReturnType, Ident, ItemVisibility,
        NodeInterner, NoirFunction, Param, Pattern, UnresolvedType, Visibility,
    },
    node_interner::{FuncId, TraitId},
    token::Attributes,
    Kind, ResolvedGeneric, Type, TypeBindings, TypeVariableKind,
};

use super::Elaborator;

impl<'context> Elaborator<'context> {
    pub fn collect_traits(
        &mut self,
        traits: BTreeMap<TraitId, UnresolvedTrait>,
        generated_items: &mut CollectedItems,
    ) {
        for (trait_id, unresolved_trait) in traits {
            self.recover_generics(|this| {
                let resolved_generics = this.interner.get_trait(trait_id).generics.clone();
                this.add_existing_generics(
                    &unresolved_trait.trait_def.generics,
                    &resolved_generics,
                );

                // Resolve order
                // 1. Trait Types ( Trait constants can have a trait type, therefore types before constants)
                let _ = this.resolve_trait_types(&unresolved_trait);
                // 2. Trait Constants ( Trait's methods can use trait types & constants, therefore they should be after)
                let _ = this.resolve_trait_constants(&unresolved_trait);
                // 3. Trait Methods
                let methods = this.resolve_trait_methods(trait_id, &unresolved_trait);

                this.interner.update_trait(trait_id, |trait_def| {
                    trait_def.set_methods(methods);
                });

                let attributes = &unresolved_trait.trait_def.attributes;
                let item = crate::hir::comptime::Value::TraitDefinition(trait_id);
                let span = unresolved_trait.trait_def.span;
                this.run_comptime_attributes_on_item(attributes, item, span, generated_items);
            });

            // This check needs to be after the trait's methods are set since
            // the interner may set `interner.ordering_type` based on the result type
            // of the Cmp trait, if this is it.
            if self.crate_id.is_stdlib() {
                self.interner.try_add_infix_operator_trait(trait_id);
                self.interner.try_add_prefix_operator_trait(trait_id);
            }
        }
    }

    fn resolve_trait_types(&mut self, _unresolved_trait: &UnresolvedTrait) -> Vec<TraitType> {
        // TODO
        vec![]
    }

    fn resolve_trait_constants(
        &mut self,
        _unresolved_trait: &UnresolvedTrait,
    ) -> Vec<TraitConstant> {
        // TODO
        vec![]
    }

    fn resolve_trait_methods(
        &mut self,
        trait_id: TraitId,
        unresolved_trait: &UnresolvedTrait,
    ) -> Vec<TraitFunction> {
        self.local_module = unresolved_trait.module_id;
        self.file = self.def_maps[&self.crate_id].file_id(unresolved_trait.module_id);

        let mut functions = vec![];

        for item in &unresolved_trait.trait_def.items {
            if let TraitItem::Function {
                name,
                generics,
                parameters,
                return_type,
                where_clause,
                body: _,
            } = item
            {
                self.recover_generics(|this| {
                    let the_trait = this.interner.get_trait(trait_id);
                    let self_typevar = the_trait.self_type_typevar.clone();
                    let self_type =
                        Type::TypeVariable(self_typevar.clone(), TypeVariableKind::Normal);
                    let name_span = the_trait.name.span();

                    this.add_existing_generic(
                        &UnresolvedGeneric::Variable(Ident::from("Self")),
                        name_span,
                        &ResolvedGeneric {
                            name: Rc::new("Self".to_owned()),
                            type_var: self_typevar,
                            span: name_span,
                            kind: Kind::Normal,
                        },
                    );
                    this.self_type = Some(self_type.clone());

                    let func_id = unresolved_trait.method_ids[&name.0.contents];

                    this.resolve_trait_function(
                        trait_id,
                        name,
                        generics,
                        parameters,
                        return_type,
                        where_clause,
                        func_id,
                    );

                    let func_meta = this.interner.function_meta(&func_id);

                    let arguments = vecmap(&func_meta.parameters.0, |(_, typ, _)| typ.clone());
                    let return_type = func_meta.return_type().clone();

                    let generics =
                        vecmap(&this.generics.clone(), |generic| generic.type_var.clone());

                    let default_impl_list: Vec<_> = unresolved_trait
                        .fns_with_default_impl
                        .functions
                        .iter()
                        .filter(|(_, _, q)| q.name() == name.0.contents)
                        .collect();

                    let default_impl = if default_impl_list.len() == 1 {
                        Some(Box::new(default_impl_list[0].2.clone()))
                    } else {
                        None
                    };

                    let no_environment = Box::new(Type::Unit);
                    let function_type =
                        Type::Function(arguments, Box::new(return_type), no_environment);

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
        generics: &UnresolvedGenerics,
        parameters: &[(Ident, UnresolvedType)],
        return_type: &FunctionReturnType,
        where_clause: &[UnresolvedTraitConstraint],
        func_id: FuncId,
    ) {
        let old_generic_count = self.generics.len();

        self.scopes.start_function();

        let kind = FunctionKind::Normal;
        let def = FunctionDefinition {
            name: name.clone(),
            attributes: Attributes::empty(),
            is_unconstrained: false,
            is_comptime: false,
            visibility: ItemVisibility::Public, // Trait functions are always public
            generics: generics.clone(),
            parameters: vecmap(parameters, |(name, typ)| Param {
                visibility: Visibility::Private,
                pattern: Pattern::Identifier(name.clone()),
                typ: typ.clone(),
                span: name.span(),
            }),
            body: BlockExpression { statements: Vec::new() },
            span: name.span(),
            where_clause: where_clause.to_vec(),
            return_type: return_type.clone(),
            return_visibility: Visibility::Private,
        };

        let mut function = NoirFunction { kind, def };
        self.define_function_meta(&mut function, func_id, Some(trait_id));
        self.elaborate_function(func_id);
        let _ = self.scopes.end_function();
        // Don't check the scope tree for unused variables, they can't be used in a declaration anyway.
        self.generics.truncate(old_generic_count);
    }

    pub fn resolve_trait_impl_generics(
        &mut self,
        trait_impl: &UnresolvedTraitImpl,
        trait_id: TraitId,
    ) -> Option<Vec<Type>> {
        let trait_def = self.interner.get_trait(trait_id);
        let resolved_generics = trait_def.generics.clone();
        if resolved_generics.len() != trait_impl.trait_generics.len() {
            self.push_err(CompilationError::TypeError(TypeCheckError::GenericCountMismatch {
                item: trait_def.name.to_string(),
                expected: resolved_generics.len(),
                found: trait_impl.trait_generics.len(),
                span: trait_impl.trait_path.span(),
            }));

            return None;
        }

        let generics = trait_impl.trait_generics.iter().zip(resolved_generics.iter());
        let mapped = generics.map(|(generic, resolved_generic)| {
            self.resolve_type_inner(generic.clone(), &resolved_generic.kind)
        });
        Some(mapped.collect())
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
/// - Self for Bar<E>
/// - A for D
/// - B for F
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
) -> Vec<TypeCheckError> {
    let meta = interner.function_meta(&function);
    let method_name = interner.function_name(&function);
    let mut errors = Vec::new();

    let definition_type = meta.typ.as_monotype();

    let impl_ =
        meta.trait_impl.expect("Trait impl function should have a corresponding trait impl");

    // If the trait implementation is not defined in the interner then there was a previous
    // error in resolving the trait path and there is likely no trait for this impl.
    let Some(impl_) = interner.try_get_trait_implementation(impl_) else {
        return errors;
    };

    let impl_ = impl_.borrow();
    let trait_info = interner.get_trait(impl_.trait_id);

    let mut bindings = TypeBindings::new();
    bindings.insert(
        trait_info.self_type_typevar_id,
        (trait_info.self_type_typevar.clone(), impl_.typ.clone()),
    );

    if trait_info.generics.len() != impl_.trait_generics.len() {
        let expected = trait_info.generics.len();
        let found = impl_.trait_generics.len();
        let span = impl_.ident.span();
        let item = trait_info.name.to_string();
        errors.push(TypeCheckError::GenericCountMismatch { item, expected, found, span });
    }

    // Substitute each generic on the trait with the corresponding generic on the impl
    for (generic, arg) in trait_info.generics.iter().zip(&impl_.trait_generics) {
        bindings.insert(generic.type_var.id(), (generic.type_var.clone(), arg.clone()));
    }

    // If this is None, the trait does not have the corresponding function.
    // This error should have been caught in name resolution already so we don't
    // issue an error for it here.
    if let Some(trait_fn_id) = trait_info.method_ids.get(method_name) {
        let trait_fn_meta = interner.function_meta(trait_fn_id);

        if trait_fn_meta.direct_generics.len() != meta.direct_generics.len() {
            let expected = trait_fn_meta.direct_generics.len();
            let found = meta.direct_generics.len();
            let span = meta.name.location.span;
            let item = method_name.to_string();
            errors.push(TypeCheckError::GenericCountMismatch { item, expected, found, span });
        }

        // Substitute each generic on the trait function with the corresponding generic on the impl function
        for (
            ResolvedGeneric { type_var: trait_fn_generic, .. },
            ResolvedGeneric { name, type_var: impl_fn_generic, .. },
        ) in trait_fn_meta.direct_generics.iter().zip(&meta.direct_generics)
        {
            let arg = Type::NamedGeneric(impl_fn_generic.clone(), name.clone(), Kind::Normal);
            bindings.insert(trait_fn_generic.id(), (trait_fn_generic.clone(), arg));
        }

        let (declaration_type, _) = trait_fn_meta.typ.instantiate_with_bindings(bindings, interner);

        check_function_type_matches_expected_type(
            &declaration_type,
            definition_type,
            method_name,
            &meta.parameters,
            meta.name.location.span,
            &trait_info.name.0.contents,
            &mut errors,
        );
    }

    errors
}

fn check_function_type_matches_expected_type(
    expected: &Type,
    actual: &Type,
    method_name: &str,
    actual_parameters: &Parameters,
    span: Span,
    trait_name: &str,
    errors: &mut Vec<TypeCheckError>,
) {
    let mut bindings = TypeBindings::new();
    // Shouldn't need to unify envs, they should always be equal since they're both free functions
    if let (Type::Function(params_a, ret_a, _env_a), Type::Function(params_b, ret_b, _env_b)) =
        (expected, actual)
    {
        if params_a.len() == params_b.len() {
            for (i, (a, b)) in params_a.iter().zip(params_b.iter()).enumerate() {
                if a.try_unify(b, &mut bindings).is_err() {
                    errors.push(TypeCheckError::TraitMethodParameterTypeMismatch {
                        method_name: method_name.to_string(),
                        expected_typ: a.to_string(),
                        actual_typ: b.to_string(),
                        parameter_span: actual_parameters.0[i].0.span(),
                        parameter_index: i + 1,
                    });
                }
            }

            if ret_b.try_unify(ret_a, &mut bindings).is_err() {
                errors.push(TypeCheckError::TypeMismatch {
                    expected_typ: ret_a.to_string(),
                    expr_typ: ret_b.to_string(),
                    expr_span: span,
                });
            }
        } else {
            errors.push(TypeCheckError::MismatchTraitImplNumParameters {
                actual_num_parameters: params_b.len(),
                expected_num_parameters: params_a.len(),
                trait_name: trait_name.to_string(),
                method_name: method_name.to_string(),
                span,
            });
        }
    }

    // If result bindings is not empty, a type variable was bound which means the two
    // signatures were not a perfect match. Note that this relies on us already binding
    // all the expected generics to each other prior to this check.
    if !bindings.is_empty() {
        let expected_typ = expected.to_string();
        let expr_typ = actual.to_string();
        errors.push(TypeCheckError::TypeMismatch { expected_typ, expr_typ, expr_span: span });
    }
}
