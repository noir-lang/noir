use std::collections::{BTreeMap, HashSet};

use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    graph::CrateId,
    hir::{
        def_collector::{
            dc_crate::{
                check_methods_signatures, CompilationError, UnresolvedTrait, UnresolvedTraitImpl,
            },
            errors::{DefCollectorErrorKind, DuplicateType},
        },
        def_map::{CrateDefMap, ModuleDefId, ModuleId},
        Context,
    },
    hir_def::traits::{TraitConstant, TraitFunction, TraitImpl, TraitType},
    node_interner::{FuncId, NodeInterner, TraitId},
    Generics, Path, Shared, TraitItem, Type, TypeVariable, TypeVariableKind,
};

use super::{
    functions, get_module_mut, get_struct_type,
    path_resolver::{PathResolver, StandardPathResolver},
    resolver::Resolver,
    take_errors,
};

/// Create the mappings from TypeId -> TraitType
/// so that expressions can access the elements of traits
pub(crate) fn resolve_traits(
    context: &mut Context,
    traits: BTreeMap<TraitId, UnresolvedTrait>,
    crate_id: CrateId,
) -> Vec<(CompilationError, FileId)> {
    for (trait_id, unresolved_trait) in &traits {
        context.def_interner.push_empty_trait(*trait_id, unresolved_trait);
    }
    let mut all_errors = Vec::new();

    for (trait_id, unresolved_trait) in traits {
        let generics = vecmap(&unresolved_trait.trait_def.generics, |_| {
            TypeVariable::unbound(context.def_interner.next_type_variable_id())
        });

        // Resolve order
        // 1. Trait Types ( Trait constants can have a trait type, therefore types before constants)
        let _ = resolve_trait_types(context, crate_id, &unresolved_trait);
        // 2. Trait Constants ( Trait's methods can use trait types & constants, therefore they should be after)
        let _ = resolve_trait_constants(context, crate_id, &unresolved_trait);
        // 3. Trait Methods
        let (methods, errors) =
            resolve_trait_methods(context, trait_id, crate_id, &unresolved_trait, &generics);

        all_errors.extend(errors);

        context.def_interner.update_trait(trait_id, |trait_def| {
            trait_def.set_methods(methods);
            trait_def.generics = generics;
        });

        // This check needs to be after the trait's methods are set since
        // the interner may set `interner.ordering_type` based on the result type
        // of the Cmp trait, if this is it.
        if crate_id.is_stdlib() {
            context.def_interner.try_add_operator_trait(trait_id);
        }
    }
    all_errors
}

fn resolve_trait_types(
    _context: &mut Context,
    _crate_id: CrateId,
    _unresolved_trait: &UnresolvedTrait,
) -> (Vec<TraitType>, Vec<(CompilationError, FileId)>) {
    // TODO
    (vec![], vec![])
}
fn resolve_trait_constants(
    _context: &mut Context,
    _crate_id: CrateId,
    _unresolved_trait: &UnresolvedTrait,
) -> (Vec<TraitConstant>, Vec<(CompilationError, FileId)>) {
    // TODO
    (vec![], vec![])
}

fn resolve_trait_methods(
    context: &mut Context,
    trait_id: TraitId,
    crate_id: CrateId,
    unresolved_trait: &UnresolvedTrait,
    trait_generics: &Generics,
) -> (Vec<TraitFunction>, Vec<(CompilationError, FileId)>) {
    let interner = &mut context.def_interner;
    let def_maps = &mut context.def_maps;

    let path_resolver = StandardPathResolver::new(ModuleId {
        local_id: unresolved_trait.module_id,
        krate: crate_id,
    });
    let file = def_maps[&crate_id].file_id(unresolved_trait.module_id);

    let mut functions = vec![];
    let mut resolver_errors = vec![];

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
            let the_trait = interner.get_trait(trait_id);
            let self_typevar = the_trait.self_type_typevar.clone();
            let self_type = Type::TypeVariable(self_typevar.clone(), TypeVariableKind::Normal);
            let name_span = the_trait.name.span();

            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.add_generics(generics);
            resolver.add_existing_generics(&unresolved_trait.trait_def.generics, trait_generics);
            resolver.add_existing_generic("Self", name_span, self_typevar);
            resolver.set_self_type(Some(self_type.clone()));

            let func_id = unresolved_trait.method_ids[&name.0.contents];
            let (_, func_meta) = resolver.resolve_trait_function(
                name,
                parameters,
                return_type,
                where_clause,
                func_id,
            );
            resolver.interner.push_fn_meta(func_meta, func_id);

            let arguments = vecmap(parameters, |param| resolver.resolve_type(param.1.clone()));
            let return_type = resolver.resolve_type(return_type.get_type().into_owned());

            let generics = vecmap(resolver.get_generics(), |(_, type_var, _)| type_var.clone());

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
            let function_type = Type::Function(arguments, Box::new(return_type), no_environment);

            functions.push(TraitFunction {
                name: name.clone(),
                typ: Type::Forall(generics, Box::new(function_type)),
                location: Location::new(name.span(), unresolved_trait.file_id),
                default_impl,
                default_impl_module_id: unresolved_trait.module_id,
            });

            let errors = resolver.take_errors().into_iter();
            resolver_errors.extend(errors.map(|resolution_error| (resolution_error.into(), file)));
        }
    }
    (functions, resolver_errors)
}

fn collect_trait_impl_methods(
    interner: &mut NodeInterner,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    crate_id: CrateId,
    trait_id: TraitId,
    trait_impl: &mut UnresolvedTraitImpl,
) -> Vec<(CompilationError, FileId)> {
    // In this Vec methods[i] corresponds to trait.methods[i]. If the impl has no implementation
    // for a particular method, the default implementation will be added at that slot.
    let mut ordered_methods = Vec::new();

    // check whether the trait implementation is in the same crate as either the trait or the type
    let mut errors =
        check_trait_impl_crate_coherence(interner, trait_id, trait_impl, crate_id, def_maps);
    // set of function ids that have a corresponding method in the trait
    let mut func_ids_in_trait = HashSet::new();

    // Temporarily take ownership of the trait's methods so we can iterate over them
    // while also mutating the interner
    let the_trait = interner.get_trait_mut(trait_id);
    let methods = std::mem::take(&mut the_trait.methods);

    for method in &methods {
        let overrides: Vec<_> = trait_impl
            .methods
            .functions
            .iter()
            .filter(|(_, _, f)| f.name() == method.name.0.contents)
            .collect();

        if overrides.is_empty() {
            if let Some(default_impl) = &method.default_impl {
                let func_id = interner.push_empty_fn();
                let module = ModuleId { local_id: trait_impl.module_id, krate: crate_id };
                let location = Location::new(default_impl.def.span, trait_impl.file_id);
                interner.push_function(func_id, &default_impl.def, module, location);
                func_ids_in_trait.insert(func_id);
                ordered_methods.push((
                    method.default_impl_module_id,
                    func_id,
                    *default_impl.clone(),
                ));
            } else {
                let error = DefCollectorErrorKind::TraitMissingMethod {
                    trait_name: interner.get_trait(trait_id).name.clone(),
                    method_name: method.name.clone(),
                    trait_impl_span: trait_impl.object_type.span.expect("type must have a span"),
                };
                errors.push((error.into(), trait_impl.file_id));
            }
        } else {
            for (_, func_id, _) in &overrides {
                func_ids_in_trait.insert(*func_id);
            }

            if overrides.len() > 1 {
                let error = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::TraitAssociatedFunction,
                    first_def: overrides[0].2.name_ident().clone(),
                    second_def: overrides[1].2.name_ident().clone(),
                };
                errors.push((error.into(), trait_impl.file_id));
            }

            ordered_methods.push(overrides[0].clone());
        }
    }

    // Restore the methods that were taken before the for loop
    let the_trait = interner.get_trait_mut(trait_id);
    the_trait.set_methods(methods);

    // Emit MethodNotInTrait error for methods in the impl block that
    // don't have a corresponding method signature defined in the trait
    for (_, func_id, func) in &trait_impl.methods.functions {
        if !func_ids_in_trait.contains(func_id) {
            let error = DefCollectorErrorKind::MethodNotInTrait {
                trait_name: the_trait.name.clone(),
                impl_method: func.name_ident().clone(),
            };
            errors.push((error.into(), trait_impl.file_id));
        }
    }

    trait_impl.methods.functions = ordered_methods;
    trait_impl.methods.trait_id = Some(trait_id);
    errors
}

fn collect_trait_impl(
    context: &mut Context,
    crate_id: CrateId,
    trait_impl: &mut UnresolvedTraitImpl,
) -> Vec<(CompilationError, FileId)> {
    let interner = &mut context.def_interner;
    let def_maps = &mut context.def_maps;
    let mut errors: Vec<(CompilationError, FileId)> = vec![];
    let unresolved_type = trait_impl.object_type.clone();
    let module = ModuleId { local_id: trait_impl.module_id, krate: crate_id };
    trait_impl.trait_id =
        match resolve_trait_by_path(def_maps, module, trait_impl.trait_path.clone()) {
            Ok(trait_id) => Some(trait_id),
            Err(error) => {
                errors.push((error.into(), trait_impl.file_id));
                None
            }
        };

    if let Some(trait_id) = trait_impl.trait_id {
        errors
            .extend(collect_trait_impl_methods(interner, def_maps, crate_id, trait_id, trait_impl));

        let path_resolver = StandardPathResolver::new(module);
        let file = def_maps[&crate_id].file_id(trait_impl.module_id);
        let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
        resolver.add_generics(&trait_impl.generics);
        let typ = resolver.resolve_type(unresolved_type);
        errors.extend(take_errors(trait_impl.file_id, resolver));

        if let Some(struct_type) = get_struct_type(&typ) {
            let struct_type = struct_type.borrow();
            let module = get_module_mut(def_maps, struct_type.id.module_id());

            for (_, method_id, method) in &trait_impl.methods.functions {
                // If this method was already declared, remove it from the module so it cannot
                // be accessed with the `TypeName::method` syntax. We'll check later whether the
                // object types in each method overlap or not. If they do, we issue an error.
                // If not, that is specialization which is allowed.
                if module.declare_function(method.name_ident().clone(), *method_id).is_err() {
                    module.remove_function(method.name_ident());
                }
            }
        }
    }
    errors
}

pub(crate) fn collect_trait_impls(
    context: &mut Context,
    crate_id: CrateId,
    collected_impls: &mut [UnresolvedTraitImpl],
) -> Vec<(CompilationError, FileId)> {
    collected_impls
        .iter_mut()
        .flat_map(|trait_impl| collect_trait_impl(context, crate_id, trait_impl))
        .collect()
}

fn check_trait_impl_crate_coherence(
    interner: &mut NodeInterner,
    trait_id: TraitId,
    trait_impl: &UnresolvedTraitImpl,
    current_crate: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
) -> Vec<(CompilationError, FileId)> {
    let mut errors: Vec<(CompilationError, FileId)> = vec![];

    let module = ModuleId { krate: current_crate, local_id: trait_impl.module_id };
    let file = def_maps[&current_crate].file_id(trait_impl.module_id);
    let path_resolver = StandardPathResolver::new(module);
    let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);

    let object_crate = match resolver.resolve_type(trait_impl.object_type.clone()) {
        Type::Struct(struct_type, _) => struct_type.borrow().id.krate(),
        _ => CrateId::Dummy,
    };

    let the_trait = interner.get_trait(trait_id);
    if current_crate != the_trait.crate_id && current_crate != object_crate {
        let error = DefCollectorErrorKind::TraitImplOrphaned {
            span: trait_impl.object_type.span.expect("object type must have a span"),
        };
        errors.push((error.into(), trait_impl.file_id));
    }

    errors
}

pub(crate) fn resolve_trait_by_path(
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    module: ModuleId,
    path: Path,
) -> Result<TraitId, DefCollectorErrorKind> {
    let path_resolver = StandardPathResolver::new(module);

    match path_resolver.resolve(def_maps, path.clone()) {
        Ok(ModuleDefId::TraitId(trait_id)) => Ok(trait_id),
        Ok(_) => Err(DefCollectorErrorKind::NotATrait { not_a_trait_name: path }),
        Err(_) => Err(DefCollectorErrorKind::TraitNotFound { trait_path: path }),
    }
}
pub(crate) fn resolve_trait_impls(
    context: &mut Context,
    traits: Vec<UnresolvedTraitImpl>,
    crate_id: CrateId,
    errors: &mut Vec<(CompilationError, FileId)>,
) -> Vec<(FileId, FuncId)> {
    let interner = &mut context.def_interner;
    let mut methods = Vec::<(FileId, FuncId)>::new();

    for trait_impl in traits {
        let unresolved_type = trait_impl.object_type;
        let local_mod_id = trait_impl.module_id;
        let module_id = ModuleId { krate: crate_id, local_id: local_mod_id };
        let path_resolver = StandardPathResolver::new(module_id);

        let self_type_span = unresolved_type.span;

        let mut resolver =
            Resolver::new(interner, &path_resolver, &context.def_maps, trait_impl.file_id);
        resolver.add_generics(&trait_impl.generics);

        let trait_generics =
            vecmap(&trait_impl.trait_generics, |generic| resolver.resolve_type(generic.clone()));

        let self_type = resolver.resolve_type(unresolved_type.clone());
        let impl_generics = resolver.get_generics().to_vec();
        let impl_id = interner.next_trait_impl_id();

        let mut impl_methods = functions::resolve_function_set(
            interner,
            crate_id,
            &context.def_maps,
            trait_impl.methods.clone(),
            Some(self_type.clone()),
            Some(impl_id),
            impl_generics.clone(),
            errors,
        );

        let maybe_trait_id = trait_impl.trait_id;
        if let Some(trait_id) = maybe_trait_id {
            for (_, func) in &impl_methods {
                interner.set_function_trait(*func, self_type.clone(), trait_id);
            }
        }

        if matches!(self_type, Type::MutableReference(_)) {
            let span = self_type_span.unwrap_or_else(|| trait_impl.trait_path.span());
            let error = DefCollectorErrorKind::MutableReferenceInTraitImpl { span };
            errors.push((error.into(), trait_impl.file_id));
        }

        let mut new_resolver =
            Resolver::new(interner, &path_resolver, &context.def_maps, trait_impl.file_id);

        new_resolver.set_generics(impl_generics.clone());
        new_resolver.set_self_type(Some(self_type.clone()));

        if let Some(trait_id) = maybe_trait_id {
            check_methods_signatures(
                &mut new_resolver,
                &impl_methods,
                trait_id,
                trait_impl.trait_path.span(),
                trait_impl.trait_generics,
                trait_impl.generics.len(),
                trait_impl.file_id,
                errors,
            );

            let where_clause = trait_impl
                .where_clause
                .into_iter()
                .flat_map(|item| new_resolver.resolve_trait_constraint(item))
                .collect();

            let resolver_errors = new_resolver.take_errors().into_iter();
            errors.extend(resolver_errors.map(|error| (error.into(), trait_impl.file_id)));

            let resolved_trait_impl = Shared::new(TraitImpl {
                ident: trait_impl.trait_path.last_segment().clone(),
                typ: self_type.clone(),
                trait_id,
                trait_generics: trait_generics.clone(),
                file: trait_impl.file_id,
                where_clause,
                methods: vecmap(&impl_methods, |(_, func_id)| *func_id),
            });

            let impl_generics = vecmap(impl_generics, |(_, type_variable, _)| type_variable);

            if let Err((prev_span, prev_file)) = interner.add_trait_implementation(
                self_type.clone(),
                trait_id,
                trait_generics,
                impl_id,
                impl_generics,
                resolved_trait_impl,
            ) {
                let error = DefCollectorErrorKind::OverlappingImpl {
                    typ: self_type.clone(),
                    span: self_type_span.unwrap_or_else(|| trait_impl.trait_path.span()),
                };
                errors.push((error.into(), trait_impl.file_id));

                // The 'previous impl defined here' note must be a separate error currently
                // since it may be in a different file and all errors have the same file id.
                let error = DefCollectorErrorKind::OverlappingImplNote { span: prev_span };
                errors.push((error.into(), prev_file));
            }

            methods.append(&mut impl_methods);
        }
    }

    methods
}
