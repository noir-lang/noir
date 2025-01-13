use iter_extended::vecmap;
use noirc_errors::{Location, Span};

use crate::ast::{Ident, Path, PathKind, UnresolvedType};
use crate::hir::def_map::{ModuleData, ModuleDefId, ModuleId, PerNs};
use crate::hir::resolution::import::{resolve_path_kind, PathResolutionError};

use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::visibility::item_in_module_is_visible;

use crate::locations::ReferencesTracker;
use crate::node_interner::{FuncId, GlobalId, StructId, TraitId, TypeAliasId};
use crate::{Shared, Type, TypeAlias};

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
    Module,
    Struct(StructId, Option<Turbofish>),
    TypeAlias(TypeAliasId, Option<Turbofish>),
    Trait(TraitId, Option<Turbofish>),
}

pub(crate) type PathResolutionResult = Result<PathResolution, PathResolutionError>;

enum StructMethodLookupResult {
    /// The method could not be found. There might be trait methods that could be imported,
    /// but none of them are.
    NotFound(Vec<TraitId>),
    /// Found a struct method.
    FoundStructMethod(PerNs),
    /// Found a trait method and it's currently in scope.
    FoundTraitMethod(PerNs, TraitId),
    /// There's only one trait method that matches, but it's not in scope
    /// (we'll warn about this to avoid introducing a large breaking change)
    FoundOneTraitMethodButNotInScope(PerNs, TraitId),
    /// Multiple trait method matches were found and they are all in scope.
    FoundMultipleTraitMethods(Vec<TraitId>),
}

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

        let mut intermediate_item = IntermediatePathResolutionItem::Module;

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

            let current_module_id_is_struct;

            (current_module_id, current_module_id_is_struct, intermediate_item) = match typ {
                ModuleDefId::ModuleId(id) => {
                    if last_segment_generics.is_some() {
                        errors.push(PathResolutionError::TurbofishNotAllowedOnItem {
                            item: format!("module `{last_ident}`"),
                            span: last_segment.turbofish_span(),
                        });
                    }

                    (id, false, IntermediatePathResolutionItem::Module)
                }
                ModuleDefId::TypeId(id) => (
                    id.module_id(),
                    true,
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
                    let Some(module_id) = get_type_alias_module_def_id(&type_alias) else {
                        return Err(PathResolutionError::Unresolved(last_ident.clone()));
                    };

                    (
                        module_id,
                        true,
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
                    false,
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
            let found_ns = if current_module_id_is_struct {
                match self.resolve_struct_function(importing_module, current_module, current_ident)
                {
                    StructMethodLookupResult::NotFound(vec) => {
                        if vec.is_empty() {
                            return Err(PathResolutionError::Unresolved(current_ident.clone()));
                        } else {
                            let traits = vecmap(vec, |trait_id| {
                                let trait_ = self.interner.get_trait(trait_id);
                                self.fully_qualified_trait_path(trait_)
                            });
                            return Err(
                                PathResolutionError::UnresolvedWithPossibleTraitsToImport {
                                    ident: current_ident.clone(),
                                    traits,
                                },
                            );
                        }
                    }
                    StructMethodLookupResult::FoundStructMethod(per_ns) => per_ns,
                    StructMethodLookupResult::FoundTraitMethod(per_ns, trait_id) => {
                        let trait_ = self.interner.get_trait(trait_id);
                        self.usage_tracker.mark_as_used(importing_module, &trait_.name);
                        per_ns
                    }
                    StructMethodLookupResult::FoundOneTraitMethodButNotInScope(
                        per_ns,
                        trait_id,
                    ) => {
                        let trait_ = self.interner.get_trait(trait_id);
                        let trait_name = self.fully_qualified_trait_path(trait_);
                        errors.push(PathResolutionError::TraitMethodNotInScope {
                            ident: current_ident.clone(),
                            trait_name,
                        });
                        per_ns
                    }
                    StructMethodLookupResult::FoundMultipleTraitMethods(vec) => {
                        let traits = vecmap(vec, |trait_id| {
                            let trait_ = self.interner.get_trait(trait_id);
                            self.usage_tracker.mark_as_used(importing_module, &trait_.name);
                            self.fully_qualified_trait_path(trait_)
                        });
                        return Err(PathResolutionError::MultipleTraitsInScope {
                            ident: current_ident.clone(),
                            traits,
                        });
                    }
                }
            } else {
                current_module.find_name(current_ident)
            };
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

    fn resolve_struct_function(
        &self,
        importing_module_id: ModuleId,
        current_module: &ModuleData,
        ident: &Ident,
    ) -> StructMethodLookupResult {
        // If the current module is a struct, next we need to find a function for it.
        // The function could be in the struct itself, or it could be defined in traits.
        let item_scope = current_module.scope();
        let Some(values) = item_scope.values().get(ident) else {
            return StructMethodLookupResult::NotFound(vec![]);
        };

        // First search if the function is defined in the struct itself
        if let Some(item) = values.get(&None) {
            return StructMethodLookupResult::FoundStructMethod(PerNs {
                types: None,
                values: Some(*item),
            });
        }

        // Otherwise, the function could be defined in zero, one or more traits.
        let starting_module = self.get_module(importing_module_id);

        // Gather a list of items for which their trait is in scope.
        let mut results = Vec::new();

        for (trait_id, item) in values.iter() {
            let trait_id = trait_id.expect("The None option was already considered before");
            let trait_ = self.interner.get_trait(trait_id);
            let Some(map) = starting_module.scope().types().get(&trait_.name) else {
                continue;
            };
            let Some(imported_item) = map.get(&None) else {
                continue;
            };
            if imported_item.0 == ModuleDefId::TraitId(trait_id) {
                results.push((trait_id, item));
            }
        }

        if results.is_empty() {
            if values.len() == 1 {
                // This is the backwards-compatible case where there's a single trait method but it's not in scope
                let (trait_id, item) = values.iter().next().expect("Expected an item");
                let trait_id = trait_id.expect("The None option was already considered before");
                let per_ns = PerNs { types: None, values: Some(*item) };
                return StructMethodLookupResult::FoundOneTraitMethodButNotInScope(
                    per_ns, trait_id,
                );
            } else {
                let trait_ids = vecmap(values, |(trait_id, _)| {
                    trait_id.expect("The none option was already considered before")
                });
                return StructMethodLookupResult::NotFound(trait_ids);
            }
        }

        if results.len() > 1 {
            let trait_ids = vecmap(results, |(trait_id, _)| trait_id);
            return StructMethodLookupResult::FoundMultipleTraitMethods(trait_ids);
        }

        let (trait_id, item) = results.remove(0);
        let per_ns = PerNs { types: None, values: Some(*item) };
        StructMethodLookupResult::FoundTraitMethod(per_ns, trait_id)
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
            IntermediatePathResolutionItem::Module => PathResolutionItem::ModuleFunction(func_id),
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

fn get_type_alias_module_def_id(type_alias: &Shared<TypeAlias>) -> Option<ModuleId> {
    let type_alias = type_alias.borrow();

    match &type_alias.typ {
        Type::Struct(struct_id, _generics) => Some(struct_id.borrow().id.module_id()),
        Type::Alias(type_alias, _generics) => get_type_alias_module_def_id(type_alias),
        Type::Error => None,
        _ => {
            // For now we only allow type aliases that point to structs.
            // The more general case is captured here: https://github.com/noir-lang/noir/issues/6398
            panic!("Type alias in path not pointing to struct not yet supported")
        }
    }
}
