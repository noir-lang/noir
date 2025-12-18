//! Path resolution for types, values, and trait methods across modules.

use iter_extended::vecmap;
use noirc_errors::{Located, Location, Span};

use crate::ast::{Ident, PathKind};
use crate::hir::def_map::{ModuleData, ModuleDefId, ModuleId, PerNs};
use crate::hir::resolution::import::{
    PathResolutionError, first_segment_is_always_visible, resolve_path_kind,
};

use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::visibility::item_in_module_is_visible;

use crate::locations::ReferencesTracker;
use crate::node_interner::{FuncId, GlobalId, TraitAssociatedTypeId, TraitId, TypeAliasId, TypeId};
use crate::{Shared, Type, TypeAlias};

use super::Elaborator;
use super::primitive_types::PrimitiveType;
use super::types::SELF_TYPE_NAME;

#[derive(Debug)]
pub(crate) struct PathResolution {
    pub(crate) item: PathResolutionItem,
    pub(crate) errors: Vec<PathResolutionError>,
}

/// All possible items that result from resolving a Path.
/// Note that this item doesn't include the last turbofish in a Path,
/// only intermediate ones, if any.
#[derive(Debug)]
pub(crate) enum PathResolutionItem {
    // These are types
    Module(ModuleId),
    Type(TypeId),
    TypeAlias(TypeAliasId),
    PrimitiveType(PrimitiveType),
    Trait(TraitId),
    TraitAssociatedType(TraitAssociatedTypeId),

    // These are values
    /// A reference to a global value.
    Global(GlobalId),
    /// A function call on a module, for example `some::module::function()`.
    ModuleFunction(FuncId),
    Method(TypeId, Option<Turbofish>, FuncId),
    /// A function call on `Self`, for example `Self::function()`. Turbofish is not allowed here.
    SelfMethod(FuncId),
    /// A function call on a type alias, for example `TypeAlias::function()`.
    TypeAliasFunction(TypeAliasId, Option<Turbofish>, FuncId),
    /// A function call on a trait, for example `Trait::function()` or `Trait::<A, B>::function()`.
    TraitFunction(TraitId, Option<Turbofish>, FuncId),
    /// A function call on a type that resolves to a trait method, for example `SomeType::from(...)`
    /// or `SomeType::<A, B>::from(..).`. The main difference from `TraitFunction` is that this
    /// holds the self type, in this case `SomeType`.
    TypeTraitFunction(Type, TraitId, FuncId),
    /// A function call on a primitive type, for example `u64::from(...)` or `u64::<A, B>::from(..)`.
    PrimitiveFunction(PrimitiveType, Option<Turbofish>, FuncId),
}

impl PathResolutionItem {
    /// Return a [FuncId] if the item refers to some kind of function, otherwise `None`.
    pub(crate) fn function_id(&self) -> Option<FuncId> {
        match self {
            PathResolutionItem::ModuleFunction(func_id)
            | PathResolutionItem::Method(_, _, func_id)
            | PathResolutionItem::SelfMethod(func_id)
            | PathResolutionItem::TypeAliasFunction(_, _, func_id)
            | PathResolutionItem::TraitFunction(_, _, func_id)
            | PathResolutionItem::TypeTraitFunction(_, _, func_id)
            | PathResolutionItem::PrimitiveFunction(_, _, func_id) => Some(*func_id),
            PathResolutionItem::Module(..)
            | PathResolutionItem::Type(..)
            | PathResolutionItem::TypeAlias(..)
            | PathResolutionItem::PrimitiveType(..)
            | PathResolutionItem::Trait(..)
            | PathResolutionItem::TraitAssociatedType(..)
            | PathResolutionItem::Global(..) => None,
        }
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            PathResolutionItem::Module(..) => "module",
            PathResolutionItem::Type(..) => "type",
            PathResolutionItem::TypeAlias(..) => "type alias",
            PathResolutionItem::PrimitiveType(..) => "primitive type",
            PathResolutionItem::Trait(..) => "trait",
            PathResolutionItem::TraitAssociatedType(..) => "associated type",
            PathResolutionItem::Global(..) => "global",
            PathResolutionItem::ModuleFunction(..)
            | PathResolutionItem::Method(..)
            | PathResolutionItem::SelfMethod(..)
            | PathResolutionItem::TypeAliasFunction(..)
            | PathResolutionItem::TraitFunction(..)
            | PathResolutionItem::TypeTraitFunction(..)
            | PathResolutionItem::PrimitiveFunction(..) => "function",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Turbofish {
    pub generics: Vec<Located<Type>>,
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
    /// There's only one trait method that matches, but it's not in scope.
    FoundOneTraitMethodButNotInScope(PerNs, TraitId),
    /// Multiple (ambiguous) trait method matches were found and they are all in scope.
    FoundMultipleTraitMethods(Vec<(TraitId, Ident)>),
}

/// Determines whether data-types found along a path are to be marked as referenced
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

/// Depending on where a path appears in the source code it should either resolve to a type
/// or a value. For example, in `let x: Foo::Bar = Foo::Bar {}` both `Foo::Bar` should resolve to
/// types, never values. On the other hand, in `Foo::Bar()` `Foo::Bar` should resolve to a value,
/// typically a function.
///
/// When using any of the `resolve` methods in this module, items in the target namespace
/// will be returned first if another one exists in the other namespace.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum PathResolutionTarget {
    Type,
    Value,
}

/// Like a [`crate::ast::Path`] but each segment has resolved turbofish types.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypedPath {
    pub segments: Vec<TypedPathSegment>,
    pub kind: PathKind,
    pub location: Location,
    /// The location of `kind` (this is the same as `location` for plain kinds)
    pub kind_location: Location,
}

impl TypedPath {
    /// Construct a [PathKind::Plain] from a number of segments.
    pub fn plain(segments: Vec<TypedPathSegment>, location: Location) -> Self {
        Self { segments, location, kind: PathKind::Plain, kind_location: location }
    }

    /// Removes and returns the last segment.
    ///
    /// Panics if there are no more segments in the path.
    pub fn pop(&mut self) -> TypedPathSegment {
        self.segments.pop().unwrap()
    }

    /// Construct a [PathKind::Plain] from a single identifier name.
    pub fn from_single(name: String, location: Location) -> TypedPath {
        let segment = Ident::from(Located::from(location, name));
        TypedPath::from_ident(segment)
    }

    /// Construct a [PathKind::Plain] from a single identifier segment.
    pub fn from_ident(name: Ident) -> TypedPath {
        let location = name.location();
        let segment = TypedPathSegment { ident: name, generics: None, location };
        TypedPath::plain(vec![segment], location)
    }

    pub fn span(&self) -> Span {
        self.location.span
    }

    /// Returns a clone of the last segment.
    ///
    /// Panics if there are no segments in the path.
    pub fn last_segment(&self) -> TypedPathSegment {
        assert!(!self.segments.is_empty());
        self.segments.last().unwrap().clone()
    }

    /// The [Ident] of the last segment.
    ///
    /// Panics if there are no segments in the path.
    pub fn last_ident(&self) -> Ident {
        self.last_segment().ident
    }

    /// The name of the [Ident] in the first segment.
    ///
    /// Returns `None` if there are no segments in the path.
    pub fn first_name(&self) -> Option<&str> {
        self.segments.first().map(|segment| segment.ident.as_str())
    }

    /// The name of the [Ident] in the last segment.
    ///
    /// Panics if there are no segments in the path.
    pub fn last_name(&self) -> &str {
        assert!(!self.segments.is_empty());
        self.segments.last().unwrap().ident.as_str()
    }

    /// Returns `Some` if the [TypedPath] consists of a single [PathKind::Plain] segment, otherwise `None`.
    pub fn as_single_segment(&self) -> Option<&TypedPathSegment> {
        if self.kind == PathKind::Plain && self.segments.len() == 1 {
            self.segments.first()
        } else {
            None
        }
    }
}

impl std::fmt::Display for TypedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let segments = vecmap(&self.segments, ToString::to_string);
        if self.kind == PathKind::Plain {
            write!(f, "{}", segments.join("::"))
        } else {
            write!(f, "{}::{}", self.kind, segments.join("::"))
        }
    }
}

/// Like a [`crate::ast::PathSegment`] but with resolved turbofish types.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypedPathSegment {
    pub ident: Ident,
    pub generics: Option<Vec<Located<Type>>>,
    pub location: Location,
}

impl TypedPathSegment {
    /// Returns the span where turbofish can happen. For example:
    ///
    /// ```noir
    ///    foo::<T>
    ///       ~^^^^
    /// ```
    ///
    /// Returns an empty [Span] at the end of `foo` if there's no turbofish.
    pub fn turbofish_span(&self) -> Span {
        if self.ident.location().file == self.location.file {
            // The `location` contains both the `ident` and the potential turbofish.
            Span::from(self.ident.span().end()..self.location.span.end())
        } else {
            self.location.span
        }
    }

    /// [Location] of any turbofish in the segment.
    ///
    /// The [Span] will be empty if there was no turbofish.
    pub fn turbofish_location(&self) -> Location {
        Location::new(self.turbofish_span(), self.location.file)
    }

    /// Returns the turbofish if there are generics in the path.
    pub fn turbofish(&self) -> Option<Turbofish> {
        self.generics.as_ref().map(|generics| Turbofish {
            location: self.turbofish_location(),
            generics: generics.clone(),
        })
    }
}

impl std::fmt::Display for TypedPathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ident.fmt(f)?;

        if let Some(generics) = &self.generics {
            let generics = vecmap(generics, |generic| generic.contents.to_string());
            write!(f, "::<{}>", generics.join(", "))?;
        }

        Ok(())
    }
}

impl Elaborator<'_> {
    /// Try to resolve a [TypedPath] into a [PathResolutionItem], marking it as _referenced_.
    pub(super) fn resolve_path_or_error(
        &mut self,
        path: TypedPath,
        target: PathResolutionTarget,
    ) -> Result<PathResolutionItem, ResolverError> {
        self.resolve_path_or_error_inner(path, target, PathResolutionMode::MarkAsReferenced)
    }

    /// Try to resolve a [TypedPath] into a [PathResolutionItem], marking it as _used_.
    pub(super) fn use_path_or_error(
        &mut self,
        path: TypedPath,
        target: PathResolutionTarget,
    ) -> Result<PathResolutionItem, ResolverError> {
        self.resolve_path_or_error_inner(path, target, PathResolutionMode::MarkAsUsed)
    }

    /// Try to resolve a [TypedPath] into a [PathResolutionItem].
    ///
    /// Pushes the `errors` from the [PathResolution], returning only the `item`.
    pub(super) fn resolve_path_or_error_inner(
        &mut self,
        path: TypedPath,
        target: PathResolutionTarget,
        mode: PathResolutionMode,
    ) -> Result<PathResolutionItem, ResolverError> {
        let path_resolution = self.resolve_path_inner(path, target, mode)?;

        self.push_errors(path_resolution.errors);

        Ok(path_resolution.item)
    }

    /// Try to resolve a [TypedPath] into a [PathResolution] with [PathResolutionTarget::Type], marking it as _referenced_.
    pub(super) fn resolve_path_as_type(&mut self, path: TypedPath) -> PathResolutionResult {
        self.resolve_path_inner(
            path,
            PathResolutionTarget::Type,
            PathResolutionMode::MarkAsReferenced,
        )
    }

    /// Try to resolve a [TypedPath] into a [PathResolution] with [PathResolutionTarget::Type], marking it as _used_.
    pub(super) fn use_path_as_type(&mut self, path: TypedPath) -> PathResolutionResult {
        self.resolve_path_inner(path, PathResolutionTarget::Type, PathResolutionMode::MarkAsUsed)
    }

    /// Resolves a path in the current module.
    ///
    /// If the referenced name can't be found, `Err` will be returned. If it can be found, `Ok`
    /// will be returned with a potential vector of errors if, for example, one of the segments
    /// is not accessible from the current module (e.g. because it's private).
    pub(super) fn resolve_path_inner(
        &mut self,
        mut path: TypedPath,
        target: PathResolutionTarget,
        mode: PathResolutionMode,
    ) -> PathResolutionResult {
        let mut module_id = self.module_id();
        let mut intermediate_item = IntermediatePathResolutionItem::Module;

        if path.kind == PathKind::Plain && path.first_name() == Some(SELF_TYPE_NAME) {
            if let Some(Type::DataType(datatype, _)) = &self.self_type {
                let datatype = datatype.borrow();
                if path.segments.len() == 1 {
                    return Ok(PathResolution {
                        item: PathResolutionItem::Type(datatype.id),
                        errors: Vec::new(),
                    });
                }

                module_id = datatype.id.module_id();
                path.segments.remove(0);
                intermediate_item = IntermediatePathResolutionItem::SelfType;
            }
        }

        let last_segment_turbofish_location = path
            .segments
            .last()
            .and_then(|segment| segment.generics.is_some().then(|| segment.turbofish_location()));

        let result = self.resolve_path_in_module(path, module_id, intermediate_item, target, mode);
        let Some(last_segment_turbofish_location) = last_segment_turbofish_location else {
            return result;
        };

        result.map(|mut resolution| {
            match resolution.item {
                PathResolutionItem::Global(..) => {
                    resolution.errors.push(PathResolutionError::TurbofishNotAllowedOnItem {
                        item: "globals".to_string(),
                        location: last_segment_turbofish_location,
                    });
                }
                PathResolutionItem::Module(..) => {
                    resolution.errors.push(PathResolutionError::TurbofishNotAllowedOnItem {
                        item: "modules".to_string(),
                        location: last_segment_turbofish_location,
                    });
                }
                PathResolutionItem::Type(..)
                | PathResolutionItem::TypeAlias(..)
                | PathResolutionItem::PrimitiveType(..)
                | PathResolutionItem::Trait(..)
                | PathResolutionItem::TraitAssociatedType(..)
                | PathResolutionItem::ModuleFunction(..)
                | PathResolutionItem::Method(..)
                | PathResolutionItem::SelfMethod(..)
                | PathResolutionItem::TypeAliasFunction(..)
                | PathResolutionItem::TraitFunction(..)
                | PathResolutionItem::TypeTraitFunction(..)
                | PathResolutionItem::PrimitiveFunction(..) => (),
            }
            resolution
        })
    }

    /// Resolves a [TypedPath].
    ///
    /// `importing_module` is the module where the lookup originally started.
    fn resolve_path_in_module(
        &mut self,
        path: TypedPath,
        importing_module: ModuleId,
        intermediate_item: IntermediatePathResolutionItem,
        target: PathResolutionTarget,
        mode: PathResolutionMode,
    ) -> PathResolutionResult {
        let references_tracker =
            self.interner.is_in_lsp_mode().then(|| ReferencesTracker::new(self.interner));

        let res =
            resolve_path_kind(path.clone(), importing_module, self.def_maps, references_tracker);

        match res {
            Ok((path, module_id, _)) => self.resolve_name_in_module(
                path,
                module_id,
                importing_module,
                intermediate_item,
                target,
                mode,
            ),
            Err(error @ PathResolutionError::Unresolved(_)) => {
                if let Some(result) =
                    self.resolve_primitive_type_or_function(path, importing_module)
                {
                    return result;
                }
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    /// Resolves a [TypedPath] assuming it is inside `starting_module`.
    ///
    /// `importing_module` is the module where the lookup originally started.
    ///
    /// Marks the segments in the path as used or referenced, depending on the [PathResolutionMode].
    /// Pushes errors if segments refer to private items.
    fn resolve_name_in_module(
        &mut self,
        path: TypedPath,
        starting_module: ModuleId,
        importing_module: ModuleId,
        mut intermediate_item: IntermediatePathResolutionItem,
        target: PathResolutionTarget,
        mode: PathResolutionMode,
    ) -> PathResolutionResult {
        // There is a possibility that the import path is empty. In that case, early return.
        if path.segments.is_empty() {
            return Ok(PathResolution {
                item: PathResolutionItem::Module(starting_module),
                errors: Vec::new(),
            });
        }

        let first_segment_is_always_visible =
            first_segment_is_always_visible(&path, importing_module, starting_module);

        // The current module and module ID as we resolve path segments
        let mut current_module_id = starting_module;
        let mut current_module = self.get_module(starting_module);

        let first_segment =
            &path.segments.first().expect("ICE: could not fetch first segment").ident;

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
        for (index, (prev_segment, current_segment)) in
            path.segments.iter().zip(path.segments.iter().skip(1)).enumerate()
        {
            let prev_ident = &prev_segment.ident;
            let current_ident = &current_segment.ident;
            let prev_segment_generics = &prev_segment.generics;

            // We are looking up the `current_segment` in the lookup result of the `prev_segment`.
            let (typ, visibility) = match current_ns.types {
                None => return Err(PathResolutionError::Unresolved(prev_ident.clone())),
                Some((typ, visibility, _)) => (typ, visibility),
            };

            let location = prev_segment.location;
            self.interner.add_module_def_id_reference(
                typ,
                location,
                prev_segment.ident.is_self_type_name(),
            );

            let current_module_id_is_type;

            (current_module_id, current_module_id_is_type, intermediate_item) = match typ {
                ModuleDefId::ModuleId(id) => {
                    if prev_segment_generics.is_some() {
                        errors.push(PathResolutionError::TurbofishNotAllowedOnItem {
                            item: format!("module `{prev_ident}`"),
                            location: prev_segment.turbofish_location(),
                        });
                    }

                    (id, false, IntermediatePathResolutionItem::Module)
                }
                ModuleDefId::TypeId(id) => {
                    let item = IntermediatePathResolutionItem::Type(id, prev_segment.turbofish());
                    (id.module_id(), true, item)
                }
                ModuleDefId::TypeAliasId(id) => {
                    let type_alias = self.interner.get_type_alias(id);
                    let Some(module_id) = get_type_alias_module_def_id(&type_alias) else {
                        return Err(PathResolutionError::Unresolved(prev_ident.clone()));
                    };

                    let item =
                        IntermediatePathResolutionItem::TypeAlias(id, prev_segment.turbofish());
                    (module_id, true, item)
                }
                ModuleDefId::TraitAssociatedTypeId(..) => {
                    // There are no items inside an associated type so we return earlier
                    return Err(PathResolutionError::Unresolved(current_ident.clone()));
                }
                ModuleDefId::TraitId(id) => {
                    let item = IntermediatePathResolutionItem::Trait(id, prev_segment.turbofish());
                    (id.0, false, item)
                }
                ModuleDefId::FunctionId(_) => panic!("functions cannot be in the type namespace"),
                ModuleDefId::GlobalId(_) => panic!("globals cannot be in the type namespace"),
            };

            // If the path is plain or crate, the first segment will always refer to
            // something that's visible from the current module.
            if !((first_segment_is_always_visible && index == 0)
                || item_in_module_is_visible(
                    self.def_maps,
                    importing_module,
                    current_module_id,
                    visibility,
                ))
            {
                errors.push(PathResolutionError::Private(prev_ident.clone()));
            }

            // Switch to the module the current segment is defined in.
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

        let (target_ns, fallback_ns) = match target {
            PathResolutionTarget::Type => (current_ns.types, current_ns.values),
            PathResolutionTarget::Value => (current_ns.values, current_ns.types),
        };

        let (module_def_id, visibility, _) =
            target_ns.or(fallback_ns).expect("A namespace should never be empty");

        let item = self.per_ns_item_to_path_resolution_item(
            path,
            importing_module,
            intermediate_item,
            current_module_id,
            &mut errors,
            module_def_id,
            visibility,
        );

        Ok(PathResolution { item, errors })
    }

    /// Transform a result from [PerNs] into a [PathResolutionItem],
    /// pushing any visibility errors.
    #[allow(clippy::too_many_arguments)]
    fn per_ns_item_to_path_resolution_item(
        &mut self,
        path: TypedPath,
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

        if !item_in_module_is_visible(
            self.def_maps,
            importing_module,
            current_module_id,
            visibility,
        ) {
            errors.push(PathResolutionError::Private(name.clone()));
        }

        item
    }

    /// Assuming that the current path segment is a type or type alias defined in the `current_module`,
    /// resolve the `ident` as a method on that type.
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

        // Gather a vector of items for which their trait is in scope.
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

    /// Try to resolve a path with 1 or 2 segments as a [PathResolutionItem::PrimitiveType] or [PathResolutionItem::PrimitiveFunction].
    ///
    /// If the path consists of 2 segments, use the 2nd segment as the method name and look up a direct method implementation,
    /// or an unambiguous trait method among the traits which are in scope.
    fn resolve_primitive_type_or_function(
        &mut self,
        path: TypedPath,
        importing_module_id: ModuleId,
    ) -> Option<PathResolutionResult> {
        if path.segments.len() != 1 && path.segments.len() != 2 {
            return None;
        }

        let object_name = path.segments[0].ident.as_str();
        let turbofish = path.segments[0].turbofish();
        let primitive_type = PrimitiveType::lookup_by_name(object_name)?;
        let typ = primitive_type.to_type();
        let mut errors = Vec::new();

        if primitive_type == PrimitiveType::StructDefinition {
            errors.push(PathResolutionError::StructDefinitionDeprecated {
                location: path.segments[0].ident.location(),
            });
        }

        if path.segments.len() == 1 {
            let item = PathResolutionItem::PrimitiveType(primitive_type);
            return Some(Ok(PathResolution { item, errors }));
        }

        let method_name_ident = &path.segments[1].ident;
        let method_name = method_name_ident.as_str();

        // Note: the logic here is similar to that of resolve_method, except that that one works by
        // searching through modules, and this one works by searching through primitive types.
        // It would be nice to refactor this to a common logic though it's a bit hard.
        // That said, the logic is "just" searching through direct methods, then through trait methods
        // checking which ones are in scope, and is unlikely to change.

        if let Some(func_id) = self.interner.lookup_direct_method(&typ, method_name, false) {
            let item = PathResolutionItem::PrimitiveFunction(primitive_type, turbofish, func_id);
            return Some(Ok(PathResolution { item, errors }));
        }

        let starting_module = self.get_module(importing_module_id);

        let trait_methods = self.interner.lookup_trait_methods(&typ, method_name, false);

        let mut results = Vec::new();
        for (func_id, trait_id) in &trait_methods {
            if let Some(name) = starting_module.find_trait_in_scope(*trait_id) {
                results.push((*trait_id, *func_id, name));
            };
        }

        if results.is_empty() {
            if trait_methods.len() == 1 {
                // This is the backwards-compatible case where there's a single trait method but it's not in scope
                let (func_id, trait_id) = trait_methods.first().expect("Expected an item");
                let trait_ = self.interner.get_trait(*trait_id);
                let trait_name = self.fully_qualified_trait_path(trait_);
                let ident = method_name_ident.clone();
                errors.push(PathResolutionError::TraitMethodNotInScope { ident, trait_name });
                let item =
                    PathResolutionItem::PrimitiveFunction(primitive_type, turbofish, *func_id);
                return Some(Ok(PathResolution { item, errors }));
            } else if trait_methods.is_empty() {
                return Some(Err(PathResolutionError::Unresolved(method_name_ident.clone())));
            } else {
                let traits = vecmap(trait_methods, |(_, trait_id)| {
                    self.fully_qualified_trait_path(self.interner.get_trait(trait_id))
                });
                let ident = method_name_ident.clone();
                let error =
                    PathResolutionError::UnresolvedWithPossibleTraitsToImport { ident, traits };
                return Some(Err(error));
            }
        }

        if results.len() > 1 {
            let traits = vecmap(results, |(trait_id, _, name)| (trait_id, name.clone()));
            let traits = vecmap(traits, |(trait_id, name)| {
                let trait_ = self.interner.get_trait(trait_id);
                self.usage_tracker.mark_as_used(importing_module_id, &name);
                self.fully_qualified_trait_path(trait_)
            });
            let ident = method_name_ident.clone();
            let error = PathResolutionError::MultipleTraitsInScope { ident, traits };
            return Some(Err(error));
        }

        let (_, func_id, _) = results.remove(0);
        Some(Ok(PathResolution {
            item: PathResolutionItem::PrimitiveFunction(primitive_type, turbofish, func_id),
            errors,
        }))
    }
}

/// Transform a [ModuleDefId] into a [PathResolutionItem].
///
/// If it's a [ModuleDefId::FunctionId], merge it with the [IntermediatePathResolutionItem]
/// representing the item it was found in, such as a module, type, trait, alias, or Self.
fn merge_intermediate_path_resolution_item_with_module_def_id(
    intermediate_item: IntermediatePathResolutionItem,
    module_def_id: ModuleDefId,
) -> PathResolutionItem {
    match module_def_id {
        ModuleDefId::ModuleId(module_id) => PathResolutionItem::Module(module_id),
        ModuleDefId::TypeId(type_id) => PathResolutionItem::Type(type_id),
        ModuleDefId::TypeAliasId(type_alias_id) => PathResolutionItem::TypeAlias(type_alias_id),
        ModuleDefId::TraitId(trait_id) => PathResolutionItem::Trait(trait_id),
        ModuleDefId::TraitAssociatedTypeId(id) => PathResolutionItem::TraitAssociatedType(id),
        ModuleDefId::GlobalId(global_id) => PathResolutionItem::Global(global_id),
        ModuleDefId::FunctionId(func_id) => match intermediate_item {
            IntermediatePathResolutionItem::SelfType => PathResolutionItem::SelfMethod(func_id),
            IntermediatePathResolutionItem::Module => PathResolutionItem::ModuleFunction(func_id),
            IntermediatePathResolutionItem::Type(type_id, generics) => {
                PathResolutionItem::Method(type_id, generics, func_id)
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
