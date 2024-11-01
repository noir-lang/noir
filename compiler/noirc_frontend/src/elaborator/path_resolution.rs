use noirc_errors::Location;

use crate::ast::{ItemVisibility, Path, PathKind};
use crate::hir::def_map::ModuleId;
use crate::hir::resolution::import::{
    resolve_import, ImportDirective, PathResolution, PathResolutionItem, PathResolutionResult,
};

use crate::hir::resolution::errors::ResolverError;
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
        let resolved_import = resolve_import(
            module_id.krate,
            &import,
            self.interner,
            self.def_maps,
            self.usage_tracker,
            path_references,
        )?;

        Ok(PathResolution { item: resolved_import.item, errors: resolved_import.errors })
    }
}
