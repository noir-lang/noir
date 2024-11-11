use noirc_errors::{Location, Span};

use crate::ast::{Path, PathKind, UnresolvedType};
use crate::hir::def_map::{ModuleDefId, ModuleId};
use crate::hir::resolution::import::{resolve_path_kind, PathResolutionError};

use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::visibility::item_in_module_is_visible;

use crate::locations::ReferencesTracker;
use crate::node_interner::{FuncId, GlobalId, StructId, TraitId, TypeAliasId};
use crate::Type;

use super::types::SELF_TYPE_NAME;
use super::Elaborator;

#[derive(Debug)]
pub(crate) struct PathResolution {
    pub(crate) item: PathResolutionItem,
    pub(crate) errors: Vec<PathResolutionError>,
}

/// All possible items that result from resolving a Path.
/// Note that this item doesn't include the last turbofish in a Path,
/// only intermediate ones, if any.
#[derive(Debug, Clone)]
pub enum PathResolutionItem {
    Module(ModuleId),
    Struct(StructId),
    TypeAlias(TypeAliasId),
    Trait(TraitId),
    Global(GlobalId),
    ModuleFunction(FuncId),
    StructFunction(StructId, Option<Turbofish>, FuncId),
    TypeAliasFunction(TypeAliasId, Option<Turbofish>, FuncId),
    TraitFunction(TraitId, Option<Turbofish>, FuncId),
}

impl PathResolutionItem {
    pub fn function_id(&self) -> Option<FuncId> {
        match self {
            PathResolutionItem::ModuleFunction(func_id)
            | PathResolutionItem::StructFunction(_, _, func_id)
            | PathResolutionItem::TypeAliasFunction(_, _, func_id)
            | PathResolutionItem::TraitFunction(_, _, func_id) => Some(*func_id),
            _ => None,
        }
    }

    pub fn module_id(&self) -> Option<ModuleId> {
        match self {
            Self::Module(module_id) => Some(*module_id),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PathResolutionItem::Module(..) => "module",
            PathResolutionItem::Struct(..) => "type",
            PathResolutionItem::TypeAlias(..) => "type alias",
            PathResolutionItem::Trait(..) => "trait",
            PathResolutionItem::Global(..) => "global",
            PathResolutionItem::ModuleFunction(..)
            | PathResolutionItem::StructFunction(..)
            | PathResolutionItem::TypeAliasFunction(..)
            | PathResolutionItem::TraitFunction(..) => "function",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Turbofish {
    pub generics: Vec<UnresolvedType>,
    pub span: Span,
}

/// Any item that can appear before the last segment in a path.
#[derive(Debug)]
enum IntermediatePathResolutionItem {
    Module(ModuleId),
    Struct(StructId, Option<Turbofish>),
    TypeAlias(TypeAliasId, Option<Turbofish>),
    Trait(TraitId, Option<Turbofish>),
}

pub(crate) type PathResolutionResult = Result<PathResolution, PathResolutionError>;

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

    /// Resolves a path in the current module.
    /// If the referenced name can't be found, `Err` will be returned. If it can be found, `Ok`
    /// will be returned with a potential list of errors if, for example, one of the segments
    /// is not accessible from the current module (e.g. because it's private).
    pub(super) fn resolve_path(&mut self, mut path: Path) -> PathResolutionResult {
        let mut module_id = self.module_id();

        if path.kind == PathKind::Plain && path.first_name() == Some(SELF_TYPE_NAME) {
            if let Some(Type::Struct(struct_type, _)) = &self.self_type {
                let struct_type = struct_type.borrow();
                if path.segments.len() == 1 {
                    return Ok(PathResolution {
                        item: PathResolutionItem::Struct(struct_type.id),
                        errors: Vec::new(),
                    });
                }

                module_id = struct_type.id.module_id();
                path.segments.remove(0);
            }
        }

        self.resolve_path_in_module(path, module_id)
    }

    /// Resolves a path in `current_module`.
    /// `importing_module` is the module where the lookup originally started.
    fn resolve_path_in_module(
        &mut self,
        path: Path,
        importing_module: ModuleId,
    ) -> PathResolutionResult {
        let references_tracker = if self.interner.is_in_lsp_mode() {
            Some(ReferencesTracker::new(self.interner, self.file))
        } else {
            None
        };
        let (path, module_id, _) =
            resolve_path_kind(path, importing_module, self.def_maps, references_tracker)?;
        self.resolve_name_in_module(path, module_id, importing_module)
    }

    /// Resolves a Path assuming we are inside `starting_module`.
    /// `importing_module` is the module where the lookup originally started.
    fn resolve_name_in_module(
        &mut self,
        path: Path,
        starting_module: ModuleId,
        importing_module: ModuleId,
    ) -> PathResolutionResult {
        // There is a possibility that the import path is empty. In that case, early return.
        if path.segments.is_empty() {
            return Ok(PathResolution {
                item: PathResolutionItem::Module(starting_module),
                errors: Vec::new(),
            });
        }

        let plain_or_crate = matches!(path.kind, PathKind::Plain | PathKind::Crate);

        // The current module and module ID as we resolve path segments
        let mut current_module_id = starting_module;
        let mut current_module = self.get_module(starting_module);

        let mut intermediate_item = IntermediatePathResolutionItem::Module(current_module_id);

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
            let last_segment_generics = &last_segment.generics;

            let (typ, visibility) = match current_ns.types {
                None => return Err(PathResolutionError::Unresolved(last_ident.clone())),
                Some((typ, visibility, _)) => (typ, visibility),
            };

            let location = Location::new(last_segment.span, self.file);
            self.interner.add_module_def_id_reference(
                typ,
                location,
                last_segment.ident.is_self_type_name(),
            );

            (current_module_id, intermediate_item) = match typ {
                ModuleDefId::ModuleId(id) => {
                    if last_segment_generics.is_some() {
                        errors.push(PathResolutionError::TurbofishNotAllowedOnItem {
                            item: format!("module `{last_ident}`"),
                            span: last_segment.turbofish_span(),
                        });
                    }

                    (id, IntermediatePathResolutionItem::Module(id))
                }
                ModuleDefId::TypeId(id) => (
                    id.module_id(),
                    IntermediatePathResolutionItem::Struct(
                        id,
                        last_segment_generics.as_ref().map(|generics| Turbofish {
                            generics: generics.clone(),
                            span: last_segment.turbofish_span(),
                        }),
                    ),
                ),
                ModuleDefId::TypeAliasId(id) => {
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
                ModuleDefId::TraitId(id) => (
                    id.0,
                    IntermediatePathResolutionItem::Trait(
                        id,
                        last_segment_generics.as_ref().map(|generics| Turbofish {
                            generics: generics.clone(),
                            span: last_segment.turbofish_span(),
                        }),
                    ),
                ),
                ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
                ModuleDefId::GlobalId(_) => panic!("globals cannot be in the type namespace"),
            };

            // If the path is plain or crate, the first segment will always refer to
            // something that's visible from the current module.
            if !((plain_or_crate && index == 0)
                || item_in_module_is_visible(
                    self.def_maps,
                    importing_module,
                    current_module_id,
                    visibility,
                ))
            {
                errors.push(PathResolutionError::Private(last_ident.clone()));
            }

            current_module = self.get_module(current_module_id);

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

        let name = path.last_ident();
        let is_self_type = name.is_self_type_name();
        let location = Location::new(name.span(), self.file);
        self.interner.add_module_def_id_reference(module_def_id, location, is_self_type);

        let item = merge_intermediate_path_resolution_item_with_module_def_id(
            intermediate_item,
            module_def_id,
        );

        if !(self.self_type_module_id() == Some(current_module_id)
            || item_in_module_is_visible(
                self.def_maps,
                importing_module,
                current_module_id,
                visibility,
            ))
        {
            errors.push(PathResolutionError::Private(name.clone()));
        }

        Ok(PathResolution { item, errors })
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
