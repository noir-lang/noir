use noirc_errors::{CustomDiagnostic, Span};
use thiserror::Error;

use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::node_interner::ReferenceId;
use crate::usage_tracker::UsageTracker;

use std::collections::BTreeMap;

use crate::ast::{Ident, ItemVisibility, Path, PathKind, PathSegment};
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleDefId, ModuleId, PerNs};

use super::errors::ResolverError;
use super::visibility::can_reference_module_id;

#[derive(Debug, Clone)]
pub struct ImportDirective {
    pub visibility: ItemVisibility,
    pub module_id: LocalModuleId,
    pub self_type_module_id: Option<ModuleId>,
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
    usage_tracker: &mut UsageTracker,
    path_references: &mut Option<&mut Vec<ReferenceId>>,
) -> Result<ResolvedImport, PathResolutionError> {
    let module_scope = import_directive.module_id;
    let NamespaceResolution { module_id: resolved_module, namespace: resolved_namespace, error } =
        resolve_path_to_ns(
            import_directive,
            crate_id,
            crate_id,
            def_maps,
            usage_tracker,
            path_references,
        )?;

    let name = resolve_path_name(import_directive);

    let visibility = resolved_namespace
        .values
        .or(resolved_namespace.types)
        .map(|(_, visibility, _)| visibility)
        .expect("Found empty namespace");

    let error = error.or_else(|| {
        if import_directive.self_type_module_id == Some(resolved_module)
            || can_reference_module_id(
                def_maps,
                crate_id,
                import_directive.module_id,
                resolved_module,
                visibility,
            )
        {
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
    usage_tracker: &mut UsageTracker,
    path_references: &mut Option<&mut Vec<ReferenceId>>,
) -> NamespaceResolutionResult {
    let import_path = &import_directive.path.segments;

    match import_directive.path.kind {
        crate::ast::PathKind::Crate => {
            // Resolve from the root of the crate
            resolve_path_from_crate_root(
                crate_id,
                importing_crate,
                import_path,
                def_maps,
                usage_tracker,
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
                    true, // plain or crate
                    usage_tracker,
                    path_references,
                );
            }

            let def_map = &def_maps[&crate_id];
            let current_mod_id = ModuleId { krate: crate_id, local_id: import_directive.module_id };
            let current_mod = &def_map.modules[current_mod_id.local_id.0];
            let first_segment =
                &import_path.first().expect("ice: could not fetch first segment").ident;
            if current_mod.find_name(first_segment).is_none() {
                // Resolve externally when first segment is unresolved
                return resolve_external_dep(
                    crate_id,
                    // def_map,
                    import_directive,
                    def_maps,
                    usage_tracker,
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
                true, // plain or crate
                usage_tracker,
                path_references,
            )
        }

        crate::ast::PathKind::Dep => resolve_external_dep(
            crate_id,
            import_directive,
            def_maps,
            usage_tracker,
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
                    false, // plain or crate
                    usage_tracker,
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

    import_path: &[PathSegment],
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    usage_tracker: &mut UsageTracker,
    path_references: &mut Option<&mut Vec<ReferenceId>>,
) -> NamespaceResolutionResult {
    let starting_mod = def_maps[&crate_id].root;
    resolve_name_in_module(
        crate_id,
        importing_crate,
        import_path,
        starting_mod,
        def_maps,
        true, // plain or crate
        usage_tracker,
        path_references,
    )
}

#[allow(clippy::too_many_arguments)]
fn resolve_name_in_module(
    krate: CrateId,
    importing_crate: CrateId,
    import_path: &[PathSegment],
    starting_mod: LocalModuleId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    plain_or_crate: bool,
    usage_tracker: &mut UsageTracker,
    path_references: &mut Option<&mut Vec<ReferenceId>>,
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

    let first_segment = &import_path.first().expect("ice: could not fetch first segment").ident;
    let mut current_ns = current_mod.find_name(first_segment);
    if current_ns.is_none() {
        return Err(PathResolutionError::Unresolved(first_segment.clone()));
    }

    usage_tracker.mark_as_referenced(current_mod_id, first_segment);

    let mut warning: Option<PathResolutionError> = None;
    for (index, (last_segment, current_segment)) in
        import_path.iter().zip(import_path.iter().skip(1)).enumerate()
    {
        let last_segment = &last_segment.ident;
        let current_segment = &current_segment.ident;

        let (typ, visibility) = match current_ns.types {
            None => return Err(PathResolutionError::Unresolved(last_segment.clone())),
            Some((typ, visibility, _)) => (typ, visibility),
        };

        // In the type namespace, only Mod can be used in a path.
        current_mod_id = match typ {
            ModuleDefId::ModuleId(id) => {
                if let Some(path_references) = path_references {
                    path_references.push(ReferenceId::Module(id));
                }
                id
            }
            ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
            // TODO: If impls are ever implemented, types can be used in a path
            ModuleDefId::TypeId(id) => {
                if let Some(path_references) = path_references {
                    path_references.push(ReferenceId::Struct(id));
                }
                id.module_id()
            }
            ModuleDefId::TypeAliasId(_) => panic!("type aliases cannot be used in type namespace"),
            ModuleDefId::TraitId(id) => {
                if let Some(path_references) = path_references {
                    path_references.push(ReferenceId::Trait(id));
                }
                id.0
            }
            ModuleDefId::GlobalId(_) => panic!("globals cannot be in the type namespace"),
        };

        warning = warning.or_else(|| {
            // If the path is plain or crate, the first segment will always refer to
            // something that's visible from the current module.
            if (plain_or_crate && index == 0)
                || can_reference_module_id(
                    def_maps,
                    importing_crate,
                    starting_mod,
                    current_mod_id,
                    visibility,
                )
            {
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

        usage_tracker.mark_as_referenced(current_mod_id, current_segment);

        current_ns = found_ns;
    }

    Ok(NamespaceResolution { module_id: current_mod_id, namespace: current_ns, error: warning })
}

fn resolve_path_name(import_directive: &ImportDirective) -> Ident {
    match &import_directive.alias {
        None => import_directive.path.last_ident(),
        Some(ident) => ident.clone(),
    }
}

fn resolve_external_dep(
    crate_id: CrateId,
    directive: &ImportDirective,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    usage_tracker: &mut UsageTracker,
    path_references: &mut Option<&mut Vec<ReferenceId>>,
    importing_crate: CrateId,
) -> NamespaceResolutionResult {
    // Use extern_prelude to get the dep
    let path = &directive.path.segments;

    let current_def_map = &def_maps[&crate_id];

    // Fetch the root module from the prelude
    let crate_name = &path.first().unwrap().ident;
    let dep_module = current_def_map
        .extern_prelude
        .get(&crate_name.0.contents)
        .ok_or_else(|| PathResolutionError::Unresolved(crate_name.to_owned()))?;

    // Create an import directive for the dependency crate
    // XXX: This will panic if the path is of the form `use std`. Ideal algorithm will not distinguish between crate and module
    // See `singleton_import.nr` test case for a check that such cases are handled elsewhere.
    let path_without_crate_name = &path[1..];

    if let Some(path_references) = path_references {
        path_references.push(ReferenceId::Module(*dep_module));
    }

    let path = Path {
        segments: path_without_crate_name.to_vec(),
        kind: PathKind::Plain,
        span: Span::default(),
    };
    let dep_directive = ImportDirective {
        visibility: ItemVisibility::Private,
        module_id: dep_module.local_id,
        self_type_module_id: directive.self_type_module_id,
        path,
        alias: directive.alias.clone(),
        is_prelude: false,
    };

    resolve_path_to_ns(
        &dep_directive,
        dep_module.krate,
        importing_crate,
        def_maps,
        usage_tracker,
        path_references,
    )
}
