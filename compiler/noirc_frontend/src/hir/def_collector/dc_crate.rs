use super::dc_mod::collect_defs;
use super::errors::{DefCollectorErrorKind, DuplicateType};
use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleDefId, ModuleId};
use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::import::PathResolutionError;
use crate::hir::resolution::path_resolver::PathResolver;
use crate::hir::resolution::resolver::Resolver;
use crate::hir::resolution::{
    import::{resolve_imports, ImportDirective},
    path_resolver::StandardPathResolver,
};
use crate::hir::type_check::{type_check_func, TypeCheckError, TypeChecker};
use crate::hir::Context;
use crate::hir_def::traits::{Trait, TraitConstant, TraitFunction, TraitImpl, TraitType};
use crate::node_interner::{
    FuncId, NodeInterner, StmtId, StructId, TraitId, TraitImplKey, TypeAliasId,
};

use crate::parser::ParserError;

use crate::{
    ExpressionKind, Generics, Ident, LetStatement, Literal, NoirFunction, NoirStruct, NoirTrait,
    NoirTypeAlias, ParsedModule, Path, Shared, StructType, TraitItem, Type, TypeBinding,
    TypeVariableKind, UnresolvedGenerics, UnresolvedType,
};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{CustomDiagnostic, Span};
use std::collections::{BTreeMap, HashMap, HashSet};
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

    pub fn resolve_trait_bounds_trait_ids(
        &mut self,
        def_maps: &BTreeMap<CrateId, CrateDefMap>,
        crate_id: CrateId,
    ) -> Vec<DefCollectorErrorKind> {
        let mut errors = Vec::new();

        for (local_id, _, func) in &mut self.functions {
            let module = ModuleId { krate: crate_id, local_id: *local_id };

            for bound in &mut func.def.where_clause {
                match resolve_trait_by_path(def_maps, module, bound.trait_bound.trait_path.clone())
                {
                    Ok(trait_id) => {
                        bound.trait_bound.trait_id = Some(trait_id);
                    }
                    Err(err) => {
                        errors.push(err);
                    }
                }
            }
        }

        errors
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
    pub crate_id: CrateId,
    pub trait_def: NoirTrait,
    pub fns_with_default_impl: UnresolvedFunctions,
}

pub struct UnresolvedTraitImpl {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub trait_id: Option<TraitId>,
    pub trait_path: Path,
    pub object_type: UnresolvedType,
    pub methods: UnresolvedFunctions,
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
    pub(crate) collected_traits_impls: Vec<UnresolvedTraitImpl>,
}

pub enum CompilationError {
    ParseError(ParserError),
    DefinitionError(DefCollectorErrorKind),
    ResolveError(ResolverError),
    TypeError(TypeCheckError),
}

impl From<CompilationError> for CustomDiagnostic {
    fn from(value: CompilationError) -> Self {
        match value {
            CompilationError::ParseError(error) => error.into(),
            CompilationError::DefinitionError(error) => error.into(),
            CompilationError::ResolveError(error) => error.into(),
            CompilationError::TypeError(error) => error.into(),
        }
    }
}

impl From<ParserError> for CompilationError {
    fn from(value: ParserError) -> Self {
        CompilationError::ParseError(value)
    }
}

impl From<DefCollectorErrorKind> for CompilationError {
    fn from(value: DefCollectorErrorKind) -> Self {
        CompilationError::DefinitionError(value)
    }
}

impl From<ResolverError> for CompilationError {
    fn from(value: ResolverError) -> Self {
        CompilationError::ResolveError(value)
    }
}
impl From<TypeCheckError> for CompilationError {
    fn from(value: TypeCheckError) -> Self {
        CompilationError::TypeError(value)
    }
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
            collected_traits_impls: vec![],
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
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        let crate_id = def_map.krate;

        // Recursively resolve the dependencies
        //
        // Dependencies are fetched from the crate graph
        // Then added these to the context of DefMaps once they are resolved
        //
        let crate_graph = &context.crate_graph[crate_id];

        for dep in crate_graph.dependencies.clone() {
            errors.extend(CrateDefMap::collect_defs(dep.crate_id, context));

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
        errors.extend(collect_defs(
            &mut def_collector,
            ast,
            root_file_id,
            crate_root,
            crate_id,
            context,
        ));

        // Add the current crate to the collection of DefMaps
        context.def_maps.insert(crate_id, def_collector.def_map);

        // Resolve unresolved imports collected from the crate
        let (resolved, unresolved_imports) =
            resolve_imports(crate_id, def_collector.collected_imports, &context.def_maps);

        {
            let current_def_map = context.def_maps.get(&crate_id).unwrap();
            errors.extend(vecmap(unresolved_imports, |(error, module_id)| {
                let file_id = current_def_map.file_id(module_id);
                let error = DefCollectorErrorKind::PathResolutionError(error);
                (error.into(), file_id)
            }));
        };

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
                    errors.push((err.into(), root_file_id));
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

        let mut resolved_globals = resolve_globals(context, literal_globals, crate_id);

        errors.extend(resolve_type_aliases(
            context,
            def_collector.collected_type_aliases,
            crate_id,
        ));

        errors.extend(resolve_traits(context, def_collector.collected_traits, crate_id));
        // Must resolve structs before we resolve globals.
        errors.extend(resolve_structs(context, def_collector.collected_types, crate_id));

        // We must wait to resolve non-integer globals until after we resolve structs since structs
        // globals will need to reference the struct type they're initialized to to ensure they are valid.
        resolved_globals.extend(resolve_globals(context, other_globals, crate_id));

        // Before we resolve any function symbols we must go through our impls and
        // re-collect the methods within into their proper module. This cannot be
        // done before resolution since we need to be able to resolve the type of the
        // impl since that determines the module we should collect into.
        errors.extend(collect_impls(context, crate_id, &def_collector.collected_impls));

        // Bind trait impls to their trait. Collect trait functions, that have a
        // default implementation, which hasn't been overriden.
        errors.extend(collect_trait_impls(
            context,
            crate_id,
            &mut def_collector.collected_traits_impls,
        ));

        // Lower each function in the crate. This is now possible since imports have been resolved
        let file_func_ids = resolve_free_functions(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_functions,
            None,
            &mut errors,
        );

        let file_method_ids = resolve_impls(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_impls,
            &mut errors,
        );
        // resolve_trait_impls can fill different type of errors, therefore we pass errors by mut ref
        let file_trait_impls_ids = resolve_trait_impls(
            context,
            def_collector.collected_traits_impls,
            crate_id,
            &mut errors,
        );

        errors.extend(resolved_globals.errors);
        errors.extend(type_check_globals(&mut context.def_interner, resolved_globals.globals));

        // Type check all of the functions in the crate
        errors.extend(type_check_functions(&mut context.def_interner, file_func_ids));
        errors.extend(type_check_functions(&mut context.def_interner, file_method_ids));
        errors.extend(type_check_functions(&mut context.def_interner, file_trait_impls_ids));
        errors
    }
}

/// Go through the list of impls and add each function within to the scope
/// of the module defined by its type.
fn collect_impls(
    context: &mut Context,
    crate_id: CrateId,
    collected_impls: &ImplMap,
) -> Vec<(CompilationError, FileId)> {
    let interner = &mut context.def_interner;
    let def_maps = &mut context.def_maps;
    let mut errors: Vec<(CompilationError, FileId)> = vec![];

    for ((unresolved_type, module_id), methods) in collected_impls {
        let path_resolver =
            StandardPathResolver::new(ModuleId { local_id: *module_id, krate: crate_id });

        let file = def_maps[&crate_id].file_id(*module_id);

        for (generics, span, unresolved) in methods {
            let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
            resolver.add_generics(generics);
            let typ = resolver.resolve_type(unresolved_type.clone());

            errors.extend(take_errors(unresolved.file_id, resolver));

            if let Some(struct_type) = get_struct_type(&typ) {
                let struct_type = struct_type.borrow();
                let type_module = struct_type.id.local_module_id();

                // `impl`s are only allowed on types defined within the current crate
                if struct_type.id.krate() != crate_id {
                    let span = *span;
                    let type_name = struct_type.name.to_string();
                    let error = DefCollectorErrorKind::ForeignImpl { span, type_name };
                    errors.push((error.into(), unresolved.file_id));
                    continue;
                }

                // Grab the module defined by the struct type. Note that impls are a case
                // where the module the methods are added to is not the same as the module
                // they are resolved in.
                let module = &mut def_maps.get_mut(&crate_id).unwrap().modules[type_module.0];

                for (_, method_id, method) in &unresolved.functions {
                    let result = module.declare_function(method.name_ident().clone(), *method_id);

                    if let Err((first_def, second_def)) = result {
                        let error = DefCollectorErrorKind::Duplicate {
                            typ: DuplicateType::Function,
                            first_def,
                            second_def,
                        };
                        errors.push((error.into(), unresolved.file_id));
                    }
                }
            // Prohibit defining impls for primitive types if we're not in the stdlib
            } else if typ != Type::Error && !crate_id.is_stdlib() {
                let span = *span;
                let error = DefCollectorErrorKind::NonStructTypeInImpl { span };
                errors.push((error.into(), unresolved.file_id));
            }
        }
    }
    errors
}

fn collect_trait_impl_methods(
    interner: &mut NodeInterner,
    def_maps: &mut BTreeMap<CrateId, CrateDefMap>,
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
                interner.push_function(func_id, &default_impl.def, module);
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
                    typ: DuplicateType::Function,
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
    errors
}

fn add_method_to_struct_namespace(
    current_def_map: &mut CrateDefMap,
    struct_type: &Shared<StructType>,
    func_id: FuncId,
    name_ident: &Ident,
) -> Result<(), DefCollectorErrorKind> {
    let struct_type = struct_type.borrow();
    let type_module = struct_type.id.local_module_id();
    let module = &mut current_def_map.modules[type_module.0];
    module.declare_function(name_ident.clone(), func_id).map_err(|(first_def, second_def)| {
        DefCollectorErrorKind::Duplicate { typ: DuplicateType::Function, first_def, second_def }
    })
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
            errors.extend(collect_trait_impl_methods(
                interner, def_maps, crate_id, trait_id, trait_impl,
            ));
            for (_, func_id, ast) in &trait_impl.methods.functions {
                let file = def_maps[&crate_id].file_id(trait_impl.module_id);

                let path_resolver = StandardPathResolver::new(module);
                let mut resolver = Resolver::new(interner, &path_resolver, def_maps, file);
                resolver.add_generics(&ast.def.generics);
                let typ = resolver.resolve_type(unresolved_type.clone());

                if let Some(struct_type) = get_struct_type(&typ) {
                    errors.extend(take_errors(trait_impl.file_id, resolver));
                    let current_def_map = def_maps.get_mut(&crate_id).unwrap();
                    match add_method_to_struct_namespace(
                        current_def_map,
                        struct_type,
                        *func_id,
                        ast.name_ident(),
                    ) {
                        Ok(()) => {},
                        Err(err) => {
                            errors.push((err.into(), trait_impl.file_id));
                        }
                    }
                } else {
                    let error = DefCollectorErrorKind::NonStructTraitImpl {
                        trait_path: trait_impl.trait_path.clone(),
                        span: trait_impl.trait_path.span(),
                    };
                    errors.push((error.into(), trait_impl.file_id));
                }
            }
        }
    errors
}

fn collect_trait_impls(
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

fn resolve_trait_by_path(
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

fn get_struct_type(typ: &Type) -> Option<&Shared<StructType>> {
    match typ {
        Type::Struct(definition, _) => Some(definition),
        _ => None,
    }
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

pub struct ResolvedGlobals {
    pub globals: Vec<(FileId, StmtId)>,
    pub errors: Vec<(CompilationError, FileId)>,
}

impl ResolvedGlobals {
    pub fn extend(&mut self, oth: Self) {
        self.globals.extend(oth.globals);
        self.errors.extend(oth.errors);
    }
}

fn resolve_globals(
    context: &mut Context,
    globals: Vec<UnresolvedGlobal>,
    crate_id: CrateId,
) -> ResolvedGlobals {
    let mut errors: Vec<(CompilationError, FileId)> = vec![];
    let globals = vecmap(globals, |global| {
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
        errors.extend(take_errors(global.file_id, resolver));

        context.def_interner.update_global(global.stmt_id, hir_stmt);

        context.def_interner.push_global(global.stmt_id, name, global.module_id, storage_slot);

        (global.file_id, global.stmt_id)
    });
    ResolvedGlobals { globals, errors }
}

fn type_check_globals(
    interner: &mut NodeInterner,
    global_ids: Vec<(FileId, StmtId)>,
) -> Vec<(CompilationError, fm::FileId)> {
    global_ids
        .iter()
        .flat_map(|(file_id, stmt_id)| {
            TypeChecker::check_global(stmt_id, interner)
                .iter()
                .cloned()
                .map(|e| (e.into(), *file_id))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn type_check_functions(
    interner: &mut NodeInterner,
    file_func_ids: Vec<(FileId, FuncId)>,
) -> Vec<(CompilationError, fm::FileId)> {
    file_func_ids
        .iter()
        .flat_map(|(file, func)| {
            type_check_func(interner, *func)
                .iter()
                .cloned()
                .map(|e| (e.into(), *file))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Create the mappings from TypeId -> StructType
/// so that expressions can access the fields of structs
fn resolve_structs(
    context: &mut Context,
    structs: BTreeMap<StructId, UnresolvedStruct>,
    crate_id: CrateId,
) -> Vec<(CompilationError, FileId)> {
    let mut errors: Vec<(CompilationError, FileId)> = vec![];
    // Resolve each field in each struct.
    // Each struct should already be present in the NodeInterner after def collection.
    for (type_id, typ) in structs {
        let file_id = typ.file_id;
        let (generics, fields, resolver_errors) = resolve_struct_fields(context, crate_id, typ);
        errors.extend(vecmap(resolver_errors, |err| (err.into(), file_id)));
        context.def_interner.update_struct(type_id, |struct_def| {
            struct_def.set_fields(fields);
            struct_def.generics = generics;
        });
    }
    errors
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
            generics: _,
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
            resolver.set_self_type(Some(self_type));

            let arguments = vecmap(parameters, |param| resolver.resolve_type(param.1.clone()));
            let resolved_return_type = resolver.resolve_type(return_type.get_type().into_owned());

            let name = name.clone();
            // TODO
            let generics: Generics = vec![];
            let span: Span = name.span();
            let default_impl_list: Vec<_> = unresolved_trait
                .fns_with_default_impl
                .functions
                .iter()
                .filter(|(_, _, q)| q.name() == name.0.contents)
                .collect();
            let default_impl = if !default_impl_list.is_empty() {
                if default_impl_list.len() > 1 {
                    // TODO(nickysn): Add check for method duplicates in the trait and emit proper error messages. This is planned in a future PR.
                    panic!("Too many functions with the same name!");
                }
                Some(Box::new(default_impl_list[0].2.clone()))
            } else {
                None
            };

            let f = TraitFunction {
                name,
                generics,
                arguments,
                return_type: resolved_return_type,
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

fn take_errors_filter_self_not_resolved(
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

fn take_errors(file_id: FileId, resolver: Resolver<'_>) -> Vec<(CompilationError, FileId)> {
    resolver.take_errors().iter().cloned().map(|e| (e.into(), file_id)).collect()
}

/// Create the mappings from TypeId -> TraitType
/// so that expressions can access the elements of traits
fn resolve_traits(
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

fn resolve_struct_fields(
    context: &mut Context,
    krate: CrateId,
    unresolved: UnresolvedStruct,
) -> (Generics, Vec<(Ident, Type)>, Vec<ResolverError>) {
    let path_resolver =
        StandardPathResolver::new(ModuleId { local_id: unresolved.module_id, krate });
    let file_id = unresolved.file_id;
    let (generics, fields, errors) =
        Resolver::new(&mut context.def_interner, &path_resolver, &context.def_maps, file_id)
            .resolve_struct_fields(unresolved.struct_def);
    (generics, fields, errors)
}

fn resolve_type_aliases(
    context: &mut Context,
    type_aliases: BTreeMap<TypeAliasId, UnresolvedTypeAlias>,
    crate_id: CrateId,
) -> Vec<(CompilationError, FileId)> {
    let mut errors: Vec<(CompilationError, FileId)> = vec![];
    for (type_id, unresolved_typ) in type_aliases {
        let path_resolver = StandardPathResolver::new(ModuleId {
            local_id: unresolved_typ.module_id,
            krate: crate_id,
        });
        let file = unresolved_typ.file_id;
        let (typ, generics, resolver_errors) =
            Resolver::new(&mut context.def_interner, &path_resolver, &context.def_maps, file)
                .resolve_type_aliases(unresolved_typ.type_alias_def);
        errors.extend(resolver_errors.iter().cloned().map(|e| (e.into(), file)));
        context.def_interner.set_type_alias(type_id, typ, generics);
    }
    errors
}

fn resolve_impls(
    interner: &mut NodeInterner,
    crate_id: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    collected_impls: ImplMap,
    errors: &mut Vec<(CompilationError, FileId)>,
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
                        errors.push((error.into(), *file_id));
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
        let trait_definition_ident = trait_impl.trait_path.last_segment();

        let self_type = {
            let mut resolver =
                Resolver::new(interner, &path_resolver, &context.def_maps, trait_impl.file_id);
            resolver.resolve_type(unresolved_type.clone())
        };

        let maybe_trait_id = trait_impl.trait_id;

        let mut impl_methods = resolve_function_set(
            interner,
            crate_id,
            &context.def_maps,
            trait_impl.methods.clone(),
            Some(self_type.clone()),
            vec![], // TODO
            errors,
        );

        let mut new_resolver =
            Resolver::new(interner, &path_resolver, &context.def_maps, trait_impl.file_id);
        new_resolver.set_self_type(Some(self_type.clone()));

        if let Some(trait_id) = maybe_trait_id {
            check_methods_signatures(&mut new_resolver, &impl_methods, trait_id, errors);

            let key = TraitImplKey { typ: self_type.clone(), trait_id };
            if let Some(prev_trait_impl_ident) = interner.get_trait_implementation(&key) {
                let err = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::TraitImplementation,
                    first_def: prev_trait_impl_ident.borrow().ident.clone(),
                    second_def: trait_definition_ident.clone(),
                };
                errors.push((err.into(), trait_impl.methods.file_id));
            } else {
                let resolved_trait_impl = Shared::new(TraitImpl {
                    ident: trait_impl.trait_path.last_segment().clone(),
                    typ: self_type.clone(),
                    trait_id,
                    methods: vecmap(&impl_methods, |(_, func_id)| *func_id),
                });
                interner.add_trait_implementation(&key, resolved_trait_impl.clone());
            }

            methods.append(&mut impl_methods);
        }
    }

    methods
}

// TODO(vitkov): Move this out of here and into type_check
fn check_methods_signatures(
    resolver: &mut Resolver,
    impl_methods: &Vec<(FileId, FuncId)>,
    trait_id: TraitId,
    errors: &mut Vec<(CompilationError, FileId)>,
) {
    let the_trait = resolver.interner.get_trait(trait_id);

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
                    errors.push((
                        DefCollectorErrorKind::MismatchTraitImplementationNumParameters {
                            actual_num_parameters: meta.parameters.0.len(),
                            expected_num_parameters: method.arguments.len(),
                            trait_name: the_trait.name.to_string(),
                            method_name: func_name.to_string(),
                            span: meta.location.span,
                        }
                        .into(),
                        *file_id,
                    ));
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

            errors.extend(typecheck_errors.iter().cloned().map(|e| (e.into(), *file_id)));
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
    errors: &mut Vec<(CompilationError, FileId)>,
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
    mut unresolved_functions: UnresolvedFunctions,
    self_type: Option<Type>,
    impl_generics: Vec<(Rc<String>, Shared<TypeBinding>, Span)>,
    errors: &mut Vec<(CompilationError, FileId)>,
) -> Vec<(FileId, FuncId)> {
    let file_id = unresolved_functions.file_id;

    let where_clause_errors =
        unresolved_functions.resolve_trait_bounds_trait_ids(def_maps, crate_id);
    errors.extend(where_clause_errors.iter().cloned().map(|e| (e.into(), file_id)));

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
        errors.extend(errs.iter().cloned().map(|e| (e.into(), file_id)));
        (file_id, func_id)
    })
}
