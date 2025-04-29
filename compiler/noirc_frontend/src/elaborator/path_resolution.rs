use iter_extended::vecmap;
use noirc_errors::Location;

use crate::ast::{Ident, Path, PathKind, UnresolvedType};
use crate::hir::def_map::{ModuleData, ModuleDefId, ModuleId, PerNs};
use crate::hir::resolution::import::{PathResolutionError, resolve_path_kind};

use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::visibility::item_in_module_is_visible;

use crate::locations::ReferencesTracker;
use crate::node_interner::{FuncId, GlobalId, TraitId, TypeAliasId, TypeId};
use crate::{Shared, Type, TypeAlias};

use super::Elaborator;
use super::types::SELF_TYPE_NAME;

#[derive(Debug)]
pub(crate) struct PathResolution {
    pub(crate) item: PathResolutionItem,
    pub(crate) errors: Vec<PathResolutionError>,
}

#[derive(Debug)]
pub(crate) enum PathResolutionItem {
    Type(PathResolutionTypeItem),
    Value(PathResolutionValueItem),
    TypeAndValue(PathResolutionTypeItem, PathResolutionValueItem),
}

impl PathResolutionItem {
    pub(crate) fn into_type(self) -> Option<PathResolutionTypeItem> {
        match self {
            Self::Type(typ) | Self::TypeAndValue(typ, _) => Some(typ),
            _ => None,
        }
    }

    pub(crate) fn as_type(&self) -> Option<&PathResolutionTypeItem> {
        match self {
            Self::Type(typ) | Self::TypeAndValue(typ, _) => Some(typ),
            _ => None,
        }
    }

    pub(crate) fn into_value(self) -> Option<PathResolutionValueItem> {
        match self {
            Self::Value(value) | Self::TypeAndValue(_, value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_value(&self) -> Option<&PathResolutionValueItem> {
        match self {
            Self::Value(value) | Self::TypeAndValue(_, value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn function_id(&self) -> Option<FuncId> {
        self.as_value().and_then(|item| item.function_id())
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            Self::Type(item) => item.description(),
            Self::Value(item) => item.description(),
            Self::TypeAndValue(item, _) => {
                // Any of the two items can be used to describe the resolution.
                item.description()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PathResolutionTypeItem {
    Module(ModuleId),
    Type(TypeId),
    TypeAlias(TypeAliasId),
    Trait(TraitId),
}

impl PathResolutionTypeItem {
    pub(crate) fn description(&self) -> &'static str {
        match self {
            Self::Module(..) => "module",
            Self::Type(..) => "type",
            Self::TypeAlias(..) => "type alias",
            Self::Trait(..) => "trait",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PathResolutionValueItem {
    Global(GlobalId),
    ModuleFunction(FuncId),
    Method(TypeId, Option<Turbofish>, FuncId),
    SelfMethod(FuncId),
    TypeAliasFunction(TypeAliasId, Option<Turbofish>, FuncId),
    TraitFunction(TraitId, Option<Turbofish>, FuncId),
}

impl PathResolutionValueItem {
    pub(crate) fn function_id(&self) -> Option<FuncId> {
        match self {
            Self::ModuleFunction(func_id)
            | Self::Method(_, _, func_id)
            | Self::SelfMethod(func_id)
            | Self::TypeAliasFunction(_, _, func_id)
            | Self::TraitFunction(_, _, func_id) => Some(*func_id),
            Self::Global(..) => None,
        }
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            Self::Global(..) => "global",
            Self::ModuleFunction(..)
            | Self::Method(..)
            | Self::SelfMethod(..)
            | Self::TypeAliasFunction(..)
            | Self::TraitFunction(..) => "function",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Turbofish {
    pub generics: Vec<UnresolvedType>,
    pub location: Location,
}

/// Any item that can appear before the last segment in a path.
#[derive(Debug, Clone)]
enum IntermediatePathResolutionItem {
    SelfType,
    Module,
    Type(TypeId, Option<Turbofish>),
    TypeAlias(TypeAliasId, Option<Turbofish>),
    Trait(TraitId, Option<Turbofish>),
}

pub(crate) type PathResolutionResult = Result<PathResolution, PathResolutionError>;

enum MethodLookupResult {
    /// The method could not be found. There might be trait methods that could be imported,
    /// but none of them are.
    NotFound(Vec<TraitId>),
    /// Found a method.
    FoundMethod(PerNs),
    /// Found a trait method and it's currently in scope.
    FoundTraitMethod(PerNs, Ident),
    /// There's only one trait method that matches, but it's not in scope
    /// (we'll warn about this to avoid introducing a large breaking change)
    FoundOneTraitMethodButNotInScope(PerNs, TraitId),
    /// Multiple trait method matches were found and they are all in scope.
    FoundMultipleTraitMethods(Vec<(TraitId, Ident)>),
}

/// Determines whether datatypes found along a path are to be marked as referenced
/// or used (see [`crate::usage_tracker::UsageTracker::mark_as_referenced`]
/// and [`crate::usage_tracker::UsageTracker::mark_as_used`])
///
/// For example, a struct `Foo` won't be marked as used (just as referenced) if it
/// mentioned in a function parameter:
///
/// ```noir
/// fn method(foo: Foo) {}
/// ```
///
/// However, if it's used in a return type it will be marked as used, even if
/// it's not explicitly constructed:
///
/// ```noir
/// fn method() -> Foo {
///     std::mem::zeroed()
/// }
/// ```
///
/// Or, for example, a struct used in a impl or trait impl won't be marked as used:
///
/// ```noir
/// impl Foo {}
/// impl Trait for Foo {}
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum PathResolutionMode {
    MarkAsReferenced,
    MarkAsUsed,
}

impl Elaborator<'_> {
    pub(super) fn resolve_path_or_error(
        &mut self,
        path: Path,
    ) -> Result<PathResolutionItem, ResolverError> {
        self.resolve_path_or_error_inner(path, PathResolutionMode::MarkAsReferenced)
    }

    pub(super) fn use_path_or_error(
        &mut self,
        path: Path,
    ) -> Result<PathResolutionItem, ResolverError> {
        self.resolve_path_or_error_inner(path, PathResolutionMode::MarkAsUsed)
    }

    pub(super) fn resolve_path_or_error_inner(
        &mut self,
        path: Path,
        mode: PathResolutionMode,
    ) -> Result<PathResolutionItem, ResolverError> {
        let path_resolution = self.resolve_path_inner(path, mode)?;

        for error in path_resolution.errors {
            self.push_err(error);
        }

        Ok(path_resolution.item)
    }

    pub(super) fn resolve_path(&mut self, path: Path) -> PathResolutionResult {
        self.resolve_path_inner(path, PathResolutionMode::MarkAsReferenced)
    }

    pub(super) fn use_path(&mut self, path: Path) -> PathResolutionResult {
        self.resolve_path_inner(path, PathResolutionMode::MarkAsUsed)
    }

    /// Resolves a path in the current module.
    /// If the referenced name can't be found, `Err` will be returned. If it can be found, `Ok`
    /// will be returned with a potential list of errors if, for example, one of the segments
    /// is not accessible from the current module (e.g. because it's private).
    pub(super) fn resolve_path_inner(
        &mut self,
        mut path: Path,
        mode: PathResolutionMode,
    ) -> PathResolutionResult {
        let mut module_id = self.module_id();
        let mut intermediate_item = IntermediatePathResolutionItem::Module;

        if path.kind == PathKind::Plain && path.first_name() == Some(SELF_TYPE_NAME) {
            if let Some(Type::DataType(datatype, _)) = &self.self_type {
                let datatype = datatype.borrow();
                if path.segments.len() == 1 {
                    return Ok(PathResolution {
                        item: PathResolutionItem::Type(PathResolutionTypeItem::Type(datatype.id)),
                        errors: Vec::new(),
                    });
                }

                module_id = datatype.id.module_id();
                path.segments.remove(0);
                intermediate_item = IntermediatePathResolutionItem::SelfType;
            }
        }

        self.resolve_path_in_module(path, module_id, intermediate_item, mode)
    }

    /// Resolves a path in `current_module`.
    /// `importing_module` is the module where the lookup originally started.
    fn resolve_path_in_module(
        &mut self,
        path: Path,
        importing_module: ModuleId,
        intermediate_item: IntermediatePathResolutionItem,
        mode: PathResolutionMode,
    ) -> PathResolutionResult {
        let references_tracker = if self.interner.is_in_lsp_mode() {
            Some(ReferencesTracker::new(self.interner))
        } else {
            None
        };
        let (path, module_id, _) =
            resolve_path_kind(path, importing_module, self.def_maps, references_tracker)?;
        self.resolve_name_in_module(path, module_id, importing_module, intermediate_item, mode)
    }

    /// Resolves a Path assuming we are inside `starting_module`.
    /// `importing_module` is the module where the lookup originally started.
    fn resolve_name_in_module(
        &mut self,
        path: Path,
        starting_module: ModuleId,
        importing_module: ModuleId,
        mut intermediate_item: IntermediatePathResolutionItem,
        mode: PathResolutionMode,
    ) -> PathResolutionResult {
        // There is a possibility that the import path is empty. In that case, early return.
        if path.segments.is_empty() {
            return Ok(PathResolution {
                item: PathResolutionItem::Type(PathResolutionTypeItem::Module(starting_module)),
                errors: Vec::new(),
            });
        }

        let plain_or_crate = matches!(path.kind, PathKind::Plain | PathKind::Crate);

        // The current module and module ID as we resolve path segments
        let mut current_module_id = starting_module;
        let mut current_module = self.get_module(starting_module);

        let first_segment =
            &path.segments.first().expect("ice: could not fetch first segment").ident;
        let mut current_ns = current_module.find_name(first_segment);
        if current_ns.is_none() {
            return Err(PathResolutionError::Unresolved(first_segment.clone()));
        }

        match mode {
            PathResolutionMode::MarkAsReferenced => {
                self.usage_tracker.mark_as_referenced(current_module_id, first_segment);
            }
            PathResolutionMode::MarkAsUsed => {
                self.usage_tracker.mark_as_used(current_module_id, first_segment);
            }
        }

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

            let location = last_segment.location;
            self.interner.add_module_def_id_reference(
                typ,
                location,
                last_segment.ident.is_self_type_name(),
            );

            let current_module_id_is_type;

            (current_module_id, current_module_id_is_type, intermediate_item) = match typ {
                ModuleDefId::ModuleId(id) => {
                    if last_segment_generics.is_some() {
                        errors.push(PathResolutionError::TurbofishNotAllowedOnItem {
                            item: format!("module `{last_ident}`"),
                            location: last_segment.turbofish_location(),
                        });
                    }

                    (id, false, IntermediatePathResolutionItem::Module)
                }
                ModuleDefId::TypeId(id) => {
                    let item = IntermediatePathResolutionItem::Type(id, last_segment.turbofish());
                    (id.module_id(), true, item)
                }
                ModuleDefId::TypeAliasId(id) => {
                    let type_alias = self.interner.get_type_alias(id);
                    let Some(module_id) = get_type_alias_module_def_id(&type_alias) else {
                        return Err(PathResolutionError::Unresolved(last_ident.clone()));
                    };

                    let item =
                        IntermediatePathResolutionItem::TypeAlias(id, last_segment.turbofish());
                    (module_id, true, item)
                }
                ModuleDefId::TraitId(id) => {
                    let item = IntermediatePathResolutionItem::Trait(id, last_segment.turbofish());
                    (id.0, false, item)
                }
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
            let found_ns = if current_module_id_is_type {
                match self.resolve_method(importing_module, current_module, current_ident) {
                    MethodLookupResult::NotFound(vec) => {
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
                    MethodLookupResult::FoundMethod(per_ns) => per_ns,
                    MethodLookupResult::FoundTraitMethod(per_ns, name) => {
                        self.usage_tracker.mark_as_used(importing_module, &name);
                        per_ns
                    }
                    MethodLookupResult::FoundOneTraitMethodButNotInScope(per_ns, trait_id) => {
                        let trait_ = self.interner.get_trait(trait_id);
                        let trait_name = self.fully_qualified_trait_path(trait_);
                        errors.push(PathResolutionError::TraitMethodNotInScope {
                            ident: current_ident.clone(),
                            trait_name,
                        });
                        per_ns
                    }
                    MethodLookupResult::FoundMultipleTraitMethods(vec) => {
                        let traits = vecmap(vec, |(trait_id, name)| {
                            let trait_ = self.interner.get_trait(trait_id);
                            self.usage_tracker.mark_as_used(importing_module, &name);
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

            match mode {
                PathResolutionMode::MarkAsReferenced => {
                    self.usage_tracker.mark_as_referenced(current_module_id, current_ident);
                }
                PathResolutionMode::MarkAsUsed => {
                    self.usage_tracker.mark_as_used(current_module_id, current_ident);
                }
            }

            current_ns = found_ns;
        }

        let type_item = current_ns.types.map(|(module_def_id, visibility, ..)| {
            self.per_ns_item_to_path_resolution_item(
                path.clone(),
                importing_module,
                intermediate_item.clone(),
                current_module_id,
                &mut errors,
                module_def_id,
                visibility,
            )
        });

        let value_item = current_ns.values.map(|(module_def_id, visibility, ..)| {
            self.per_ns_item_to_path_resolution_item(
                path,
                importing_module,
                intermediate_item,
                current_module_id,
                &mut errors,
                module_def_id,
                visibility,
            )
        });

        let type_item = type_item.map(|type_item| match type_item {
            PathResolutionItem::Type(item) | PathResolutionItem::TypeAndValue(item, _) => item,
            PathResolutionItem::Value(_) => {
                unreachable!("A type should have been produced from the type namespace")
            }
        });
        let value_item = value_item.map(|value_item| match value_item {
            PathResolutionItem::Value(item) | PathResolutionItem::TypeAndValue(_, item) => item,
            PathResolutionItem::Type(_) => {
                unreachable!("A value should have been produced from the value namespace")
            }
        });
        let item = match (type_item, value_item) {
            (Some(type_item), None) => PathResolutionItem::Type(type_item),
            (None, Some(value_item)) => PathResolutionItem::Value(value_item),
            (Some(type_item), Some(value_item)) => {
                PathResolutionItem::TypeAndValue(type_item, value_item)
            }
            (None, None) => unreachable!("Found an empty namespace"),
        };

        Ok(PathResolution { item, errors })
    }

    #[allow(clippy::too_many_arguments)]
    fn per_ns_item_to_path_resolution_item(
        &mut self,
        path: Path,
        importing_module: ModuleId,
        intermediate_item: IntermediatePathResolutionItem,
        current_module_id: ModuleId,
        errors: &mut Vec<PathResolutionError>,
        module_def_id: ModuleDefId,
        visibility: crate::ast::ItemVisibility,
    ) -> PathResolutionItem {
        let name = path.last_ident();
        let is_self_type = name.is_self_type_name();
        let location = name.location();
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
        item
    }

    fn self_type_module_id(&self) -> Option<ModuleId> {
        if let Some(Type::DataType(datatype, _)) = &self.self_type {
            Some(datatype.borrow().id.module_id())
        } else {
            None
        }
    }

    fn resolve_method(
        &self,
        importing_module_id: ModuleId,
        current_module: &ModuleData,
        ident: &Ident,
    ) -> MethodLookupResult {
        // If the current module is a type, next we need to find a function for it.
        // The function could be in the type itself, or it could be defined in traits.
        let item_scope = current_module.scope();
        let Some(values) = item_scope.values().get(ident) else {
            return MethodLookupResult::NotFound(vec![]);
        };

        // First search if the function is defined in the type itself
        if let Some(item) = values.get(&None) {
            return MethodLookupResult::FoundMethod(PerNs { types: None, values: Some(*item) });
        }

        // Otherwise, the function could be defined in zero, one or more traits.
        let starting_module = self.get_module(importing_module_id);

        // Gather a list of items for which their trait is in scope.
        let mut results = Vec::new();

        for (trait_id, item) in values.iter() {
            let trait_id = trait_id.expect("The None option was already considered before");
            if let Some(name) = starting_module.find_trait_in_scope(trait_id) {
                results.push((trait_id, name, item));
            };
        }

        if results.is_empty() {
            if values.len() == 1 {
                // This is the backwards-compatible case where there's a single trait method but it's not in scope
                let (trait_id, item) = values.iter().next().expect("Expected an item");
                let trait_id = trait_id.expect("The None option was already considered before");
                let per_ns = PerNs { types: None, values: Some(*item) };
                return MethodLookupResult::FoundOneTraitMethodButNotInScope(per_ns, trait_id);
            } else {
                let trait_ids = vecmap(values, |(trait_id, _)| {
                    trait_id.expect("The none option was already considered before")
                });
                return MethodLookupResult::NotFound(trait_ids);
            }
        }

        if results.len() > 1 {
            let trait_ids = vecmap(results, |(trait_id, name, _)| (trait_id, name.clone()));
            return MethodLookupResult::FoundMultipleTraitMethods(trait_ids);
        }

        let (_, name, item) = results.remove(0);
        let per_ns = PerNs { types: None, values: Some(*item) };
        MethodLookupResult::FoundTraitMethod(per_ns, name.clone())
    }
}

fn merge_intermediate_path_resolution_item_with_module_def_id(
    intermediate_item: IntermediatePathResolutionItem,
    module_def_id: ModuleDefId,
) -> PathResolutionItem {
    match module_def_id {
        ModuleDefId::ModuleId(module_id) => {
            PathResolutionItem::Type(PathResolutionTypeItem::Module(module_id))
        }
        ModuleDefId::TypeId(type_id) => {
            PathResolutionItem::Type(PathResolutionTypeItem::Type(type_id))
        }
        ModuleDefId::TypeAliasId(type_alias_id) => {
            PathResolutionItem::Type(PathResolutionTypeItem::TypeAlias(type_alias_id))
        }
        ModuleDefId::TraitId(trait_id) => {
            PathResolutionItem::Type(PathResolutionTypeItem::Trait(trait_id))
        }
        ModuleDefId::GlobalId(global_id) => {
            PathResolutionItem::Value(PathResolutionValueItem::Global(global_id))
        }
        ModuleDefId::FunctionId(func_id) => match intermediate_item {
            IntermediatePathResolutionItem::SelfType => {
                PathResolutionItem::Value(PathResolutionValueItem::SelfMethod(func_id))
            }
            IntermediatePathResolutionItem::Module => {
                PathResolutionItem::Value(PathResolutionValueItem::ModuleFunction(func_id))
            }
            IntermediatePathResolutionItem::Type(type_id, generics) => PathResolutionItem::Value(
                PathResolutionValueItem::Method(type_id, generics, func_id),
            ),
            IntermediatePathResolutionItem::TypeAlias(alias_id, generics) => {
                PathResolutionItem::Value(PathResolutionValueItem::TypeAliasFunction(
                    alias_id, generics, func_id,
                ))
            }
            IntermediatePathResolutionItem::Trait(trait_id, generics) => PathResolutionItem::Value(
                PathResolutionValueItem::TraitFunction(trait_id, generics, func_id),
            ),
        },
    }
}

fn get_type_alias_module_def_id(type_alias: &Shared<TypeAlias>) -> Option<ModuleId> {
    let type_alias = type_alias.borrow();

    match &type_alias.typ {
        Type::DataType(type_id, _generics) => Some(type_id.borrow().id.module_id()),
        Type::Alias(type_alias, _generics) => get_type_alias_module_def_id(type_alias),
        Type::Error => None,
        _ => {
            // For now we only allow type aliases that point to data types.
            // The more general case is captured here: https://github.com/noir-lang/noir/issues/6398
            panic!("Type alias in path not pointing to a data type is not yet supported")
        }
    }
}
