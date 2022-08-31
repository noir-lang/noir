use super::dc_mod::collect_defs;
use super::errors::DefCollectorErrorKind;
use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};
use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::resolver::Resolver;
use crate::hir::resolution::{
    import::{resolve_imports, ImportDirective},
    path_resolver::StandardPathResolver,
};
use crate::hir::type_check::type_check;
use crate::hir::type_check::type_check_func;
use crate::hir::Context;
use crate::node_interner::{FuncId, NodeInterner, StmtId, StructId};
use crate::util::vecmap;
use crate::{Ident, NoirFunction, NoirStruct, ParsedModule, Path, Pattern, Statement, Type};
use fm::FileId;
use noirc_errors::CollectedErrors;
use noirc_errors::DiagnosableError;
use std::collections::HashMap;

/// Stores all of the unresolved functions in a particular file/mod
pub struct UnresolvedFunctions {
    pub file_id: FileId,
    pub functions: Vec<(LocalModuleId, FuncId, NoirFunction)>,
}

impl UnresolvedFunctions {
    pub fn push_fn(&mut self, mod_id: LocalModuleId, func_id: FuncId, func: NoirFunction) {
        self.functions.push((mod_id, func_id, func))
    }
}

pub struct UnresolvedStruct {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub struct_def: NoirStruct,
}

pub struct UnresolvedGlobalConst {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub stmt_def: Statement,
}

impl Clone for UnresolvedGlobalConst {
    fn clone(&self) -> Self {
        UnresolvedGlobalConst {
            file_id: self.file_id,
            module_id: self.module_id,
            stmt_def: self.stmt_def.clone(),
        }
    }
}

struct FunctionResolutionInfo {
    file_func_ids: Vec<(FileId, FuncId)>,
    file_const_ids: Vec<(FileId, StmtId)>
}

impl FunctionResolutionInfo {
    pub fn new() -> Self {
        Self {
            file_func_ids: Vec::new(),
            file_const_ids: Vec::new()
        }
    }
}

/// Given a Crate root, collect all definitions in that crate
pub struct DefCollector {
    pub(crate) def_map: CrateDefMap,
    pub(crate) collected_imports: Vec<ImportDirective>,
    pub(crate) collected_functions: Vec<UnresolvedFunctions>,
    pub(crate) collected_types: HashMap<StructId, UnresolvedStruct>,
    pub(crate) collected_consts: Vec<UnresolvedGlobalConst>,
    /// collected impls maps the type name and the module id in which
    /// the impl is defined to the functions contained in that impl
    pub(crate) collected_impls: HashMap<(Path, LocalModuleId), Vec<UnresolvedFunctions>>,
}

impl DefCollector {
    fn new(def_map: CrateDefMap) -> DefCollector {
        DefCollector {
            def_map,
            collected_imports: vec![],
            collected_functions: vec![],
            collected_types: HashMap::new(),
            collected_impls: HashMap::new(),
            collected_consts: vec![],
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
        errors: &mut Vec<CollectedErrors>,
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
                context.def_map(dep.crate_id).expect("ice: def map was just created").root;
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
        let (unresolved, resolved) =
            resolve_imports(crate_id, def_collector.collected_imports, &context.def_maps);

        let current_def_map = context.def_maps.get(&crate_id).unwrap();
        for unresolved_import in unresolved.into_iter() {
            // File if that the import was declared
            let file_id = current_def_map.modules[unresolved_import.module_id.0].origin.file_id();
            let diagnostic = DefCollectorErrorKind::UnresolvedImport { import: unresolved_import }
                .to_diagnostic();
            let err = CollectedErrors { file_id, errors: vec![diagnostic] };
            errors.push(err);
        }

        // Populate module namespaces according to the imports used
        let current_def_map = context.def_maps.get_mut(&crate_id).unwrap();
        for resolved_import in resolved {
            let name = resolved_import.name;
            for ns in resolved_import.resolved_namespace.iter_defs() {
                let result = current_def_map.modules[resolved_import.module_scope.0]
                    .scope
                    .add_item_to_namespace(name.clone(), ns);

                if let Err((first_def, second_def)) = result {
                    let err = DefCollectorErrorKind::DuplicateImport { first_def, second_def };

                    errors.push(CollectedErrors {
                        file_id: root_file_id,
                        errors: vec![err.to_diagnostic()],
                    });
                }
            }
        }

        resolve_structs(context, def_collector.collected_types, crate_id, errors);

        // Collect global constants and check for multiple declarations within a crate
        collect_global_constants(context, def_collector.collected_consts.clone(), crate_id, errors);

        // Before we resolve any function symbols we must go through our impls and
        // re-collect the methods within into their proper module. This cannot be
        // done before resolution since we need to be able to resolve the type of the
        // impl since that determines the module we should collect into.
        collect_impls(context, crate_id, &def_collector.collected_impls, errors);

        // Lower each function in the crate. This is now possible since imports have been resolved
        let funcs_resolution_ids = resolve_functions(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_functions,
            def_collector.collected_consts.clone(),
            None,
            errors,
        );

        let impls_resolution_ids = resolve_impls(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_impls,
            def_collector.collected_consts,
            errors,
        );

        type_check_global_consts(&mut context.def_interner, funcs_resolution_ids.file_const_ids, errors);
        type_check_global_consts(&mut context.def_interner, impls_resolution_ids.file_const_ids, errors);
        // Type check all of the functions in the crate
        type_check_functions(&mut context.def_interner, funcs_resolution_ids.file_func_ids, errors);
        type_check_functions(&mut context.def_interner, impls_resolution_ids.file_func_ids, errors);
    }
}

/// Go through the list of impls and add each function within to the scope
/// of the module defined by its type.
fn collect_impls(
    context: &mut Context,
    crate_id: CrateId,
    collected_impls: &HashMap<(Path, LocalModuleId), Vec<UnresolvedFunctions>>,
    errors: &mut Vec<CollectedErrors>,
) {
    let interner = &mut context.def_interner;
    let def_maps = &mut context.def_maps;

    for ((path, module_id), methods) in collected_impls {
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: *module_id, krate: crate_id });

        let file = def_maps[&crate_id].module_file_id(*module_id);

        for unresolved in methods {
            let resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            let (typ, more_errors) = resolver.lookup_type_for_impl(path.clone());
            if !more_errors.is_empty() {
                errors.push(CollectedErrors {
                    file_id: unresolved.file_id,
                    errors: vecmap(more_errors, |err| err.into_diagnostic(interner)),
                })
            }

            if typ != StructId::dummy_id() {
                // Grab the scope defined by the struct type. Note that impls are a case
                // where the scope the methods are added to is not the same as the scope
                // they are resolved in.
                let type_module = typ.0.local_id;
                let scope = &mut def_maps.get_mut(&crate_id).unwrap().modules[type_module.0].scope;

                // .define_func_def(name, func_id);
                for (_, method_id, method) in &unresolved.functions {
                    let result = scope.define_func_def(method.name_ident().clone(), *method_id);
                    if let Err((first_def, second_def)) = result {
                        let err =
                            DefCollectorErrorKind::DuplicateFunction { first_def, second_def };
                        errors.push(CollectedErrors {
                            file_id: unresolved.file_id,
                            errors: vec![err.to_diagnostic()],
                        });
                    }
                }
            }
        }
    }
}

fn collect_global_constants(
    context: &mut Context,
    global_constants: Vec<UnresolvedGlobalConst>,
    crate_id: CrateId,
    errors: &mut Vec<CollectedErrors>,
) {
    for global_constant in global_constants {
        let path_resolver = StandardPathResolver::new(ModuleId {
            local_id: global_constant.module_id,
            krate: crate_id,
        });

        let mut resolver = Resolver::new(
            &mut context.def_interner,
            &path_resolver,
            &context.def_maps,
            global_constant.file_id,
        );

        let name = match global_constant.stmt_def.clone() {
            Statement::Let(let_stmt) => {
                match let_stmt.pattern {
                    Pattern::Identifier(ident) => ident,
                    _ => panic!("pattern for const statement must be an identifier"), // TODO: change this to use errors
                }
            }
            _ => panic!("global consts must be a let statement"), // TODO: change this to use errors
        };

        // This is a junk stmt id only used in the item scope for finding duplicate global consts
        let stmt_id = resolver.intern_stmt(global_constant.stmt_def, true);

        // NOTE: This is done in resolve_global_consts so that the resolver matches the scopes used by functions or impl functions in the module
        // resolver.push_global_const(name.clone(), stmt_id);

        let current_def_map = context.def_maps.get_mut(&crate_id).unwrap();

        // This simply checks for repeat global constants within the crate
        let result = current_def_map.modules[global_constant.module_id.0]
            .scope
            .define_global_const_def(name, stmt_id);
        if let Err((first_def, second_def)) = result {
            let err = DefCollectorErrorKind::DuplicateGlobalConst { first_def, second_def };
            errors.push(CollectedErrors {
                file_id: global_constant.file_id,
                errors: vec![err.to_diagnostic()],
            });
        }
    }
}

fn resolve_global_constants(
    resolver: &mut Resolver,
    global_constants: Vec<UnresolvedGlobalConst>,
) -> Vec<(FileId, StmtId)> {
    let mut global_const_ids = Vec::new();

    // NOTE: it is still necessary to intern global const statements to check for duplicate global const declarations,
    // repeated variable names inside functions, consts in functions params, and consts specifying array size
    for global_constant in global_constants {
        let name = match global_constant.stmt_def.clone() {
            Statement::Let(let_stmt) => {
                match let_stmt.pattern {
                    Pattern::Identifier(ident) => ident,
                    _ => panic!("pattern for const statement must be an identifier"), // TODO: change this to use errors
                }
            }
            _ => panic!("global consts must be a let statement"), // TODO: change this to use errors
        };
        let stmt_id = resolver.intern_stmt(global_constant.stmt_def, true);
        
        // Check if global const is already inside node interner
        // Otherwise the stmt id generated when collecting the global consts will be overridden in the interner
        if resolver.get_global_const(&name).is_none() {
            resolver.push_global_const(name, stmt_id);
            global_const_ids.push((global_constant.file_id, stmt_id));
        }

    }
    global_const_ids
}

fn type_check_global_consts(
    interner: &mut NodeInterner,
    global_const_ids: Vec<(FileId, StmtId)>,
    errors: &mut Vec<CollectedErrors>,
) {
    for (file_id, stmt_id) in global_const_ids {
        let mut type_check_errs = Vec::new();
        let _stmt_type = type_check(interner, &stmt_id, &mut type_check_errs);
        let type_check_err_diagnostics: Vec<_> = type_check_errs
            .clone()
            .into_iter()
            .map(|error| error.into_diagnostic(interner))
            .collect();

        if !type_check_err_diagnostics.is_empty() {
            let collected_errors = CollectedErrors { file_id, errors: type_check_err_diagnostics };
            errors.push(collected_errors)
        }
    }
}

/// Create the mappings from TypeId -> StructType
/// so that expressions can access the fields of structs
fn resolve_structs(
    context: &mut Context,
    structs: HashMap<StructId, UnresolvedStruct>,
    crate_id: CrateId,
    errors: &mut Vec<CollectedErrors>,
) {
    // We must first go through the struct list once to ensure all IDs are pushed to
    // the def_interner map. This lets structs refer to each other regardless of declaration order
    // without resolve_struct_fields nondeterministically unwrapping a value
    // that isn't in the HashMap.
    for (type_id, typ) in &structs {
        context.def_interner.push_empty_struct(*type_id, typ);
    }

    for (type_id, typ) in structs {
        let fields = resolve_struct_fields(context, crate_id, typ, errors);
        context.def_interner.update_struct(type_id, |struct_def| {
            assert!(struct_def.fields.is_empty());
            struct_def.fields = fields;
        });
    }
}

fn resolve_struct_fields(
    context: &mut Context,
    krate: CrateId,
    unresolved: UnresolvedStruct,
    errors: &mut Vec<CollectedErrors>,
) -> Vec<(Ident, Type)> {
    let path_resolver =
        StandardPathResolver::new(ModuleId { local_id: unresolved.module_id, krate });

    let file = unresolved.file_id;

    let (typ, errs) =
        Resolver::new(&mut context.def_interner, &path_resolver, &context.def_maps, file)
            .resolve_struct_fields(unresolved.struct_def);

    if !errs.is_empty() {
        errors.push(CollectedErrors {
            file_id: unresolved.file_id,
            errors: vecmap(errs, |err| err.into_diagnostic(&context.def_interner)),
        })
    }

    typ
}

fn resolve_impls(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &HashMap<CrateId, CrateDefMap>,
    collected_impls: HashMap<(Path, LocalModuleId), Vec<UnresolvedFunctions>>,
    collected_consts: Vec<UnresolvedGlobalConst>,
    errors: &mut Vec<CollectedErrors>,
) -> FunctionResolutionInfo {
    let mut impls_resolution_ids = FunctionResolutionInfo::new();

    for ((path, module_id), methods) in collected_impls {
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: module_id, krate: crate_id });

        let file = def_maps[&crate_id].module_file_id(module_id);

        let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
        let self_type = resolver.lookup_struct(path);
        let self_type_id = self_type.as_ref().map(|typ| typ.borrow().id);

        let mut funcs_resolution_ids = resolve_functions(
            interner,
            crate_id,
            def_maps,
            methods,
            collected_consts.clone(),
            self_type_id,
            errors,
        );

        if let Some(typ) = self_type {
            for (file_id, method_id) in &funcs_resolution_ids.file_func_ids {
                let method_name = interner.function_name(method_id).to_owned();
                let mut typ = typ.borrow_mut();

                if let Some(first_fn) = typ.methods.insert(method_name, *method_id) {
                    let error = ResolverError::DuplicateDefinition {
                        first_ident: interner.function_meta(&first_fn).name,
                        second_ident: interner.function_meta(method_id).name,
                    };

                    errors.push(CollectedErrors {
                        file_id: *file_id,
                        errors: vec![error.into_diagnostic(interner)],
                    });
                }
            }
        }

        impls_resolution_ids.file_func_ids.append(&mut funcs_resolution_ids.file_func_ids);
        impls_resolution_ids.file_const_ids.append(&mut funcs_resolution_ids.file_const_ids);
    }

    impls_resolution_ids
}

fn resolve_functions(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &HashMap<CrateId, CrateDefMap>,
    collected_functions: Vec<UnresolvedFunctions>,
    collected_consts: Vec<UnresolvedGlobalConst>,
    self_type: Option<StructId>,
    errors: &mut Vec<CollectedErrors>,
) -> FunctionResolutionInfo {
    let mut funcs_resolution_ids = FunctionResolutionInfo::new();
    // let mut file_func_ids = Vec::new();
    // let mut file_const_ids: Vec<(FileId, StmtId)> = Vec::new();
    // Lower each function in the crate. This is now possible since imports have been resolved
    for unresolved_functions in collected_functions {
        let file_id = unresolved_functions.file_id;
        let mut collected_errors = CollectedErrors { file_id, errors: Vec::new() };

        for (mod_id, func_id, func) in unresolved_functions.functions {
            funcs_resolution_ids.file_func_ids.push((file_id, func_id));

            let path_resolver =
                StandardPathResolver::new(ModuleId { local_id: mod_id, krate: crate_id });

            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file_id);
            resolver.set_self_type(self_type);

            let mut resolved_const_ids =
                resolve_global_constants(&mut resolver, collected_consts.clone());
                funcs_resolution_ids.file_const_ids.append(&mut resolved_const_ids);

            let (hir_func, func_meta, errs) = resolver.resolve_function(func);
            interner.push_fn_meta(func_meta, func_id);
            interner.update_fn(func_id, hir_func);
            collected_errors
                .errors
                .extend(errs.into_iter().map(|err| err.into_diagnostic(interner)));
        }
        if !collected_errors.errors.is_empty() {
            errors.push(collected_errors);
        }
    }

    // (file_func_ids, file_const_ids)
    funcs_resolution_ids
}

fn type_check_functions(
    interner: &mut NodeInterner,
    file_func_ids: Vec<(FileId, FuncId)>,
    errors: &mut Vec<CollectedErrors>,
) {
    file_func_ids
        .into_iter()
        .map(|(file_id, func_id)| {
            let errors =
                vecmap(type_check_func(interner, func_id), |error| error.into_diagnostic(interner));

            CollectedErrors { file_id, errors }
        })
        .filter(|collected| !collected.errors.is_empty())
        .for_each(|error| errors.push(error));
}
