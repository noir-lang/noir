use iter_extended::vecmap;
use noirc_errors::{CustomDiagnostic, Location};
use thiserror::Error;

use crate::elaborator::{TypedPath, TypedPathSegment};
use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::CompilationError;

use crate::locations::ReferencesTracker;
use crate::usage_tracker::UsageTracker;

use std::collections::BTreeMap;

use crate::ast::{Ident, ItemVisibility, Path, PathKind, PathSegment};
use crate::hir::def_map::{
    CrateDefMap, DefMaps, LocalModuleId, ModuleData, ModuleDefId, ModuleId, PerNs,
};

use super::errors::ResolverError;
use super::visibility::item_in_module_is_visible;

#[derive(Debug, Clone)]
pub struct ImportDirective {
    pub visibility: ItemVisibility,
    pub module_id: LocalModuleId,
    pub path: Path,
    pub alias: Option<Ident>,
    pub is_prelude: bool,
}

impl ImportDirective {
    /// Returns the name that's brought into scope: either the alias or the last segment of the path
    pub fn name(&self) -> Ident {
        match &self.alias {
            None => self.path.last_ident(),
            Some(ident) => ident.clone(),
        }
    }
}

type ImportResolutionResult = Result<ResolvedImport, PathResolutionError>;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PathResolutionError {
    #[error("Could not resolve '{0}' in path")]
    Unresolved(Ident),
    #[error("{0} is private and not visible from the current module")]
    Private(Ident),
    #[error("There is no super module")]
    NoSuper(Location),
    #[error("turbofish (`::<_>`) not allowed on {item}")]
    TurbofishNotAllowedOnItem { item: String, location: Location },
    #[error("{ident} is a {kind}, not a module")]
    NotAModule { ident: Ident, kind: &'static str },
    #[error(
        "trait `{trait_name}` which provides `{ident}` is implemented but not in scope, please import it"
    )]
    TraitMethodNotInScope { ident: Ident, trait_name: String },
    #[error("Could not resolve '{ident}' in path")]
    UnresolvedWithPossibleTraitsToImport { ident: Ident, traits: Vec<String> },
    #[error("Multiple applicable items in scope")]
    MultipleTraitsInScope { ident: Ident, traits: Vec<String> },
    #[error("`StructDefinition` is deprecated. It has been renamed to `TypeDefinition`")]
    StructDefinitionDeprecated { location: Location },
}

impl PathResolutionError {
    pub fn location(&self) -> Location {
        match self {
            PathResolutionError::NoSuper(location)
            | PathResolutionError::TurbofishNotAllowedOnItem { location, .. }
            | PathResolutionError::StructDefinitionDeprecated { location } => *location,
            PathResolutionError::Unresolved(ident)
            | PathResolutionError::Private(ident)
            | PathResolutionError::NotAModule { ident, .. }
            | PathResolutionError::TraitMethodNotInScope { ident, .. }
            | PathResolutionError::MultipleTraitsInScope { ident, .. }
            | PathResolutionError::UnresolvedWithPossibleTraitsToImport { ident, .. } => {
                ident.location()
            }
        }
    }
}

#[derive(Debug)]
pub struct ResolvedImport {
    // The symbol which we have resolved to
    pub namespace: PerNs,
    // The module which we must add the resolved namespace to
    pub errors: Vec<PathResolutionError>,
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
                CustomDiagnostic::simple_error(error.to_string(), String::new(), ident.location())
            }
            PathResolutionError::Private(ident) => CustomDiagnostic::simple_error(
                error.to_string(),
                format!("{ident} is private"),
                ident.location(),
            ),
            PathResolutionError::NoSuper(location) => {
                CustomDiagnostic::simple_error(error.to_string(), String::new(), *location)
            }
            PathResolutionError::TurbofishNotAllowedOnItem { item: _, location } => {
                CustomDiagnostic::simple_error(error.to_string(), String::new(), *location)
            }
            PathResolutionError::NotAModule { ident, kind: _ } => {
                CustomDiagnostic::simple_error(error.to_string(), String::new(), ident.location())
            }
            PathResolutionError::TraitMethodNotInScope { ident, .. } => {
                CustomDiagnostic::simple_error(error.to_string(), String::new(), ident.location())
            }
            PathResolutionError::UnresolvedWithPossibleTraitsToImport { ident, traits } => {
                let mut traits = vecmap(traits, |trait_name| format!("`{trait_name}`"));
                traits.sort();
                CustomDiagnostic::simple_error(
                    error.to_string(),
                    format!(
                        "The following traits which provide `{ident}` are implemented but not in scope: {}",
                        traits.join(", ")
                    ),
                    ident.location(),
                )
            }
            PathResolutionError::MultipleTraitsInScope { ident, traits } => {
                let mut traits = vecmap(traits, |trait_name| format!("`{trait_name}`"));
                traits.sort();
                CustomDiagnostic::simple_error(
                    error.to_string(),
                    format!(
                        "All these trait which provide `{ident}` are implemented and in scope: {}",
                        traits.join(", ")
                    ),
                    ident.location(),
                )
            }
            PathResolutionError::StructDefinitionDeprecated { location } => {
                CustomDiagnostic::simple_warning(
                    "`StructDefinition` is deprecated. It has been renamed to `TypeDefinition`"
                        .to_string(),
                    String::new(),
                    *location,
                )
            }
        }
    }
}

/// Resolves a Path in a `use` statement, assuming it's located in `importing_module`.
///
/// If the imported name can't be found, `Err` will be returned. If it can be found, `Ok`
/// will be returned with a potential list of errors if, for example, one of the segments
/// is not accessible from the importing module (e.g. because it's private).
pub fn resolve_import(
    path: Path,
    importing_module: ModuleId,
    def_maps: &DefMaps,
    usage_tracker: &mut UsageTracker,
    references_tracker: Option<ReferencesTracker>,
) -> ImportResolutionResult {
    let path = path_to_typed_path(path);
    let (path, module_id, references_tracker) =
        resolve_path_kind(path, importing_module, def_maps, references_tracker)?;
    let mut solver =
        ImportSolver::new(importing_module, def_maps, usage_tracker, references_tracker);
    solver.resolve_name_in_module(path, module_id)
}

fn path_to_typed_path(path: Path) -> TypedPath {
    let segments = vecmap(path.segments, path_segment_to_typed_path_segment);
    let kind_location = path.kind_location;
    TypedPath { segments, kind: path.kind, location: path.location, kind_location }
}

fn path_segment_to_typed_path_segment(segment: PathSegment) -> TypedPathSegment {
    assert!(segment.generics.is_none(), "generics should not be present in a use path segment");
    TypedPathSegment { ident: segment.ident, generics: None, location: segment.location }
}

/// Given a `TypedPath` and a [ModuleId] it's being used in, this function returns a `TypedPath`
/// and a [ModuleId] where that `TypedPath` should be resolved.
///
/// For a [PathKind::Dep] with a value such as `dep::foo::bar::baz`, the path will be turned into a
/// [PathKind::Plain] with the first segment (the crate `foo`) removed, leaving just `bar::baz`
/// to be resolved within `foo`. For other cases the path kind stays the same, it's just paired
/// up with the module where it should be looked up. If the module cannot be found, and error is
/// returned.
///
/// The third value in the tuple is a reference tracker that must be passed to this
/// method, which is used in case the path kind is `dep`: the segment after `dep`
/// will be linked to the root module of the external dependency.
pub fn resolve_path_kind<'r>(
    path: TypedPath,
    importing_module: ModuleId,
    def_maps: &DefMaps,
    references_tracker: Option<ReferencesTracker<'r>>,
) -> Result<(TypedPath, ModuleId, Option<ReferencesTracker<'r>>), PathResolutionError> {
    let mut solver =
        PathResolutionTargetResolver { importing_module, def_maps, references_tracker };
    let (path, module_id) = solver.resolve(path)?;
    Ok((path, module_id, solver.references_tracker))
}

struct PathResolutionTargetResolver<'def_maps, 'references_tracker> {
    importing_module: ModuleId,
    def_maps: &'def_maps BTreeMap<CrateId, CrateDefMap>,
    references_tracker: Option<ReferencesTracker<'references_tracker>>,
}

impl PathResolutionTargetResolver<'_, '_> {
    /// Resolve a `TypedPath` based on its [PathKind] to the target [ModuleId].
    fn resolve(&mut self, path: TypedPath) -> Result<(TypedPath, ModuleId), PathResolutionError> {
        match path.kind {
            PathKind::Crate => self.resolve_crate_path(path, self.importing_module.krate),
            PathKind::Plain => self.resolve_plain_path(path, self.importing_module),
            PathKind::Dep => self.resolve_dep_path(path),
            PathKind::Super => self.resolve_super_path(path),
            PathKind::Resolved(crate_id) => self.resolve_crate_path(path, crate_id),
        }
    }

    /// Resolve a path such as `crate::foo::bar` or `$crate::foo::bar`.
    ///
    /// Returns a path with its kind unchanged, paired up with the importing or defining module itself as the target.
    fn resolve_crate_path(
        &mut self,
        path: TypedPath,
        krate: CrateId,
    ) -> Result<(TypedPath, ModuleId), PathResolutionError> {
        let root_module = self.def_maps[&krate].root();
        let current_module = ModuleId { krate, local_id: root_module };
        Ok((path, current_module))
    }

    /// Resolve a path such as `foo::bar`:
    /// * check if `foo` module can be found in the current importing module
    /// * if not, treat the path as if it were `dep::foo::bar` and look for a `foo` crate instead
    fn resolve_plain_path(
        &mut self,
        path: TypedPath,
        current_module: ModuleId,
    ) -> Result<(TypedPath, ModuleId), PathResolutionError> {
        // There is a possibility that the import path is empty. In that case, early return.
        // This happens on import statements such as `use crate` or `use std`.
        if path.segments.is_empty() {
            return Ok((path, current_module));
        }

        let first_segment =
            &path.segments.first().expect("ICE: could not fetch first segment").ident;
        if get_module(self.def_maps, current_module).find_name(first_segment).is_none() {
            // Resolve externally when first segment is unresolved
            return self.resolve_dep_path(path);
        }

        Ok((path, current_module))
    }

    /// Resolve a path such as `dep::foo:bar::baz`:
    /// * find the `foo` crate among the dependencies of the current importing module
    /// * change the crate `foo` from the path, returning a plain path `bar::baz` along with the dependency module
    fn resolve_dep_path(
        &mut self,
        mut path: TypedPath,
    ) -> Result<(TypedPath, ModuleId), PathResolutionError> {
        // Use extern_prelude to get the dep
        let current_def_map = &self.def_maps[&self.importing_module.krate];

        // Fetch the root module from the prelude
        let crate_name = &path.segments.first().unwrap().ident;
        let dep_module = current_def_map
            .extern_prelude
            .get(crate_name.as_str())
            .ok_or_else(|| PathResolutionError::Unresolved(crate_name.to_owned()))?;

        if let Some(references_tracker) = &mut self.references_tracker {
            let location = crate_name.location();
            references_tracker.add_reference(ModuleDefId::ModuleId(*dep_module), location, false);
        }

        // Now the path can be solved starting from the second segment as a plain path
        path.kind = PathKind::Plain;
        path.segments.remove(0);

        Ok((path, *dep_module))
    }

    /// Resolve a path such as `super::foo::bar`:
    /// * get the parent of the current importing module
    /// * return the path still with [PathKind::Super], paired up with the parent module
    fn resolve_super_path(
        &mut self,
        path: TypedPath,
    ) -> Result<(TypedPath, ModuleId), PathResolutionError> {
        let Some(parent_module_id) = get_module(self.def_maps, self.importing_module).parent else {
            return Err(PathResolutionError::NoSuper(path.kind_location));
        };

        let current_module =
            ModuleId { krate: self.importing_module.krate, local_id: parent_module_id };
        Ok((path, current_module))
    }
}

struct ImportSolver<'def_maps, 'usage_tracker, 'references_tracker> {
    importing_module: ModuleId,
    def_maps: &'def_maps BTreeMap<CrateId, CrateDefMap>,
    usage_tracker: &'usage_tracker mut UsageTracker,
    references_tracker: Option<ReferencesTracker<'references_tracker>>,
}

impl<'def_maps, 'usage_tracker, 'references_tracker>
    ImportSolver<'def_maps, 'usage_tracker, 'references_tracker>
{
    fn new(
        importing_module: ModuleId,
        def_maps: &'def_maps BTreeMap<CrateId, CrateDefMap>,
        usage_tracker: &'usage_tracker mut UsageTracker,
        references_tracker: Option<ReferencesTracker<'references_tracker>>,
    ) -> Self {
        Self { importing_module, def_maps, usage_tracker, references_tracker }
    }

    fn resolve_name_in_module(
        &mut self,
        path: TypedPath,
        starting_module: ModuleId,
    ) -> ImportResolutionResult {
        // There is a possibility that the import path is empty. In that case, early return.
        if path.segments.is_empty() {
            return Ok(ResolvedImport {
                namespace: PerNs::types(starting_module.into()),
                errors: Vec::new(),
            });
        }

        let first_segment_is_always_visible = match path.kind {
            PathKind::Crate => true,
            PathKind::Plain => self.importing_module == starting_module,
            PathKind::Dep | PathKind::Super | PathKind::Resolved(_) => false,
        };

        // The current module and module ID as we resolve path segments
        let mut current_module_id = starting_module;
        let mut current_module = get_module(self.def_maps, starting_module);

        let first_segment =
            &path.segments.first().expect("ICE: could not fetch first segment").ident;
        let mut current_ns = current_module.find_name(first_segment);
        if current_ns.is_none() {
            return Err(PathResolutionError::Unresolved(first_segment.clone()));
        }

        self.usage_tracker.mark_as_referenced(current_module_id, first_segment);

        let mut errors = Vec::new();
        for (index, (last_segment, current_segment)) in
            path.segments.iter().zip(path.segments.iter().skip(1)).enumerate()
        {
            let last_ident = &last_segment.ident;
            let current_ident = &current_segment.ident;

            let (typ, visibility) = match current_ns.types {
                None => {
                    return Err(PathResolutionError::Unresolved(last_ident.clone()));
                }
                Some((typ, visibility, _)) => (typ, visibility),
            };

            self.add_reference(typ, last_segment.location, last_segment.ident.is_self_type_name());

            // In the type namespace, only Mod can be used in a path.
            current_module_id = match typ {
                ModuleDefId::ModuleId(id) => id,
                ModuleDefId::TypeId(id) => id.module_id(),
                ModuleDefId::TypeAliasId(..) => {
                    return Err(PathResolutionError::NotAModule {
                        ident: last_segment.ident.clone(),
                        kind: "type alias",
                    });
                }
                ModuleDefId::TraitAssociatedTypeId(..) => {
                    return Err(PathResolutionError::NotAModule {
                        ident: last_segment.ident.clone(),
                        kind: "associated type",
                    });
                }
                ModuleDefId::TraitId(id) => id.0,
                ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
                ModuleDefId::GlobalId(_) => panic!("globals cannot be in the type namespace"),
            };

            if !((first_segment_is_always_visible && index == 0)
                || self.item_in_module_is_visible(current_module_id, visibility))
            {
                errors.push(PathResolutionError::Private(last_ident.clone()));
            }

            current_module = &self.def_maps[&current_module_id.krate][current_module_id.local_id];

            // Check if namespace
            let found_ns = current_module.find_name(current_ident);
            if found_ns.is_none() {
                return Err(PathResolutionError::Unresolved(current_ident.clone()));
            }

            self.usage_tracker.mark_as_referenced(current_module_id, current_ident);

            current_ns = found_ns;
        }

        let (module_def_id, visibility, _) =
            current_ns.values.or(current_ns.types).expect("Found empty namespace");

        self.add_reference(module_def_id, path.segments.last().unwrap().ident.location(), false);

        if !self.item_in_module_is_visible(current_module_id, visibility) {
            errors.push(PathResolutionError::Private(path.last_ident()));
        }

        Ok(ResolvedImport { namespace: current_ns, errors })
    }

    fn add_reference(
        &mut self,
        reference_id: ModuleDefId,
        location: Location,
        is_self_type_name: bool,
    ) {
        if let Some(references_tracker) = &mut self.references_tracker {
            references_tracker.add_reference(reference_id, location, is_self_type_name);
        }
    }

    fn item_in_module_is_visible(&self, module: ModuleId, visibility: ItemVisibility) -> bool {
        item_in_module_is_visible(self.def_maps, self.importing_module, module, visibility)
    }
}

fn get_module(def_maps: &DefMaps, module: ModuleId) -> &ModuleData {
    let message = "A crate should always be present for a given crate id";
    &def_maps.get(&module.krate).expect(message)[module.local_id]
}
