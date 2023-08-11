use super::dc_mod::collect_defs;
use super::errors::{DefCollectorErrorKind, DuplicateType};
use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};
use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::resolver::Resolver;
use crate::hir::resolution::{
    import::{resolve_imports, ImportDirective},
    path_resolver::StandardPathResolver,
};
use crate::hir::type_check::{type_check_func, TypeChecker};
use crate::hir::Context;
use crate::node_interner::{FuncId, NodeInterner, StmtId, StructId, TypeAliasId};
use crate::{
    ExpressionKind, Generics, Ident, LetStatement, Literal, NoirFunction, NoirStruct,
    NoirTypeAlias, ParsedModule, Shared, StructType, Type, TypeBinding, UnresolvedGenerics,
    UnresolvedType,
};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::Span;
use noirc_errors::{CustomDiagnostic, FileDiagnostic};
use std::collections::HashMap;
use std::rc::Rc;

/// Stores all of the unresolved functions in a particular file/mod
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
    pub(crate) collected_types: HashMap<StructId, UnresolvedStruct>,
    pub(crate) collected_type_aliases: HashMap<TypeAliasId, UnresolvedTypeAlias>,
    pub(crate) collected_globals: Vec<UnresolvedGlobal>,
    pub(crate) collected_impls: ImplMap,
}

/// Maps the type and the module id in which the impl is defined to the functions contained in that
/// impl along with the generics declared on the impl itself. This also contains the Span
/// of the object_type of the impl, used to issue an error if the object type fails to resolve.
type ImplMap =
    HashMap<(UnresolvedType, LocalModuleId), Vec<(UnresolvedGenerics, Span, UnresolvedFunctions)>>;

impl DefCollector {
    fn new(def_map: CrateDefMap) -> DefCollector {
        DefCollector {
            def_map,
            collected_imports: vec![],
            collected_functions: vec![],
            collected_types: HashMap::new(),
            collected_type_aliases: HashMap::new(),
            collected_impls: HashMap::new(),
            collected_globals: vec![],
        }
    }

    /// Collect all of the definitions in a given crate into a CrateDefMap
    /// Modules which are not a part of the module hierarchy starting with
    /// the root module, will be ignored.
    pub fn collect(
        mut def_map: CrateDefMap,
        context: &mut Context,
        ast: ParsedModule,
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
            let file_id = current_def_map.modules[module_id.0].origin.file_id();
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

        type_check_globals(&mut context.def_interner, file_global_ids, errors);

        // Type check all of the functions in the crate
        type_check_functions(&mut context.def_interner, file_func_ids, errors);
        type_check_functions(&mut context.def_interner, file_method_ids, errors);
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

        let file = def_maps[&crate_id].module_file_id(*module_id);

        for (generics, span, unresolved) in methods {
            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.add_generics(generics);
            let typ = resolver.resolve_type(unresolved_type.clone());

            extend_errors(errors, unresolved.file_id, resolver.take_errors());

            if let Some(struct_type) = get_struct_type(&typ) {
                let struct_type = struct_type.borrow();
                let type_module = struct_type.id.0.local_id;

                // `impl`s are only allowed on types defined within the current crate
                if struct_type.id.0.krate != crate_id {
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
    structs: HashMap<StructId, UnresolvedStruct>,
    crate_id: CrateId,
    errors: &mut Vec<FileDiagnostic>,
) {
    // We must first go through the struct list once to ensure all IDs are pushed to
    // the def_interner map. This lets structs refer to each other regardless of declaration order
    // without resolve_struct_fields non-deterministically unwrapping a value
    // that isn't in the HashMap.
    for (type_id, typ) in &structs {
        let type_index = type_id.0.local_id.0;
        let module_path = context.def_map(&crate_id).unwrap().get_module_path_with_separator(
            type_index,
            Some(typ.module_id),
            "::",
        );
        let crate_name = context
            .crate_graph
            .get_crate(crate_id)
            .and_then(|c| c.name.to_owned())
            .map_or(String::new(), |n| n.to_string());
        let full_path = format!("{crate_name}::{module_path}");
        context.def_interner.push_empty_struct(*type_id, full_path, typ);
    }

    for (type_id, typ) in structs {
        let (generics, fields) = resolve_struct_fields(context, crate_id, typ, errors);
        context.def_interner.update_struct(type_id, |struct_def| {
            struct_def.set_fields(fields);
            struct_def.generics = generics;
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
    type_aliases: HashMap<TypeAliasId, UnresolvedTypeAlias>,
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
    def_maps: &HashMap<CrateId, CrateDefMap>,
    collected_impls: ImplMap,
    errors: &mut Vec<FileDiagnostic>,
) -> Vec<(FileId, FuncId)> {
    let mut file_method_ids = Vec::new();

    for ((unresolved_type, module_id), methods) in collected_impls {
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: module_id, krate: crate_id });

        let file = def_maps[&crate_id].module_file_id(module_id);

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

fn resolve_free_functions(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &HashMap<CrateId, CrateDefMap>,
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
    def_maps: &HashMap<CrateId, CrateDefMap>,
    unresolved_functions: UnresolvedFunctions,
    self_type: Option<Type>,
    impl_generics: Vec<(Rc<String>, Shared<TypeBinding>, Span)>,
    errors: &mut Vec<FileDiagnostic>,
) -> Vec<(FileId, FuncId)> {
    let file_id = unresolved_functions.file_id;

    vecmap(unresolved_functions.functions, |(mod_id, func_id, func)| {
        let module_id = ModuleId { krate: crate_id, local_id: mod_id };
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: mod_id, krate: crate_id });

        let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file_id);
        // Must use set_generics here to ensure we re-use the same generics from when
        // the impl was originally collected. Otherwise the function will be using different
        // TypeVariables for the same generic, causing it to instantiate incorrectly.
        resolver.set_generics(impl_generics.clone());
        resolver.set_self_type(self_type.clone());

        let (hir_func, func_meta, errs) = resolver.resolve_function(func, func_id, module_id);
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
