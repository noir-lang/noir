use noirc_errors::{Location, Span};

use crate::ast::{Ident, ItemVisibility, Path, PathKind, PathSegment};
use crate::graph::CrateId;
use crate::hir::def_map::{LocalModuleId, ModuleDefId, ModuleId, PerNs};
use crate::hir::resolution::import::{
    ImportDirective, IntermediatePathResolutionItem, NamespaceResolution,
    NamespaceResolutionResult, PathResolution, PathResolutionError, PathResolutionItem,
    PathResolutionResult, ResolvedImport, Turbofish,
};

use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::visibility::can_reference_module_id;
use crate::node_interner::ReferenceId;
use crate::Type;

use super::types::SELF_TYPE_NAME;
use super::Elaborator;

impl<'context> Elaborator<'context> {
    pub(super) fn resolve_path_or_error(
        &mut self,
        path: Path,
    ) -> Result<PathResolutionItem, ResolverError> {
        let path_resolution = self.resolve_path(path)?;

        for error in path_resolution.errors {
            self.push_err(error);
        }

        Ok(path_resolution.item)
    }

    pub(super) fn resolve_path(&mut self, path: Path) -> PathResolutionResult {
        let mut module_id = self.module_id();
        let mut path = path;

        if path.kind == PathKind::Plain && path.first_name() == SELF_TYPE_NAME {
            if let Some(Type::Struct(struct_type, _)) = &self.self_type {
                let struct_type = struct_type.borrow();
                if path.segments.len() == 1 {
                    return Ok(PathResolution {
                        item: PathResolutionItem::Struct(struct_type.id),
                        errors: Vec::new(),
                    });
                }

                module_id = struct_type.id.module_id();
                path = Path {
                    segments: path.segments[1..].to_vec(),
                    kind: PathKind::Plain,
                    span: path.span(),
                };
            }
        }

        self.resolve_path_in_module(path, module_id)
    }

    fn resolve_path_in_module(&mut self, path: Path, module_id: ModuleId) -> PathResolutionResult {
        let self_type_module_id = if let Some(Type::Struct(struct_type, _)) = &self.self_type {
            Some(struct_type.borrow().id.module_id())
        } else {
            None
        };

        self.resolve_path_impl(module_id, self_type_module_id, path)
    }

    fn resolve_path_impl(
        &mut self,
        module_id: ModuleId,
        self_type_module_id: Option<ModuleId>,
        path: Path,
    ) -> PathResolutionResult {
        if !self.interner.lsp_mode {
            return self.resolve_path_impl_with_references(
                module_id,
                self_type_module_id,
                path,
                &mut None,
            );
        }

        let last_segment = path.last_ident();
        let location = Location::new(last_segment.span(), self.file);
        let is_self_type_name = last_segment.is_self_type_name();

        let mut references: Vec<_> = Vec::new();
        let path_resolution = self.resolve_path_impl_with_references(
            module_id,
            self_type_module_id,
            path.clone(),
            &mut Some(&mut references),
        );

        for (referenced, segment) in references.iter().zip(path.segments) {
            self.interner.add_reference(
                *referenced,
                Location::new(segment.ident.span(), self.file),
                segment.ident.is_self_type_name(),
            );
        }

        let path_resolution = match path_resolution {
            Ok(path_resolution) => path_resolution,
            Err(err) => return Err(err),
        };

        self.interner.add_path_resolution_kind_reference(
            path_resolution.item.clone(),
            location,
            is_self_type_name,
        );

        Ok(path_resolution)
    }

    fn resolve_path_impl_with_references(
        &mut self,
        module_id: ModuleId,
        self_type_module_id: Option<ModuleId>,
        path: Path,
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
        let resolved_import = self.resolve_import(module_id.krate, &import, path_references)?;

        Ok(PathResolution { item: resolved_import.item, errors: resolved_import.errors })
    }

    fn resolve_import(
        &mut self,
        crate_id: CrateId,
        import_directive: &ImportDirective,
        path_references: &mut Option<&mut Vec<ReferenceId>>,
    ) -> Result<ResolvedImport, PathResolutionError> {
        let module_scope = import_directive.module_id;
        let NamespaceResolution {
            module_id: resolved_module,
            item,
            namespace: resolved_namespace,
            mut errors,
        } = self.resolve_path_to_ns(import_directive, crate_id, crate_id, path_references)?;

        let name = resolve_path_name(import_directive);

        let visibility = resolved_namespace
            .values
            .or(resolved_namespace.types)
            .map(|(_, visibility, _)| visibility)
            .expect("Found empty namespace");

        if !(import_directive.self_type_module_id == Some(resolved_module)
            || can_reference_module_id(
                self.def_maps,
                crate_id,
                import_directive.module_id,
                resolved_module,
                visibility,
            ))
        {
            errors.push(PathResolutionError::Private(name.clone()));
        }

        Ok(ResolvedImport {
            name,
            resolved_namespace,
            item,
            module_scope,
            is_prelude: import_directive.is_prelude,
            errors,
        })
    }

    fn resolve_path_to_ns(
        &mut self,
        import_directive: &ImportDirective,
        crate_id: CrateId,
        importing_crate: CrateId,
        path_references: &mut Option<&mut Vec<ReferenceId>>,
    ) -> NamespaceResolutionResult {
        let import_path = &import_directive.path.segments;

        match import_directive.path.kind {
            crate::ast::PathKind::Crate => {
                // Resolve from the root of the crate
                self.resolve_path_from_crate_root(
                    crate_id,
                    importing_crate,
                    import_path,
                    path_references,
                )
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
                        path_references,
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
                    return self.resolve_external_dep(
                        crate_id,
                        // def_map,
                        import_directive,
                        path_references,
                        importing_crate,
                    );
                }

                self.resolve_name_in_module(
                    crate_id,
                    importing_crate,
                    import_path,
                    import_directive.module_id,
                    true, // plain or crate
                    path_references,
                )
            }

            crate::ast::PathKind::Dep => self.resolve_external_dep(
                crate_id,
                import_directive,
                path_references,
                importing_crate,
            ),

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
        &mut self,
        crate_id: CrateId,
        importing_crate: CrateId,
        import_path: &[PathSegment],
        path_references: &mut Option<&mut Vec<ReferenceId>>,
    ) -> NamespaceResolutionResult {
        let starting_mod = self.def_maps[&crate_id].root;
        self.resolve_name_in_module(
            crate_id,
            importing_crate,
            import_path,
            starting_mod,
            true, // plain or crate
            path_references,
        )
    }

    fn resolve_name_in_module(
        &mut self,
        krate: CrateId,
        importing_crate: CrateId,
        import_path: &[PathSegment],
        starting_mod: LocalModuleId,
        plain_or_crate: bool,
        path_references: &mut Option<&mut Vec<ReferenceId>>,
    ) -> NamespaceResolutionResult {
        let def_map = &self.def_maps[&krate];
        let mut current_mod_id = ModuleId { krate, local_id: starting_mod };
        let mut current_mod = &def_map.modules[current_mod_id.local_id.0];

        let mut intermediate_item = IntermediatePathResolutionItem::Module(current_mod_id);

        // There is a possibility that the import path is empty
        // In that case, early return
        if import_path.is_empty() {
            return Ok(NamespaceResolution {
                module_id: current_mod_id,
                item: PathResolutionItem::Module(current_mod_id),
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
            let last_segment_generics = &last_segment.generics;

            let (typ, visibility) = match current_ns.types {
                None => return Err(PathResolutionError::Unresolved(last_ident.clone())),
                Some((typ, visibility, _)) => (typ, visibility),
            };

            // In the type namespace, only Mod can be used in a path.
            (current_mod_id, intermediate_item) = match typ {
                ModuleDefId::ModuleId(id) => {
                    if let Some(path_references) = path_references {
                        path_references.push(ReferenceId::Module(id));
                    }

                    if last_segment_generics.is_some() {
                        errors.push(PathResolutionError::TurbofishNotAllowedOnItem {
                            item: format!("module `{last_ident}`"),
                            span: last_segment.turbofish_span(),
                        });
                    }

                    (id, IntermediatePathResolutionItem::Module(id))
                }
                ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
                // TODO: If impls are ever implemented, types can be used in a path
                ModuleDefId::TypeId(id) => {
                    if let Some(path_references) = path_references {
                        path_references.push(ReferenceId::Struct(id));
                    }

                    (
                        id.module_id(),
                        IntermediatePathResolutionItem::Struct(
                            id,
                            last_segment_generics.as_ref().map(|generics| Turbofish {
                                generics: generics.clone(),
                                span: last_segment.turbofish_span(),
                            }),
                        ),
                    )
                }
                ModuleDefId::TypeAliasId(id) => {
                    if let Some(path_references) = path_references {
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

                    (
                        module_id,
                        IntermediatePathResolutionItem::TypeAlias(
                            id,
                            last_segment_generics.as_ref().map(|generics| Turbofish {
                                generics: generics.clone(),
                                span: last_segment.turbofish_span(),
                            }),
                        ),
                    )
                }
                ModuleDefId::TraitId(id) => {
                    if let Some(path_references) = path_references {
                        path_references.push(ReferenceId::Trait(id));
                    }

                    (
                        id.0,
                        IntermediatePathResolutionItem::Trait(
                            id,
                            last_segment_generics.as_ref().map(|generics| Turbofish {
                                generics: generics.clone(),
                                span: last_segment.turbofish_span(),
                            }),
                        ),
                    )
                }
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

        let module_def_id = current_ns
            .values
            .or(current_ns.types)
            .map(|(id, _, _)| id)
            .expect("Found empty namespace");

        let item = merge_intermediate_path_resolution_item_with_module_def_id(
            intermediate_item,
            module_def_id,
        );

        Ok(NamespaceResolution { module_id: current_mod_id, item, namespace: current_ns, errors })
    }

    fn resolve_external_dep(
        &mut self,
        crate_id: CrateId,
        directive: &ImportDirective,
        path_references: &mut Option<&mut Vec<ReferenceId>>,
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

        self.resolve_path_to_ns(&dep_directive, dep_module.krate, importing_crate, path_references)
    }
}

fn resolve_path_name(import_directive: &ImportDirective) -> Ident {
    match &import_directive.alias {
        None => import_directive.path.last_ident(),
        Some(ident) => ident.clone(),
    }
}

fn merge_intermediate_path_resolution_item_with_module_def_id(
    intermediate_item: IntermediatePathResolutionItem,
    module_def_id: ModuleDefId,
) -> PathResolutionItem {
    match module_def_id {
        ModuleDefId::ModuleId(module_id) => PathResolutionItem::Module(module_id),
        ModuleDefId::TypeId(struct_id) => PathResolutionItem::Struct(struct_id),
        ModuleDefId::TypeAliasId(type_alias_id) => PathResolutionItem::TypeAlias(type_alias_id),
        ModuleDefId::TraitId(trait_id) => PathResolutionItem::Trait(trait_id),
        ModuleDefId::GlobalId(global_id) => PathResolutionItem::Global(global_id),
        ModuleDefId::FunctionId(func_id) => match intermediate_item {
            IntermediatePathResolutionItem::Module(_) => {
                PathResolutionItem::ModuleFunction(func_id)
            }
            IntermediatePathResolutionItem::Struct(struct_id, generics) => {
                PathResolutionItem::StructFunction(struct_id, generics, func_id)
            }
            IntermediatePathResolutionItem::TypeAlias(alias_id, generics) => {
                PathResolutionItem::TypeAliasFunction(alias_id, generics, func_id)
            }
            IntermediatePathResolutionItem::Trait(trait_id, generics) => {
                PathResolutionItem::TraitFunction(trait_id, generics, func_id)
            }
        },
    }
}
