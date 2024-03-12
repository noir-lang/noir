use super::dc_mod::collect_defs;
use super::errors::{DefCollectorErrorKind, DuplicateType};
use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};
use crate::hir::resolution::errors::ResolverError;

use crate::hir::resolution::import::{resolve_import, ImportDirective};
use crate::hir::resolution::{
    collect_impls, collect_trait_impls, path_resolver, resolve_free_functions, resolve_globals,
    resolve_impls, resolve_structs, resolve_trait_by_path, resolve_trait_impls, resolve_traits,
    resolve_type_aliases,
};
use crate::hir::type_check::{
    check_trait_impl_method_matches_declaration, type_check_func, TypeCheckError, TypeChecker,
};
use crate::hir::Context;

use crate::macros_api::{MacroError, MacroProcessor};
use crate::node_interner::{FuncId, GlobalId, NodeInterner, StructId, TraitId, TypeAliasId};

use crate::parser::{ParserError, SortedModule};
use crate::{
    ExpressionKind, Ident, LetStatement, Literal, NoirFunction, NoirStruct, NoirTrait,
    NoirTypeAlias, Path, PathKind, UnresolvedGenerics, UnresolvedTraitConstraint, UnresolvedType,
};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{CustomDiagnostic, Span};
use std::collections::{BTreeMap, HashMap};

use std::vec;

/// Stores all of the unresolved functions in a particular file/mod
#[derive(Clone)]
pub struct UnresolvedFunctions {
    pub file_id: FileId,
    pub functions: Vec<(LocalModuleId, FuncId, NoirFunction)>,
    pub trait_id: Option<TraitId>,
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
    pub method_ids: HashMap<String, FuncId>,
    pub fns_with_default_impl: UnresolvedFunctions,
}

pub struct UnresolvedTraitImpl {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub trait_id: Option<TraitId>,
    pub trait_generics: Vec<UnresolvedType>,
    pub trait_path: Path,
    pub object_type: UnresolvedType,
    pub methods: UnresolvedFunctions,
    pub generics: UnresolvedGenerics,
    pub where_clause: Vec<UnresolvedTraitConstraint>,
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
    pub global_id: GlobalId,
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

/// Maps the type and the module id in which the impl is defined to the functions contained in that
/// impl along with the generics declared on the impl itself. This also contains the Span
/// of the object_type of the impl, used to issue an error if the object type fails to resolve.
///
/// Note that because these are keyed by unresolved types, the impl map is one of the few instances
/// of HashMap rather than BTreeMap. For this reason, we should be careful not to iterate over it
/// since it would be non-deterministic.
pub(crate) type ImplMap =
    HashMap<(UnresolvedType, LocalModuleId), Vec<(UnresolvedGenerics, Span, UnresolvedFunctions)>>;

#[derive(Debug, Clone)]
pub enum CompilationError {
    ParseError(ParserError),
    DefinitionError(DefCollectorErrorKind),
    ResolverError(ResolverError),
    TypeError(TypeCheckError),
}

impl From<CompilationError> for CustomDiagnostic {
    fn from(value: CompilationError) -> Self {
        match value {
            CompilationError::ParseError(error) => error.into(),
            CompilationError::DefinitionError(error) => error.into(),
            CompilationError::ResolverError(error) => error.into(),
            CompilationError::TypeError(error) => error.into(),
        }
    }
}

impl From<MacroError> for CompilationError {
    fn from(value: MacroError) -> Self {
        CompilationError::DefinitionError(DefCollectorErrorKind::MacroError(value))
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
        CompilationError::ResolverError(value)
    }
}
impl From<TypeCheckError> for CompilationError {
    fn from(value: TypeCheckError) -> Self {
        CompilationError::TypeError(value)
    }
}

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
        ast: SortedModule,
        root_file_id: FileId,
        macro_processors: &[&dyn MacroProcessor],
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
            errors.extend(CrateDefMap::collect_defs(dep.crate_id, context, macro_processors));

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

        let submodules = vecmap(def_collector.def_map.modules().iter(), |(index, _)| index);
        // Add the current crate to the collection of DefMaps
        context.def_maps.insert(crate_id, def_collector.def_map);

        // TODO(#4653): generalize this function
        for macro_processor in macro_processors {
            macro_processor
                .process_unresolved_traits_impls(
                    &crate_id,
                    context,
                    &def_collector.collected_traits_impls,
                    &mut def_collector.collected_functions,
                )
                .unwrap_or_else(|(macro_err, file_id)| {
                    errors.push((macro_err.into(), file_id));
                });
        }

        inject_prelude(crate_id, context, crate_root, &mut def_collector.collected_imports);
        for submodule in submodules {
            inject_prelude(
                crate_id,
                context,
                LocalModuleId(submodule),
                &mut def_collector.collected_imports,
            );
        }

        // Resolve unresolved imports collected from the crate, one by one.
        for collected_import in def_collector.collected_imports {
            match resolve_import(crate_id, &collected_import, &context.def_maps) {
                Ok(resolved_import) => {
                    // Populate module namespaces according to the imports used
                    let current_def_map = context.def_maps.get_mut(&crate_id).unwrap();

                    let name = resolved_import.name;
                    for ns in resolved_import.resolved_namespace.iter_defs() {
                        let result = current_def_map.modules[resolved_import.module_scope.0]
                            .import(name.clone(), ns, resolved_import.is_prelude);

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
                Err((error, module_id)) => {
                    let current_def_map = context.def_maps.get(&crate_id).unwrap();
                    let file_id = current_def_map.file_id(module_id);
                    let error = DefCollectorErrorKind::PathResolutionError(error);
                    errors.push((error.into(), file_id));
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

        // We must wait to resolve non-integer globals until after we resolve structs since struct
        // globals will need to reference the struct type they're initialized to to ensure they are valid.
        resolved_globals.extend(resolve_globals(context, other_globals, crate_id));
        errors.extend(resolved_globals.errors);

        // Bind trait impls to their trait. Collect trait functions, that have a
        // default implementation, which hasn't been overridden.
        errors.extend(collect_trait_impls(
            context,
            crate_id,
            &mut def_collector.collected_traits_impls,
        ));

        // Before we resolve any function symbols we must go through our impls and
        // re-collect the methods within into their proper module. This cannot be
        // done before resolution since we need to be able to resolve the type of the
        // impl since that determines the module we should collect into.
        //
        // These are resolved after trait impls so that struct methods are chosen
        // over trait methods if there are name conflicts.
        errors.extend(collect_impls(context, crate_id, &def_collector.collected_impls));

        // Resolve each function in the crate. This is now possible since imports have been resolved
        let mut functions = Vec::new();
        functions.extend(resolve_free_functions(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_functions,
            None,
            &mut errors,
        ));

        functions.extend(resolve_impls(
            &mut context.def_interner,
            crate_id,
            &context.def_maps,
            def_collector.collected_impls,
            &mut errors,
        ));

        let impl_functions = resolve_trait_impls(
            context,
            def_collector.collected_traits_impls,
            crate_id,
            &mut errors,
        );

        for macro_processor in macro_processors {
            macro_processor.process_typed_ast(&crate_id, context).unwrap_or_else(
                |(macro_err, file_id)| {
                    errors.push((macro_err.into(), file_id));
                },
            );
        }

        errors.extend(context.def_interner.check_for_dependency_cycles());

        errors.extend(type_check_globals(&mut context.def_interner, resolved_globals.globals));
        errors.extend(type_check_functions(&mut context.def_interner, functions));
        errors.extend(type_check_trait_impl_signatures(&mut context.def_interner, &impl_functions));
        errors.extend(type_check_functions(&mut context.def_interner, impl_functions));
        errors
    }
}

fn inject_prelude(
    crate_id: CrateId,
    context: &Context,
    crate_root: LocalModuleId,
    collected_imports: &mut Vec<ImportDirective>,
) {
    let segments: Vec<_> = "std::prelude"
        .split("::")
        .map(|segment| crate::Ident::new(segment.into(), Span::default()))
        .collect();

    let path =
        Path { segments: segments.clone(), kind: crate::PathKind::Dep, span: Span::default() };

    if !crate_id.is_stdlib() {
        if let Ok(module_def) = path_resolver::resolve_path(
            &context.def_maps,
            ModuleId { krate: crate_id, local_id: crate_root },
            path,
        ) {
            let module_id = module_def.as_module().expect("std::prelude should be a module");
            let prelude = context.module(module_id).scope().names();

            for path in prelude {
                let mut segments = segments.clone();
                segments.push(Ident::new(path.to_string(), Span::default()));

                collected_imports.insert(
                    0,
                    ImportDirective {
                        module_id: crate_root,
                        path: Path { segments, kind: PathKind::Dep, span: Span::default() },
                        alias: None,
                        is_prelude: true,
                    },
                );
            }
        }
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

fn type_check_globals(
    interner: &mut NodeInterner,
    global_ids: Vec<(FileId, GlobalId)>,
) -> Vec<(CompilationError, fm::FileId)> {
    global_ids
        .into_iter()
        .flat_map(|(file_id, global_id)| {
            TypeChecker::check_global(global_id, interner)
                .iter()
                .cloned()
                .map(|e| (e.into(), file_id))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn type_check_functions(
    interner: &mut NodeInterner,
    file_func_ids: Vec<(FileId, FuncId)>,
) -> Vec<(CompilationError, fm::FileId)> {
    file_func_ids
        .into_iter()
        .flat_map(|(file, func)| {
            type_check_func(interner, func)
                .into_iter()
                .map(|e| (e.into(), file))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn type_check_trait_impl_signatures(
    interner: &mut NodeInterner,
    file_func_ids: &[(FileId, FuncId)],
) -> Vec<(CompilationError, fm::FileId)> {
    file_func_ids
        .iter()
        .flat_map(|(file, func)| {
            check_trait_impl_method_matches_declaration(interner, *func)
                .into_iter()
                .map(|e| (e.into(), *file))
                .collect::<Vec<_>>()
        })
        .collect()
}
