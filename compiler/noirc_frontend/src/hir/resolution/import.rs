use noirc_errors::{CustomDiagnostic, Span};
use thiserror::Error;

use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::CompilationError;

use crate::node_interner::{NodeInterner, ReferenceId};
use crate::usage_tracker::UsageTracker;
use crate::Type;

use std::collections::BTreeMap;

use crate::ast::{Ident, ItemVisibility, Path, PathKind, PathSegment};
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleDefId, ModuleId, PerNs};

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

struct NamespaceResolution {
    module_id: ModuleId,
    namespace: PerNs,
    errors: Vec<PathResolutionError>,
}

type NamespaceResolutionResult = Result<NamespaceResolution, PathResolutionError>;

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
}

#[derive(Debug)]
pub struct ResolvedImport {
    // The symbol which we have resolved to
    pub resolved_namespace: PerNs,
    // The module which we must add the resolved namespace to
    pub module_scope: LocalModuleId,
    pub is_prelude: bool,
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
        }
    }
}

pub fn resolve_import<'p>(
    crate_id: CrateId,
    import_directive: &ImportDirective,
    interner: &NodeInterner,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    usage_tracker: &mut UsageTracker,
    path_references: &'p mut Option<&'p mut Vec<ReferenceId>>,
) -> Result<ResolvedImport, PathResolutionError> {
    let mut solver = ImportSolver::new(interner, def_maps, usage_tracker, path_references);
    solver.solve(import_directive, crate_id)
}

struct ImportSolver<'interner, 'def_maps, 'usage_tracker, 'path_references> {
    interner: &'interner NodeInterner,
    def_maps: &'def_maps BTreeMap<CrateId, CrateDefMap>,
    usage_tracker: &'usage_tracker mut UsageTracker,
    path_references: &'path_references mut Option<&'path_references mut Vec<ReferenceId>>,
}

impl<'interner, 'def_maps, 'usage_tracker, 'path_references>
    ImportSolver<'interner, 'def_maps, 'usage_tracker, 'path_references>
{
    fn new(
        interner: &'interner NodeInterner,
        def_maps: &'def_maps BTreeMap<CrateId, CrateDefMap>,
        usage_tracker: &'usage_tracker mut UsageTracker,
        path_references: &'path_references mut Option<&'path_references mut Vec<ReferenceId>>,
    ) -> Self {
        Self { interner, def_maps, usage_tracker, path_references }
    }

    fn solve(
        &mut self,
        import_directive: &ImportDirective,
        crate_id: CrateId,
    ) -> Result<ResolvedImport, PathResolutionError> {
        let NamespaceResolution {
            module_id: resolved_module,
            namespace: resolved_namespace,
            mut errors,
        } = self.resolve_path_to_ns(import_directive, crate_id, crate_id)?;

        let visibility = resolved_namespace
            .values
            .or(resolved_namespace.types)
            .map(|(_, visibility, _)| visibility)
            .expect("Found empty namespace");

        if !can_reference_module_id(
            self.def_maps,
            crate_id,
            import_directive.module_id,
            resolved_module,
            visibility,
        ) {
            errors.push(PathResolutionError::Private(import_directive.path.last_ident()));
        }

        Ok(ResolvedImport {
            resolved_namespace,
            module_scope: import_directive.module_id,
            is_prelude: import_directive.is_prelude,
            errors,
        })
    }

    fn resolve_path_to_ns(
        &mut self,
        import_directive: &ImportDirective,
        crate_id: CrateId,
        importing_crate: CrateId,
    ) -> NamespaceResolutionResult {
        let import_path = &import_directive.path.segments;

        match import_directive.path.kind {
            crate::ast::PathKind::Crate => {
                // Resolve from the root of the crate
                self.resolve_path_from_crate_root(crate_id, importing_crate, import_path)
            }
            crate::ast::PathKind::Plain => {
                // There is a possibility that the import path is empty
                // In that case, early return
                if import_path.is_empty() {
                    return self.resolve_name_in_module(
                        crate_id,
                        importing_crate,
                        import_path,
                        import_directive.module_id,
                        true, // plain or crate
                    );
                }

                let def_map = &self.def_maps[&crate_id];
                let current_mod_id =
                    ModuleId { krate: crate_id, local_id: import_directive.module_id };
                let current_mod = &def_map.modules[current_mod_id.local_id.0];
                let first_segment =
                    &import_path.first().expect("ice: could not fetch first segment").ident;
                if current_mod.find_name(first_segment).is_none() {
                    // Resolve externally when first segment is unresolved
                    return self.resolve_external_dep(crate_id, import_directive, importing_crate);
                }

                self.resolve_name_in_module(
                    crate_id,
                    importing_crate,
                    import_path,
                    import_directive.module_id,
                    true, // plain or crate
                )
            }

            crate::ast::PathKind::Dep => {
                self.resolve_external_dep(crate_id, import_directive, importing_crate)
            }

            crate::ast::PathKind::Super => {
                if let Some(parent_module_id) =
                    self.def_maps[&crate_id].modules[import_directive.module_id.0].parent
                {
                    self.resolve_name_in_module(
                        crate_id,
                        importing_crate,
                        import_path,
                        parent_module_id,
                        false, // plain or crate
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
        &mut self,
        crate_id: CrateId,
        importing_crate: CrateId,
        import_path: &[PathSegment],
    ) -> NamespaceResolutionResult {
        let starting_mod = self.def_maps[&crate_id].root;
        self.resolve_name_in_module(
            crate_id,
            importing_crate,
            import_path,
            starting_mod,
            true, // plain or crate
        )
    }

    fn resolve_name_in_module(
        &mut self,
        krate: CrateId,
        importing_crate: CrateId,
        import_path: &[PathSegment],
        starting_mod: LocalModuleId,
        plain_or_crate: bool,
    ) -> NamespaceResolutionResult {
        let def_map = &self.def_maps[&krate];
        let mut current_mod_id = ModuleId { krate, local_id: starting_mod };
        let mut current_mod = &def_map.modules[current_mod_id.local_id.0];

        // There is a possibility that the import path is empty
        // In that case, early return
        if import_path.is_empty() {
            return Ok(NamespaceResolution {
                module_id: current_mod_id,
                namespace: PerNs::types(current_mod_id.into()),
                errors: Vec::new(),
            });
        }

        let first_segment = &import_path.first().expect("ice: could not fetch first segment").ident;
        let mut current_ns = current_mod.find_name(first_segment);
        if current_ns.is_none() {
            return Err(PathResolutionError::Unresolved(first_segment.clone()));
        }

        self.usage_tracker.mark_as_referenced(current_mod_id, first_segment);

        let mut errors = Vec::new();
        for (index, (last_segment, current_segment)) in
            import_path.iter().zip(import_path.iter().skip(1)).enumerate()
        {
            let last_ident = &last_segment.ident;
            let current_ident = &current_segment.ident;

            let (typ, visibility) = match current_ns.types {
                None => return Err(PathResolutionError::Unresolved(last_ident.clone())),
                Some((typ, visibility, _)) => (typ, visibility),
            };

            // In the type namespace, only Mod can be used in a path.
            current_mod_id = match typ {
                ModuleDefId::ModuleId(id) => {
                    if let Some(path_references) = self.path_references {
                        path_references.push(ReferenceId::Module(id));
                    }

                    id
                }
                ModuleDefId::TypeId(id) => {
                    if let Some(path_references) = self.path_references {
                        path_references.push(ReferenceId::Struct(id));
                    }

                    id.module_id()
                }
                ModuleDefId::TypeAliasId(id) => {
                    if let Some(path_references) = self.path_references {
                        path_references.push(ReferenceId::Alias(id));
                    }

                    let type_alias = self.interner.get_type_alias(id);
                    let type_alias = type_alias.borrow();

                    let module_id = match &type_alias.typ {
                        Type::Struct(struct_id, _generics) => struct_id.borrow().id.module_id(),
                        Type::Error => {
                            return Err(PathResolutionError::Unresolved(last_ident.clone()));
                        }
                        _ => {
                            // For now we only allow type aliases that point to structs.
                            // The more general case is captured here: https://github.com/noir-lang/noir/issues/6398
                            panic!("Type alias in path not pointing to struct not yet supported")
                        }
                    };

                    module_id
                }
                ModuleDefId::TraitId(id) => {
                    if let Some(path_references) = self.path_references {
                        path_references.push(ReferenceId::Trait(id));
                    }

                    id.0
                }
                ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
                ModuleDefId::GlobalId(_) => panic!("globals cannot be in the type namespace"),
            };

            // If the path is plain or crate, the first segment will always refer to
            // something that's visible from the current module.
            if !((plain_or_crate && index == 0)
                || can_reference_module_id(
                    self.def_maps,
                    importing_crate,
                    starting_mod,
                    current_mod_id,
                    visibility,
                ))
            {
                errors.push(PathResolutionError::Private(last_ident.clone()));
            }

            current_mod = &self.def_maps[&current_mod_id.krate].modules[current_mod_id.local_id.0];

            // Check if namespace
            let found_ns = current_mod.find_name(current_ident);
            if found_ns.is_none() {
                return Err(PathResolutionError::Unresolved(current_ident.clone()));
            }

            self.usage_tracker.mark_as_referenced(current_mod_id, current_ident);

            current_ns = found_ns;
        }

        Ok(NamespaceResolution { module_id: current_mod_id, namespace: current_ns, errors })
    }

    fn resolve_external_dep(
        &mut self,
        crate_id: CrateId,
        directive: &ImportDirective,
        importing_crate: CrateId,
    ) -> NamespaceResolutionResult {
        // Use extern_prelude to get the dep
        let path = &directive.path.segments;

        let current_def_map = &self.def_maps[&crate_id];

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

        if let Some(path_references) = self.path_references {
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
            path,
            alias: directive.alias.clone(),
            is_prelude: false,
        };

        self.resolve_path_to_ns(&dep_directive, dep_module.krate, importing_crate)
    }
}
