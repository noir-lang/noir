use noirc_errors::{CustomDiagnostic, Span};
use thiserror::Error;

use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::CompilationError;
use std::collections::BTreeMap;

use crate::ast::{Ident, ItemVisibility, Path, PathKind};
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleDefId, ModuleId, PerNs};

use super::errors::ResolverError;

#[derive(Debug, Clone)]
pub struct ImportDirective {
    pub module_id: LocalModuleId,
    pub path: Path,
    pub alias: Option<Ident>,
    pub is_prelude: bool,
}

struct NamespaceResolution {
    module_id: ModuleId,
    namespace: PerNs,
    error: Option<PathResolutionError>,
}

type NamespaceResolutionResult = Result<NamespaceResolution, PathResolutionError>;

pub struct PathResolution {
    pub module_def_id: ModuleDefId,

    pub error: Option<PathResolutionError>,
}

pub(crate) type PathResolutionResult = Result<PathResolution, PathResolutionError>;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PathResolutionError {
    #[error("Could not resolve '{0}' in path")]
    Unresolved(Ident),
    #[error("Contract variable '{0}' referenced from outside the contract")]
    ExternalContractUsed(Ident),
    #[error("{0} is private and not visible from the current module")]
    Private(Ident),
}

#[derive(Debug)]
pub struct ResolvedImport {
    // name of the namespace, either last path segment or an alias
    pub name: Ident,
    // The symbol which we have resolved to
    pub resolved_namespace: PerNs,
    // The module which we must add the resolved namespace to
    pub module_scope: LocalModuleId,
    pub is_prelude: bool,
    pub error: Option<PathResolutionError>,
}

impl From<PathResolutionError> for CompilationError {
    fn from(error: PathResolutionError) -> Self {
        Self::ResolverError(ResolverError::PathResolutionError(error))
    }
}

impl<'a> From<&'a PathResolutionError> for CustomDiagnostic {
    fn from(error: &'a PathResolutionError) -> Self {
        match &error {
            PathResolutionError::Unresolved(ident) => {
                CustomDiagnostic::simple_error(error.to_string(), String::new(), ident.span())
            }
            PathResolutionError::ExternalContractUsed(ident) => CustomDiagnostic::simple_error(
                error.to_string(),
                "Contracts may only be referenced from within a contract".to_string(),
                ident.span(),
            ),
            // This will be upgraded to an error in future versions
            PathResolutionError::Private(ident) => CustomDiagnostic::simple_warning(
                error.to_string(),
                format!("{ident} is private"),
                ident.span(),
            ),
        }
    }
}

pub fn resolve_import(
    crate_id: CrateId,
    import_directive: &ImportDirective,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
) -> Result<ResolvedImport, PathResolutionError> {
    let allow_contracts =
        allow_referencing_contracts(def_maps, crate_id, import_directive.module_id);

    let module_scope = import_directive.module_id;
    let NamespaceResolution {
        module_id: resolved_module,
        namespace: resolved_namespace,
        mut error,
    } = resolve_path_to_ns(import_directive, crate_id, crate_id, def_maps, allow_contracts)?;

    let name = resolve_path_name(import_directive);

    let visibility = resolved_namespace
        .values
        .or(resolved_namespace.types)
        .map(|(_, visibility, _)| visibility)
        .expect("Found empty namespace");

    error = error.or_else(|| {
        if can_reference_module_id(
            def_maps,
            crate_id,
            import_directive.module_id,
            resolved_module,
            visibility,
        ) {
            None
        } else {
            Some(PathResolutionError::Private(name.clone()))
        }
    });

    Ok(ResolvedImport {
        name,
        resolved_namespace,
        module_scope,
        is_prelude: import_directive.is_prelude,
        error,
    })
}

fn allow_referencing_contracts(
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    krate: CrateId,
    local_id: LocalModuleId,
) -> bool {
    ModuleId { krate, local_id }.module(def_maps).is_contract
}

fn resolve_path_to_ns(
    import_directive: &ImportDirective,
    crate_id: CrateId,
    importing_crate: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    allow_contracts: bool,
) -> NamespaceResolutionResult {
    let import_path = &import_directive.path.segments;
    let def_map = &def_maps[&crate_id];

    match import_directive.path.kind {
        crate::ast::PathKind::Crate => {
            // Resolve from the root of the crate
            resolve_path_from_crate_root(
                crate_id,
                importing_crate,
                import_path,
                def_maps,
                allow_contracts,
            )
        }
        crate::ast::PathKind::Dep => resolve_external_dep(
            def_map,
            import_directive,
            def_maps,
            allow_contracts,
            importing_crate,
        ),
        crate::ast::PathKind::Plain => {
            // Plain paths are only used to import children modules. It's possible to allow import of external deps, but maybe this distinction is better?
            // In Rust they can also point to external Dependencies, if no children can be found with the specified name
            resolve_name_in_module(
                crate_id,
                importing_crate,
                import_path,
                import_directive.module_id,
                def_maps,
                allow_contracts,
            )
        }
    }
}

fn resolve_path_from_crate_root(
    crate_id: CrateId,
    importing_crate: CrateId,

    import_path: &[Ident],
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    allow_contracts: bool,
) -> NamespaceResolutionResult {
    resolve_name_in_module(
        crate_id,
        importing_crate,
        import_path,
        def_maps[&crate_id].root,
        def_maps,
        allow_contracts,
    )
}

fn resolve_name_in_module(
    krate: CrateId,
    importing_crate: CrateId,
    import_path: &[Ident],
    starting_mod: LocalModuleId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    allow_contracts: bool,
) -> NamespaceResolutionResult {
    let def_map = &def_maps[&krate];
    let mut current_mod_id = ModuleId { krate, local_id: starting_mod };
    let mut current_mod = &def_map.modules[current_mod_id.local_id.0];

    // There is a possibility that the import path is empty
    // In that case, early return
    if import_path.is_empty() {
        return Ok(NamespaceResolution {
            module_id: current_mod_id,
            namespace: PerNs::types(current_mod_id.into()),
            error: None,
        });
    }

    let first_segment = import_path.first().expect("ice: could not fetch first segment");
    let mut current_ns = current_mod.find_name(first_segment);
    if current_ns.is_none() {
        return Err(PathResolutionError::Unresolved(first_segment.clone()));
    }

    let mut warning: Option<PathResolutionError> = None;
    for (last_segment, current_segment) in import_path.iter().zip(import_path.iter().skip(1)) {
        let (typ, visibility) = match current_ns.types {
            None => return Err(PathResolutionError::Unresolved(last_segment.clone())),
            Some((typ, visibility, _)) => (typ, visibility),
        };

        // In the type namespace, only Mod can be used in a path.
        current_mod_id = match typ {
            ModuleDefId::ModuleId(id) => id,
            ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
            // TODO: If impls are ever implemented, types can be used in a path
            ModuleDefId::TypeId(id) => id.module_id(),
            ModuleDefId::TypeAliasId(_) => panic!("type aliases cannot be used in type namespace"),
            ModuleDefId::TraitId(id) => id.0,
            ModuleDefId::GlobalId(_) => panic!("globals cannot be in the type namespace"),
        };

        warning = warning.or_else(|| {
            if can_reference_module_id(
                def_maps,
                importing_crate,
                starting_mod,
                current_mod_id,
                visibility,
            ) {
                None
            } else {
                Some(PathResolutionError::Private(last_segment.clone()))
            }
        });

        current_mod = &def_maps[&current_mod_id.krate].modules[current_mod_id.local_id.0];

        // Check if namespace
        let found_ns = current_mod.find_name(current_segment);

        if found_ns.is_none() {
            return Err(PathResolutionError::Unresolved(current_segment.clone()));
        }

        // Check if it is a contract and we're calling from a non-contract context
        if current_mod.is_contract && !allow_contracts {
            return Err(PathResolutionError::ExternalContractUsed(current_segment.clone()));
        }
        current_ns = found_ns;
    }

    Ok(NamespaceResolution { module_id: current_mod_id, namespace: current_ns, error: warning })
}

fn resolve_path_name(import_directive: &ImportDirective) -> Ident {
    match &import_directive.alias {
        None => import_directive.path.segments.last().unwrap().clone(),
        Some(ident) => ident.clone(),
    }
}

fn resolve_external_dep(
    current_def_map: &CrateDefMap,
    directive: &ImportDirective,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    allow_contracts: bool,
    importing_crate: CrateId,
) -> NamespaceResolutionResult {
    // Use extern_prelude to get the dep
    let path = &directive.path.segments;

    // Fetch the root module from the prelude
    let crate_name = path.first().unwrap();
    let dep_module = current_def_map
        .extern_prelude
        .get(&crate_name.0.contents)
        .ok_or_else(|| PathResolutionError::Unresolved(crate_name.to_owned()))?;

    // Create an import directive for the dependency crate
    let path_without_crate_name = &path[1..]; // XXX: This will panic if the path is of the form `use dep::std` Ideal algorithm will not distinguish between crate and module

    let path = Path {
        segments: path_without_crate_name.to_vec(),
        kind: PathKind::Plain,
        span: Span::default(),
    };
    let dep_directive = ImportDirective {
        module_id: dep_module.local_id,
        path,
        alias: directive.alias.clone(),
        is_prelude: false,
    };

    resolve_path_to_ns(&dep_directive, dep_module.krate, importing_crate, def_maps, allow_contracts)
}

// Issue an error if the given private function is being called from a non-child module, or
// if the given pub(crate) function is being called from another crate
fn can_reference_module_id(
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    importing_crate: CrateId,
    current_module: LocalModuleId,
    target_module: ModuleId,
    visibility: ItemVisibility,
) -> bool {
    // Note that if the target module is in a different crate from the current module then we will either
    // return true as the target module is public or return false as it is private without looking at the `CrateDefMap` in either case.
    let same_crate = target_module.krate == importing_crate;
    let target_crate_def_map = &def_maps[&target_module.krate];

    match visibility {
        ItemVisibility::Public => true,
        ItemVisibility::PublicCrate => same_crate,
        ItemVisibility::Private => {
            same_crate
                && module_descendent_of_target(
                    target_crate_def_map,
                    target_module.local_id,
                    current_module,
                )
        }
    }
}

// Returns true if `current` is a (potentially nested) child module of `target`.
// This is also true if `current == target`.
fn module_descendent_of_target(
    def_map: &CrateDefMap,
    target: LocalModuleId,
    current: LocalModuleId,
) -> bool {
    if current == target {
        return true;
    }

    def_map.modules[current.0]
        .parent
        .map_or(false, |parent| module_descendent_of_target(def_map, target, parent))
}
