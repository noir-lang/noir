use iter_extended::partition_results;
use noirc_errors::CustomDiagnostic;

use crate::graph::CrateId;
use std::collections::HashMap;

use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleDefId, ModuleId, PerNs};
use crate::{Ident, Path, PathKind};

#[derive(Debug, Clone)]
pub struct ImportDirective {
    pub module_id: LocalModuleId,
    pub path: Path,
    pub alias: Option<Ident>,
}

pub type PathResolution = Result<PerNs, PathResolutionError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResolutionError {
    Unresolved(Ident),
    ExternalContractUsed(Ident),
}

#[derive(Debug)]
pub struct ResolvedImport {
    // name of the namespace, either last path segment or an alias
    pub name: Ident,
    // The symbol which we have resolved to
    pub resolved_namespace: PerNs,
    // The module which we must add the resolved namespace to
    pub module_scope: LocalModuleId,
}

impl From<PathResolutionError> for CustomDiagnostic {
    fn from(error: PathResolutionError) -> Self {
        match error {
            PathResolutionError::Unresolved(ident) => CustomDiagnostic::simple_error(
                format!("Could not resolve '{ident}' in path"),
                String::new(),
                ident.span(),
            ),
            PathResolutionError::ExternalContractUsed(ident) => CustomDiagnostic::simple_error(
                format!("Contract variable '{ident}' referenced from outside the contract"),
                "Contracts may only be referenced from within a contract".to_string(),
                ident.span(),
            ),
        }
    }
}

pub fn resolve_imports(
    crate_id: CrateId,
    imports_to_resolve: Vec<ImportDirective>,
    def_maps: &HashMap<CrateId, CrateDefMap>,
) -> (Vec<ResolvedImport>, Vec<(PathResolutionError, LocalModuleId)>) {
    let def_map = &def_maps[&crate_id];

    partition_results(imports_to_resolve, |import_directive| {
        let allow_contracts =
            allow_referencing_contracts(def_maps, crate_id, import_directive.module_id);

        let module_scope = import_directive.module_id;
        let resolved_namespace =
            resolve_path_to_ns(&import_directive, def_map, def_maps, allow_contracts)
                .map_err(|error| (error, module_scope))?;

        let name = resolve_path_name(&import_directive);
        Ok(ResolvedImport { name, resolved_namespace, module_scope })
    })
}

pub(super) fn allow_referencing_contracts(
    def_maps: &HashMap<CrateId, CrateDefMap>,
    krate: CrateId,
    local_id: LocalModuleId,
) -> bool {
    ModuleId { krate, local_id }.module(def_maps).is_contract
}

pub fn resolve_path_to_ns(
    import_directive: &ImportDirective,
    def_map: &CrateDefMap,
    def_maps: &HashMap<CrateId, CrateDefMap>,
    allow_contracts: bool,
) -> PathResolution {
    let import_path = &import_directive.path.segments;

    match import_directive.path.kind {
        crate::ast::PathKind::Crate => {
            // Resolve from the root of the crate
            resolve_path_from_crate_root(def_map, import_path, def_maps, allow_contracts)
        }
        crate::ast::PathKind::Dep => {
            resolve_external_dep(def_map, import_directive, def_maps, allow_contracts)
        }
        crate::ast::PathKind::Plain => {
            // Plain paths are only used to import children modules. It's possible to allow import of external deps, but maybe this distinction is better?
            // In Rust they can also point to external Dependencies, if no children can be found with the specified name
            resolve_name_in_module(
                def_map,
                import_path,
                import_directive.module_id,
                def_maps,
                allow_contracts,
            )
        }
    }
}

fn resolve_path_from_crate_root(
    def_map: &CrateDefMap,
    import_path: &[Ident],
    def_maps: &HashMap<CrateId, CrateDefMap>,
    allow_contracts: bool,
) -> PathResolution {
    resolve_name_in_module(def_map, import_path, def_map.root, def_maps, allow_contracts)
}

fn resolve_name_in_module(
    def_map: &CrateDefMap,
    import_path: &[Ident],
    starting_mod: LocalModuleId,
    def_maps: &HashMap<CrateId, CrateDefMap>,
    allow_contracts: bool,
) -> PathResolution {
    let mut current_mod = &def_map.modules[starting_mod.0];

    // There is a possibility that the import path is empty
    // In that case, early return
    if import_path.is_empty() {
        let mod_id = ModuleId { krate: def_map.krate, local_id: starting_mod };
        return Ok(PerNs::types(mod_id.into()));
    }

    let mut import_path = import_path.iter();
    let first_segment = import_path.next().expect("ice: could not fetch first segment");
    let mut current_ns = current_mod.find_name(first_segment);
    if current_ns.is_none() {
        return Err(PathResolutionError::Unresolved(first_segment.clone()));
    }

    for segment in import_path {
        let typ = match current_ns.take_types() {
            None => return Err(PathResolutionError::Unresolved(segment.clone())),
            Some(typ) => typ,
        };

        // In the type namespace, only Mod can be used in a path.
        let new_module_id = match typ {
            ModuleDefId::ModuleId(id) => id,
            ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
            // TODO: If impls are ever implemented, types can be used in a path
            ModuleDefId::TypeId(id) => id.0,
            ModuleDefId::TypeAliasId(_) => panic!("type aliases cannot be in the type namespace"),
            ModuleDefId::GlobalId(_) => panic!("globals cannot be in the type namespace"),
        };

        current_mod = &def_maps[&new_module_id.krate].modules[new_module_id.local_id.0];

        // Check if namespace
        let found_ns = current_mod.find_name(segment);
        if found_ns.is_none() {
            return Err(PathResolutionError::Unresolved(segment.clone()));
        }

        // Check if it is a contract and we're calling from a non-contract context
        if current_mod.is_contract && !allow_contracts {
            return Err(PathResolutionError::ExternalContractUsed(segment.clone()));
        }

        current_ns = found_ns;
    }

    Ok(current_ns)
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
    def_maps: &HashMap<CrateId, CrateDefMap>,
    allow_contracts: bool,
) -> PathResolution {
    // Use extern_prelude to get the dep
    //
    let path = &directive.path.segments;
    //
    // Fetch the root module from the prelude
    let crate_name = path.first().unwrap();
    let dep_module = current_def_map
        .extern_prelude
        .get(&crate_name.0.contents)
        .ok_or_else(|| PathResolutionError::Unresolved(crate_name.to_owned()))?;

    // Create an import directive for the dependency crate
    let path_without_crate_name = &path[1..]; // XXX: This will panic if the path is of the form `use dep::std` Ideal algorithm will not distinguish between crate and module

    let path = Path { segments: path_without_crate_name.to_vec(), kind: PathKind::Plain };
    let dep_directive =
        ImportDirective { module_id: dep_module.local_id, path, alias: directive.alias.clone() };

    let dep_def_map = def_maps.get(&dep_module.krate).unwrap();

    resolve_path_to_ns(&dep_directive, dep_def_map, def_maps, allow_contracts)
}
