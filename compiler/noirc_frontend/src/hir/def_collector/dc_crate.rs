use super::dc_mod::collect_defs;
use super::errors::{DefCollectorErrorKind, DuplicateType};
use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};
use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::import::PathResolutionError;
use crate::hir::resolution::resolver::Resolver;
use crate::hir::resolution::{
    import::{resolve_imports, ImportDirective},
    path_resolver::StandardPathResolver,
};
use crate::hir::type_check::{type_check_func, TypeCheckError, TypeChecker};
use crate::hir::Context;
use crate::hir_def::traits::{TraitConstant, TraitFunction, TraitImpl, TraitType};
use crate::node_interner::{
    FuncId, NodeInterner, StmtId, StructId, TraitId, TraitImplKey, TypeAliasId,
};
use crate::parser::UnorderParsedModule;
use crate::{
    ExpressionKind, Generics, Ident, LetStatement, Literal, NoirFunction, NoirStruct, NoirTrait,
    NoirTypeAlias, Shared, StructType, TraitItem, Type, TypeBinding, TypeVariableKind,
    UnresolvedGenerics, UnresolvedType,
};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::Span;
use noirc_errors::{CustomDiagnostic, FileDiagnostic};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::vec;

/// Stores all of the unresolved functions in a particular file/mod
#[derive(Clone)]
pub struct UnresolvedFunctions {
    pub file_id: FileId,
    pub functions: Vec<(LocalModuleId, FuncId, NoirFunction)>,
}

impl UnresolvedFunctions {
    pub fn push_fn(&mut self, mod_id: LocalModuleId, func_id: FuncId, func: NoirFunction) {
        self.functions.push((mod_id, func_id, func));
    }
}

pub struct UnresolvedStruct {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub struct_def: NoirStruct,
}

#[derive(Clone)]
pub struct UnresolvedTrait {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub trait_def: NoirTrait,
}

pub struct UnresolvedTraitImpl {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub the_trait: UnresolvedTrait,
    pub methods: UnresolvedFunctions,
    pub trait_impl_ident: Ident, // for error reporting
}

#[derive(Clone)]
pub struct UnresolvedTypeAlias {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub type_alias_def: NoirTypeAlias,
}

#[derive(Clone)]
pub struct UnresolvedGlobal {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub stmt_id: StmtId,
    pub stmt_def: LetStatement,
}

/// Given a Crate root, collect all definitions in that crate
pub struct DefCollector {
    pub(crate) def_map: CrateDefMap,
    pub(crate) collected_imports: Vec<ImportDirective>,
    pub(crate) collected_functions: Vec<UnresolvedFunctions>,
    pub(crate) collected_types: BTreeMap<StructId, UnresolvedStruct>,
    pub(crate) collected_type_aliases: BTreeMap<TypeAliasId, UnresolvedTypeAlias>,
    pub(crate) collected_traits: BTreeMap<TraitId, UnresolvedTrait>,
    pub(crate) collected_globals: Vec<UnresolvedGlobal>,
    pub(crate) collected_impls: ImplMap,
    pub(crate) collected_traits_impls: TraitImplMap,
}

/// Maps the type and the module id in which the impl is defined to the functions contained in that
/// impl along with the generics declared on the impl itself. This also contains the Span
/// of the object_type of the impl, used to issue an error if the object type fails to resolve.
///
/// Note that because these are keyed by unresolved types, the impl map is one of the few instances
/// of HashMap rather than BTreeMap. For this reason, we should be careful not to iterate over it
/// since it would be non-deterministic.
type ImplMap =
    HashMap<(UnresolvedType, LocalModuleId), Vec<(UnresolvedGenerics, Span, UnresolvedFunctions)>>;

type TraitImplMap = HashMap<(UnresolvedType, LocalModuleId, TraitId), UnresolvedTraitImpl>;

impl DefCollector {
    fn new(def_map: CrateDefMap) -> DefCollector {
        DefCollector {
            def_map,
            collected_imports: vec![],
            collected_functions: vec![],
            collected_types: BTreeMap::new(),
            collected_type_aliases: BTreeMap::new(),
            collected_traits: BTreeMap::new(),
            collected_impls: HashMap::new(),
            collected_globals: vec![],
            collected_traits_impls: HashMap::new(),
        }
    }

    /// Collect all of the definitions in a given crate into a CrateDefMap
    /// Modules which are not a part of the module hierarchy starting with
    /// the root module, will be ignored.
    pub fn collect(
        mut def_map: CrateDefMap,
        context: &mut Context,
        ast: UnorderParsedModule,
        root_file_id: FileId,
        errors: &mut Vec<FileDiagnostic>,
    ) {
        let crate_id = def_map.krate;

        // Recursively resolve the dependencies
        //
        // Dependencies are fetched from the crate graph
        // Then added these to the context of DefMaps once they are resolved
        //
        let crate_graph = &context.crate_graph[crate_id];

        for dep in crate_graph.dependencies.clone() {
            CrateDefMap::collect_defs(dep.crate_id, context, errors);

            let dep_def_root =
                context.def_map(&dep.crate_id).expect("ice: def map was just created").root;
            let module_id = ModuleId { krate: dep.crate_id, local_id: dep_def_root };
            // Add this crate as a dependency by linking it's root module
            def_map.extern_prelude.insert(dep.as_name(), module_id);
        }

        // At this point, all dependencies are resolved and type checked.
        //
        // It is now possible to collect all of the definitions of this crate.
        let crate_root = def_map.root;
        let mut def_collector = DefCollector::new(def_map);

        // Collecting module declarations with ModCollector
        // and lowering the functions
        // i.e. Use a mod collector to collect the nodes at the root module
        // and process them
        collect_defs(&mut def_collector, ast, root_file_id, crate_root, crate_id, context, errors);

        // Add the current crate to the collection of DefMaps
        context.def_maps.insert(crate_id, def_collector.def_map);

        // Resolve unresolved imports collected from the crate
        let (resolved, unresolved_imports) =
            resolve_imports(crate_id, def_collector.collected_imports, &context.def_maps);

        let current_def_map = context.def_maps.get(&crate_id).unwrap();

        errors.extend(vecmap(unresolved_imports, |(error, module_id)| {
            let file_id = current_def_map.file_id(module_id);
            let error = DefCollectorErrorKind::PathResolutionError(error);
            error.into_file_diagnostic(file_id)
        }));

        // Populate module namespaces according to the imports used
        let current_def_map = context.def_maps.get_mut(&crate_id).unwrap();
        for resolved_import in resolved {
            let name = resolved_import.name;
            for ns in resolved_import.resolved_namespace.iter_defs() {
                let result = current_def_map.modules[resolved_import.module_scope.0]
                    .import(name.clone(), ns);

                if let Err((first_def, second_def)) = result {
                    let err = DefCollectorErrorKind::Duplicate {
                        typ: DuplicateType::Import,
                        first_def,
                        second_def,
                    };
                    errors.push(err.into_file_diagnostic(root_file_id));
                }
            }
        }

        // We must first resolve and intern the globals before we can resolve any stmts inside each function.
        // Each function uses its own resolver with a newly created ScopeForest, and must be resolved again to be within a function's scope
        //
        // Additionally, we must resolve integer globals before structs since structs may refer to
        // the values of integer globals as numeric generics.
        let (literal_globals, other_globals) =
            filter_literal_globals(def_collector.collected_globals);

        let mut file_global_ids = resolve_globals(context, literal_globals, crate_id, errors);

        resolve_type_aliases(context, def_collector.collected_type_aliases, crate_id, errors);

        resolve_traits(context, def_collector.collected_traits, crate_id, errors);
        // Must resolve structs before we resolve globals.
        resolve_structs(context, def_collector.collected_types, crate_id, errors);

        // We must wait to resolve non-integer globals until after we resolve structs since structs
        // globals will need to reference the struct type they're initialized to to ensure they are valid.
        let mut more_global_ids = resolve_globals(context, other_globals, crate_id, errors);

        file_global_ids.append(&mut more_global_ids);

        // Before we resolve any function symbols we must go through our impls and
        // re-collect the methods within into their proper module. This cannot be
        // done before resolution since we need to be able to resolve the type of the
        // impl since that determines the module we should collect into.
        collect_impls(context, crate_id, &def_collector.collected_impls, errors);

        collect_trait_impls(context, crate_id, &def_collector.collected_traits_impls, errors);

        // Lower each function in the crate. This is now possible since imports have been resolved
        let file_func_ids = resolve_free_functions(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_functions,
            None,
            errors,
        );

        let file_method_ids = resolve_impls(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_impls,
            errors,
        );

        let file_trait_impls_ids =
            resolve_trait_impls(context, def_collector.collected_traits_impls, crate_id, errors);

        type_check_globals(&mut context.def_interner, file_global_ids, errors);

        // Type check all of the functions in the crate
        type_check_functions(&mut context.def_interner, file_func_ids, errors);
        type_check_functions(&mut context.def_interner, file_method_ids, errors);
        type_check_functions(&mut context.def_interner, file_trait_impls_ids, errors);
    }
}

/// Go through the list of impls and add each function within to the scope
/// of the module defined by its type.
fn collect_impls(
    context: &mut Context,
    crate_id: CrateId,
    collected_impls: &ImplMap,
    errors: &mut Vec<FileDiagnostic>,
) {
    let interner = &mut context.def_interner;
    let def_maps = &mut context.def_maps;

    for ((unresolved_type, module_id), methods) in collected_impls {
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: *module_id, krate: crate_id });

        let file = def_maps[&crate_id].file_id(*module_id);

        for (generics, span, unresolved) in methods {
            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.add_generics(generics);
            let typ = resolver.resolve_type(unresolved_type.clone());

            extend_errors(errors, unresolved.file_id, resolver.take_errors());

            if let Some(struct_type) = get_struct_type(&typ) {
                let struct_type = struct_type.borrow();
                let type_module = struct_type.id.local_module_id();

                // `impl`s are only allowed on types defined within the current crate
                if struct_type.id.krate() != crate_id {
                    let span = *span;
                    let type_name = struct_type.name.to_string();
                    let error = DefCollectorErrorKind::ForeignImpl { span, type_name };
                    errors.push(error.into_file_diagnostic(unresolved.file_id));
                    continue;
                }

                // Grab the module defined by the struct type. Note that impls are a case
                // where the module the methods are added to is not the same as the module
                // they are resolved in.
                let module = &mut def_maps.get_mut(&crate_id).unwrap().modules[type_module.0];

                for (_, method_id, method) in &unresolved.functions {
                    let result = module.declare_function(method.name_ident().clone(), *method_id);

                    if let Err((first_def, second_def)) = result {
                        let err = DefCollectorErrorKind::Duplicate {
                            typ: DuplicateType::Function,
                            first_def,
                            second_def,
                        };
                        errors.push(err.into_file_diagnostic(unresolved.file_id));
                    }
                }
            // Prohibit defining impls for primitive types if we're not in the stdlib
            } else if typ != Type::Error && !crate_id.is_stdlib() {
                let span = *span;
                let error = DefCollectorErrorKind::NonStructTypeInImpl { span };
                errors.push(error.into_file_diagnostic(unresolved.file_id));
            }
        }
    }
}

fn collect_trait_impls(
    context: &mut Context,
    crate_id: CrateId,
    collected_impls: &TraitImplMap,
    errors: &mut Vec<FileDiagnostic>,
) {
    let interner = &mut context.def_interner;
    let def_maps = &mut context.def_maps;

    // TODO: To follow the semantics of Rust, we must allow the impl if either
    //     1. The type is a struct and it's defined in the current crate
    //     2. The trait is defined in the current crate
    for ((unresolved_type, module_id, _), trait_impl) in collected_impls {
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: *module_id, krate: crate_id });

        for (_, func_id, ast) in &trait_impl.methods.functions {
            let file = def_maps[&crate_id].file_id(*module_id);

            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.add_generics(&ast.def.generics);
            let typ = resolver.resolve_type(unresolved_type.clone());

            // Add the method to the struct's namespace
            if let Some(struct_type) = get_struct_type(&typ) {
                extend_errors(errors, trait_impl.file_id, resolver.take_errors());

                let struct_type = struct_type.borrow();
                let type_module = struct_type.id.local_module_id();

                let module = &mut def_maps.get_mut(&crate_id).unwrap().modules[type_module.0];

                let result = module.declare_function(ast.name_ident().clone(), *func_id);

                if let Err((first_def, second_def)) = result {
                    let err = DefCollectorErrorKind::Duplicate {
                        typ: DuplicateType::Function,
                        first_def,
                        second_def,
                    };
                    errors.push(err.into_file_diagnostic(trait_impl.file_id));
                }
            } else {
                let span = trait_impl.trait_impl_ident.span();
                let trait_ident = trait_impl.the_trait.trait_def.name.clone();
                let error = DefCollectorErrorKind::NonStructTraitImpl { trait_ident, span };
                errors.push(error.into_file_diagnostic(trait_impl.file_id));
            }
        }
    }
}

fn get_struct_type(typ: &Type) -> Option<&Shared<StructType>> {
    match typ {
        Type::Struct(definition, _) => Some(definition),
        _ => None,
    }
}

fn extend_errors<Err, Errs>(errors: &mut Vec<FileDiagnostic>, file: fm::FileId, new_errors: Errs)
where
    Errs: IntoIterator<Item = Err>,
    Err: Into<CustomDiagnostic>,
{
    errors.extend(new_errors.into_iter().map(|err| err.into().in_file(file)));
}

/// Separate the globals Vec into two. The first element in the tuple will be the
/// literal globals, except for arrays, and the second will be all other globals.
/// We exclude array literals as they can contain complex types
fn filter_literal_globals(
    globals: Vec<UnresolvedGlobal>,
) -> (Vec<UnresolvedGlobal>, Vec<UnresolvedGlobal>) {
    globals.into_iter().partition(|global| match &global.stmt_def.expression.kind {
        ExpressionKind::Literal(literal) => !matches!(literal, Literal::Array(_)),
        _ => false,
    })
}

fn resolve_globals(
    context: &mut Context,
    globals: Vec<UnresolvedGlobal>,
    crate_id: CrateId,
    errors: &mut Vec<FileDiagnostic>,
) -> Vec<(FileId, StmtId)> {
    vecmap(globals, |global| {
        let module_id = ModuleId { local_id: global.module_id, krate: crate_id };
        let path_resolver = StandardPathResolver::new(module_id);
        let storage_slot = context.next_storage_slot(module_id);

        let mut resolver = Resolver::new(
            &mut context.def_interner,
            &path_resolver,
            &context.def_maps,
            global.file_id,
        );

        let name = global.stmt_def.pattern.name_ident().clone();

        let hir_stmt = resolver.resolve_global_let(global.stmt_def);
        extend_errors(errors, global.file_id, resolver.take_errors());

        context.def_interner.update_global(global.stmt_id, hir_stmt);

        context.def_interner.push_global(global.stmt_id, name, global.module_id, storage_slot);

        (global.file_id, global.stmt_id)
    })
}

fn type_check_globals(
    interner: &mut NodeInterner,
    global_ids: Vec<(FileId, StmtId)>,
    all_errors: &mut Vec<FileDiagnostic>,
) {
    for (file_id, stmt_id) in global_ids {
        let errors = TypeChecker::check_global(&stmt_id, interner);
        extend_errors(all_errors, file_id, errors);
    }
}

/// Create the mappings from TypeId -> StructType
/// so that expressions can access the fields of structs
fn resolve_structs(
    context: &mut Context,
    structs: BTreeMap<StructId, UnresolvedStruct>,
    crate_id: CrateId,
    errors: &mut Vec<FileDiagnostic>,
) {
    // Resolve each field in each struct.
    // Each struct should already be present in the NodeInterner after def collection.
    for (type_id, typ) in structs {
        let (generics, fields) = resolve_struct_fields(context, crate_id, typ, errors);
        context.def_interner.update_struct(type_id, |struct_def| {
            struct_def.set_fields(fields);
            struct_def.generics = generics;
        });
    }
}

fn resolve_trait_types(
    _context: &mut Context,
    _crate_id: CrateId,
    _unresolved_trait: &UnresolvedTrait,
    _errors: &mut [FileDiagnostic],
) -> Vec<TraitType> {
    // TODO
    vec![]
}
fn resolve_trait_constants(
    _context: &mut Context,
    _crate_id: CrateId,
    _unresolved_trait: &UnresolvedTrait,
    _errors: &mut [FileDiagnostic],
) -> Vec<TraitConstant> {
    // TODO
    vec![]
}

fn resolve_trait_methods(
    context: &mut Context,
    trait_id: TraitId,
    crate_id: CrateId,
    unresolved_trait: &UnresolvedTrait,
    errors: &mut Vec<FileDiagnostic>,
) -> Vec<TraitFunction> {
    let interner = &mut context.def_interner;
    let def_maps = &mut context.def_maps;

    let path_resolver = StandardPathResolver::new(ModuleId {
        local_id: unresolved_trait.module_id,
        krate: crate_id,
    });
    let file = def_maps[&crate_id].file_id(unresolved_trait.module_id);

    let mut res = vec![];

    for item in &unresolved_trait.trait_def.items {
        if let TraitItem::Function {
            name,
            generics: _,
            parameters,
            return_type,
            where_clause: _,
            body: _,
        } = item
        {
            let the_trait = interner.get_trait(trait_id);
            let self_type = Type::TypeVariable(
                the_trait.borrow().self_type_typevar.clone(),
                TypeVariableKind::Normal,
            );

            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.set_self_type(Some(self_type));

            let arguments = vecmap(parameters, |param| resolver.resolve_type(param.1.clone()));
            let resolved_return_type = resolver.resolve_type(return_type.get_type().into_owned());

            let name = name.clone();
            // TODO
            let generics: Generics = vec![];
            let span: Span = name.span();
            let f = TraitFunction {
                name,
                generics,
                arguments,
                return_type: resolved_return_type,
                span,
            };
            res.push(f);
            let new_errors = take_errors_filter_self_not_resolved(resolver);
            extend_errors(errors, file, new_errors);
        }
    }
    res
}

fn take_errors_filter_self_not_resolved(resolver: Resolver<'_>) -> Vec<ResolverError> {
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
        .collect()
}

/// Create the mappings from TypeId -> TraitType
/// so that expressions can access the elements of traits
fn resolve_traits(
    context: &mut Context,
    traits: BTreeMap<TraitId, UnresolvedTrait>,
    crate_id: CrateId,
    errors: &mut Vec<FileDiagnostic>,
) {
    for (trait_id, unresolved_trait) in &traits {
        context.def_interner.push_empty_trait(*trait_id, unresolved_trait);
    }
    for (trait_id, unresolved_trait) in traits {
        // Resolve order
        // 1. Trait Types ( Trait constants can have a trait type, therefore types before constants)
        let _ = resolve_trait_types(context, crate_id, &unresolved_trait, errors);
        // 2. Trait Constants ( Trait's methods can use trait types & constants, therefore they should be after)
        let _ = resolve_trait_constants(context, crate_id, &unresolved_trait, errors);
        // 3. Trait Methods
        let methods = resolve_trait_methods(context, trait_id, crate_id, &unresolved_trait, errors);

        context.def_interner.update_trait(trait_id, |trait_def| {
            trait_def.set_methods(methods);
        });
    }
}

fn resolve_struct_fields(
    context: &mut Context,
    krate: CrateId,
    unresolved: UnresolvedStruct,
    all_errors: &mut Vec<FileDiagnostic>,
) -> (Generics, Vec<(Ident, Type)>) {
    let path_resolver =
        StandardPathResolver::new(ModuleId { local_id: unresolved.module_id, krate });

    let file = unresolved.file_id;

    let (generics, fields, errors) =
        Resolver::new(&mut context.def_interner, &path_resolver, &context.def_maps, file)
            .resolve_struct_fields(unresolved.struct_def);

    extend_errors(all_errors, unresolved.file_id, errors);
    (generics, fields)
}

fn resolve_type_aliases(
    context: &mut Context,
    type_aliases: BTreeMap<TypeAliasId, UnresolvedTypeAlias>,
    crate_id: CrateId,
    all_errors: &mut Vec<FileDiagnostic>,
) {
    for (type_id, unresolved_typ) in type_aliases {
        let path_resolver = StandardPathResolver::new(ModuleId {
            local_id: unresolved_typ.module_id,
            krate: crate_id,
        });
        let file = unresolved_typ.file_id;
        let (typ, generics, errors) =
            Resolver::new(&mut context.def_interner, &path_resolver, &context.def_maps, file)
                .resolve_type_aliases(unresolved_typ.type_alias_def);
        extend_errors(all_errors, file, errors);

        context.def_interner.set_type_alias(type_id, typ, generics);
    }
}

fn resolve_impls(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    collected_impls: ImplMap,
    errors: &mut Vec<FileDiagnostic>,
) -> Vec<(FileId, FuncId)> {
    let mut file_method_ids = Vec::new();

    for ((unresolved_type, module_id), methods) in collected_impls {
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: module_id, krate: crate_id });

        let file = def_maps[&crate_id].file_id(module_id);

        for (generics, _, functions) in methods {
            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.add_generics(&generics);
            let generics = resolver.get_generics().to_vec();
            let self_type = resolver.resolve_type(unresolved_type.clone());

            let mut file_func_ids = resolve_function_set(
                interner,
                crate_id,
                def_maps,
                functions,
                Some(self_type.clone()),
                generics,
                errors,
            );
            if self_type != Type::Error {
                for (file_id, method_id) in &file_func_ids {
                    let method_name = interner.function_name(method_id).to_owned();

                    if let Some(first_fn) =
                        interner.add_method(&self_type, method_name.clone(), *method_id)
                    {
                        let error = ResolverError::DuplicateDefinition {
                            name: method_name,
                            first_span: interner.function_ident(&first_fn).span(),
                            second_span: interner.function_ident(method_id).span(),
                        };

                        errors.push(error.into_file_diagnostic(*file_id));
                    }
                }
            }
            file_method_ids.append(&mut file_func_ids);
        }
    }

    file_method_ids
}

fn resolve_trait_impls(
    context: &mut Context,
    traits: TraitImplMap,
    crate_id: CrateId,
    errors: &mut Vec<FileDiagnostic>,
) -> Vec<(FileId, FuncId)> {
    let interner = &mut context.def_interner;
    let mut methods = Vec::<(FileId, FuncId)>::new();

    for ((unresolved_type, _, trait_id), trait_impl) in traits {
        let local_mod_id = trait_impl.module_id;
        let module_id = ModuleId { krate: crate_id, local_id: local_mod_id };
        let path_resolver = StandardPathResolver::new(module_id);

        let self_type = {
            let mut resolver =
                Resolver::new(interner, &path_resolver, &context.def_maps, trait_impl.file_id);
            resolver.resolve_type(unresolved_type.clone())
        };

        let mut impl_methods = resolve_function_set(
            interner,
            crate_id,
            &context.def_maps,
            trait_impl.methods.clone(),
            Some(self_type.clone()),
            vec![], // TODO
            errors,
        );

        let resolved_trait_impl = Shared::new(TraitImpl {
            ident: trait_impl.trait_impl_ident.clone(),
            typ: self_type.clone(),
            trait_id,
            methods: vecmap(&impl_methods, |(_, func_id)| *func_id),
        });

        let mut new_resolver =
            Resolver::new(interner, &path_resolver, &context.def_maps, trait_impl.file_id);
        new_resolver.set_self_type(Some(self_type.clone()));

        check_methods_signatures(&mut new_resolver, &impl_methods, trait_id, errors);

        let trait_definition_ident = &trait_impl.trait_impl_ident;
        let key = TraitImplKey { typ: self_type.clone(), trait_id };

        if let Some(prev_trait_impl_ident) = interner.get_trait_implementation(&key) {
            let err = DefCollectorErrorKind::Duplicate {
                typ: DuplicateType::TraitImplementation,
                first_def: prev_trait_impl_ident.borrow().ident.clone(),
                second_def: trait_definition_ident.clone(),
            };
            errors.push(err.into_file_diagnostic(trait_impl.methods.file_id));
        } else {
            interner.add_trait_implementation(&key, resolved_trait_impl);
        }

        methods.append(&mut impl_methods);
    }

    methods
}

// TODO(vitkov): Move this out of here and into type_check
fn check_methods_signatures(
    resolver: &mut Resolver,
    impl_methods: &Vec<(FileId, FuncId)>,
    trait_id: TraitId,
    errors: &mut Vec<FileDiagnostic>,
) {
    let the_trait_shared = resolver.interner.get_trait(trait_id);
    let the_trait = the_trait_shared.borrow();

    let self_type = resolver.get_self_type().expect("trait impl must have a Self type");

    // Temporarily bind the trait's Self type to self_type so we can type check
    let _ = the_trait.self_type_typevar.borrow_mut().bind_to(self_type.clone(), the_trait.span);

    for (file_id, func_id) in impl_methods {
        let meta = resolver.interner.function_meta(func_id);
        let func_name = resolver.interner.function_name(func_id).to_owned();

        let mut typecheck_errors = Vec::new();

        // `method` is None in the case where the impl block has a method that's not part of the trait.
        // If that's the case, a `MethodNotInTrait` error has already been thrown, and we can ignore
        // the impl method, since there's nothing in the trait to match its signature against.
        if let Some(method) =
            the_trait.methods.iter().find(|method| method.name.0.contents == func_name)
        {
            let function_typ = meta.typ.instantiate(resolver.interner);

            if let Type::Function(params, _, _) = function_typ.0 {
                if method.arguments.len() == params.len() {
                    // Check the parameters of the impl method against the parameters of the trait method
                    for (parameter_index, ((expected, actual), (hir_pattern, _, _))) in
                        method.arguments.iter().zip(&params).zip(&meta.parameters.0).enumerate()
                    {
                        expected.unify(actual, &mut typecheck_errors, || {
                            TypeCheckError::TraitMethodParameterTypeMismatch {
                                method_name: func_name.to_string(),
                                expected_typ: expected.to_string(),
                                actual_typ: actual.to_string(),
                                parameter_span: hir_pattern.span(),
                                parameter_index: parameter_index + 1,
                            }
                        });
                    }
                } else {
                    errors.push(
                        DefCollectorErrorKind::MismatchTraitImplementationNumParameters {
                            actual_num_parameters: meta.parameters.0.len(),
                            expected_num_parameters: method.arguments.len(),
                            trait_name: the_trait.name.to_string(),
                            method_name: func_name.to_string(),
                            span: meta.location.span,
                        }
                        .into_file_diagnostic(*file_id),
                    );
                }
            }

            // Check that impl method return type matches trait return type:
            let resolved_return_type =
                resolver.resolve_type(meta.return_type.get_type().into_owned());

            method.return_type.unify(&resolved_return_type, &mut typecheck_errors, || {
                let ret_type_span =
                    meta.return_type.get_type().span.expect("return type must always have a span");

                TypeCheckError::TypeMismatch {
                    expected_typ: method.return_type.to_string(),
                    expr_typ: meta.return_type().to_string(),
                    expr_span: ret_type_span,
                }
            });

            extend_errors(errors, *file_id, typecheck_errors);
        }
    }

    the_trait.self_type_typevar.borrow_mut().unbind(the_trait.self_type_typevar_id);
}

fn resolve_free_functions(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    collected_functions: Vec<UnresolvedFunctions>,
    self_type: Option<Type>,
    errors: &mut Vec<FileDiagnostic>,
) -> Vec<(FileId, FuncId)> {
    // Lower each function in the crate. This is now possible since imports have been resolved
    collected_functions
        .into_iter()
        .flat_map(|unresolved_functions| {
            resolve_function_set(
                interner,
                crate_id,
                def_maps,
                unresolved_functions,
                self_type.clone(),
                vec![], // no impl generics
                errors,
            )
        })
        .collect()
}

fn resolve_function_set(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    unresolved_functions: UnresolvedFunctions,
    self_type: Option<Type>,
    impl_generics: Vec<(Rc<String>, Shared<TypeBinding>, Span)>,
    errors: &mut Vec<FileDiagnostic>,
) -> Vec<(FileId, FuncId)> {
    let file_id = unresolved_functions.file_id;

    vecmap(unresolved_functions.functions, |(mod_id, func_id, func)| {
        let module_id = ModuleId { krate: crate_id, local_id: mod_id };
        let path_resolver = StandardPathResolver::new(module_id);

        let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file_id);
        // Must use set_generics here to ensure we re-use the same generics from when
        // the impl was originally collected. Otherwise the function will be using different
        // TypeVariables for the same generic, causing it to instantiate incorrectly.
        resolver.set_generics(impl_generics.clone());
        resolver.set_self_type(self_type.clone());

        let (hir_func, func_meta, errs) = resolver.resolve_function(func, func_id);
        interner.push_fn_meta(func_meta, func_id);
        interner.update_fn(func_id, hir_func);
        extend_errors(errors, file_id, errs);
        (file_id, func_id)
    })
}

fn type_check_functions(
    interner: &mut NodeInterner,
    file_func_ids: Vec<(FileId, FuncId)>,
    errors: &mut Vec<FileDiagnostic>,
) {
    for (file, func) in file_func_ids {
        extend_errors(errors, file, type_check_func(interner, func));
    }
}
