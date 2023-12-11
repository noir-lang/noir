use std::collections::{BTreeMap, HashSet};

use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span};

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
    hir_def::traits::{Trait, TraitConstant, TraitFunction, TraitImpl, TraitType},
    node_interner::{FuncId, NodeInterner, TraitId},
    Path, Shared, TraitItem, Type, TypeBinding, TypeVariableKind,
};

use super::{
    errors::ResolverError,
    functions, get_module_mut, get_struct_type,
    import::PathResolutionError,
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
    let mut res: Vec<(CompilationError, FileId)> = vec![];
    for (trait_id, unresolved_trait) in traits {
        // Resolve order
        // 1. Trait Types ( Trait constants can have a trait type, therefore types before constants)
        let _ = resolve_trait_types(context, crate_id, &unresolved_trait);
        // 2. Trait Constants ( Trait's methods can use trait types & constants, therefore they should be after)
        let _ = resolve_trait_constants(context, crate_id, &unresolved_trait);
        // 3. Trait Methods
        let (methods, errors) =
            resolve_trait_methods(context, trait_id, crate_id, &unresolved_trait);
        res.extend(errors);
        context.def_interner.update_trait(trait_id, |trait_def| {
            trait_def.set_methods(methods);
        });
    }
    res
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
) -> (Vec<TraitFunction>, Vec<(CompilationError, FileId)>) {
    let interner = &mut context.def_interner;
    let def_maps = &mut context.def_maps;

    let path_resolver = StandardPathResolver::new(ModuleId {
        local_id: unresolved_trait.module_id,
        krate: crate_id,
    });
    let file = def_maps[&crate_id].file_id(unresolved_trait.module_id);

    let mut res = vec![];
    let mut resolver_errors = vec![];
    for item in &unresolved_trait.trait_def.items {
        if let TraitItem::Function {
            name,
            generics,
            parameters,
            return_type,
            where_clause: _,
            body: _,
        } = item
        {
            let the_trait = interner.get_trait(trait_id);
            let self_type =
                Type::TypeVariable(the_trait.self_type_typevar.clone(), TypeVariableKind::Normal);

            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.add_generics(generics);
            resolver.set_self_type(Some(self_type));

            let arguments = vecmap(parameters, |param| resolver.resolve_type(param.1.clone()));
            let return_type = resolver.resolve_type(return_type.get_type().into_owned());

            let mut generics = vecmap(resolver.get_generics(), |(_, type_var, _)| match &*type_var
                .borrow()
            {
                TypeBinding::Unbound(id) => (*id, type_var.clone()),
                TypeBinding::Bound(binding) => unreachable!("Trait generic was bound to {binding}"),
            });

            // Ensure the trait is generic over the Self type as well
            generics.push((the_trait.self_type_typevar_id, the_trait.self_type_typevar));

            let name = name.clone();
            let span: Span = name.span();
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
            let typ = Type::Forall(generics, Box::new(function_type));

            let f = TraitFunction {
                name,
                typ,
                span,
                default_impl,
                default_impl_file_id: unresolved_trait.file_id,
                default_impl_module_id: unresolved_trait.module_id,
            };
            res.push(f);
            resolver_errors.extend(take_errors_filter_self_not_resolved(file, resolver));
        }
    }
    (res, resolver_errors)
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

    let the_trait = interner.get_trait(trait_id);

    // check whether the trait implementation is in the same crate as either the trait or the type
    let mut errors =
        check_trait_impl_crate_coherence(interner, &the_trait, trait_impl, crate_id, def_maps);
    // set of function ids that have a corresponding method in the trait
    let mut func_ids_in_trait = HashSet::new();

    for method in &the_trait.methods {
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
                    trait_name: the_trait.name.clone(),
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
    the_trait: &Trait,
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
        let self_type = resolver.resolve_type(unresolved_type.clone());
        let generics = resolver.get_generics().to_vec();

        let impl_id = interner.next_trait_impl_id();

        let mut impl_methods = functions::resolve_function_set(
            interner,
            crate_id,
            &context.def_maps,
            trait_impl.methods.clone(),
            Some(self_type.clone()),
            Some(impl_id),
            generics.clone(),
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

        new_resolver.set_generics(generics);
        new_resolver.set_self_type(Some(self_type.clone()));

        if let Some(trait_id) = maybe_trait_id {
            check_methods_signatures(
                &mut new_resolver,
                &impl_methods,
                trait_id,
                trait_impl.generics.len(),
                errors,
            );

            let where_clause = trait_impl
                .where_clause
                .into_iter()
                .flat_map(|item| new_resolver.resolve_trait_constraint(item))
                .collect();

            let resolved_trait_impl = Shared::new(TraitImpl {
                ident: trait_impl.trait_path.last_segment().clone(),
                typ: self_type.clone(),
                trait_id,
                file: trait_impl.file_id,
                where_clause,
                methods: vecmap(&impl_methods, |(_, func_id)| *func_id),
            });

            if let Err((prev_span, prev_file)) = interner.add_trait_implementation(
                self_type.clone(),
                trait_id,
                impl_id,
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

pub(crate) fn take_errors_filter_self_not_resolved(
    file_id: FileId,
    resolver: Resolver<'_>,
) -> Vec<(CompilationError, FileId)> {
    resolver
        .take_errors()
        .iter()
        .filter(|resolution_error| match resolution_error {
            ResolverError::PathResolutionError(PathResolutionError::Unresolved(ident)) => {
                &ident.0.contents != "Self"
            }
            _ => true,
        })
        .cloned()
        .map(|resolution_error| (resolution_error.into(), file_id))
        .collect()
}
