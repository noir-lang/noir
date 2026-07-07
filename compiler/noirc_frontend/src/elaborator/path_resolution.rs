//! Path resolution for types, values, and trait methods across modules.

use iter_extended::vecmap;
use itertools::Itertools;
use noirc_errors::{Located, Location, Span};

use crate::ast::{Ident, PathKind};
use crate::hir::def_map::{ModuleData, ModuleDefId, ModuleId, Namespace, NamespaceItem, PerNs};
use crate::hir::resolution::import::{
    PathResolutionError, first_segment_is_always_visible, resolve_path_kind,
};

use crate::hir::resolution::errors::ResolverError;
use crate::hir::resolution::visibility::{
    item_in_module_is_visible, trait_visibility_for_method_is_satisfied,
};

use crate::hir_def::traits::NamedType;
use crate::locations::ReferencesTracker;
use crate::node_interner::{
    DefinitionId, FuncId, GlobalId, NodeInterner, TraitAssociatedTypeId, TraitId, TraitLookupMode,
    TypeAliasId, TypeId,
};
use crate::{Kind, Shared, Type, TypeAlias};

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
    /// A fieldless enum variant such as `Foo::Spam`, which is lowered to a global. Unlike an
    /// ordinary global it may carry the enum's generics, so a turbofish is allowed (e.g.
    /// `Foo::Spam::<u32>`); the generics are bound when the variable is elaborated.
    EnumVariant(GlobalId),
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
    /// An associated constant accessed via `Type::CONSTANT` syntax, for example `Foo::N`.
    TraitConstant(TypeId, TraitId, DefinitionId),
}

impl PathResolutionItem {
    /// Return a [`FuncId`] if the item refers to some kind of function, otherwise `None`.
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
            | PathResolutionItem::Global(..)
            | PathResolutionItem::EnumVariant(..)
            | PathResolutionItem::TraitConstant(..) => None,
        }
    }

    pub(crate) fn description(&self, interner: &NodeInterner) -> String {
        match self {
            PathResolutionItem::Module(module) => {
                let module_data = interner.try_module_attributes(*module);
                module_data
                    .map_or_else(|| "module".to_string(), |data| format!("module `{}`", data.name))
            }
            PathResolutionItem::Type(type_id) => {
                let datatype = interner.get_type(*type_id);
                let datatype = datatype.borrow();
                if datatype.is_enum() {
                    format!("enum `{}`", datatype.name)
                } else {
                    format!("struct `{}`", datatype.name)
                }
            }
            PathResolutionItem::TypeAlias(type_alias_id) => {
                let type_alias = interner.get_type_alias(*type_alias_id);
                let type_alias = type_alias.borrow();
                format!("type alias `{}`", type_alias.name)
            }
            PathResolutionItem::PrimitiveType(kind) => {
                format!("primitive type `{}`", kind.name())
            }
            PathResolutionItem::Trait(trait_id) => {
                let trait_ = interner.get_trait(*trait_id);
                format!("trait `{}`", trait_.name)
            }
            PathResolutionItem::TraitAssociatedType(id) => {
                let associated_type = interner.get_trait_associated_type(*id);
                let trait_ = interner.get_trait(associated_type.trait_id);
                format!("associated type `{}::{}`", trait_.name, associated_type.name)
            }
            PathResolutionItem::Global(id) => {
                let global = interner.get_global_definition(*id);
                format!("global `{}`", global.name)
            }
            PathResolutionItem::EnumVariant(id) => {
                let global = interner.get_global_definition(*id);
                format!("enum variant `{}`", global.name)
            }
            PathResolutionItem::ModuleFunction(func_id)
            | PathResolutionItem::Method(_, _, func_id)
            | PathResolutionItem::SelfMethod(func_id)
            | PathResolutionItem::TypeAliasFunction(_, _, func_id)
            | PathResolutionItem::TraitFunction(_, _, func_id)
            | PathResolutionItem::TypeTraitFunction(_, _, func_id)
            | PathResolutionItem::PrimitiveFunction(_, _, func_id) => {
                let name = interner.function_name(func_id);
                format!("function `{name}`")
            }
            PathResolutionItem::TraitConstant(type_id, trait_id, def_id) => {
                let datatype = interner.get_type(*type_id);
                let datatype = datatype.borrow();
                let trait_ = interner.get_trait(*trait_id);
                let def_info = interner.definition(*def_id);
                format!(
                    "associated constant `{}` from trait `{}` on type `{}`",
                    def_info.name, trait_.name, datatype.name
                )
            }
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
    /// Construct a [`PathKind::Plain`] from a number of segments.
    pub fn plain(segments: Vec<TypedPathSegment>, location: Location) -> Self {
        Self { segments, location, kind: PathKind::Plain, kind_location: location }
    }

    /// Removes and returns the last segment.
    ///
    /// Panics if there are no more segments in the path.
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn pop(&mut self) -> TypedPathSegment {
        self.segments.pop().unwrap()
    }

    /// Construct a [`PathKind::Plain`] from a single identifier name.
    pub fn from_single(name: String, location: Location) -> TypedPath {
        let segment = Ident::from(Located::from(location, name));
        TypedPath::from_ident(segment)
    }

    /// Construct a [`PathKind::Plain`] from a single identifier segment.
    pub fn from_ident(name: Ident) -> TypedPath {
        let location = name.location();
        let segment = TypedPathSegment::without_generics(name, location);
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

    /// Returns `Some` if the [`TypedPath`] consists of a single [`PathKind::Plain`] segment, otherwise `None`.
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
    pub(crate) fn without_generics(ident: Ident, location: Location) -> TypedPathSegment {
        TypedPathSegment { ident, generics: None, location }
    }

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
    /// Try to resolve a [`TypedPath`] into a [`PathResolutionItem`], marking it as _referenced_.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_path_or_error(
        &mut self,
        path: TypedPath,
        target: PathResolutionTarget,
    ) -> Result<PathResolutionItem, ResolverError> {
        self.resolve_path_or_error_inner(path, target, PathResolutionMode::MarkAsReferenced)
    }

    /// Try to resolve a [`TypedPath`] into a [`PathResolutionItem`], marking it as _used_.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn use_path_or_error(
        &mut self,
        path: TypedPath,
        target: PathResolutionTarget,
    ) -> Result<PathResolutionItem, ResolverError> {
        self.resolve_path_or_error_inner(path, target, PathResolutionMode::MarkAsUsed)
    }

    /// Try to resolve a [`TypedPath`] into a [`PathResolutionItem`].
    ///
    /// Pushes the `errors` from the [PathResolution], returning only the `item`.
    #[tracing::instrument(level = "trace", skip_all)]
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

    /// Try to resolve a [`TypedPath`] into a [PathResolution] with [`PathResolutionTarget::Type`], marking it as _referenced_.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_path_as_type(&mut self, path: TypedPath) -> PathResolutionResult {
        self.resolve_path_inner(
            path,
            PathResolutionTarget::Type,
            PathResolutionMode::MarkAsReferenced,
        )
    }

    /// Try to resolve a [`TypedPath`] into a [PathResolution] with [`PathResolutionTarget::Type`], marking it as _used_.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn use_path_as_type(&mut self, path: TypedPath) -> PathResolutionResult {
        self.resolve_path_inner(path, PathResolutionTarget::Type, PathResolutionMode::MarkAsUsed)
    }

    /// Resolves a path in the current module.
    ///
    /// If the referenced name can't be found, `Err` will be returned. If it can be found, `Ok`
    /// will be returned with a potential list of errors if, for example, one of the segments
    /// is not accessible from the current module (e.g. because it's private).
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn resolve_path_inner(
        &mut self,
        mut path: TypedPath,
        target: PathResolutionTarget,
        mode: PathResolutionMode,
    ) -> PathResolutionResult {
        let mut starting_module = self.module_id();
        let mut intermediate_item = IntermediatePathResolutionItem::Module;

        if path.kind == PathKind::Plain
            && path.first_name() == Some(SELF_TYPE_NAME)
            && let Some(Type::DataType(datatype, _)) = &self.self_type
        {
            let datatype = datatype.borrow();
            if path.segments.len() == 1 {
                return Ok(PathResolution {
                    item: PathResolutionItem::Type(datatype.id),
                    errors: Vec::new(),
                });
            }

            starting_module = datatype.id.module_id();
            path.segments.remove(0);
            intermediate_item = IntermediatePathResolutionItem::SelfType;
        }

        let turbofished_leaf =
            path.segments.last().filter(|segment| segment.generics.is_some()).cloned();
        let result =
            self.resolve_path_in_module(path, starting_module, intermediate_item, target, mode);
        Self::check_leaf_turbofish(result, turbofished_leaf.as_ref())
    }

    /// If `turbofished_leaf` (a resolved path's last segment, present only when it carries a
    /// turbofish) is set, reject the turbofish unless the item it resolved to is one a turbofish is
    /// allowed on. A fieldless enum variant may carry the enum's generics, so a turbofish is allowed
    /// there (e.g. `Foo::Spam::<u32>`); it is bound when the variable is elaborated. Any other
    /// errors, and the `Err` case, pass through untouched.
    fn check_leaf_turbofish(
        mut result: PathResolutionResult,
        turbofished_leaf: Option<&TypedPathSegment>,
    ) -> PathResolutionResult {
        let Some(leaf) = turbofished_leaf else {
            return result;
        };
        let Ok(resolution) = &mut result else {
            return result;
        };

        let location = leaf.turbofish_location();
        match resolution.item {
            PathResolutionItem::EnumVariant(..) => {}
            PathResolutionItem::Global(..) => {
                resolution.errors.push(PathResolutionError::TurbofishNotAllowedOnItem {
                    item: "globals".to_string(),
                    location,
                });
            }
            PathResolutionItem::Module(..) => {
                resolution.errors.push(PathResolutionError::TurbofishNotAllowedOnItem {
                    item: format!("module `{}`", leaf.ident),
                    location,
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
            | PathResolutionItem::PrimitiveFunction(..)
            | PathResolutionItem::TraitConstant(..) => (),
        }
        result
    }

    /// Resolve `path` as a value, looking it up directly in the already-resolved `module_id`
    /// instead of from the current module. The current module stays the importing module, so
    /// visibility is still checked from where the lookup originates. The [`Self::use_path_or_error`]
    /// counterpart for when a path's prefix module is already known.
    pub(super) fn use_value_in_module(
        &mut self,
        path: TypedPath,
        module_id: ModuleId,
    ) -> Result<PathResolutionItem, ResolverError> {
        let turbofished_leaf =
            path.segments.last().filter(|segment| segment.generics.is_some()).cloned();
        let result = self.resolve_name_in_module(
            path,
            module_id,
            IntermediatePathResolutionItem::Module,
            PathResolutionTarget::Value,
            PathResolutionMode::MarkAsUsed,
        );

        let resolution = Self::check_leaf_turbofish(result, turbofished_leaf.as_ref())?;
        self.push_errors(resolution.errors);
        Ok(resolution.item)
    }

    /// Resolve `last_segment` as a value member of the already-resolved type `typ`, instead of
    /// re-resolving the whole path. This is the type-prefix counterpart of [`Self::use_value_in_module`]:
    /// the member is either an enum-variant constructor (kept in the type's module value scope) or a
    /// trait associated constant (`Type::CONST`). `turbofish` carries the prefix's generics, used to
    /// finalize an enum variant. Only a concrete data type has value members; anything else (e.g. a
    /// primitive type) resolves nothing here.
    pub(super) fn use_value_in_type(
        &mut self,
        last_segment: &TypedPathSegment,
        typ: &Type,
        turbofish: Option<Turbofish>,
    ) -> Result<PathResolutionItem, ResolverError> {
        let Type::DataType(datatype, _) = typ else {
            return Err(PathResolutionError::Unresolved(last_segment.ident.clone()).into());
        };
        let type_id = datatype.borrow().id;

        // The associated constant is looked up on the resolved `typ` (so an alias's generics are
        // applied correctly).
        let mut errors = Vec::new();
        let item = self.resolve_value_member_of_type(
            last_segment,
            type_id.module_id(),
            IntermediatePathResolutionItem::Type(type_id, turbofish),
            Some((type_id, typ)),
            PathResolutionMode::MarkAsUsed,
            &mut errors,
        )?;
        self.push_errors(errors);
        Ok(item)
    }

    /// Resolves a [`TypedPath`] assuming it is inside `starting_module`.
    ///
    /// This method first checks the path's kind and resolves it accordingly.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_path_in_module(
        &mut self,
        path: TypedPath,
        starting_module: ModuleId,
        intermediate_item: IntermediatePathResolutionItem,
        target: PathResolutionTarget,
        mode: PathResolutionMode,
    ) -> PathResolutionResult {
        let references_tracker =
            self.interner.is_in_lsp_mode().then(|| ReferencesTracker::new(self.interner));

        let res =
            resolve_path_kind(path.clone(), starting_module, self.def_maps, references_tracker);

        match res {
            Ok((path, module_id, _)) => {
                self.resolve_name_in_module(path, module_id, intermediate_item, target, mode)
            }
            Err(error @ PathResolutionError::Unresolved(_)) => {
                if let Some(result) = self.resolve_primitive_type_or_function(path) {
                    return result;
                }
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    /// Mark a path segment's definition as used or referenced, depending on the [`PathResolutionMode`].
    fn mark_segment(
        &mut self,
        mode: PathResolutionMode,
        module_id: ModuleId,
        name: &Ident,
        namespace: Namespace,
    ) {
        match mode {
            PathResolutionMode::MarkAsReferenced => {
                self.usage_tracker.mark_as_referenced(module_id, name, namespace);
            }
            PathResolutionMode::MarkAsUsed => {
                self.usage_tracker.mark_as_used(module_id, name, namespace);
            }
        }
    }

    /// Resolves a [`TypedPath`] assuming it is inside `starting_module`.
    ///
    /// This method does not check the path kind, it just checks its segments.
    ///
    /// Marks the segments in the path as used or referenced, depending on the [`PathResolutionMode`].
    /// Pushes errors if segments refer to private items.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_name_in_module(
        &mut self,
        path: TypedPath,
        starting_module: ModuleId,
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

        // The module to use for visibility check.
        // Use the caller's module if set, else the module the lookup started in.
        let visibility_module = self.caller_module.unwrap_or(self.module_id());

        let first_segment_is_always_visible =
            first_segment_is_always_visible(&path, self.module_id(), starting_module);

        // The current module and module ID as we resolve path segments
        let mut current_module_id = starting_module;
        let mut current_module = self.get_module(starting_module);

        let first_segment =
            &path.segments.first().expect("ICE: could not fetch first segment").ident;

        let mut current_ns = current_module.find_name(first_segment);
        if current_ns.is_none() {
            return Err(PathResolutionError::Unresolved(first_segment.clone()));
        }

        // When the path has more than one segment, the first segment is traversed as a module or
        // type, so it lives in the type namespace. A single-segment path's only segment is the
        // leaf, which is marked after the loop with the namespace it actually resolved to.
        if path.segments.len() > 1 {
            self.mark_segment(mode, current_module_id, first_segment, Namespace::Type);
        }

        let mut errors = Vec::new();
        for (index, (prev_segment, current_segment)) in
            path.segments.iter().tuple_windows().enumerate()
        {
            let prev_ident = &prev_segment.ident;
            let current_ident = &current_segment.ident;
            let prev_segment_generics = &prev_segment.generics;

            // We are looking up the `current_segment` in the lookup result of the `prev_segment`.
            let (typ, visibility) = match current_ns.types {
                None => return Err(PathResolutionError::Unresolved(prev_ident.clone())),
                Some(scope) => (scope.id, scope.visibility),
            };

            let location = prev_segment.location;
            self.interner.add_module_def_id_reference(
                typ,
                location,
                prev_segment.ident.is_self_type_name(),
            );

            // An item brought into scope by an import that is not visible from here (kept in scope only
            // because a colliding item in the other namespace was visible) is private when referenced.
            if self.get_module(current_module_id).is_private_import_deferred(typ) {
                errors.push(PathResolutionError::Private(prev_ident.clone()));
            }

            let current_module_id_is_type;

            // The module `prev_segment` is declared in (its visibility is checked against this),
            // captured before stepping `current_module_id` into the module/type it refers to.
            let prev_segment_module_id = current_module_id;

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
                    match get_type_alias_target(&type_alias) {
                        Some(TypeAliasTarget::Module(module_id)) => {
                            let item = IntermediatePathResolutionItem::TypeAlias(
                                id,
                                prev_segment.turbofish(),
                            );
                            (module_id, true, item)
                        }
                        Some(TypeAliasTarget::Primitive(typ)) => {
                            // The alias points to a primitive type. Look up the method
                            // directly via the interner rather than through a module.
                            return self.resolve_primitive_type_alias_method(
                                typ,
                                id,
                                prev_segment.turbofish(),
                                current_ident,
                                &mut errors,
                            );
                        }
                        None => {
                            return Err(PathResolutionError::Unresolved(prev_ident.clone()));
                        }
                    }
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
                    visibility_module,
                    prev_segment_module_id,
                    visibility,
                ))
            {
                errors.push(PathResolutionError::Private(prev_ident.clone()));
            }

            // Switch to the module the current segment is defined in.
            current_module = self.get_module(current_module_id);

            let is_last_segment = index == path.segments.len() - 2;

            // A type's path members (an enum-variant constructor or a trait associated constant) are
            // terminal — they have no members of their own — so resolve and finalize the member here
            // and reject any trailing segment.
            if current_module_id_is_type {
                // Associated constants apply to a concrete type, not a type-alias intermediate.
                let self_type;
                let associated_constant =
                    if let IntermediatePathResolutionItem::Type(type_id, turbofish) =
                        &intermediate_item
                    {
                        self_type = self.data_type_as_self_type(*type_id, turbofish.as_ref());
                        Some((*type_id, &self_type))
                    } else {
                        None
                    };

                let item = self.resolve_value_member_of_type(
                    current_segment,
                    current_module_id,
                    intermediate_item,
                    associated_constant,
                    mode,
                    &mut errors,
                )?;

                if !is_last_segment {
                    let kind = match &item {
                        PathResolutionItem::EnumVariant(..) => "enum variant",
                        _ => "associated constant",
                    };
                    return Err(PathResolutionError::NoAssociatedItems {
                        name: current_ident.clone(),
                        kind,
                    });
                }
                return Ok(PathResolution { item, errors });
            }

            let found_ns = current_module.find_name(current_ident);
            if found_ns.is_none() {
                return Err(PathResolutionError::Unresolved(current_ident.clone()));
            }

            // Every segment but the last is traversed as a module or type. The last segment is
            // the leaf, marked after the loop with the namespace it actually resolved to.
            if !is_last_segment {
                self.mark_segment(mode, current_module_id, current_ident, Namespace::Type);
            }

            current_ns = found_ns;
        }

        let (target_ns, fallback_ns) = match target {
            PathResolutionTarget::Type => (current_ns.types, current_ns.values),
            PathResolutionTarget::Value => (current_ns.values, current_ns.types),
        };

        let scope = target_ns.or(fallback_ns).expect("A namespace should never be empty");

        let item = self.finalize_resolved_leaf(
            path,
            intermediate_item,
            current_module_id,
            scope,
            mode,
            &mut errors,
        );

        Ok(PathResolution { item, errors })
    }

    /// Finalize a path's leaf: mark its segment as used/referenced in the namespace it resolved to
    /// (so a same-named sibling in the other namespace stays tracked), then turn the namespace item
    /// into a [`PathResolutionItem`], pushing any visibility errors onto `errors`. Shared by the
    /// segment loop's tail and [`Self::use_value_in_type`].
    fn finalize_resolved_leaf(
        &mut self,
        path: TypedPath,
        intermediate_item: IntermediatePathResolutionItem,
        current_module_id: ModuleId,
        scope: NamespaceItem,
        mode: PathResolutionMode,
        errors: &mut Vec<PathResolutionError>,
    ) -> PathResolutionItem {
        // Use the caller's module if set, else the module the lookup started in.
        let visibility_module = self.caller_module.unwrap_or(self.module_id());
        self.mark_segment(mode, current_module_id, &path.last_ident(), scope.id.namespace());
        self.per_ns_item_to_path_resolution_item(
            path,
            visibility_module,
            intermediate_item,
            current_module_id,
            errors,
            scope.id,
            scope.visibility,
        )
    }

    /// Transform a result from [`PerNs`] into a [`PathResolutionItem`],
    /// pushing any visibility errors.
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(level = "trace", skip_all)]
    fn per_ns_item_to_path_resolution_item(
        &mut self,
        path: TypedPath,
        visibility_module: ModuleId,
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

        let mut item = merge_intermediate_path_resolution_item_with_module_def_id(
            intermediate_item,
            module_def_id,
        );

        // Fieldless enum variants are lowered to globals; surface them as a dedicated variant
        // so callers (e.g. the turbofish check) can tell them apart from ordinary globals.
        if let PathResolutionItem::Global(global_id) = item
            && self.interner.is_enum_variant_global(global_id)
        {
            item = PathResolutionItem::EnumVariant(global_id);
        }

        // For inherent impl methods, check visibility against the impl's defining module
        // (source_module), not just the type's module where the method was declared for lookup.
        // This prevents private methods defined in `impl super::S` inside `mod private` from
        // being accessible outside `mod private` via `S::method()`.
        if let ModuleDefId::FunctionId(func_id) = module_def_id
            && let Some(func_meta) = self.interner.try_function_meta(&func_id)
            && func_meta.type_id.is_some()
            && func_meta.trait_impl.is_none()
        {
            let source_module =
                ModuleId { krate: func_meta.source_crate, local_id: func_meta.source_module };
            if !item_in_module_is_visible(
                self.def_maps,
                visibility_module,
                source_module,
                visibility,
            ) {
                errors.push(PathResolutionError::Private(name.clone()));
            }
        } else if !item_in_module_is_visible(
            self.def_maps,
            visibility_module,
            current_module_id,
            visibility,
        ) {
            errors.push(PathResolutionError::Private(name.clone()));
        }

        // An item brought into scope by an import that is not visible from here (kept in scope only
        // because a colliding item in the other namespace was visible) is private when referenced.
        if self.get_module(current_module_id).is_private_import_deferred(module_def_id) {
            errors.push(PathResolutionError::Private(name.clone()));
        }

        // A trait method imported via `Type::method` must also be reachable through its trait's
        // visibility (e.g. a `pub(crate) trait` is not accessible from another crate, even if the
        // method's own visibility check above passes).
        if let ModuleDefId::FunctionId(func_id) = module_def_id
            && !trait_visibility_for_method_is_satisfied(
                func_id,
                visibility_module,
                self.interner,
                self.def_maps,
            )
        {
            errors.push(PathResolutionError::Private(name));
        }

        item
    }

    /// Assuming that the current path segment is a type defined in `current_module`, resolve
    /// `ident` as one of that type's enum variant constructors.
    ///
    /// These are the only function-like items kept in a type's module scope: inherent and trait
    /// methods are not declared there and resolve through the interner's type-directed lookup.
    fn resolve_method(&self, current_module: &ModuleData, ident: &Ident) -> Option<PerNs> {
        current_module
            .scope()
            .values()
            .get(ident)
            .map(|item| PerNs { types: None, values: Some(*item) })
    }

    /// Build the `self` type for a data type accessed in a path: its generics come from the path's
    /// turbofish if present, otherwise from the type's own (fresh) generics.
    fn data_type_as_self_type(&self, type_id: TypeId, turbofish: Option<&Turbofish>) -> Type {
        let datatype = self.interner.get_type(type_id);
        let generics = if let Some(turbofish) = turbofish {
            turbofish.generics.iter().map(|t| t.contents.clone()).collect()
        } else {
            datatype.borrow().generic_types()
        };
        Type::DataType(datatype, generics)
    }

    /// Resolve `member` as a value member of a data type whose own module is `module`: an
    /// enum-variant constructor kept in that module's value scope, or — for a concrete type, with
    /// `associated_constant` set to its `(TypeId, self type)` — a trait associated constant
    /// (`Type::CONST`). Returns the finalized item, or an error if it names neither. Shared by the
    /// segment loop and [`Self::use_value_in_type`].
    fn resolve_value_member_of_type(
        &mut self,
        member: &TypedPathSegment,
        module: ModuleId,
        intermediate_item: IntermediatePathResolutionItem,
        associated_constant: Option<(TypeId, &Type)>,
        mode: PathResolutionMode,
        errors: &mut Vec<PathResolutionError>,
    ) -> Result<PathResolutionItem, PathResolutionError> {
        if let Some(per_ns) = self.resolve_method(self.get_module(module), &member.ident) {
            let scope = per_ns.values.expect("resolve_method only returns a value namespace");
            let path = TypedPath::plain(vec![member.clone()], member.ident.location());
            return Ok(self.finalize_resolved_leaf(
                path,
                intermediate_item,
                module,
                scope,
                mode,
                errors,
            ));
        }

        match associated_constant {
            Some((type_id, self_type)) => {
                self.resolve_associated_constant_or_unresolved(type_id, self_type, &member.ident)
            }
            None => Err(PathResolutionError::Unresolved(member.ident.clone())),
        }
    }

    /// A segment that named the data type `self_type` but is not an enum-variant constructor in its
    /// module scope may still be a trait associated constant (`Type::CONST`) on it; otherwise it is
    /// unresolved. `type_id` is the data type, carried into the resulting
    /// [`PathResolutionItem::TraitConstant`]. Shared by the segment loop and [`Self::use_value_in_type`].
    fn resolve_associated_constant_or_unresolved(
        &self,
        type_id: TypeId,
        self_type: &Type,
        ident: &Ident,
    ) -> Result<PathResolutionItem, PathResolutionError> {
        if let Some(result) = self.try_resolve_trait_constant(type_id, self_type, ident) {
            return result;
        }
        // Not an associated constant. If a trait defines an associated item with this name, point
        // the user at it rather than reporting a bare "could not resolve".
        if let Some(error) = self.resolve_associated_item_diagnostic(self_type, ident) {
            return Err(error);
        }
        Err(PathResolutionError::Unresolved(ident.clone()))
    }

    /// Try to resolve an identifier as a trait associated constant (e.g., `Foo::N`) on `self_type`
    /// (`type_id` is its data type, carried into the resulting [`PathResolutionItem::TraitConstant`]).
    ///
    /// Returns `Some(Ok(PathResolutionItem))` if the constant was found,
    /// `Some(Err(PathResolutionError))` if there's an ambiguity error,
    /// or `None` if no matching constant was found.
    fn try_resolve_trait_constant(
        &self,
        type_id: TypeId,
        self_type: &Type,
        ident: &Ident,
    ) -> Option<Result<PathResolutionItem, PathResolutionError>> {
        // Look up constants matching the identifier
        let constants =
            self.interner.lookup_trait_impl_constants_for_type(self_type, ident.as_str());

        if constants.is_empty() {
            return None;
        }

        // Filter to traits that are in scope
        let starting_module = self.get_module(self.module_id());
        let in_scope: Vec<_> = constants
            .iter()
            .filter(|(_, trait_id, _)| starting_module.find_trait_in_scope(*trait_id).is_some())
            .collect();

        match in_scope.len() {
            0 => {
                // Constants exist but none of their traits are in scope
                // Return None to fall through to the method error handling,
                // which will suggest importing the traits
                None
            }
            1 => {
                // Exactly one matching constant with trait in scope
                let (def_id, trait_id, _impl_id) = in_scope[0];
                Some(Ok(PathResolutionItem::TraitConstant(type_id, *trait_id, *def_id)))
            }
            _ => {
                // Multiple matching constants - ambiguous. If all candidates are from the
                // same trait, this is multiple impls of one trait — report it with the
                // specific impl signatures so the user can see what to disambiguate.
                let first_trait_id = in_scope[0].1;
                let same_trait =
                    in_scope.iter().all(|(_, trait_id, _)| *trait_id == first_trait_id);
                if same_trait {
                    let trait_name = self.fully_qualified_trait_path_by_id(first_trait_id);
                    let type_name = self_type.to_string();
                    let impls = vecmap(&in_scope, |(_, _, impl_id)| {
                        let ordered = &self.interner.get_trait_generics_for_impl(*impl_id).ordered;
                        let signature = if ordered.is_empty() {
                            trait_name.clone()
                        } else {
                            let args = vecmap(ordered, |t| t.to_string()).join(", ");
                            format!("{trait_name}<{args}>")
                        };
                        let location =
                            self.interner.get_trait_implementation(*impl_id).borrow().location;
                        (signature, location)
                    });
                    Some(Err(PathResolutionError::MultipleApplicableImpls {
                        ident: ident.clone(),
                        trait_name,
                        type_name,
                        impls,
                    }))
                } else {
                    let mut traits = vecmap(&in_scope, |(_, trait_id, _)| {
                        self.fully_qualified_trait_path_by_id(*trait_id)
                    });
                    traits.sort();
                    traits.dedup();
                    Some(Err(PathResolutionError::MultipleTraitsInScope {
                        ident: ident.clone(),
                        traits,
                    }))
                }
            }
        }
    }

    /// Build a helpful diagnostic for a `Type::item` path whose `item` is neither a method nor a
    /// resolvable associated constant of `Type`, but does name an associated item of some trait.
    ///
    /// - If `item` is an associated type of a trait that `Type` implements, direct access isn't
    ///   supported: suggest the fully-qualified `<Type as Trait>::item` form.
    /// - Otherwise, if a trait defines an associated item with this name, `Type` simply doesn't
    ///   implement it: report that and name the trait(s).
    ///
    /// Returns `None` when no trait defines an associated item with this name, leaving the caller
    /// to fall back to its generic "could not resolve" handling.
    fn resolve_associated_item_diagnostic(
        &self,
        self_type: &Type,
        ident: &Ident,
    ) -> Option<PathResolutionError> {
        let name = ident.as_str();
        // Traits that define an associated type named `name` and which `self_type` implements.
        let mut accessible_type_traits = Vec::new();
        // Every trait that defines an associated item (type or constant) named `name`.
        let mut defining_traits = Vec::new();

        for trait_id in self.interner.trait_ids() {
            let the_trait = self.interner.get_trait(trait_id);
            let defines_type = the_trait.get_associated_type(name).is_some();
            let defines_constant = the_trait.associated_constant_ids.contains_key(name);
            if !defines_type && !defines_constant {
                continue;
            }

            let trait_name = self.fully_qualified_trait_path_by_id(trait_id);
            defining_traits.push(trait_name.clone());
            if defines_type && self.type_implements_trait(self_type, trait_id) {
                accessible_type_traits.push(trait_name);
            }
        }

        if defining_traits.is_empty() {
            return None;
        }

        let type_name = self_type.to_string();
        if !accessible_type_traits.is_empty() {
            return Some(PathResolutionError::AssociatedTypeNotAccessibleDirectly {
                ident: ident.clone(),
                type_name,
                traits: accessible_type_traits,
            });
        }

        Some(PathResolutionError::AssociatedItemNotImplemented {
            ident: ident.clone(),
            type_name,
            traits: defining_traits,
        })
    }

    /// Returns whether `typ` implements `trait_id` for some instantiation of the trait's generics.
    fn type_implements_trait(&self, typ: &Type, trait_id: TraitId) -> bool {
        let the_trait = self.interner.get_trait(trait_id);
        let ordered =
            vecmap(&the_trait.generics, |_| self.interner.next_type_variable_with_kind(Kind::Any));
        let named = vecmap(&the_trait.associated_types, |generic| NamedType {
            name: Ident::new(generic.name.to_string(), Location::dummy()),
            typ: self.interner.next_type_variable_with_kind(Kind::Any),
        });
        self.interner
            .try_lookup_trait_implementation(
                typ,
                trait_id,
                &ordered,
                &named,
                TraitLookupMode::Default,
            )
            .is_ok()
    }

    /// Resolve a method on a type alias that points to a primitive type.
    ///
    /// This handles paths like `MyAlias::method()` where `MyAlias` aliases
    /// a primitive type. Because they do not have module, we look up the method directly
    /// like what is done in [`Self::resolve_primitive_type_or_function`].
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_primitive_type_alias_method(
        &mut self,
        typ: Type,
        alias_id: TypeAliasId,
        turbofish: Option<Turbofish>,
        method_name_ident: &Ident,
        errors: &mut Vec<PathResolutionError>,
    ) -> PathResolutionResult {
        self.resolve_primitive_type_method(typ, method_name_ident, errors).map(|func_id| {
            let item = PathResolutionItem::TypeAliasFunction(alias_id, turbofish, func_id);
            PathResolution { item, errors: std::mem::take(errors) }
        })
    }

    /// Try to resolve a path with 1 or 2 segments as a [`PathResolutionItem::PrimitiveType`] or [`PathResolutionItem::PrimitiveFunction`].
    ///
    /// If the path consists of 2 segments, use the 2nd segment as the method name and look up a direct method implementation,
    /// or an unambiguous trait method among the traits which are in scope.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_primitive_type_or_function(
        &mut self,
        path: TypedPath,
    ) -> Option<PathResolutionResult> {
        if path.segments.len() != 1 && path.segments.len() != 2 {
            return None;
        }

        let object_name = path.segments[0].ident.as_str();
        let turbofish = path.segments[0].turbofish();
        let primitive_type = PrimitiveType::lookup_by_name(object_name)?;
        let typ = primitive_type.to_type();
        let mut errors = Vec::new();

        if path.segments.len() == 1 {
            let item = PathResolutionItem::PrimitiveType(primitive_type);
            return Some(Ok(PathResolution { item, errors }));
        }

        let method_name_ident = &path.segments[1].ident;
        let method = self.resolve_primitive_type_method(typ, method_name_ident, &mut errors);
        Some(method.map(|func_id| PathResolution {
            item: PathResolutionItem::PrimitiveFunction(primitive_type, turbofish, func_id),
            errors,
        }))
    }

    fn resolve_primitive_type_method(
        &mut self,
        typ: Type,
        method_name_ident: &Ident,
        errors: &mut Vec<PathResolutionError>,
    ) -> Result<FuncId, PathResolutionError> {
        let method_name = method_name_ident.as_str();

        // An inherent method takes precedence over any trait methods.
        if let Some(func_id) = self.lookup_direct_method(&typ, method_name, false) {
            return Ok(func_id);
        }

        // Split the matching trait methods by whether their trait is currently in scope.
        let current_module_id = self.module_id();
        let trait_methods = self.lookup_trait_methods(&typ, method_name, false);
        let total = trait_methods.len();
        let starting_module = self.get_module(current_module_id);
        let mut in_scope = Vec::new();
        let mut out_of_scope = Vec::new();
        for (func_id, trait_id, _) in trait_methods {
            if let Some(name) = starting_module.find_trait_in_scope(trait_id) {
                in_scope.push((trait_id, name.clone(), func_id));
            } else {
                out_of_scope.push((trait_id, func_id));
            }
        }

        match in_scope.len() {
            // A single trait method that isn't in scope is still resolved, with a warning.
            0 if total == 1 => {
                let (trait_id, func_id) = out_of_scope.into_iter().next().expect("total == 1");
                let trait_name = self.fully_qualified_trait_path_by_id(trait_id);
                errors.push(PathResolutionError::TraitMethodNotInScope {
                    ident: method_name_ident.clone(),
                    trait_name,
                });
                Ok(func_id)
            }
            0 if total == 0 => Err(PathResolutionError::Unresolved(method_name_ident.clone())),
            // No matching trait is in scope, but some could be imported.
            0 => {
                let traits = vecmap(out_of_scope, |(trait_id, _)| {
                    self.fully_qualified_trait_path(self.interner.get_trait(trait_id))
                });
                Err(PathResolutionError::UnresolvedWithPossibleTraitsToImport {
                    ident: method_name_ident.clone(),
                    traits,
                })
            }
            1 => {
                let (_, _, func_id) = in_scope.into_iter().next().expect("len == 1");
                Ok(func_id)
            }
            _ => {
                let traits = vecmap(in_scope, |(trait_id, name, _)| {
                    let trait_ = self.interner.get_trait(trait_id);
                    self.usage_tracker.mark_as_used(current_module_id, &name, Namespace::Type);
                    self.fully_qualified_trait_path(trait_)
                });
                Err(PathResolutionError::MultipleTraitsInScope {
                    ident: method_name_ident.clone(),
                    traits,
                })
            }
        }
    }
}

/// Transform a [`ModuleDefId`] into a [`PathResolutionItem`].
///
/// If it's a [`ModuleDefId::FunctionId`], merge it with the [`IntermediatePathResolutionItem`]
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

/// The target that a type alias ultimately resolves to for path resolution purposes.
/// Indicate directly the primitive type in case of an alias to a primitive type, because
/// they do not have module.
enum TypeAliasTarget {
    /// The alias points to a data type (struct/enum) with this module.
    Module(ModuleId),
    /// The alias points to a primitive type.
    Primitive(Type),
}

fn get_type_alias_target(type_alias: &Shared<TypeAlias>) -> Option<TypeAliasTarget> {
    let type_alias = type_alias.borrow();

    match &type_alias.typ {
        Type::DataType(type_id, _generics) => {
            Some(TypeAliasTarget::Module(type_id.borrow().id.module_id()))
        }
        Type::Alias(type_alias, _generics) => get_type_alias_target(type_alias),
        Type::Error => None,
        other => {
            if PrimitiveType::from_type(other).is_some() {
                Some(TypeAliasTarget::Primitive(other.clone()))
            } else {
                None
            }
        }
    }
}
