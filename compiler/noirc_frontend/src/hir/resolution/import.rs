use noirc_errors::{CustomDiagnostic, Span};
use thiserror::Error;

use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::CompilationError;

use crate::locations::ReferencesTracker;
use crate::usage_tracker::UsageTracker;

use std::collections::BTreeMap;

use crate::ast::{Ident, ItemVisibility, Path, PathKind};
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleData, ModuleDefId, ModuleId, PerNs};

use super::errors::ResolverError;
use super::visibility::can_reference_module_id;

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
    NoSuper(Span),
    #[error("turbofish (`::<_>`) not allowed on {item}")]
    TurbofishNotAllowedOnItem { item: String, span: Span },
    #[error("{ident} is a {kind}, not a module")]
    NotAModule { ident: Ident, kind: &'static str },
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
            PathResolutionError::TurbofishNotAllowedOnItem { item: _, span } => {
                CustomDiagnostic::simple_error(error.to_string(), String::new(), *span)
            }
            PathResolutionError::NotAModule { ident, kind: _ } => {
                CustomDiagnostic::simple_error(error.to_string(), String::new(), ident.span())
            }
        }
    }
}

/// Resolves a Path in a `use` statement, assuming it's located in `importing_module`.
/// If the imported name can't be found, `Err` will be returned. If it can be found, `Ok`
/// will be returned with a potential list of errors if, for example, one of the segments
/// is not accessible from the importing module (e.g. because it's private).
pub fn resolve_import(
    path: Path,
    importing_module: ModuleId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    usage_tracker: &mut UsageTracker,
    references_tracker: Option<ReferencesTracker>,
) -> ImportResolutionResult {
    let mut solver = ImportSolver::new(def_maps, usage_tracker, references_tracker);
    solver.solve(path, importing_module)
}

struct ImportSolver<'def_maps, 'usage_tracker, 'references_tracker> {
    def_maps: &'def_maps BTreeMap<CrateId, CrateDefMap>,
    usage_tracker: &'usage_tracker mut UsageTracker,
    references_tracker: Option<ReferencesTracker<'references_tracker>>,
}

impl<'def_maps, 'usage_tracker, 'references_tracker>
    ImportSolver<'def_maps, 'usage_tracker, 'references_tracker>
{
    fn new(
        def_maps: &'def_maps BTreeMap<CrateId, CrateDefMap>,
        usage_tracker: &'usage_tracker mut UsageTracker,
        references_tracker: Option<ReferencesTracker<'references_tracker>>,
    ) -> Self {
        Self { def_maps, usage_tracker, references_tracker }
    }

    fn solve(&mut self, path: Path, importing_module: ModuleId) -> ImportResolutionResult {
        match path.kind {
            PathKind::Crate => self.resolve_crate_path(path, importing_module),
            PathKind::Plain => self.resolve_plain_path(path, importing_module, importing_module),
            PathKind::Dep => self.resolve_dep_path(path, importing_module),
            PathKind::Super => self.resolve_super_path(path, importing_module),
        }
    }

    fn resolve_crate_path(
        &mut self,
        path: Path,
        importing_module: ModuleId,
    ) -> ImportResolutionResult {
        let root_module = self.def_maps[&importing_module.krate].root;
        let current_module = ModuleId { krate: importing_module.krate, local_id: root_module };
        self.resolve_name_in_module(path, current_module, importing_module)
    }

    fn resolve_plain_path(
        &mut self,
        path: Path,
        current_module: ModuleId,
        importing_module: ModuleId,
    ) -> ImportResolutionResult {
        // There is a possibility that the import path is empty. In that case, early return.
        if path.segments.is_empty() {
            return self.resolve_name_in_module(path, current_module, importing_module);
        }

        let first_segment =
            &path.segments.first().expect("ice: could not fetch first segment").ident;
        if self.get_module(current_module).find_name(first_segment).is_none() {
            // Resolve externally when first segment is unresolved
            return self.resolve_dep_path(path, importing_module);
        }

        self.resolve_name_in_module(path, current_module, importing_module)
    }

    fn resolve_dep_path(
        &mut self,
        mut path: Path,
        importing_module: ModuleId,
    ) -> ImportResolutionResult {
        // Use extern_prelude to get the dep
        let current_def_map = &self.def_maps[&importing_module.krate];

        // Fetch the root module from the prelude
        let crate_name = &path.segments.first().unwrap().ident;
        let dep_module = current_def_map
            .extern_prelude
            .get(&crate_name.0.contents)
            .ok_or_else(|| PathResolutionError::Unresolved(crate_name.to_owned()))?;

        let span = crate_name.span();
        self.add_reference(ModuleDefId::ModuleId(*dep_module), span, false);

        // Create an import directive for the dependency crate
        // XXX: This will panic if the path is of the form `use std`. Ideal algorithm will not distinguish between crate and module
        // See `singleton_import.nr` test case for a check that such cases are handled elsewhere.
        path.kind = PathKind::Plain;
        path.segments.remove(0);

        self.resolve_plain_path(path, *dep_module, importing_module)
    }

    fn resolve_super_path(
        &mut self,
        path: Path,
        importing_module: ModuleId,
    ) -> ImportResolutionResult {
        let Some(parent_module_id) = self.get_module(importing_module).parent else {
            let span_start = path.span.start();
            let span = Span::from(span_start..span_start + 5); // 5 == "super".len()
            return Err(PathResolutionError::NoSuper(span));
        };

        let current_module = ModuleId { krate: importing_module.krate, local_id: parent_module_id };
        self.resolve_name_in_module(path, current_module, importing_module)
    }

    fn resolve_name_in_module(
        &mut self,
        path: Path,
        starting_module: ModuleId,
        importing_module: ModuleId,
    ) -> ImportResolutionResult {
        let plain_or_crate = matches!(path.kind, PathKind::Plain | PathKind::Crate);

        // The current module and module ID as we resolve path segments
        let mut current_module_id = starting_module;
        let mut current_module = self.get_module(starting_module);

        // There is a possibility that the import path is empty. In that case, early return.
        if path.segments.is_empty() {
            return Ok(ResolvedImport {
                namespace: PerNs::types(current_module_id.into()),
                errors: Vec::new(),
            });
        }

        let first_segment =
            &path.segments.first().expect("ice: could not fetch first segment").ident;
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
                None => return Err(PathResolutionError::Unresolved(last_ident.clone())),
                Some((typ, visibility, _)) => (typ, visibility),
            };

            self.add_reference(typ, last_segment.span, last_segment.ident.is_self_type_name());

            // In the type namespace, only Mod can be used in a path.
            current_module_id = match typ {
                ModuleDefId::ModuleId(id) => id,
                ModuleDefId::TypeId(_) | ModuleDefId::TypeAliasId(_) | ModuleDefId::TraitId(_) => {
                    let ident = last_segment.ident.clone();
                    return Err(PathResolutionError::NotAModule { ident, kind: typ.as_str() });
                }
                ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
                ModuleDefId::GlobalId(_) => panic!("globals cannot be in the type namespace"),
            };

            // If the path is plain or crate, the first segment will always refer to
            // something that's visible from the current module.
            if !((plain_or_crate && index == 0)
                || can_reference_module_id(
                    self.def_maps,
                    importing_module.krate,
                    starting_module.local_id,
                    current_module_id,
                    visibility,
                ))
            {
                errors.push(PathResolutionError::Private(last_ident.clone()));
            }

            current_module =
                &self.def_maps[&current_module_id.krate].modules[current_module_id.local_id.0];

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

        self.add_reference(module_def_id, path.segments.last().unwrap().ident.span(), false);

        if !can_reference_module_id(
            self.def_maps,
            importing_module.krate,
            importing_module.local_id,
            current_module_id,
            visibility,
        ) {
            errors.push(PathResolutionError::Private(path.last_ident()));
        }

        Ok(ResolvedImport { namespace: current_ns, errors })
    }

    fn get_module(&self, module: ModuleId) -> &ModuleData {
        let message = "A crate should always be present for a given crate id";
        &self.def_maps.get(&module.krate).expect(message).modules[module.local_id.0]
    }

    fn add_reference(&mut self, reference_id: ModuleDefId, span: Span, is_self_type_name: bool) {
        if let Some(references_tracker) = &mut self.references_tracker {
            references_tracker.add_reference(reference_id, span, is_self_type_name);
        }
    }
}
