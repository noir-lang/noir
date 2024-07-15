use noirc_errors::{CustomDiagnostic, Span};
use thiserror::Error;

use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::node_interner::ReferenceId;
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
    #[error("{0} is private and not visible from the current module")]
    Private(Ident),
    #[error("There is no super module")]
    NoSuper(Span),
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
            // This will be upgraded to an error in future versions
            PathResolutionError::Private(ident) => CustomDiagnostic::simple_warning(
                error.to_string(),
                format!("{ident} is private"),
                ident.span(),
            ),
            PathResolutionError::NoSuper(span) => {
                CustomDiagnostic::simple_error(error.to_string(), String::new(), *span)
            }
        }
    }
}

pub fn resolve_import(
    crate_id: CrateId,
    import_directive: &ImportDirective,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    path_references: &mut Option<&mut Vec<Option<ReferenceId>>>,
) -> Result<ResolvedImport, PathResolutionError> {
    let module_scope = import_directive.module_id;
    let NamespaceResolution {
        module_id: resolved_module,
        namespace: resolved_namespace,
        mut error,
    } = resolve_path_to_ns(import_directive, crate_id, crate_id, def_maps, path_references)?;

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

fn resolve_path_to_ns(
    import_directive: &ImportDirective,
    crate_id: CrateId,
    importing_crate: CrateId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    path_references: &mut Option<&mut Vec<Option<ReferenceId>>>,
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
                path_references,
            )
        }
        crate::ast::PathKind::Plain => {
            // There is a possibility that the import path is empty
            // In that case, early return
            if import_path.is_empty() {
                return resolve_name_in_module(
                    crate_id,
                    importing_crate,
                    import_path,
                    import_directive.module_id,
                    def_maps,
                    path_references,
                );
            }

            let current_mod_id = ModuleId { krate: crate_id, local_id: import_directive.module_id };
            let current_mod = &def_map.modules[current_mod_id.local_id.0];
            let first_segment = import_path.first().expect("ice: could not fetch first segment");
            if current_mod.find_name(first_segment).is_none() {
                // Resolve externally when first segment is unresolved
                return resolve_external_dep(
                    def_map,
                    import_directive,
                    def_maps,
                    path_references,
                    importing_crate,
                );
            }

            resolve_name_in_module(
                crate_id,
                importing_crate,
                import_path,
                import_directive.module_id,
                def_maps,
                path_references,
            )
        }

        crate::ast::PathKind::Dep => resolve_external_dep(
            def_map,
            import_directive,
            def_maps,
            path_references,
            importing_crate,
        ),

        crate::ast::PathKind::Super => {
            if let Some(parent_module_id) =
                def_maps[&crate_id].modules[import_directive.module_id.0].parent
            {
                resolve_name_in_module(
                    crate_id,
                    importing_crate,
                    import_path,
                    parent_module_id,
                    def_maps,
                    path_references,
                )
            } else {
                let span_start = import_directive.path.span().start();
                let span = Span::from(span_start..span_start + 5); // 5 == "super".len()
                Err(PathResolutionError::NoSuper(span))
            }
        }
    }
}

fn resolve_path_from_crate_root(
    crate_id: CrateId,
    importing_crate: CrateId,

    import_path: &[Ident],
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    path_references: &mut Option<&mut Vec<Option<ReferenceId>>>,
) -> NamespaceResolutionResult {
    resolve_name_in_module(
        crate_id,
        importing_crate,
        import_path,
        def_maps[&crate_id].root,
        def_maps,
        path_references,
    )
}

fn resolve_name_in_module(
    krate: CrateId,
    importing_crate: CrateId,
    import_path: &[Ident],
    starting_mod: LocalModuleId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    path_references: &mut Option<&mut Vec<Option<ReferenceId>>>,
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
            ModuleDefId::ModuleId(id) => {
                if let Some(path_references) = path_references {
                    path_references.push(Some(ReferenceId::Module(id)));
                }
                id
            }
            ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
            // TODO: If impls are ever implemented, types can be used in a path
            ModuleDefId::TypeId(id) => {
                if let Some(path_references) = path_references {
                    path_references.push(Some(ReferenceId::Struct(id)));
                }
                id.module_id()
            }
            ModuleDefId::TypeAliasId(_) => panic!("type aliases cannot be used in type namespace"),
            ModuleDefId::TraitId(id) => {
                if let Some(path_references) = path_references {
                    path_references.push(Some(ReferenceId::Trait(id)));
                }
                id.0
            }
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
    path_references: &mut Option<&mut Vec<Option<ReferenceId>>>,
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
    // XXX: This will panic if the path is of the form `use std`. Ideal algorithm will not distinguish between crate and module
    // See `singleton_import.nr` test case for a check that such cases are handled elsewhere.
    let path_without_crate_name = &path[1..];

    // Given that we skipped the first segment, record that it doesn't refer to any module or type.
    if let Some(path_references) = path_references {
        path_references.push(None);
    }

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

    resolve_path_to_ns(&dep_directive, dep_module.krate, importing_crate, def_maps, path_references)
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
