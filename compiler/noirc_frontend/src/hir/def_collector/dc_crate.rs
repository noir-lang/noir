use super::dc_mod::collect_defs;
use super::errors::{DefCollectorErrorKind, DuplicateType};
use crate::elaborator::Elaborator;
use crate::graph::CrateId;
use crate::hir::comptime::InterpreterError;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};
use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::path_resolver;
use crate::hir::type_check::TypeCheckError;
use crate::{Type, TypeVariable};

use crate::hir::resolution::import::{resolve_import, ImportDirective, PathResolution};
use crate::hir::Context;

use crate::macros_api::{MacroError, MacroProcessor};
use crate::node_interner::{FuncId, GlobalId, StructId, TraitId, TraitImplId, TypeAliasId};

use crate::ast::{
    ExpressionKind, Ident, LetStatement, Literal, NoirFunction, NoirStruct, NoirTrait,
    NoirTypeAlias, Path, PathKind, UnresolvedGenerics, UnresolvedTraitConstraint, UnresolvedType,
};
use crate::parser::{ParserError, SortedModule};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{CustomDiagnostic, Span};
use std::collections::{BTreeMap, HashMap};

use std::rc::Rc;
use std::vec;

/// Stores all of the unresolved functions in a particular file/mod
#[derive(Clone)]
pub struct UnresolvedFunctions {
    pub file_id: FileId,
    pub functions: Vec<(LocalModuleId, FuncId, NoirFunction)>,
    pub trait_id: Option<TraitId>,

    // The object type this set of functions was declared on, if there is one.
    pub self_type: Option<Type>,
}

impl UnresolvedFunctions {
    pub fn push_fn(&mut self, mod_id: LocalModuleId, func_id: FuncId, func: NoirFunction) {
        self.functions.push((mod_id, func_id, func));
    }

    pub fn function_ids(&self) -> Vec<FuncId> {
        vecmap(&self.functions, |(_, id, _)| *id)
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
    pub trait_generics: Vec<UnresolvedType>,
    pub trait_path: Path,
    pub object_type: UnresolvedType,
    pub methods: UnresolvedFunctions,
    pub generics: UnresolvedGenerics,
    pub where_clause: Vec<UnresolvedTraitConstraint>,

    // Every field after this line is filled in later in the elaborator
    pub trait_id: Option<TraitId>,
    pub impl_id: Option<TraitImplId>,
    pub resolved_object_type: Option<Type>,
    pub resolved_generics: Vec<(Rc<String>, TypeVariable, Span)>,

    // The resolved generic on the trait itself. E.g. it is the `<C, D>` in
    // `impl<A, B> Foo<C, D> for Bar<E, F> { ... }`
    pub resolved_trait_generics: Vec<Type>,
}

#[derive(Clone)]
pub struct UnresolvedTypeAlias {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub type_alias_def: NoirTypeAlias,
}

#[derive(Debug, Clone)]
pub struct UnresolvedGlobal {
    pub file_id: FileId,
    pub module_id: LocalModuleId,
    pub global_id: GlobalId,
    pub stmt_def: LetStatement,
}

/// Given a Crate root, collect all definitions in that crate
pub struct DefCollector {
    pub(crate) def_map: CrateDefMap,
    pub(crate) imports: Vec<ImportDirective>,
    pub(crate) items: CollectedItems,
}

pub struct CollectedItems {
    pub(crate) functions: Vec<UnresolvedFunctions>,
    pub(crate) types: BTreeMap<StructId, UnresolvedStruct>,
    pub(crate) type_aliases: BTreeMap<TypeAliasId, UnresolvedTypeAlias>,
    pub(crate) traits: BTreeMap<TraitId, UnresolvedTrait>,
    pub(crate) globals: Vec<UnresolvedGlobal>,
    pub(crate) impls: ImplMap,
    pub(crate) trait_impls: Vec<UnresolvedTraitImpl>,
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
    InterpreterError(InterpreterError),
}

impl<'a> From<&'a CompilationError> for CustomDiagnostic {
    fn from(value: &'a CompilationError) -> Self {
        match value {
            CompilationError::ParseError(error) => error.into(),
            CompilationError::DefinitionError(error) => error.into(),
            CompilationError::ResolverError(error) => error.into(),
            CompilationError::TypeError(error) => error.into(),
            CompilationError::InterpreterError(error) => error.into(),
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
            imports: vec![],
            items: CollectedItems {
                functions: vec![],
                types: BTreeMap::new(),
                type_aliases: BTreeMap::new(),
                traits: BTreeMap::new(),
                impls: HashMap::new(),
                globals: vec![],
                trait_impls: vec![],
            },
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
            macro_processors,
        ));

        let submodules = vecmap(def_collector.def_map.modules().iter(), |(index, _)| index);
        // Add the current crate to the collection of DefMaps
        context.def_maps.insert(crate_id, def_collector.def_map);

        inject_prelude(crate_id, context, crate_root, &mut def_collector.imports);
        for submodule in submodules {
            inject_prelude(crate_id, context, LocalModuleId(submodule), &mut def_collector.imports);
        }

        // Resolve unresolved imports collected from the crate, one by one.
        for collected_import in std::mem::take(&mut def_collector.imports) {
            match resolve_import(crate_id, &collected_import, &context.def_maps) {
                Ok(resolved_import) => {
                    if let Some(error) = resolved_import.error {
                        errors.push((
                            DefCollectorErrorKind::PathResolutionError(error).into(),
                            root_file_id,
                        ));
                    }

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
                Err(error) => {
                    let current_def_map = context.def_maps.get(&crate_id).unwrap();
                    let file_id = current_def_map.file_id(collected_import.module_id);
                    let error = DefCollectorErrorKind::PathResolutionError(error);
                    errors.push((error.into(), file_id));
                }
            }
        }

        let mut more_errors = Elaborator::elaborate(context, crate_id, def_collector.items);
        errors.append(&mut more_errors);
        errors
    }
}

fn inject_prelude(
    crate_id: CrateId,
    context: &Context,
    crate_root: LocalModuleId,
    collected_imports: &mut Vec<ImportDirective>,
) {
    if !crate_id.is_stdlib() {
        let segments: Vec<_> = "std::prelude"
            .split("::")
            .map(|segment| crate::ast::Ident::new(segment.into(), Span::default()))
            .collect();

        let path = Path {
            segments: segments.clone(),
            kind: crate::ast::PathKind::Dep,
            span: Span::default(),
        };

        if let Ok(PathResolution { module_def_id, error }) = path_resolver::resolve_path(
            &context.def_maps,
            ModuleId { krate: crate_id, local_id: crate_root },
            path,
        ) {
            assert!(error.is_none(), "Tried to add private item to prelude");
            let module_id = module_def_id.as_module().expect("std::prelude should be a module");
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
pub fn filter_literal_globals(
    globals: Vec<UnresolvedGlobal>,
) -> (Vec<UnresolvedGlobal>, Vec<UnresolvedGlobal>) {
    globals.into_iter().partition(|global| match &global.stmt_def.expression.kind {
        ExpressionKind::Literal(literal) => !matches!(literal, Literal::Array(_)),
        _ => false,
    })
}
