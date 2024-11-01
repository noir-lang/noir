use noirc_errors::{Location, Span};

use crate::ast::{Path, PathKind};
use crate::graph::CrateId;
use crate::hir::def_map::{ModuleDefId, ModuleId};
use crate::hir::resolution::import::{
    IntermediatePathResolutionItem, PathResolution, PathResolutionError, PathResolutionItem,
    PathResolutionResult, Turbofish,
};

use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::visibility::can_reference_module_id;

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
        self.resolve_path_to_ns(path, module_id, module_id.krate)
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_path_to_ns(
        &mut self,
        path: Path,
        starting_module: ModuleId,
        importing_crate: CrateId,
    ) -> PathResolutionResult {
        match path.kind {
            PathKind::Crate => {
                // Resolve from the root of the crate
                self.resolve_path_from_crate_root(path, starting_module.krate, importing_crate)
            }
            PathKind::Plain => {
                // There is a possibility that the import path is empty
                // In that case, early return
                if path.segments.is_empty() {
                    return self.resolve_name_in_module(
                        path,
                        starting_module,
                        importing_crate,
                        true, // plain or crate
                    );
                }

                let def_map = &self.def_maps[&starting_module.krate];
                let current_mod = &def_map.modules[starting_module.local_id.0];
                let first_segment =
                    &path.segments.first().expect("ice: could not fetch first segment").ident;
                if current_mod.find_name(first_segment).is_none() {
                    // Resolve externally when first segment is unresolved
                    return self.resolve_external_dep(path, importing_crate);
                }

                self.resolve_name_in_module(
                    path,
                    starting_module,
                    importing_crate,
                    true, // plain or crate
                )
            }

            PathKind::Dep => self.resolve_external_dep(path, importing_crate),

            PathKind::Super => {
                if let Some(parent_module_id) =
                    self.def_maps[&starting_module.krate].modules[starting_module.local_id.0].parent
                {
                    let starting_module =
                        ModuleId { krate: starting_module.krate, local_id: parent_module_id };
                    self.resolve_name_in_module(
                        path,
                        starting_module,
                        importing_crate,
                        false, // plain or crate
                    )
                } else {
                    let span_start = path.span.start();
                    let span = Span::from(span_start..span_start + 5); // 5 == "super".len()
                    Err(PathResolutionError::NoSuper(span))
                }
            }
        }
    }

    fn resolve_path_from_crate_root(
        &mut self,
        path: Path,
        crate_id: CrateId,
        importing_crate: CrateId,
    ) -> PathResolutionResult {
        let root_module = self.def_maps[&crate_id].root;
        self.resolve_name_in_module(
            path,
            ModuleId { krate: crate_id, local_id: root_module },
            importing_crate,
            true, // plain or crate
        )
    }

    fn resolve_name_in_module(
        &mut self,
        path: Path,
        starting_module: ModuleId,
        importing_crate: CrateId,
        plain_or_crate: bool,
    ) -> PathResolutionResult {
        let def_map = &self.def_maps[&starting_module.krate];
        let mut current_mod_id = starting_module;
        let mut current_mod = &def_map.modules[current_mod_id.local_id.0];

        let mut intermediate_item = IntermediatePathResolutionItem::Module(current_mod_id);

        // There is a possibility that the import path is empty
        // In that case, early return
        if path.segments.is_empty() {
            return Ok(PathResolution {
                item: PathResolutionItem::Module(current_mod_id),
                errors: Vec::new(),
            });
        }

        let first_segment =
            &path.segments.first().expect("ice: could not fetch first segment").ident;
        let mut current_ns = current_mod.find_name(first_segment);
        if current_ns.is_none() {
            return Err(PathResolutionError::Unresolved(first_segment.clone()));
        }

        self.usage_tracker.mark_as_referenced(current_mod_id, first_segment);

        let mut errors = Vec::new();
        for (index, (last_segment, current_segment)) in
            path.segments.iter().zip(path.segments.iter().skip(1)).enumerate()
        {
            let last_ident = &last_segment.ident;
            let current_ident = &current_segment.ident;
            let last_segment_generics = &last_segment.generics;

            let (typ, visibility) = match current_ns.types {
                None => return Err(PathResolutionError::Unresolved(last_ident.clone())),
                Some((typ, visibility, _)) => (typ, visibility),
            };

            let location = Location::new(last_segment.span, self.file);

            // In the type namespace, only Mod can be used in a path.
            (current_mod_id, intermediate_item) = match typ {
                ModuleDefId::ModuleId(id) => {
                    self.interner.add_module_reference(id, location);

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
                    let is_self_type_name = last_segment.ident.is_self_type_name();
                    self.interner.add_struct_reference(id, location, is_self_type_name);

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
                    self.interner.add_alias_reference(id, location);

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
                    let is_self_type_name = last_segment.ident.is_self_type_name();
                    self.interner.add_trait_reference(id, location, is_self_type_name);

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
                    starting_module.local_id,
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

        let last_segment = path.segments.last().unwrap();
        let location = Location::new(last_segment.span, self.file);
        self.interner.add_module_def_id_reference(
            module_def_id,
            location,
            last_segment.ident.is_self_type_name(),
        );

        let item = merge_intermediate_path_resolution_item_with_module_def_id(
            intermediate_item,
            module_def_id,
        );

        let name = &path.segments.last().unwrap().ident;

        let visibility = current_ns
            .values
            .or(current_ns.types)
            .map(|(_, visibility, _)| visibility)
            .expect("Found empty namespace");

        if !(self.self_type_module_id() == Some(current_mod_id)
            || can_reference_module_id(
                self.def_maps,
                starting_module.krate,
                starting_module.local_id,
                current_mod_id,
                visibility,
            ))
        {
            errors.push(PathResolutionError::Private(name.clone()));
        }

        Ok(PathResolution { item, errors })
    }

    fn resolve_external_dep(
        &mut self,
        mut path: Path,
        importing_crate: CrateId,
    ) -> PathResolutionResult {
        // Use extern_prelude to get the dep
        let current_def_map = &self.def_maps[&importing_crate];

        // Fetch the root module from the prelude
        let crate_name = &path.segments.first().unwrap().ident;
        let crate_span = crate_name.span();
        let dep_module = current_def_map
            .extern_prelude
            .get(&crate_name.0.contents)
            .ok_or_else(|| PathResolutionError::Unresolved(crate_name.to_owned()))?;

        // Create an import directive for the dependency crate
        // XXX: This will panic if the path is of the form `use std`. Ideal algorithm will not distinguish between crate and module
        // See `singleton_import.nr` test case for a check that such cases are handled elsewhere.
        path.kind = PathKind::Plain;
        path.segments.remove(0);

        let location = Location::new(crate_span, self.file);
        self.interner.add_module_reference(*dep_module, location);

        self.resolve_path_to_ns(path, *dep_module, importing_crate)
    }

    fn self_type_module_id(&self) -> Option<ModuleId> {
        if let Some(Type::Struct(struct_type, _)) = &self.self_type {
            Some(struct_type.borrow().id.module_id())
        } else {
            None
        }
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
