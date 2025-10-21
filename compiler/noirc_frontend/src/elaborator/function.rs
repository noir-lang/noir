//! Function metadata and function body elaboration.
//!
//! This module handles the definition and elaboration of functions:
//! - Collect and resolve function metadata (i.e., signatures with type information). Type information includes:
//!   - Generics, parameters, trait constraints which will all be resolved while collecting function metas.
//!   - This metadata is also used for elaboration of impls and trait impls
//! - Second stage elaboration strategy of function bodies and their return type.
//!   - Shared strategy for all types of functions (standalone, impl, trait impl)

use std::rc::Rc;

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    Kind, NamedGeneric, Type, TypeVariable,
    ast::{
        BlockExpression, FunctionKind, GenericTypeArgs, NoirFunction, Param, UnresolvedGenerics,
        UnresolvedTraitConstraint, UnresolvedType, UnresolvedTypeData,
    },
    elaborator::lints,
    hir::{
        def_collector::dc_crate::{ImplMap, UnresolvedFunctions, UnresolvedTraitImpl},
        resolution::errors::ResolverError,
        type_check::TypeCheckError,
    },
    hir_def::{
        expr::HirIdent,
        function::{FuncMeta, FunctionBody, HirFunction},
        stmt::HirPattern,
        traits::TraitConstraint,
    },
    node_interner::{DefinitionKind, DependencyId, FuncId, FunctionModifiers, TraitId},
    shared::Visibility,
    validity::length_is_zero,
};

use super::Elaborator;

type ResolvedParametersInfo = (Vec<(HirPattern, Type, Visibility)>, Vec<Type>, Vec<HirIdent>);

impl Elaborator<'_> {
    /// Defines function metadata for all functions, impl methods, and trait impl methods.
    /// This is the first pass of function elaboration that extracts type signatures and
    /// resolves generics before the function bodies are elaborated.
    pub(super) fn define_function_metas(
        &mut self,
        functions: &mut [UnresolvedFunctions],
        impls: &mut ImplMap,
        trait_impls: &mut [UnresolvedTraitImpl],
    ) {
        // Define metas for regular functions
        for function_set in functions {
            self.define_function_metas_for_functions(function_set, &[]);
        }

        // Define metas for impl functions
        for ((self_type, local_module), function_sets) in impls {
            self.define_function_metas_for_impl(self_type, *local_module, function_sets);
        }

        // Define metas for trait impl functions
        for trait_impl in trait_impls {
            self.define_function_metas_for_trait_impl(trait_impl);
        }
    }

    /// Defines function metadata for a set of functions with optional extra trait constraints.
    /// This is used for both standalone functions and methods within impls/trait impls.
    fn define_function_metas_for_functions(
        &mut self,
        function_set: &mut UnresolvedFunctions,
        extra_constraints: &[(TraitConstraint, Location)],
    ) {
        for (local_module, id, func) in &mut function_set.functions {
            self.local_module = *local_module;
            self.recover_generics(|this| {
                this.define_function_meta(func, *id, None, extra_constraints);
            });
        }
    }

    /// Defines function metadata for all methods within an impl block.
    /// Resolves the self type and adds it to scope for method resolution.
    fn define_function_metas_for_impl(
        &mut self,
        self_type: &UnresolvedType,
        local_module: crate::hir::def_map::LocalModuleId,
        function_sets: &mut Vec<(UnresolvedGenerics, Location, UnresolvedFunctions)>,
    ) {
        self.local_module = local_module;

        for (generics, _, function_set) in function_sets {
            self.add_generics(generics);
            let wildcard_allowed = false;
            let self_type = self.resolve_type(self_type.clone(), wildcard_allowed);

            function_set.self_type = Some(self_type.clone());
            self.self_type = Some(self_type);
            self.define_function_metas_for_functions(function_set, &[]);
            self.self_type = None;
            self.generics.clear();
        }
    }

    /// Defines function metadata for all methods within a trait impl.
    /// This handles trait resolution, generics, associated types, and constraint checking.
    fn define_function_metas_for_trait_impl(&mut self, trait_impl: &mut UnresolvedTraitImpl) {
        self.local_module = trait_impl.module_id;

        let (trait_id, trait_generics, path_location) =
            self.resolve_trait_impl_trait_path(trait_impl);

        // If we could not find a trait we can skip elaborating the remaining functions in the trait impl
        // as we should push errors that a trait is missing.
        if trait_id.is_none() {
            return;
        }
        trait_impl.trait_id = trait_id;

        let (constraints, new_generics_trait_constraints) =
            self.setup_trait_impl_generics(trait_impl);

        self.resolve_trait_impl_associated_types(
            trait_impl,
            trait_generics,
            trait_id,
            path_location,
        );

        self.remove_trait_constraints_from_scope(
            constraints
                .iter()
                .chain(new_generics_trait_constraints.iter().map(|(constraint, _)| constraint)),
        );

        let wildcard_allowed = false;
        let unresolved_type = trait_impl.object_type.clone();
        let self_type = self.resolve_type(unresolved_type, wildcard_allowed);
        self.self_type = Some(self_type.clone());
        trait_impl.methods.self_type = Some(self_type);

        self.define_function_metas_for_functions(
            &mut trait_impl.methods,
            &new_generics_trait_constraints,
        );

        trait_impl.resolved_object_type = self.self_type.take();
        // We should only take the `current_trait_impl` after defining the function metas for trait impl methods.
        // The impl ID is needed later to perform extra trait impl validation checks.
        trait_impl.impl_id = self.current_trait_impl.take();
        self.generics.clear();

        // Add trait reference
        if let Some(trait_id) = trait_id {
            let (location, is_self_type_name) = match &trait_impl.r#trait.typ {
                UnresolvedTypeData::Named(trait_path, _, _) => {
                    let trait_name = trait_path.last_ident();
                    (trait_name.location(), trait_name.is_self_type_name())
                }
                _ => (trait_impl.r#trait.location, false),
            };
            self.interner.add_trait_reference(trait_id, location, is_self_type_name);
        }
    }

    /// Resolves the trait path from a trait impl declaration.
    /// Returns (trait_id, trait_generics, path_location).
    fn resolve_trait_impl_trait_path(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> (Option<TraitId>, GenericTypeArgs, Location) {
        match &trait_impl.r#trait.typ {
            UnresolvedTypeData::Named(trait_path, trait_generics, _) => {
                let mut trait_generics = trait_generics.clone();
                let location = trait_path.location;
                let trait_path = self.validate_path(trait_path.clone());
                let trait_id = self.resolve_trait_by_path(trait_path);

                // Check and remove and any generic that is specifying an associated item
                if !trait_generics.named_args.is_empty() {
                    if let Some(trait_id) = trait_id {
                        let associated_types =
                            self.interner.get_trait(trait_id).associated_types.clone();
                        trait_generics.named_args.retain(|(name, typ)| {
                            let associated_type = associated_types.iter().find(|associated_type| {
                                associated_type.name.as_str() == name.as_str()
                            });
                            if associated_type.is_some() {
                                let location = name.location().merge(typ.location);
                                self.push_err(
                                    ResolverError::AssociatedItemConstraintsNotAllowedInGenerics {
                                        location,
                                    },
                                );
                                false
                            } else {
                                true
                            }
                        });
                    }
                }

                (trait_id, trait_generics.clone(), location)
            }
            UnresolvedTypeData::Resolved(quoted_type_id) => {
                let typ = self.interner.get_quoted_type(*quoted_type_id);
                let location = trait_impl.r#trait.location;
                let Type::TraitAsType(trait_id, _, trait_generics) = typ else {
                    let found = typ.to_string();
                    self.push_err(ResolverError::ExpectedTrait { location, found });
                    return (None, GenericTypeArgs::default(), location);
                };

                // In order to take associated types into account we turn these resolved generics
                // into unresolved ones, but ones that point to solved types.
                let trait_id = *trait_id;
                let trait_generics = trait_generics.clone();
                let trait_generics = GenericTypeArgs {
                    ordered_args: vecmap(&trait_generics.ordered, |typ| {
                        let quoted_type_id = self.interner.push_quoted_type(typ.clone());
                        let typ = UnresolvedTypeData::Resolved(quoted_type_id);
                        UnresolvedType { typ, location }
                    }),
                    named_args: vecmap(&trait_generics.named, |named_type| {
                        let quoted_type_id = self.interner.push_quoted_type(named_type.typ.clone());
                        let typ = UnresolvedTypeData::Resolved(quoted_type_id);
                        (named_type.name.clone(), UnresolvedType { typ, location })
                    }),
                    kinds: Vec::new(),
                };

                (Some(trait_id), trait_generics, location)
            }
            _ => {
                let location = trait_impl.r#trait.location;
                let found = trait_impl.r#trait.typ.to_string();
                self.push_err(ResolverError::ExpectedTrait { location, found });
                (None, GenericTypeArgs::default(), location)
            }
        }
    }

    /// Sets up generics for a trait impl and processes trait constraints from the where clause.
    /// Returns tuple of (resolved constraints, new generic constraints).
    fn setup_trait_impl_generics(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> (Vec<TraitConstraint>, Vec<(TraitConstraint, Location)>) {
        self.add_generics(&trait_impl.generics);
        trait_impl.resolved_generics = self.generics.clone();

        let new_generics = self.desugar_trait_constraints(&mut trait_impl.where_clause);
        let mut new_generics_trait_constraints = Vec::new();
        for (new_generic, bounds) in new_generics {
            for bound in bounds {
                let typ = Type::TypeVariable(new_generic.type_var.clone());
                let location = new_generic.location;
                self.add_trait_bound_to_scope(location, &typ, &bound, bound.trait_id);
                new_generics_trait_constraints
                    .push((TraitConstraint { typ, trait_bound: bound }, location));
            }
            trait_impl.resolved_generics.push(new_generic.clone());
            self.generics.push(new_generic);
        }

        // We need to resolve the where clause before any associated types to be
        // able to resolve trait as type syntax, eg. `<T as Foo>` in case there
        // is a where constraint for `T: Foo`.
        let constraints = self.resolve_trait_constraints(&trait_impl.where_clause);

        // Attach any trait constraints on the impl to the function
        for (_, _, method) in trait_impl.methods.functions.iter_mut() {
            method.def.where_clause.append(&mut trait_impl.where_clause.clone());
        }

        // Return the constraints along with the new generics trait constraints
        // so they can be removed from scope later
        (constraints, new_generics_trait_constraints)
    }

    /// Resolves associated types for a trait impl and checks for missing generics.
    /// Sets resolved_trait_generics, impl_id, and unresolved_associated_types on trait_impl.
    fn resolve_trait_impl_associated_types(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
        mut trait_generics: GenericTypeArgs,
        trait_id: Option<TraitId>,
        path_location: Location,
    ) {
        let impl_id = self.interner.next_trait_impl_id();
        self.current_trait_impl = Some(impl_id);

        // Add each associated type to the list of named type arguments
        let associated_types = self.take_unresolved_associated_types(trait_impl);

        // Put every associated type behind a type variable (inside a NamedGeneric).
        // This way associated types can be referred to even if their actual value (for associated constants)
        // is not known yet. This is to allow associated constants to refer to associated constants
        // in other trait impls.
        let associated_types_behind_type_vars = vecmap(&associated_types, |(name, _typ, kind)| {
            let new_generic_id = self.interner.next_type_variable_id();
            let type_var = TypeVariable::unbound(new_generic_id, kind.clone());
            let typ = Type::NamedGeneric(NamedGeneric {
                type_var: type_var.clone(),
                name: Rc::new(name.to_string()),
                implicit: false,
            });
            let typ = self.interner.push_quoted_type(typ);
            let typ = UnresolvedTypeData::Resolved(typ).with_location(name.location());
            (name.clone(), typ)
        });

        trait_generics.named_args.extend(associated_types_behind_type_vars);

        let associated_types = vecmap(associated_types, |(name, typ, _kind)| (name, typ));

        let (ordered_generics, named_generics) = trait_id
            .map(|trait_id| {
                // Check for missing generics & associated types for the trait being implemented
                self.resolve_trait_args_from_trait_impl(trait_generics, trait_id, path_location)
            })
            .unwrap_or_default();

        trait_impl.resolved_trait_generics = ordered_generics;
        self.interner.set_associated_types_for_impl(impl_id, named_generics);

        // trait_impl.impl_id = self.current_trait_impl.take();
        trait_impl.unresolved_associated_types = associated_types;
    }

    /// Extracts and stores metadata from a function definition.
    ///
    /// This resolves the function's signature including generics, parameters, return type,
    /// and trait constraints. The function body is stored unresolved and will be elaborated
    /// later by [Self::elaborate_function].
    ///
    /// Prerequisite: any implicit generics from enclosing impls have already been added
    /// to scope via [Self::add_generics].
    pub(super) fn define_function_meta(
        &mut self,
        func: &mut NoirFunction,
        func_id: FuncId,
        trait_id: Option<TraitId>,
        extra_trait_constraints: &[(TraitConstraint, Location)],
    ) {
        self.scopes.start_function();
        self.current_item = Some(DependencyId::Function(func_id));

        let location = func.name_ident().location();
        let id = self.interner.function_definition_id(func_id);
        let name_ident = HirIdent::non_trait_method(id, location);

        // Add generics to scope
        let (mut generics, associated_generics_trait_constraints) =
            self.add_function_generics_to_scope(&func.def.generics, &mut func.def.where_clause);

        // Setup trait constraints
        for (extra_constraint, location) in extra_trait_constraints {
            let bound = &extra_constraint.trait_bound;
            self.add_trait_bound_to_scope(*location, &extra_constraint.typ, bound, bound.trait_id);
        }

        let mut trait_constraints = self.resolve_trait_constraints(&func.def.where_clause);
        let mut extra_trait_constraints =
            vecmap(extra_trait_constraints, |(constraint, _)| constraint.clone());
        extra_trait_constraints.extend(associated_generics_trait_constraints);

        // Resolve parameters
        let (parameters, parameter_types, parameter_idents) =
            self.resolve_function_parameters(func, &mut generics, &mut trait_constraints);

        // Resolve return type
        let wildcard_allowed = false;
        let return_type = Box::new(self.use_type(func.return_type(), wildcard_allowed));

        let is_entry_point = func.is_entry_point(self.is_function_in_contract());
        // Temporary allow slices for contract functions, until contracts are re-factored.
        if !func.attributes().has_contract_library_method() {
            self.check_if_type_is_valid_for_program_output(
                &return_type,
                is_entry_point || func.is_test_or_fuzz(),
                func.has_inline_attribute(),
                location,
            );
        }

        // Build function type
        let mut typ = Type::Function(
            parameter_types,
            return_type,
            Box::new(Type::Unit),
            func.def.is_unconstrained,
        );
        if !generics.is_empty() {
            typ = Type::Forall(generics, Box::new(typ));
        }
        self.interner.push_definition_type(name_ident.id, typ.clone());

        // Set up metadata to place on main FuncMeta structure
        let direct_generics = func.def.generics.iter();
        let direct_generics = direct_generics
            .filter_map(|generic| {
                generic.ident().ident().and_then(|name| self.find_generic(name.as_str())).cloned()
            })
            .collect();

        let statements = std::mem::take(&mut func.def.body.statements);
        let body = BlockExpression { statements };

        let struct_id = if let Some(Type::DataType(struct_type, _)) = &self.self_type {
            Some(struct_type.borrow().id)
        } else {
            None
        };

        // Remove the traits assumed by `resolve_trait_constraints` from scope
        self.remove_trait_constraints_from_scope(
            trait_constraints.iter().chain(extra_trait_constraints.iter()),
        );

        let meta = FuncMeta {
            name: name_ident,
            kind: func.kind,
            location,
            typ,
            direct_generics,
            all_generics: self.generics.clone(),
            type_id: struct_id,
            trait_id,
            trait_impl: self.current_trait_impl,
            enum_variant_index: None,
            parameters: parameters.into(),
            parameter_idents,
            return_type: func.def.return_type.clone(),
            return_visibility: func.def.return_visibility,
            has_body: !func.def.body.is_empty(),
            trait_constraints,
            extra_trait_constraints,
            is_entry_point,
            has_inline_attribute: func.has_inline_attribute(),
            source_crate: self.crate_id,
            source_module: self.local_module,
            function_body: FunctionBody::Unresolved(func.kind, body, func.def.location),
            self_type: self.self_type.clone(),
            source_file: location.file,
        };

        let modifiers = self.interner.function_modifiers(&func_id).clone();

        self.run_function_lints(&meta, &modifiers);
        self.check_function_visibility(&meta, &modifiers, func.name_ident(), location);

        self.interner.push_fn_meta(meta, func_id);
        self.scopes.end_function();
        self.current_item = None;
    }

    /// Adds function generics and associated generics (from where clause) to scope.
    ///
    /// Returns (generics, associated_generics_trait_constraints) where generics contains
    /// both associated and explicit generics in the correct order (associated first, then explicit function generics).
    fn add_function_generics_to_scope(
        &mut self,
        func_generics: &UnresolvedGenerics,
        where_clause: &mut [UnresolvedTraitConstraint],
    ) -> (Vec<TypeVariable>, Vec<TraitConstraint>) {
        self.add_generics(func_generics);

        let func_generics = vecmap(&self.generics, |generic| generic.type_var.clone());

        let associated_generics = self.desugar_trait_constraints(where_clause);

        let mut generics = Vec::with_capacity(associated_generics.len());
        let mut associated_generics_trait_constraints = Vec::new();

        for (associated_generic, bounds) in associated_generics {
            for bound in bounds {
                let typ = Type::TypeVariable(associated_generic.type_var.clone());
                let location = associated_generic.location;
                self.add_trait_bound_to_scope(location, &typ, &bound, bound.trait_id);
                associated_generics_trait_constraints
                    .push(TraitConstraint { typ, trait_bound: bound });
            }

            generics.push(associated_generic.type_var);
        }

        // We put associated generics first, as they are implicit and implicit generics
        // come before explicit generics (see `Type::instantiate_with`).
        generics.extend(func_generics);

        (generics, associated_generics_trait_constraints)
    }

    fn is_function_in_contract(&self) -> bool {
        if self.self_type.is_some() {
            // Without this, impl methods can accidentally be placed in contracts.
            // See: https://github.com/noir-lang/noir/issues/3254
            false
        } else {
            self.in_contract()
        }
    }

    /// True if the `pub` keyword is allowed on parameters in this function
    /// `pub` on function parameters is only allowed for entry point functions
    fn pub_allowed(&self, func: &NoirFunction, in_contract: bool) -> bool {
        func.is_entry_point(in_contract) || func.attributes().is_foldable()
    }

    /// Resolves function parameters and validates their types for entry points.
    ///
    /// Returns (parameters, parameter_types, parameter_idents) where generics and
    /// trait_constraints may be extended due to `impl Trait` desugaring.
    fn resolve_function_parameters(
        &mut self,
        func: &NoirFunction,
        generics: &mut Vec<TypeVariable>,
        trait_constraints: &mut Vec<TraitConstraint>,
    ) -> ResolvedParametersInfo {
        let is_entry_point = func.is_entry_point(self.is_function_in_contract());
        let is_test_or_fuzz = func.is_test_or_fuzz();

        let has_inline_attribute = func.has_inline_attribute();
        let is_pub_allowed = self.pub_allowed(func, self.is_function_in_contract());

        let mut parameters = Vec::new();
        let mut parameter_types = Vec::new();
        let mut parameter_idents = Vec::new();
        let wildcard_allowed = false;

        for Param { visibility, pattern, typ, location: _ } in func.parameters().iter().cloned() {
            self.run_lint(|_| {
                lints::unnecessary_pub_argument(func, visibility, is_pub_allowed).map(Into::into)
            });

            let type_location = typ.location;
            let typ = match typ.typ {
                UnresolvedTypeData::TraitAsType(path, args) => {
                    self.desugar_impl_trait_arg(path, args, generics, trait_constraints)
                }
                // Function parameters have Kind::Normal
                _ => self.resolve_type_with_kind(typ, &Kind::Normal, wildcard_allowed),
            };

            self.check_if_type_is_valid_for_program_input(
                &typ,
                is_entry_point || is_test_or_fuzz,
                has_inline_attribute,
                type_location,
            );

            if is_entry_point || is_test_or_fuzz {
                self.mark_type_as_used(&typ);
            }

            let pattern = self.elaborate_pattern_and_store_ids(
                pattern,
                typ.clone(),
                DefinitionKind::Local(None),
                &mut parameter_idents,
                true, // warn_if_unused
            );

            parameters.push((pattern, typ.clone(), visibility));
            parameter_types.push(typ);
        }

        (parameters, parameter_types, parameter_idents)
    }

    /// Only sized types are valid to be used as main's parameters or the parameters to a contract
    /// function. If the given type is not sized (e.g. contains a slice or NamedGeneric type), an
    /// error is issued.
    fn check_if_type_is_valid_for_program_input(
        &mut self,
        typ: &Type,
        is_entry_point: bool,
        has_inline_attribute: bool,
        location: Location,
    ) {
        if is_entry_point {
            if let Some(invalid_type) = typ.program_input_validity() {
                self.push_err(TypeCheckError::InvalidTypeForEntryPoint { invalid_type, location });
                return;
            }
        }

        if has_inline_attribute {
            if let Some(invalid_type) = typ.non_inlined_function_input_validity() {
                self.push_err(TypeCheckError::InvalidTypeForEntryPoint { invalid_type, location });
            }
        }
    }

    fn check_if_type_is_valid_for_program_output(
        &mut self,
        typ: &Type,
        is_entry_point: bool,
        has_inline_attribute: bool,
        location: Location,
    ) {
        match typ {
            Type::Unit => return,
            Type::Array(length, _) | Type::String(length) => {
                if length_is_zero(length) {
                    //returning zero length arrays is allowed
                    return;
                }
            }
            _ => (),
        }

        self.check_if_type_is_valid_for_program_input(
            typ,
            is_entry_point,
            has_inline_attribute,
            location,
        );
    }

    fn run_function_lints(&mut self, func: &FuncMeta, modifiers: &FunctionModifiers) {
        self.run_lint(|_| lints::inlining_attributes(func, modifiers).map(Into::into));
        self.run_lint(|_| lints::missing_pub(func, modifiers).map(Into::into));
        self.run_lint(|_| {
            let pub_allowed = func.is_entry_point || modifiers.attributes.is_foldable();
            lints::unnecessary_pub_return(func, modifiers, pub_allowed).map(Into::into)
        });
        self.run_lint(|_| lints::oracle_not_marked_unconstrained(func, modifiers).map(Into::into));
        self.run_lint(|elaborator| {
            lints::low_level_function_outside_stdlib(modifiers, elaborator.crate_id).map(Into::into)
        });
    }

    /// Elaborates a function's body and performs type checking.
    ///
    /// This is the second pass of function elaboration that processes the function body,
    /// resolves all expressions and statements, performs type checking, and verifies
    /// trait constraints. The function metadata must already be defined by [Self::define_function_meta].
    pub(crate) fn elaborate_function(&mut self, id: FuncId) {
        let func_meta = self.interner.func_meta.get_mut(&id);
        let func_meta =
            func_meta.expect("FuncMetas should be declared before a function is elaborated");

        let (kind, body, body_location) = match func_meta.take_body() {
            FunctionBody::Unresolved(kind, body, location) => (kind, body, location),
            FunctionBody::Resolved => return,
            // Do not error for the still-resolving case. If there is a dependency cycle,
            // the dependency cycle check will find it later on.
            FunctionBody::Resolving => return,
        };

        let func_meta = func_meta.clone();

        assert_eq!(
            self.crate_id, func_meta.source_crate,
            "Functions in other crates should be already elaborated"
        );

        self.local_module = func_meta.source_module;
        self.self_type = func_meta.self_type.clone();
        self.current_trait_impl = func_meta.trait_impl;

        self.scopes.start_function();
        let old_item = self.current_item.replace(DependencyId::Function(id));

        self.trait_bounds = func_meta.all_trait_constraints().cloned().collect();
        self.push_function_context();

        self.introduce_generics_into_scope(func_meta.all_generics.clone());

        // The DefinitionIds for each parameter were already created in define_function_meta
        // so we need to reintroduce the same IDs into scope here.
        for parameter in &func_meta.parameter_idents {
            let name = self.interner.definition_name(parameter.id).to_owned();
            if name == "_" {
                continue;
            }
            let warn_if_unused = !(func_meta.trait_impl.is_some() && name == "self");
            self.add_existing_variable_to_scope(name, parameter.clone(), warn_if_unused);
        }

        self.add_trait_constraints_to_scope(func_meta.all_trait_constraints(), func_meta.location);

        let (hir_func, body_type) = match kind {
            FunctionKind::Builtin
            | FunctionKind::LowLevel
            | FunctionKind::TraitFunctionWithoutBody => {
                if !body.statements.is_empty() {
                    panic!(
                        "Builtin, low-level, and trait function declarations cannot have a body"
                    );
                }
                (HirFunction::empty(), Type::Error)
            }
            FunctionKind::Oracle => {
                if !body.statements.is_empty() {
                    self.push_err(ResolverError::OracleWithBody {
                        location: func_meta.name.location,
                    });
                }
                (HirFunction::empty(), Type::Error)
            }
            FunctionKind::Normal => {
                let return_type = func_meta.return_type();
                let (block, body_type) = self.elaborate_block(body, Some(return_type));
                let expr_id = self.intern_expr(block, body_location);
                self.interner.push_expr_type(expr_id, body_type.clone());
                (HirFunction::unchecked_from_expr(expr_id), body_type)
            }
        };

        // Don't verify the return type for builtin functions & trait function declarations
        if !func_meta.is_stub() {
            self.type_check_function_body(body_type, &func_meta, hir_func.as_expr());
        }

        // Default any type variables that still need defaulting and
        // verify any remaining trait constraints arising from the function body.
        // This is done before trait impl search since leaving them bindable can lead to errors
        // when multiple impls are available. Instead we default first to choose the Field or u64 impl.
        self.check_and_pop_function_context();

        self.remove_trait_constraints_from_scope(func_meta.all_trait_constraints());

        let func_scope_tree = self.scopes.end_function();

        // The arguments to low-level and oracle functions are always unused so we do not produce warnings for them.
        if !func_meta.is_stub() {
            self.check_for_unused_variables_in_scope_tree(func_scope_tree);
        }

        // Check that the body can return without calling the function.
        if let FunctionKind::Normal = kind {
            self.run_lint(|elaborator| {
                lints::unbounded_recursion(
                    elaborator.interner,
                    id,
                    || elaborator.interner.definition_name(func_meta.name.id),
                    func_meta.name.location,
                    hir_func.as_expr(),
                )
                .map(Into::into)
            });
        }

        let meta = self
            .interner
            .func_meta
            .get_mut(&id)
            .expect("FuncMetas should be declared before a function is elaborated");

        meta.function_body = FunctionBody::Resolved;

        self.trait_bounds.clear();
        self.interner.update_fn(id, hir_func);
        self.current_item = old_item;
    }
}
