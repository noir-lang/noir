use super::import::{
    resolve_import, GenericTypeInPath, GenericTypeInPathKind, ImportDirective, PathResolution,
    PathResolutionKind, PathResolutionResult,
};
use crate::ast::{ItemVisibility, Path};
use crate::node_interner::ReferenceId;
use crate::usage_tracker::UsageTracker;

use std::collections::BTreeMap;

use crate::graph::CrateId;
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleDefId, ModuleId};

pub trait PathResolver {
    /// Resolve the given path returning the resolved ModuleDefId.
    /// If `path_references` is `Some`, a `ReferenceId` for each segment in `path`
    /// will be resolved and pushed (some entries will be None if they don't refer to
    /// a module or type).
    fn resolve(
        &self,
        def_maps: &BTreeMap<CrateId, CrateDefMap>,
        path: Path,
        usage_tracker: &mut UsageTracker,
        path_references: &mut Option<&mut Vec<ReferenceId>>,
    ) -> PathResolutionResult;

    fn local_module_id(&self) -> LocalModuleId;

    fn module_id(&self) -> ModuleId;
}

pub struct StandardPathResolver {
    // Module that we are resolving the path in
    module_id: ModuleId,
    // The module of the self type, if any (for example, the ModuleId of a struct)
    self_type_module_id: Option<ModuleId>,
}

impl StandardPathResolver {
    pub fn new(module_id: ModuleId, self_type_module_id: Option<ModuleId>) -> StandardPathResolver {
        Self { module_id, self_type_module_id }
    }
}

impl PathResolver for StandardPathResolver {
    fn resolve(
        &self,
        def_maps: &BTreeMap<CrateId, CrateDefMap>,
        path: Path,
        usage_tracker: &mut UsageTracker,
        path_references: &mut Option<&mut Vec<ReferenceId>>,
    ) -> PathResolutionResult {
        resolve_path(
            def_maps,
            self.module_id,
            self.self_type_module_id,
            path,
            usage_tracker,
            path_references,
        )
    }

    fn local_module_id(&self) -> LocalModuleId {
        self.module_id.local_id
    }

    fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

/// Resolve the given path to a function or a type.
/// In the case of a conflict, functions are given priority
pub fn resolve_path(
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    module_id: ModuleId,
    self_type_module_id: Option<ModuleId>,
    path: Path,
    usage_tracker: &mut UsageTracker,
    path_references: &mut Option<&mut Vec<ReferenceId>>,
) -> PathResolutionResult {
    // lets package up the path into an ImportDirective and resolve it using that
    let import = ImportDirective {
        visibility: ItemVisibility::Private,
        module_id: module_id.local_id,
        self_type_module_id,
        path,
        alias: None,
        is_prelude: false,
    };
    let resolved_import =
        resolve_import(module_id.krate, &import, def_maps, usage_tracker, path_references)?;

    let namespace = resolved_import.resolved_namespace;
    let module_def_id =
        namespace.values.or(namespace.types).map(|(id, _, _)| id).expect("Found empty namespace");

    let kind = path_resolution_kind_from_module_def_id_and_generic_type_in_path(
        module_def_id,
        resolved_import.generic_type_in_path,
    );

    Ok(PathResolution { kind, errors: resolved_import.errors })
}

fn path_resolution_kind_from_module_def_id_and_generic_type_in_path(
    module_def_id: ModuleDefId,
    generic_type_in_path: Option<GenericTypeInPath>,
) -> PathResolutionKind {
    if let Some(generic_type_in_path) = generic_type_in_path {
        match generic_type_in_path.kind {
            GenericTypeInPathKind::StructId(struct_id) => match module_def_id {
                ModuleDefId::FunctionId(func_id) => PathResolutionKind::StructFunction(
                    struct_id,
                    Some(generic_type_in_path.generics),
                    func_id,
                ),
                _ => path_resolution_kind_from_module_def_if(module_def_id),
            },
            GenericTypeInPathKind::TypeAliasId(type_alias_id) => match module_def_id {
                ModuleDefId::FunctionId(func_id) => PathResolutionKind::TypeAliasFunction(
                    type_alias_id,
                    Some(generic_type_in_path.generics),
                    func_id,
                ),
                _ => path_resolution_kind_from_module_def_if(module_def_id),
            },
            GenericTypeInPathKind::TraitId(trait_id) => match module_def_id {
                ModuleDefId::FunctionId(func_id) => PathResolutionKind::TraitFunction(
                    trait_id,
                    Some(generic_type_in_path.generics),
                    func_id,
                ),
                _ => path_resolution_kind_from_module_def_if(module_def_id),
            },
        }
    } else {
        path_resolution_kind_from_module_def_if(module_def_id)
    }
}

fn path_resolution_kind_from_module_def_if(module_def_id: ModuleDefId) -> PathResolutionKind {
    match module_def_id {
        ModuleDefId::ModuleId(module_id) => PathResolutionKind::Module(module_id),
        ModuleDefId::FunctionId(func_id) => PathResolutionKind::ModuleFunction(func_id),
        ModuleDefId::TypeId(struct_id) => PathResolutionKind::Struct(struct_id, None),
        ModuleDefId::TypeAliasId(type_alias_id) => {
            PathResolutionKind::TypeAlias(type_alias_id, None)
        }
        ModuleDefId::TraitId(trait_id) => PathResolutionKind::Trait(trait_id, None),
        ModuleDefId::GlobalId(global_id) => PathResolutionKind::Global(global_id),
    }
}
