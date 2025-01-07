use super::dc_mod::collect_defs;
use super::errors::{DefCollectorErrorKind, DuplicateType};
use crate::elaborator::Elaborator;
use crate::graph::CrateId;
use crate::hir::comptime::InterpreterError;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleId};
use crate::hir::resolution::errors::ResolverError;
use crate::hir::type_check::TypeCheckError;
use crate::locations::ReferencesTracker;
use crate::token::SecondaryAttribute;
use crate::usage_tracker::UnusedItem;
use crate::{Generics, Type};

use crate::hir::resolution::import::{resolve_import, ImportDirective};
use crate::hir::Context;

use crate::ast::Expression;
use crate::node_interner::{
    FuncId, GlobalId, ModuleAttributes, NodeInterner, ReferenceId, StructId, TraitId, TraitImplId,
    TypeAliasId,
};

use crate::ast::{
    ExpressionKind, GenericTypeArgs, Ident, ItemVisibility, LetStatement, Literal, NoirFunction,
    NoirStruct, NoirTrait, NoirTypeAlias, Path, PathKind, PathSegment, UnresolvedGenerics,
    UnresolvedTraitConstraint, UnresolvedType, UnsupportedNumericGenericType,
};

use crate::parser::{ParserError, SortedModule};
use noirc_errors::{CustomDiagnostic, Location, Span};

use fm::FileId;
use iter_extended::vecmap;
use rustc_hash::FxHashMap as HashMap;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::PathBuf;
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
    pub trait_generics: GenericTypeArgs,
    pub trait_path: Path,
    pub object_type: UnresolvedType,
    pub methods: UnresolvedFunctions,
    pub generics: UnresolvedGenerics,
    pub where_clause: Vec<UnresolvedTraitConstraint>,

    pub associated_types: Vec<(Ident, UnresolvedType)>,
    pub associated_constants: Vec<(Ident, UnresolvedType, Expression)>,

    // Every field after this line is filled in later in the elaborator
    pub trait_id: Option<TraitId>,
    pub impl_id: Option<TraitImplId>,
    pub resolved_object_type: Option<Type>,
    pub resolved_generics: Generics,

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
    pub visibility: ItemVisibility,
}

pub struct ModuleAttribute {
    // The file in which the module is defined
    pub file_id: FileId,
    // The module this attribute is attached to
    pub module_id: LocalModuleId,

    // The file where the attribute exists (it could be the same as `file_id`
    // or a different one if it is an outer attribute in the parent of the module it applies to)
    pub attribute_file_id: FileId,

    // The module where the attribute is defined (similar to `attribute_file_id`,
    // it could be different than `module_id` for inner attributes)
    pub attribute_module_id: LocalModuleId,
    pub attribute: SecondaryAttribute,
    pub is_inner: bool,
}

/// Given a Crate root, collect all definitions in that crate
pub struct DefCollector {
    pub(crate) def_map: CrateDefMap,
    pub(crate) imports: Vec<ImportDirective>,
    pub(crate) items: CollectedItems,
}

#[derive(Default)]
pub struct CollectedItems {
    pub functions: Vec<UnresolvedFunctions>,
    pub(crate) types: BTreeMap<StructId, UnresolvedStruct>,
    pub(crate) type_aliases: BTreeMap<TypeAliasId, UnresolvedTypeAlias>,
    pub(crate) traits: BTreeMap<TraitId, UnresolvedTrait>,
    pub globals: Vec<UnresolvedGlobal>,
    pub(crate) impls: ImplMap,
    pub(crate) trait_impls: Vec<UnresolvedTraitImpl>,
    pub(crate) module_attributes: Vec<ModuleAttribute>,
}

impl CollectedItems {
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
            && self.types.is_empty()
            && self.type_aliases.is_empty()
            && self.traits.is_empty()
            && self.globals.is_empty()
            && self.impls.is_empty()
            && self.trait_impls.is_empty()
    }
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
    DebugComptimeScopeNotFound(Vec<PathBuf>),
}

impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::ParseError(error) => write!(f, "{}", error),
            CompilationError::DefinitionError(error) => write!(f, "{}", error),
            CompilationError::ResolverError(error) => write!(f, "{}", error),
            CompilationError::TypeError(error) => write!(f, "{}", error),
            CompilationError::InterpreterError(error) => write!(f, "{:?}", error),
            CompilationError::DebugComptimeScopeNotFound(error) => write!(f, "{:?}", error),
        }
    }
}

impl<'a> From<&'a CompilationError> for CustomDiagnostic {
    fn from(value: &'a CompilationError) -> Self {
        match value {
            CompilationError::ParseError(error) => error.into(),
            CompilationError::DefinitionError(error) => error.into(),
            CompilationError::ResolverError(error) => error.into(),
            CompilationError::TypeError(error) => error.into(),
            CompilationError::InterpreterError(error) => error.into(),
            CompilationError::DebugComptimeScopeNotFound(error) => {
                let msg = "multiple files found matching --debug-comptime path".into();
                let secondary = error.iter().fold(String::new(), |mut output, path| {
                    let _ = writeln!(output, "    {}", path.display());
                    output
                });
                // NOTE: this span is empty as it is not expected to be displayed
                let dummy_span = Span::default();
                CustomDiagnostic::simple_error(msg, secondary, dummy_span)
            }
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
        CompilationError::ResolverError(value)
    }
}

impl From<TypeCheckError> for CompilationError {
    fn from(value: TypeCheckError) -> Self {
        CompilationError::TypeError(value)
    }
}

impl From<UnsupportedNumericGenericType> for CompilationError {
    fn from(value: UnsupportedNumericGenericType) -> Self {
        Self::ResolverError(value.into())
    }
}

impl DefCollector {
    pub fn new(def_map: CrateDefMap) -> DefCollector {
        DefCollector {
            def_map,
            imports: vec![],
            items: CollectedItems {
                functions: vec![],
                types: BTreeMap::new(),
                type_aliases: BTreeMap::new(),
                traits: BTreeMap::new(),
                impls: HashMap::default(),
                globals: vec![],
                trait_impls: vec![],
                module_attributes: vec![],
            },
        }
    }

    /// Collect all of the definitions in a given crate into a CrateDefMap
    /// Modules which are not a part of the module hierarchy starting with
    /// the root module, will be ignored.
    #[allow(clippy::too_many_arguments)]
    pub fn collect_crate_and_dependencies(
        mut def_map: CrateDefMap,
        context: &mut Context,
        ast: SortedModule,
        root_file_id: FileId,
        debug_comptime_in_file: Option<&str>,
        pedantic_solving: bool,
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
            errors.extend(CrateDefMap::collect_defs(
                dep.crate_id,
                context,
                debug_comptime_in_file,
                pedantic_solving,
            ));

            let dep_def_map =
                context.def_map(&dep.crate_id).expect("ice: def map was just created");

            let dep_def_root = dep_def_map.root;
            let module_id = ModuleId { krate: dep.crate_id, local_id: dep_def_root };
            // Add this crate as a dependency by linking it's root module
            def_map.extern_prelude.insert(dep.as_name(), module_id);

            let location = dep_def_map[dep_def_root].location;
            let attributes = ModuleAttributes {
                name: dep.as_name(),
                location,
                parent: None,
                visibility: ItemVisibility::Public,
            };
            context.def_interner.add_module_attributes(module_id, attributes);
        }

        // At this point, all dependencies are resolved and type checked.
        //
        // It is now possible to collect all of the definitions of this crate.
        let crate_root = def_map.root;
        let mut def_collector = DefCollector::new(def_map);

        let module_id = ModuleId { krate: crate_id, local_id: crate_root };
        context
            .def_interner
            .set_doc_comments(ReferenceId::Module(module_id), ast.inner_doc_comments.clone());

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

        inject_prelude(crate_id, context, crate_root, &mut def_collector.imports);
        for submodule in submodules {
            inject_prelude(crate_id, context, LocalModuleId(submodule), &mut def_collector.imports);
        }

        // Resolve unresolved imports collected from the crate, one by one.
        for collected_import in std::mem::take(&mut def_collector.imports) {
            let local_module_id = collected_import.module_id;
            let module_id = ModuleId { krate: crate_id, local_id: local_module_id };
            let current_def_map = context.def_maps.get(&crate_id).unwrap();
            let file_id = current_def_map.file_id(local_module_id);

            let resolved_import = resolve_import(
                collected_import.path.clone(),
                module_id,
                &context.def_maps,
                &mut context.usage_tracker,
                Some(ReferencesTracker::new(&mut context.def_interner, file_id)),
            );

            match resolved_import {
                Ok(resolved_import) => {
                    let current_def_map = context.def_maps.get_mut(&crate_id).unwrap();
                    let file_id = current_def_map.file_id(local_module_id);

                    let has_path_resolution_error = !resolved_import.errors.is_empty();
                    for error in resolved_import.errors {
                        errors.push((
                            DefCollectorErrorKind::PathResolutionError(error).into(),
                            file_id,
                        ));
                    }

                    // Populate module namespaces according to the imports used
                    let name = collected_import.name();
                    let visibility = collected_import.visibility;
                    let is_prelude = collected_import.is_prelude;
                    for (module_def_id, item_visibility, _) in
                        resolved_import.namespace.iter_items()
                    {
                        if item_visibility < visibility {
                            errors.push((
                                DefCollectorErrorKind::CannotReexportItemWithLessVisibility {
                                    item_name: name.clone(),
                                    desired_visibility: visibility,
                                }
                                .into(),
                                file_id,
                            ));
                        }
                        let visibility = visibility.min(item_visibility);

                        let result = current_def_map.modules[local_module_id.0].import(
                            name.clone(),
                            visibility,
                            module_def_id,
                            is_prelude,
                        );

                        // If we error on path resolution don't also say it's unused (in case it ends up being unused)
                        if !has_path_resolution_error {
                            let defining_module =
                                ModuleId { krate: crate_id, local_id: local_module_id };

                            context.usage_tracker.add_unused_item(
                                defining_module,
                                name.clone(),
                                UnusedItem::Import,
                                visibility,
                            );

                            if visibility != ItemVisibility::Private {
                                context.def_interner.register_name_for_auto_import(
                                    name.to_string(),
                                    module_def_id,
                                    visibility,
                                    Some(defining_module),
                                );
                            }
                        }

                        if let Some(ref alias) = collected_import.alias {
                            add_import_reference(
                                module_def_id,
                                alias,
                                &mut context.def_interner,
                                file_id,
                            );
                        }

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

        let debug_comptime_in_file = debug_comptime_in_file.and_then(|debug_comptime_in_file| {
            let file = context.file_manager.find_by_path_suffix(debug_comptime_in_file);
            file.unwrap_or_else(|error| {
                errors.push((CompilationError::DebugComptimeScopeNotFound(error), root_file_id));
                None
            })
        });

        let mut more_errors = Elaborator::elaborate(
            context,
            crate_id,
            def_collector.items,
            debug_comptime_in_file,
            pedantic_solving,
        );

        errors.append(&mut more_errors);

        Self::check_unused_items(context, crate_id, &mut errors);

        errors
    }

    fn check_unused_items(
        context: &Context,
        crate_id: CrateId,
        errors: &mut Vec<(CompilationError, FileId)>,
    ) {
        let unused_imports = context.usage_tracker.unused_items().iter();
        let unused_imports = unused_imports.filter(|(module_id, _)| module_id.krate == crate_id);

        errors.extend(unused_imports.flat_map(|(module_id, usage_tracker)| {
            let module = &context.def_maps[&crate_id].modules()[module_id.local_id.0];
            usage_tracker.iter().map(|(ident, unused_item)| {
                let ident = ident.clone();
                let error = CompilationError::ResolverError(ResolverError::UnusedItem {
                    ident,
                    item: *unused_item,
                });
                (error, module.location.file)
            })
        }));
    }
}

fn add_import_reference(
    def_id: crate::hir::def_map::ModuleDefId,
    name: &Ident,
    interner: &mut NodeInterner,
    file_id: FileId,
) {
    if name.span() == Span::empty(0) {
        // We ignore empty spans at 0 location, this must be Stdlib
        return;
    }

    let location = Location::new(name.span(), file_id);
    interner.add_module_def_id_reference(def_id, location, false);
}

fn inject_prelude(
    crate_id: CrateId,
    context: &mut Context,
    crate_root: LocalModuleId,
    collected_imports: &mut Vec<ImportDirective>,
) {
    if !crate_id.is_stdlib() {
        let segments: Vec<_> = "std::prelude"
            .split("::")
            .map(|segment| {
                crate::ast::PathSegment::from(crate::ast::Ident::new(
                    segment.into(),
                    Span::default(),
                ))
            })
            .collect();

        let path = Path {
            segments: segments.clone(),
            kind: crate::ast::PathKind::Plain,
            span: Span::default(),
        };

        if let Ok(resolved_import) = resolve_import(
            path,
            ModuleId { krate: crate_id, local_id: crate_root },
            &context.def_maps,
            &mut context.usage_tracker,
            None, // references tracker
        ) {
            assert!(resolved_import.errors.is_empty(), "Tried to add private item to prelude");

            let (module_def_id, _, _) =
                resolved_import.namespace.types.expect("couldn't resolve std::prelude");
            let module_id = module_def_id.as_module().expect("std::prelude should be a module");
            let prelude = context.module(module_id).scope().names();

            for path in prelude {
                let mut segments = segments.clone();
                segments.push(PathSegment::from(Ident::new(path.to_string(), Span::default())));

                collected_imports.insert(
                    0,
                    ImportDirective {
                        visibility: ItemVisibility::Private,
                        module_id: crate_root,
                        path: Path { segments, kind: PathKind::Plain, span: Span::default() },
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
