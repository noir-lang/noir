use std::borrow::Cow;
use std::fmt;
use std::hash::Hash;
use std::marker::Copy;
use std::ops::Deref;

use fm::FileId;
use iter_extended::vecmap;
use noirc_arena::{Arena, Index};
use noirc_errors::{Location, Span, Spanned};
use petgraph::algo::tarjan_scc;
use petgraph::prelude::DiGraph;
use petgraph::prelude::NodeIndex as PetGraphIndex;
use rustc_hash::FxHashMap as HashMap;

use crate::ast::Ident;
use crate::graph::CrateId;
use crate::hir::comptime;
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::hir::def_collector::dc_crate::{UnresolvedStruct, UnresolvedTrait, UnresolvedTypeAlias};
use crate::hir::def_map::{LocalModuleId, ModuleId};
use crate::macros_api::ModuleDefId;
use crate::macros_api::UnaryOp;
use crate::QuotedType;

use crate::ast::{BinaryOpKind, FunctionDefinition, ItemVisibility};
use crate::hir::resolution::errors::ResolverError;
use crate::hir_def::expr::HirIdent;
use crate::hir_def::stmt::HirLetStatement;
use crate::hir_def::traits::TraitImpl;
use crate::hir_def::traits::{Trait, TraitConstraint};
use crate::hir_def::types::{StructType, Type};
use crate::hir_def::{
    expr::HirExpression,
    function::{FuncMeta, HirFunction},
    stmt::HirStatement,
};
use crate::locations::LocationIndices;
use crate::token::{Attributes, SecondaryAttribute};
use crate::GenericTypeVars;
use crate::Generics;
use crate::{Shared, TypeAlias, TypeBindings, TypeVariable, TypeVariableId, TypeVariableKind};

/// An arbitrary number to limit the recursion depth when searching for trait impls.
/// This is needed to stop recursing for cases such as `impl<T> Foo for T where T: Eq`
const IMPL_SEARCH_RECURSION_LIMIT: u32 = 10;

#[derive(Debug)]
pub struct ModuleAttributes {
    pub name: String,
    pub location: Location,
    pub parent: LocalModuleId,
}

type StructAttributes = Vec<SecondaryAttribute>;

/// The node interner is the central storage location of all nodes in Noir's Hir (the
/// various node types can be found in hir_def). The interner is also used to collect
/// extra information about the Hir, such as the type of each node, information about
/// each definition or struct, etc. Because it is used on the Hir, the NodeInterner is
/// useful in passes where the Hir is used - name resolution, type checking, and
/// monomorphization - and it is not useful afterward.
#[derive(Debug)]
pub struct NodeInterner {
    pub(crate) nodes: Arena<Node>,
    pub(crate) func_meta: HashMap<FuncId, FuncMeta>,

    function_definition_ids: HashMap<FuncId, DefinitionId>,

    // For a given function ID, this gives the function's modifiers which includes
    // its visibility and whether it is unconstrained, among other information.
    // Unlike func_meta, this map is filled out during definition collection rather than name resolution.
    function_modifiers: HashMap<FuncId, FunctionModifiers>,

    // Contains the source module each function was defined in
    function_modules: HashMap<FuncId, ModuleId>,

    // The location of each module
    module_attributes: HashMap<ModuleId, ModuleAttributes>,

    /// This graph tracks dependencies between different global definitions.
    /// This is used to ensure the absence of dependency cycles for globals and types.
    dependency_graph: DiGraph<DependencyId, ()>,

    /// To keep track of where each DependencyId is in `dependency_graph`, we need
    /// this separate graph to map between the ids and indices.
    dependency_graph_indices: HashMap<DependencyId, PetGraphIndex>,

    // Map each `Index` to it's own location
    pub(crate) id_to_location: HashMap<Index, Location>,

    // Maps each DefinitionId to a DefinitionInfo.
    definitions: Vec<DefinitionInfo>,

    // Type checking map
    //
    // This should only be used with indices from the `nodes` arena.
    // Otherwise the indices used may overwrite other existing indices.
    // Each type for each index is filled in during type checking.
    id_to_type: HashMap<Index, Type>,

    // Similar to `id_to_type` but maps definitions to their type
    definition_to_type: HashMap<DefinitionId, Type>,

    // Struct map.
    //
    // Each struct definition is possibly shared across multiple type nodes.
    // It is also mutated through the RefCell during name resolution to append
    // methods from impls to the type.
    structs: HashMap<StructId, Shared<StructType>>,

    struct_attributes: HashMap<StructId, StructAttributes>,

    // Maps TypeAliasId -> Shared<TypeAlias>
    //
    // Map type aliases to the actual type.
    // When resolving types, check against this map to see if a type alias is defined.
    pub(crate) type_aliases: Vec<Shared<TypeAlias>>,

    // Trait map.
    //
    // Each trait definition is possibly shared across multiple type nodes.
    // It is also mutated through the RefCell during name resolution to append
    // methods from impls to the type.
    pub(crate) traits: HashMap<TraitId, Trait>,

    // Trait implementation map
    // For each type that implements a given Trait ( corresponding TraitId), there should be an entry here
    // The purpose for this hashmap is to detect duplication of trait implementations ( if any )
    //
    // Indexed by TraitImplIds
    pub(crate) trait_implementations: HashMap<TraitImplId, Shared<TraitImpl>>,

    next_trait_implementation_id: usize,

    /// Trait implementations on each type. This is expected to always have the same length as
    /// `self.trait_implementations`.
    ///
    /// For lack of a better name, this maps a trait id and type combination
    /// to a corresponding impl if one is available for the type. Due to generics,
    /// we cannot map from Type directly to impl, we need to iterate a Vec of all impls
    /// of that trait to see if any type may match. This can be further optimized later
    /// by splitting it up by type.
    trait_implementation_map: HashMap<TraitId, Vec<(Type, TraitImplKind)>>,

    /// When impls are found during type checking, we tag the function call's Ident
    /// with the impl that was selected. For cases with where clauses, this may be
    /// an Assumed (but verified) impl. In this case the monomorphizer should have
    /// the context to get the concrete type of the object and select the correct impl itself.
    selected_trait_implementations: HashMap<ExprId, TraitImplKind>,

    /// Holds the trait ids of the traits used for infix operator overloading
    infix_operator_traits: HashMap<BinaryOpKind, TraitId>,

    /// Holds the trait ids of the traits used for prefix operator overloading
    prefix_operator_traits: HashMap<UnaryOp, TraitId>,

    /// The `Ordering` type is a semi-builtin type that is the result of the comparison traits.
    ordering_type: Option<Type>,

    /// Map from ExprId (referring to a Function/Method call) to its corresponding TypeBindings,
    /// filled out during type checking from instantiated variables. Used during monomorphization
    /// to map call site types back onto function parameter types, and undo this binding as needed.
    instantiation_bindings: HashMap<ExprId, TypeBindings>,

    /// Remembers the field index a given HirMemberAccess expression was resolved to during type
    /// checking.
    field_indices: HashMap<ExprId, usize>,

    // Maps GlobalId -> GlobalInfo
    // NOTE: currently only used for checking repeat globals and restricting their scope to a module
    globals: Vec<GlobalInfo>,
    global_attributes: HashMap<GlobalId, Vec<SecondaryAttribute>>,

    next_type_variable_id: std::cell::Cell<usize>,

    /// A map from a struct type and method name to a function id for the method.
    /// This can resolve to potentially multiple methods if the same method name is
    /// specialized for different generics on the same type. E.g. for `Struct<T>`, we
    /// may have both `impl Struct<u32> { fn foo(){} }` and `impl Struct<u8> { fn foo(){} }`.
    /// If this happens, the returned Vec will have 2 entries and we'll need to further
    /// disambiguate them by checking the type of each function.
    struct_methods: HashMap<StructId, HashMap<String, Methods>>,

    /// Methods on primitive types defined in the stdlib.
    primitive_methods: HashMap<TypeMethodKey, HashMap<String, Methods>>,

    // For trait implementation functions, this is their self type and trait they belong to
    func_id_to_trait: HashMap<FuncId, (Type, TraitId)>,

    /// A list of all type aliases that are referenced in the program.
    /// Searched by LSP to resolve [Location]s of [TypeAliasType]s
    pub(crate) type_alias_ref: Vec<(TypeAliasId, Location)>,

    /// Stores the [Location] of a [Type] reference
    pub(crate) type_ref_locations: Vec<(Type, Location)>,

    /// In Noir's metaprogramming, a noir type has the type `Type`. When these are spliced
    /// into `quoted` expressions, we preserve the original type by assigning it a unique id
    /// and creating a `Token::QuotedType(id)` from this id. We cannot create a token holding
    /// the actual type since types do not implement Send or Sync.
    quoted_types: noirc_arena::Arena<Type>,

    /// Determins whether to run in LSP mode. In LSP mode references are tracked.
    pub(crate) lsp_mode: bool,

    /// Store the location of the references in the graph.
    /// Edges are directed from reference nodes to referenced nodes.
    /// For example:
    ///
    /// ```text
    /// let foo = 3;
    /// //  referenced
    /// //   ^
    /// //   |
    /// //   +------------+
    /// let bar = foo;    |
    /// //      reference |
    /// //         v      |
    /// //         |      |
    /// //         +------+
    /// ```
    pub(crate) reference_graph: DiGraph<ReferenceId, ()>,

    /// Tracks the index of the references in the graph
    pub(crate) reference_graph_indices: HashMap<ReferenceId, PetGraphIndex>,

    /// Store the location of the references in the graph
    pub(crate) location_indices: LocationIndices,

    // The module where each reference is
    // (ReferenceId::Reference and ReferenceId::Local aren't included here)
    pub(crate) reference_modules: HashMap<ReferenceId, ModuleId>,

    // All names (and their definitions) that can be offered for auto_import.
    // These include top-level functions, global variables and types, but excludes
    // impl and trait-impl methods.
    pub(crate) auto_import_names: HashMap<String, Vec<(ModuleDefId, ItemVisibility)>>,

    /// Each value currently in scope in the comptime interpreter.
    /// Each element of the Vec represents a scope with every scope together making
    /// up all currently visible definitions. The first scope is always the global scope.
    ///
    /// This is stored in the NodeInterner so that the Elaborator from each crate can
    /// share the same global values.
    pub(crate) comptime_scopes: Vec<HashMap<DefinitionId, comptime::Value>>,
}

/// A dependency in the dependency graph may be a type or a definition.
/// Types can depend on definitions too. E.g. `Foo` depends on `COUNT` in:
///
/// ```struct
/// global COUNT = 3;
///
/// struct Foo {
///     array: [Field; COUNT],
/// }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DependencyId {
    Struct(StructId),
    Global(GlobalId),
    Function(FuncId),
    Alias(TypeAliasId),
    Variable(Location),
}

/// A reference to a module, struct, trait, etc., mainly used by the LSP code
/// to keep track of how symbols reference each other.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ReferenceId {
    Module(ModuleId),
    Struct(StructId),
    StructMember(StructId, usize),
    Trait(TraitId),
    Global(GlobalId),
    Function(FuncId),
    Alias(TypeAliasId),
    Local(DefinitionId),
    Reference(Location, bool /* is Self */),
}

impl ReferenceId {
    pub fn is_self_type_name(&self) -> bool {
        matches!(self, Self::Reference(_, true))
    }
}

/// A trait implementation is either a normal implementation that is present in the source
/// program via an `impl` block, or it is assumed to exist from a `where` clause or similar.
#[derive(Debug, Clone)]
pub enum TraitImplKind {
    Normal(TraitImplId),

    /// Assumed impls don't have an impl id since they don't link back to any concrete part of the source code.
    Assumed {
        object_type: Type,

        /// The trait generics to use - if specified.
        /// This is allowed to be empty when they are inferred. E.g. for:
        ///
        /// ```
        /// trait Into<T> {
        ///     fn into(self) -> T;
        /// }
        /// ```
        ///
        /// The reference `Into::into(x)` would have inferred generics, but
        /// `x.into()` with a `X: Into<Y>` in scope would not.
        trait_generics: Vec<Type>,
    },
}

/// Represents the methods on a given type that each share the same name.
///
/// Methods are split into inherent methods and trait methods. If there is
/// ever a name that is defined on both a type directly, and defined indirectly
/// via a trait impl, the direct (inherent) name will always take precedence.
///
/// Additionally, types can define specialized impls with methods of the same name
/// as long as these specialized impls do not overlap. E.g. `impl Struct<u32>` and `impl Struct<u64>`
#[derive(Default, Debug, Clone)]
pub struct Methods {
    pub direct: Vec<FuncId>,
    pub trait_impl_methods: Vec<FuncId>,
}

/// All the information from a function that is filled out during definition collection rather than
/// name resolution. As a result, if information about a function is needed during name resolution,
/// this is the only place where it is safe to retrieve it (where all fields are guaranteed to be initialized).
#[derive(Debug, Clone)]
pub struct FunctionModifiers {
    pub name: String,

    /// Whether the function is `pub` or not.
    pub visibility: ItemVisibility,

    pub attributes: Attributes,

    pub is_unconstrained: bool,

    pub generic_count: usize,

    pub is_comptime: bool,

    /// The location of the function's name rather than the entire function
    pub name_location: Location,
}

impl FunctionModifiers {
    /// A semi-reasonable set of default FunctionModifiers used for testing.
    #[cfg(test)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            name: String::new(),
            visibility: ItemVisibility::Public,
            attributes: Attributes::empty(),
            is_unconstrained: false,
            generic_count: 0,
            is_comptime: false,
            name_location: Location::dummy(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct DefinitionId(usize);

impl DefinitionId {
    //dummy id for error reporting
    pub fn dummy_id() -> DefinitionId {
        DefinitionId(std::usize::MAX)
    }
}

/// An ID for a global value
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct GlobalId(usize);

impl GlobalId {
    // Dummy id for error reporting
    pub fn dummy_id() -> Self {
        GlobalId(std::usize::MAX)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct StmtId(Index);

impl StmtId {
    //dummy id for error reporting
    // This can be anything, as the program will ultimately fail
    // after resolution
    pub fn dummy_id() -> StmtId {
        StmtId(Index::dummy())
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct ExprId(Index);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct FuncId(Index);

impl FuncId {
    //dummy id for error reporting
    // This can be anything, as the program will ultimately fail
    // after resolution
    pub fn dummy_id() -> FuncId {
        FuncId(Index::dummy())
    }
}

impl fmt::Display for FuncId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct StructId(ModuleId);

impl StructId {
    //dummy id for error reporting
    // This can be anything, as the program will ultimately fail
    // after resolution
    pub fn dummy_id() -> StructId {
        StructId(ModuleId { krate: CrateId::dummy_id(), local_id: LocalModuleId::dummy_id() })
    }

    pub fn module_id(self) -> ModuleId {
        self.0
    }

    pub fn krate(self) -> CrateId {
        self.0.krate
    }

    pub fn local_module_id(self) -> LocalModuleId {
        self.0.local_id
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct TypeAliasId(pub usize);

impl TypeAliasId {
    pub fn dummy_id() -> TypeAliasId {
        TypeAliasId(std::usize::MAX)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TraitId(pub ModuleId);

impl TraitId {
    // dummy id for error reporting
    // This can be anything, as the program will ultimately fail
    // after resolution
    pub fn dummy_id() -> TraitId {
        TraitId(ModuleId { krate: CrateId::dummy_id(), local_id: LocalModuleId::dummy_id() })
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct TraitImplId(pub usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TraitMethodId {
    pub trait_id: TraitId,
    pub method_index: usize, // index in Trait::methods
}

macro_rules! into_index {
    ($id_type:ty) => {
        impl From<$id_type> for Index {
            fn from(t: $id_type) -> Self {
                t.0
            }
        }

        impl From<&$id_type> for Index {
            fn from(t: &$id_type) -> Self {
                t.0
            }
        }
    };
}

into_index!(ExprId);
into_index!(StmtId);

/// A Definition enum specifies anything that we can intern in the NodeInterner
/// We use one Arena for all types that can be interned as that has better cache locality
/// This data structure is never accessed directly, so API wise there is no difference between using
/// Multiple arenas and a single Arena
#[derive(Debug, Clone)]
pub(crate) enum Node {
    Function(HirFunction),
    Statement(HirStatement),
    Expression(HirExpression),
}

#[derive(Debug, Clone)]
pub struct DefinitionInfo {
    pub name: String,
    pub mutable: bool,
    pub comptime: bool,
    pub kind: DefinitionKind,
    pub location: Location,
}

impl DefinitionInfo {
    /// True if this definition is for a global variable.
    /// Note that this returns false for top-level functions.
    pub fn is_global(&self) -> bool {
        self.kind.is_global()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DefinitionKind {
    Function(FuncId),

    Global(GlobalId),

    /// Locals may be defined in let statements or parameters,
    /// in which case they will not have an associated ExprId
    Local(Option<ExprId>),

    /// Generic types in functions (T, U in `fn foo<T, U>(...)` are declared as variables
    /// in scope in case they resolve to numeric generics later.
    GenericType(TypeVariable),
}

impl DefinitionKind {
    /// True if this definition is for a global variable.
    /// Note that this returns false for top-level functions.
    pub fn is_global(&self) -> bool {
        matches!(self, DefinitionKind::Global(..))
    }

    pub fn get_rhs(&self) -> Option<ExprId> {
        match self {
            DefinitionKind::Function(_) => None,
            DefinitionKind::Global(_) => None,
            DefinitionKind::Local(id) => *id,
            DefinitionKind::GenericType(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalInfo {
    pub id: GlobalId,
    pub definition_id: DefinitionId,
    pub ident: Ident,
    pub local_id: LocalModuleId,
    pub crate_id: CrateId,
    pub location: Location,
    pub let_statement: StmtId,
    pub value: Option<comptime::Value>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuotedTypeId(noirc_arena::Index);

impl Default for NodeInterner {
    fn default() -> Self {
        NodeInterner {
            nodes: Arena::default(),
            func_meta: HashMap::default(),
            function_definition_ids: HashMap::default(),
            function_modifiers: HashMap::default(),
            function_modules: HashMap::default(),
            module_attributes: HashMap::default(),
            func_id_to_trait: HashMap::default(),
            dependency_graph: petgraph::graph::DiGraph::new(),
            dependency_graph_indices: HashMap::default(),
            id_to_location: HashMap::default(),
            definitions: vec![],
            id_to_type: HashMap::default(),
            definition_to_type: HashMap::default(),
            structs: HashMap::default(),
            struct_attributes: HashMap::default(),
            type_aliases: Vec::new(),
            traits: HashMap::default(),
            trait_implementations: HashMap::default(),
            next_trait_implementation_id: 0,
            trait_implementation_map: HashMap::default(),
            selected_trait_implementations: HashMap::default(),
            infix_operator_traits: HashMap::default(),
            prefix_operator_traits: HashMap::default(),
            ordering_type: None,
            instantiation_bindings: HashMap::default(),
            field_indices: HashMap::default(),
            next_type_variable_id: std::cell::Cell::new(0),
            globals: Vec::new(),
            global_attributes: HashMap::default(),
            struct_methods: HashMap::default(),
            primitive_methods: HashMap::default(),
            type_alias_ref: Vec::new(),
            type_ref_locations: Vec::new(),
            quoted_types: Default::default(),
            lsp_mode: false,
            location_indices: LocationIndices::default(),
            reference_graph: petgraph::graph::DiGraph::new(),
            reference_graph_indices: HashMap::default(),
            reference_modules: HashMap::default(),
            auto_import_names: HashMap::default(),
            comptime_scopes: vec![HashMap::default()],
        }
    }
}

// XXX: Add check that insertions are not overwrites for maps
// XXX: Maybe change push to intern, and remove comments
impl NodeInterner {
    /// Interns a HIR statement.
    pub fn push_stmt(&mut self, stmt: HirStatement) -> StmtId {
        StmtId(self.nodes.insert(Node::Statement(stmt)))
    }
    /// Interns a HIR expression.
    pub fn push_expr(&mut self, expr: HirExpression) -> ExprId {
        ExprId(self.nodes.insert(Node::Expression(expr)))
    }

    /// Stores the span for an interned expression.
    pub fn push_expr_location(&mut self, expr_id: ExprId, span: Span, file: FileId) {
        self.id_to_location.insert(expr_id.into(), Location::new(span, file));
    }

    /// Interns a HIR Function.
    pub fn push_fn(&mut self, func: HirFunction) -> FuncId {
        FuncId(self.nodes.insert(Node::Function(func)))
    }

    /// Store the type for an interned expression
    pub fn push_expr_type(&mut self, expr_id: ExprId, typ: Type) {
        self.id_to_type.insert(expr_id.into(), typ);
    }

    /// Store the type for a definition
    pub fn push_definition_type(&mut self, definition_id: DefinitionId, typ: Type) {
        self.definition_to_type.insert(definition_id, typ);
    }

    pub fn push_empty_trait(
        &mut self,
        type_id: TraitId,
        unresolved_trait: &UnresolvedTrait,
        generics: Generics,
    ) {
        let self_type_typevar_id = self.next_type_variable_id();

        let new_trait = Trait {
            id: type_id,
            name: unresolved_trait.trait_def.name.clone(),
            crate_id: unresolved_trait.crate_id,
            location: Location::new(unresolved_trait.trait_def.span, unresolved_trait.file_id),
            generics,
            self_type_typevar_id,
            self_type_typevar: TypeVariable::unbound(self_type_typevar_id),
            methods: Vec::new(),
            method_ids: unresolved_trait.method_ids.clone(),
            constants: Vec::new(),
            types: Vec::new(),
        };

        self.traits.insert(type_id, new_trait);
    }

    pub fn new_struct(
        &mut self,
        typ: &UnresolvedStruct,
        generics: Generics,
        krate: CrateId,
        local_id: LocalModuleId,
        file_id: FileId,
    ) -> StructId {
        let struct_id = StructId(ModuleId { krate, local_id });
        let name = typ.struct_def.name.clone();

        // Fields will be filled in later
        let no_fields = Vec::new();

        let location = Location::new(typ.struct_def.span, file_id);
        let new_struct = StructType::new(struct_id, name, location, no_fields, generics);
        self.structs.insert(struct_id, Shared::new(new_struct));
        self.struct_attributes.insert(struct_id, typ.struct_def.attributes.clone());
        struct_id
    }

    pub fn push_type_alias(
        &mut self,
        typ: &UnresolvedTypeAlias,
        generics: Generics,
    ) -> TypeAliasId {
        let type_id = TypeAliasId(self.type_aliases.len());

        self.type_aliases.push(Shared::new(TypeAlias::new(
            type_id,
            typ.type_alias_def.name.clone(),
            Location::new(typ.type_alias_def.span, typ.file_id),
            Type::Error,
            generics,
        )));

        type_id
    }

    /// Adds [TypeLiasId] and [Location] to the type_alias_ref vector
    /// So that we can later resolve [Location]s type aliases from the LSP requests
    pub fn add_type_alias_ref(&mut self, type_id: TypeAliasId, location: Location) {
        self.type_alias_ref.push((type_id, location));
    }
    pub fn update_struct(&mut self, type_id: StructId, f: impl FnOnce(&mut StructType)) {
        let mut value = self.structs.get_mut(&type_id).unwrap().borrow_mut();
        f(&mut value);
    }

    pub fn update_trait(&mut self, trait_id: TraitId, f: impl FnOnce(&mut Trait)) {
        let value = self.traits.get_mut(&trait_id).unwrap();
        f(value);
    }

    pub fn update_struct_attributes(
        &mut self,
        type_id: StructId,
        f: impl FnOnce(&mut StructAttributes),
    ) {
        let value = self.struct_attributes.get_mut(&type_id).unwrap();
        f(value);
    }

    pub fn set_type_alias(&mut self, type_id: TypeAliasId, typ: Type, generics: Generics) {
        let type_alias_type = &mut self.type_aliases[type_id.0];
        type_alias_type.borrow_mut().set_type_and_generics(typ, generics);
    }

    /// Returns the interned statement corresponding to `stmt_id`
    pub fn update_statement(&mut self, stmt_id: &StmtId, f: impl FnOnce(&mut HirStatement)) {
        let def =
            self.nodes.get_mut(stmt_id.0).expect("ice: all statement ids should have definitions");

        match def {
            Node::Statement(stmt) => f(stmt),
            _ => panic!("ice: all statement ids should correspond to a statement in the interner"),
        }
    }

    /// Updates the interned expression corresponding to `expr_id`
    pub fn update_expression(&mut self, expr_id: ExprId, f: impl FnOnce(&mut HirExpression)) {
        let def =
            self.nodes.get_mut(expr_id.0).expect("ice: all expression ids should have definitions");

        match def {
            Node::Expression(expr) => f(expr),
            _ => {
                panic!("ice: all expression ids should correspond to a expression in the interner")
            }
        }
    }

    /// Store [Location] of [Type] reference
    pub fn push_type_ref_location(&mut self, typ: Type, location: Location) {
        self.type_ref_locations.push((typ, location));
    }

    #[allow(clippy::too_many_arguments)]
    fn push_global(
        &mut self,
        ident: Ident,
        local_id: LocalModuleId,
        crate_id: CrateId,
        let_statement: StmtId,
        file: FileId,
        attributes: Vec<SecondaryAttribute>,
        mutable: bool,
        comptime: bool,
    ) -> GlobalId {
        let id = GlobalId(self.globals.len());
        let location = Location::new(ident.span(), file);
        let name = ident.to_string();
        let definition_id =
            self.push_definition(name, mutable, comptime, DefinitionKind::Global(id), location);

        self.globals.push(GlobalInfo {
            id,
            definition_id,
            ident,
            local_id,
            crate_id,
            let_statement,
            location,
            value: None,
        });
        self.global_attributes.insert(id, attributes);
        id
    }

    pub fn next_global_id(&mut self) -> GlobalId {
        GlobalId(self.globals.len())
    }

    /// Intern an empty global. Used for collecting globals before they're defined
    #[allow(clippy::too_many_arguments)]
    pub fn push_empty_global(
        &mut self,
        name: Ident,
        local_id: LocalModuleId,
        crate_id: CrateId,
        file: FileId,
        attributes: Vec<SecondaryAttribute>,
        mutable: bool,
        comptime: bool,
    ) -> GlobalId {
        let statement = self.push_stmt(HirStatement::Error);
        let span = name.span();
        let id = self
            .push_global(name, local_id, crate_id, statement, file, attributes, mutable, comptime);
        self.push_stmt_location(statement, span, file);
        id
    }

    /// Intern an empty function.
    pub fn push_empty_fn(&mut self) -> FuncId {
        self.push_fn(HirFunction::empty())
    }
    /// Updates the underlying interned Function.
    ///
    /// This method is used as we eagerly intern empty functions to
    /// generate function identifiers and then we update at a later point in
    /// time.
    pub fn update_fn(&mut self, func_id: FuncId, hir_func: HirFunction) {
        let def =
            self.nodes.get_mut(func_id.0).expect("ice: all function ids should have definitions");

        let func = match def {
            Node::Function(func) => func,
            _ => panic!("ice: all function ids should correspond to a function in the interner"),
        };
        *func = hir_func;
    }

    pub fn find_function(&self, function_name: &str) -> Option<FuncId> {
        self.func_meta
            .iter()
            .find(|(func_id, _func_meta)| self.function_name(func_id) == function_name)
            .map(|(func_id, _meta)| *func_id)
    }

    ///Interns a function's metadata.
    ///
    /// Note that the FuncId has been created already.
    /// See ModCollector for it's usage.
    pub fn push_fn_meta(&mut self, func_data: FuncMeta, func_id: FuncId) {
        self.func_meta.insert(func_id, func_data);
    }

    pub fn push_definition(
        &mut self,
        name: String,
        mutable: bool,
        comptime: bool,
        definition: DefinitionKind,
        location: Location,
    ) -> DefinitionId {
        let id = DefinitionId(self.definitions.len());
        let is_local = matches!(definition, DefinitionKind::Local(_));

        if let DefinitionKind::Function(func_id) = definition {
            self.function_definition_ids.insert(func_id, id);
        }

        let kind = definition;
        self.definitions.push(DefinitionInfo { name, mutable, comptime, kind, location });

        if is_local {
            self.add_definition_location(ReferenceId::Local(id), None);
        }

        id
    }

    /// Push a function with the default modifiers and [`ModuleId`] for testing
    #[cfg(test)]
    pub fn push_test_function_definition(&mut self, name: String) -> FuncId {
        let id = self.push_fn(HirFunction::empty());
        let mut modifiers = FunctionModifiers::new();
        modifiers.name = name;
        let module = ModuleId::dummy_id();
        let location = Location::dummy();
        self.push_function_definition(id, modifiers, module, location);
        id
    }

    pub fn push_function(
        &mut self,
        id: FuncId,
        function: &FunctionDefinition,
        module: ModuleId,
        location: Location,
    ) -> DefinitionId {
        let modifiers = FunctionModifiers {
            name: function.name.0.contents.clone(),
            visibility: function.visibility,
            attributes: function.attributes.clone(),
            is_unconstrained: function.is_unconstrained,
            generic_count: function.generics.len(),
            is_comptime: function.is_comptime,
            name_location: Location::new(function.name.span(), location.file),
        };
        let definition_id = self.push_function_definition(id, modifiers, module, location);

        // This needs to be done after pushing the definition since it will reference the
        // location that was stored
        self.add_definition_location(ReferenceId::Function(id), Some(module));

        definition_id
    }

    pub fn push_function_definition(
        &mut self,
        func: FuncId,
        modifiers: FunctionModifiers,
        module: ModuleId,
        location: Location,
    ) -> DefinitionId {
        let name = modifiers.name.clone();
        let comptime = modifiers.is_comptime;
        self.function_modifiers.insert(func, modifiers);
        self.function_modules.insert(func, module);
        self.push_definition(name, false, comptime, DefinitionKind::Function(func), location)
    }

    pub fn set_function_trait(&mut self, func: FuncId, self_type: Type, trait_id: TraitId) {
        self.func_id_to_trait.insert(func, (self_type, trait_id));
    }

    pub fn get_function_trait(&self, func: &FuncId) -> Option<(Type, TraitId)> {
        self.func_id_to_trait.get(func).cloned()
    }

    /// Returns the visibility of the given function.
    ///
    /// The underlying function_visibilities map is populated during def collection,
    /// so this function can be called anytime afterward.
    pub fn function_visibility(&self, func: FuncId) -> ItemVisibility {
        self.function_modifiers[&func].visibility
    }

    /// Returns the module this function was defined within
    pub fn function_module(&self, func: FuncId) -> ModuleId {
        self.function_modules[&func]
    }

    /// Returns the [`FuncId`] corresponding to the function referred to by `expr_id`
    pub fn lookup_function_from_expr(&self, expr: &ExprId) -> Option<FuncId> {
        if let HirExpression::Ident(HirIdent { id, .. }, _) = self.expression(expr) {
            match self.try_definition(id).map(|def| &def.kind) {
                Some(DefinitionKind::Function(func_id)) => Some(*func_id),
                Some(DefinitionKind::Local(Some(expr_id))) => {
                    self.lookup_function_from_expr(expr_id)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Returns the interned HIR function corresponding to `func_id`
    //
    // Cloning HIR structures is cheap, so we return owned structures
    pub fn function(&self, func_id: &FuncId) -> HirFunction {
        let def = self.nodes.get(func_id.0).expect("ice: all function ids should have definitions");

        match def {
            Node::Function(func) => func.clone(),
            _ => panic!("ice: all function ids should correspond to a function in the interner"),
        }
    }

    /// Returns the interned meta data corresponding to `func_id`
    pub fn function_meta(&self, func_id: &FuncId) -> &FuncMeta {
        self.func_meta.get(func_id).expect("ice: all function ids should have metadata")
    }

    pub fn function_meta_mut(&mut self, func_id: &FuncId) -> &mut FuncMeta {
        self.func_meta.get_mut(func_id).expect("ice: all function ids should have metadata")
    }

    pub fn try_function_meta(&self, func_id: &FuncId) -> Option<&FuncMeta> {
        self.func_meta.get(func_id)
    }

    pub fn function_ident(&self, func_id: &FuncId) -> crate::ast::Ident {
        let name = self.function_name(func_id).to_owned();
        let span = self.function_meta(func_id).name.location.span;
        crate::ast::Ident(Spanned::from(span, name))
    }

    pub fn function_name(&self, func_id: &FuncId) -> &str {
        &self.function_modifiers[func_id].name
    }

    pub fn function_modifiers(&self, func_id: &FuncId) -> &FunctionModifiers {
        &self.function_modifiers[func_id]
    }

    pub fn function_modifiers_mut(&mut self, func_id: &FuncId) -> &mut FunctionModifiers {
        self.function_modifiers.get_mut(func_id).expect("func_id should always have modifiers")
    }

    pub fn function_attributes(&self, func_id: &FuncId) -> &Attributes {
        &self.function_modifiers[func_id].attributes
    }

    pub fn struct_attributes(&self, struct_id: &StructId) -> &StructAttributes {
        &self.struct_attributes[struct_id]
    }

    pub fn add_module_attributes(&mut self, module_id: ModuleId, attributes: ModuleAttributes) {
        self.module_attributes.insert(module_id, attributes);
    }

    pub fn module_attributes(&self, module_id: &ModuleId) -> &ModuleAttributes {
        &self.module_attributes[module_id]
    }

    pub fn try_module_attributes(&self, module_id: &ModuleId) -> Option<&ModuleAttributes> {
        self.module_attributes.get(module_id)
    }

    pub fn try_module_parent(&self, module_id: &ModuleId) -> Option<LocalModuleId> {
        self.try_module_attributes(module_id).map(|attrs| attrs.parent)
    }

    pub fn global_attributes(&self, global_id: &GlobalId) -> &[SecondaryAttribute] {
        &self.global_attributes[global_id]
    }

    /// Returns the interned statement corresponding to `stmt_id`
    pub fn statement(&self, stmt_id: &StmtId) -> HirStatement {
        let def =
            self.nodes.get(stmt_id.0).expect("ice: all statement ids should have definitions");

        match def {
            Node::Statement(stmt) => stmt.clone(),
            _ => panic!("ice: all statement ids should correspond to a statement in the interner"),
        }
    }

    /// Try to get the `HirLetStatement` which defines a given global value
    pub fn get_global_let_statement(&self, global: GlobalId) -> Option<HirLetStatement> {
        let global = self.get_global(global);
        let def = self.nodes.get(global.let_statement.0)?;

        match def {
            Node::Statement(hir_stmt) => match hir_stmt {
                HirStatement::Let(let_stmt) => Some(let_stmt.clone()),
                HirStatement::Error => None,
                other => {
                    panic!("ice: all globals should correspond to a let statement in the interner: {other:?}")
                }
            },
            _ => panic!("ice: all globals should correspond to a statement in the interner"),
        }
    }

    /// Returns the interned expression corresponding to `expr_id`
    pub fn expression(&self, expr_id: &ExprId) -> HirExpression {
        let def =
            self.nodes.get(expr_id.0).expect("ice: all expression ids should have definitions");

        match def {
            Node::Expression(expr) => expr.clone(),
            _ => {
                panic!("ice: all expression ids should correspond to a expression in the interner")
            }
        }
    }

    /// Retrieves the definition where the given id was defined.
    /// This will panic if given DefinitionId::dummy_id. Use try_definition for
    /// any call with a possibly undefined variable.
    pub fn definition(&self, id: DefinitionId) -> &DefinitionInfo {
        &self.definitions[id.0]
    }

    /// Retrieves the definition where the given id was defined.
    /// This will panic if given DefinitionId::dummy_id. Use try_definition for
    /// any call with a possibly undefined variable.
    pub fn definition_mut(&mut self, id: DefinitionId) -> &mut DefinitionInfo {
        &mut self.definitions[id.0]
    }

    /// Tries to retrieve the given id's definition.
    /// This function should be used during name resolution or type checking when we cannot be sure
    /// all variables have corresponding definitions (in case of an error in the user's code).
    pub fn try_definition(&self, id: DefinitionId) -> Option<&DefinitionInfo> {
        self.definitions.get(id.0)
    }

    /// Returns the name of the definition
    ///
    /// This is needed as the Environment needs to map variable names to witness indices
    pub fn definition_name(&self, id: DefinitionId) -> &str {
        &self.definition(id).name
    }

    pub fn expr_span(&self, expr_id: &ExprId) -> Span {
        self.id_location(expr_id).span
    }

    pub fn try_expr_span(&self, expr_id: &ExprId) -> Option<Span> {
        self.try_id_location(expr_id).map(|location| location.span)
    }

    pub fn expr_location(&self, expr_id: &ExprId) -> Location {
        self.id_location(expr_id)
    }

    pub fn statement_span(&self, stmt_id: StmtId) -> Span {
        self.id_location(stmt_id).span
    }

    pub fn statement_location(&self, stmt_id: StmtId) -> Location {
        self.id_location(stmt_id)
    }

    pub fn push_stmt_location(&mut self, id: StmtId, span: Span, file: FileId) {
        self.id_to_location.insert(id.into(), Location::new(span, file));
    }

    pub fn get_struct(&self, id: StructId) -> Shared<StructType> {
        self.structs[&id].clone()
    }

    pub fn get_struct_methods(&self, id: StructId) -> Option<&HashMap<String, Methods>> {
        self.struct_methods.get(&id)
    }

    fn get_primitive_methods(&self, key: TypeMethodKey) -> Option<&HashMap<String, Methods>> {
        self.primitive_methods.get(&key)
    }

    pub fn get_type_methods(&self, typ: &Type) -> Option<&HashMap<String, Methods>> {
        match typ {
            Type::Struct(struct_type, _) => {
                let struct_type = struct_type.borrow();
                self.get_struct_methods(struct_type.id)
            }
            Type::Alias(type_alias, generics) => {
                let type_alias = type_alias.borrow();
                let typ = type_alias.get_type(generics);
                self.get_type_methods(&typ)
            }
            _ => get_type_method_key(typ).and_then(|key| self.get_primitive_methods(key)),
        }
    }

    pub fn get_trait(&self, id: TraitId) -> &Trait {
        &self.traits[&id]
    }

    pub fn get_trait_mut(&mut self, id: TraitId) -> &mut Trait {
        self.traits.get_mut(&id).expect("get_trait_mut given invalid TraitId")
    }

    pub fn try_get_trait(&self, id: TraitId) -> Option<&Trait> {
        self.traits.get(&id)
    }

    pub fn get_type_alias(&self, id: TypeAliasId) -> Shared<TypeAlias> {
        self.type_aliases[id.0].clone()
    }

    pub fn get_global(&self, global_id: GlobalId) -> &GlobalInfo {
        &self.globals[global_id.0]
    }

    pub fn get_global_mut(&mut self, global_id: GlobalId) -> &mut GlobalInfo {
        &mut self.globals[global_id.0]
    }

    pub fn get_global_definition(&self, global_id: GlobalId) -> &DefinitionInfo {
        let global = self.get_global(global_id);
        self.definition(global.definition_id)
    }

    pub fn get_global_definition_mut(&mut self, global_id: GlobalId) -> &mut DefinitionInfo {
        let global = self.get_global(global_id);
        self.definition_mut(global.definition_id)
    }

    pub fn get_all_globals(&self) -> &[GlobalInfo] {
        &self.globals
    }

    /// Returns the type of an item stored in the Interner or Error if it was not found.
    pub fn id_type(&self, index: impl Into<Index>) -> Type {
        self.id_to_type.get(&index.into()).cloned().unwrap_or(Type::Error)
    }

    /// Returns the type of the definition or `Type::Error` if it was not found.
    pub fn definition_type(&self, id: DefinitionId) -> Type {
        self.definition_to_type.get(&id).cloned().unwrap_or(Type::Error)
    }

    pub fn id_type_substitute_trait_as_type(&self, def_id: DefinitionId) -> Type {
        let typ = self.definition_type(def_id);
        if let Type::Function(args, ret, env, unconstrained) = &typ {
            let def = self.definition(def_id);
            if let Type::TraitAsType(..) = ret.as_ref() {
                if let DefinitionKind::Function(func_id) = def.kind {
                    let f = self.function(&func_id);
                    let func_body = f.as_expr();
                    let ret_type = self.id_type(func_body);
                    let new_type = Type::Function(
                        args.clone(),
                        Box::new(ret_type),
                        env.clone(),
                        *unconstrained,
                    );
                    return new_type;
                }
            }
        }
        typ
    }

    /// Returns the span of an item stored in the Interner
    pub fn id_location(&self, index: impl Into<Index> + Copy) -> Location {
        self.try_id_location(index)
            .unwrap_or_else(|| panic!("ID is missing a source location: {:?}", index.into()))
    }

    /// Returns the span of an item stored in the Interner, if present
    pub fn try_id_location(&self, index: impl Into<Index>) -> Option<Location> {
        self.id_to_location.get(&index.into()).copied()
    }

    /// Replaces the HirExpression at the given ExprId with a new HirExpression
    pub fn replace_expr(&mut self, id: &ExprId, new: HirExpression) {
        let old = self.nodes.get_mut(id.into()).unwrap();
        *old = Node::Expression(new);
    }

    /// Replaces the HirStatement at the given StmtId with a new HirStatement
    pub fn replace_statement(&mut self, stmt_id: StmtId, hir_stmt: HirStatement) {
        let old = self.nodes.get_mut(stmt_id.0).unwrap();
        *old = Node::Statement(hir_stmt);
    }

    pub fn next_type_variable_id(&self) -> TypeVariableId {
        let id = self.next_type_variable_id.get();
        self.next_type_variable_id.set(id + 1);
        TypeVariableId(id)
    }

    pub fn next_type_variable(&self) -> Type {
        Type::type_variable(self.next_type_variable_id())
    }

    pub fn store_instantiation_bindings(
        &mut self,
        expr_id: ExprId,
        instantiation_bindings: TypeBindings,
    ) {
        self.instantiation_bindings.insert(expr_id, instantiation_bindings);
    }

    pub fn get_instantiation_bindings(&self, expr_id: ExprId) -> &TypeBindings {
        &self.instantiation_bindings[&expr_id]
    }

    pub fn get_field_index(&self, expr_id: ExprId) -> usize {
        self.field_indices[&expr_id]
    }

    pub fn set_field_index(&mut self, expr_id: ExprId, index: usize) {
        self.field_indices.insert(expr_id, index);
    }

    pub fn function_definition_id(&self, function: FuncId) -> DefinitionId {
        self.function_definition_ids[&function]
    }

    /// Returns the DefinitionId of a trait's method, panics if the given trait method
    /// is not a valid method of the trait or if the trait has not yet had
    /// its methods ids set during name resolution.
    pub fn trait_method_id(&self, trait_method: TraitMethodId) -> DefinitionId {
        let the_trait = self.get_trait(trait_method.trait_id);
        let method_name = &the_trait.methods[trait_method.method_index].name;
        let function_id = the_trait.method_ids[&method_name.0.contents];
        self.function_definition_id(function_id)
    }

    /// Adds a non-trait method to a type.
    ///
    /// Returns `Some(duplicate)` if a matching method was already defined.
    /// Returns `None` otherwise.
    pub fn add_method(
        &mut self,
        self_type: &Type,
        method_name: String,
        method_id: FuncId,
        is_trait_method: bool,
    ) -> Option<FuncId> {
        match self_type {
            Type::Struct(struct_type, _generics) => {
                let id = struct_type.borrow().id;

                if let Some(existing) = self.lookup_method(self_type, id, &method_name, true) {
                    return Some(existing);
                }

                self.struct_methods
                    .entry(id)
                    .or_default()
                    .entry(method_name)
                    .or_default()
                    .add_method(method_id, is_trait_method);
                None
            }
            Type::Error => None,
            Type::MutableReference(element) => {
                self.add_method(element, method_name, method_id, is_trait_method)
            }

            other => {
                let key = get_type_method_key(self_type).unwrap_or_else(|| {
                    unreachable!("Cannot add a method to the unsupported type '{}'", other)
                });
                self.primitive_methods
                    .entry(key)
                    .or_default()
                    .entry(method_name)
                    .or_default()
                    .add_method(method_id, is_trait_method);
                None
            }
        }
    }

    pub fn try_get_trait_implementation(&self, id: TraitImplId) -> Option<Shared<TraitImpl>> {
        self.trait_implementations.get(&id).cloned()
    }

    pub fn get_trait_implementation(&self, id: TraitImplId) -> Shared<TraitImpl> {
        self.trait_implementations[&id].clone()
    }

    /// If the given function belongs to a trait impl, return its trait method id.
    /// Otherwise, return None.
    pub fn get_trait_method_id(&self, function: FuncId) -> Option<TraitMethodId> {
        let impl_id = self.function_meta(&function).trait_impl?;
        let trait_impl = self.get_trait_implementation(impl_id);
        let trait_impl = trait_impl.borrow();

        let method_index = trait_impl.methods.iter().position(|id| *id == function)?;
        Some(TraitMethodId { trait_id: trait_impl.trait_id, method_index })
    }

    /// Given a `ObjectType: TraitId` pair, try to find an existing impl that satisfies the
    /// constraint. If an impl cannot be found, this will return a vector of each constraint
    /// in the path to get to the failing constraint. Usually this is just the single failing
    /// constraint, but when where clauses are involved, the failing constraint may be several
    /// constraints deep. In this case, all of the constraints are returned, starting with the
    /// failing one.
    /// If this list of failing constraints is empty, this means type annotations are required.
    pub fn lookup_trait_implementation(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
    ) -> Result<TraitImplKind, Vec<TraitConstraint>> {
        let (impl_kind, bindings) =
            self.try_lookup_trait_implementation(object_type, trait_id, trait_generics)?;

        Type::apply_type_bindings(bindings);
        Ok(impl_kind)
    }

    /// Given a `ObjectType: TraitId` pair, find all implementations without taking constraints into account or
    /// applying any type bindings. Useful to look for a specific trait in a type that is used in a macro.
    pub fn lookup_all_trait_implementations(
        &self,
        object_type: &Type,
        trait_id: TraitId,
    ) -> Vec<&TraitImplKind> {
        let trait_impl = self.trait_implementation_map.get(&trait_id);

        trait_impl
            .map(|trait_impl| {
                trait_impl
                    .iter()
                    .filter_map(|(typ, impl_kind)| match &typ {
                        Type::Forall(_, typ) => {
                            if typ.deref() == object_type {
                                Some(impl_kind)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Similar to `lookup_trait_implementation` but does not apply any type bindings on success.
    /// On error returns either:
    /// - 1+ failing trait constraints, including the original.
    ///   Each constraint after the first represents a `where` clause that was followed.
    /// - 0 trait constraints indicating type annotations are needed to choose an impl.
    pub fn try_lookup_trait_implementation(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
    ) -> Result<(TraitImplKind, TypeBindings), Vec<TraitConstraint>> {
        let mut bindings = TypeBindings::new();
        let impl_kind = self.lookup_trait_implementation_helper(
            object_type,
            trait_id,
            trait_generics,
            &mut bindings,
            IMPL_SEARCH_RECURSION_LIMIT,
        )?;
        Ok((impl_kind, bindings))
    }

    /// Returns the trait implementation if found.
    /// On error returns either:
    /// - 1+ failing trait constraints, including the original.
    ///   Each constraint after the first represents a `where` clause that was followed.
    /// - 0 trait constraints indicating type annotations are needed to choose an impl.
    fn lookup_trait_implementation_helper(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
        type_bindings: &mut TypeBindings,
        recursion_limit: u32,
    ) -> Result<TraitImplKind, Vec<TraitConstraint>> {
        let make_constraint = || {
            TraitConstraint::new(
                object_type.clone(),
                trait_id,
                trait_generics.to_vec(),
                Span::default(),
            )
        };

        // Prevent infinite recursion when looking for impls
        if recursion_limit == 0 {
            return Err(vec![make_constraint()]);
        }

        let object_type = object_type.substitute(type_bindings);

        // If the object type isn't known, just return an error saying type annotations are needed.
        if object_type.is_bindable() {
            return Err(Vec::new());
        }

        let impls =
            self.trait_implementation_map.get(&trait_id).ok_or_else(|| vec![make_constraint()])?;

        let mut matching_impls = Vec::new();

        let mut where_clause_errors = Vec::new();

        for (existing_object_type2, impl_kind) in impls {
            // Bug: We're instantiating only the object type's generics here, not all of the trait's generics like we need to
            let (existing_object_type, instantiation_bindings) =
                existing_object_type2.instantiate(self);

            let mut fresh_bindings = type_bindings.clone();

            let mut check_trait_generics = |impl_generics: &[Type]| {
                trait_generics.iter().zip(impl_generics).all(|(trait_generic, impl_generic2)| {
                    let impl_generic = impl_generic2.substitute(&instantiation_bindings);
                    trait_generic.try_unify(&impl_generic, &mut fresh_bindings).is_ok()
                })
            };

            let generics_match = match impl_kind {
                TraitImplKind::Normal(id) => {
                    let shared_impl = self.get_trait_implementation(*id);
                    let shared_impl = shared_impl.borrow();
                    check_trait_generics(&shared_impl.trait_generics)
                }
                TraitImplKind::Assumed { trait_generics, .. } => {
                    check_trait_generics(trait_generics)
                }
            };

            if !generics_match {
                continue;
            }

            if object_type.try_unify(&existing_object_type, &mut fresh_bindings).is_ok() {
                if let TraitImplKind::Normal(impl_id) = impl_kind {
                    let trait_impl = self.get_trait_implementation(*impl_id);
                    let trait_impl = trait_impl.borrow();

                    if let Err(errors) = self.validate_where_clause(
                        &trait_impl.where_clause,
                        &mut fresh_bindings,
                        &instantiation_bindings,
                        recursion_limit,
                    ) {
                        // Only keep the first errors we get from a failing where clause
                        if where_clause_errors.is_empty() {
                            where_clause_errors.extend(errors);
                        }
                        continue;
                    }
                }

                matching_impls.push((impl_kind.clone(), fresh_bindings));
            }
        }

        if matching_impls.len() == 1 {
            let (impl_, fresh_bindings) = matching_impls.pop().unwrap();
            *type_bindings = fresh_bindings;
            Ok(impl_)
        } else if matching_impls.is_empty() {
            where_clause_errors.push(make_constraint());
            Err(where_clause_errors)
        } else {
            // multiple matching impls, type annotations needed
            Err(vec![])
        }
    }

    /// Verifies that each constraint in the given where clause is valid.
    /// If an impl cannot be found for any constraint, the erroring constraint is returned.
    fn validate_where_clause(
        &self,
        where_clause: &[TraitConstraint],
        type_bindings: &mut TypeBindings,
        instantiation_bindings: &TypeBindings,
        recursion_limit: u32,
    ) -> Result<(), Vec<TraitConstraint>> {
        for constraint in where_clause {
            // Instantiation bindings are generally safe to force substitute into the same type.
            // This is needed here to undo any bindings done to trait methods by monomorphization.
            // Otherwise, an impl for (A, B) could get narrowed to only an impl for e.g. (u8, u16).
            let constraint_type =
                constraint.typ.force_substitute(instantiation_bindings).substitute(type_bindings);

            let trait_generics = vecmap(&constraint.trait_generics, |generic| {
                generic.force_substitute(instantiation_bindings).substitute(type_bindings)
            });

            self.lookup_trait_implementation_helper(
                &constraint_type,
                constraint.trait_id,
                &trait_generics,
                // Use a fresh set of type bindings here since the constraint_type originates from
                // our impl list, which we don't want to bind to.
                type_bindings,
                recursion_limit - 1,
            )?;
        }

        Ok(())
    }

    /// Adds an "assumed" trait implementation to the currently known trait implementations.
    /// Unlike normal trait implementations, these are only assumed to exist. They often correspond
    /// to `where` clauses in functions where we assume there is some `T: Eq` even though we do
    /// not yet know T. For these cases, we store an impl here so that we assume they exist and
    /// can resolve them. They are then later verified when the function is called, and linked
    /// properly after being monomorphized to the correct variant.
    ///
    /// Returns true on success, or false if there is already an overlapping impl in scope.
    pub fn add_assumed_trait_implementation(
        &mut self,
        object_type: Type,
        trait_id: TraitId,
        trait_generics: Vec<Type>,
    ) -> bool {
        // Make sure there are no overlapping impls
        if self.try_lookup_trait_implementation(&object_type, trait_id, &trait_generics).is_ok() {
            return false;
        }

        let entries = self.trait_implementation_map.entry(trait_id).or_default();
        entries.push((object_type.clone(), TraitImplKind::Assumed { object_type, trait_generics }));
        true
    }

    /// Adds a trait implementation to the list of known implementations.
    pub fn add_trait_implementation(
        &mut self,
        object_type: Type,
        trait_id: TraitId,
        trait_generics: Vec<Type>,
        impl_id: TraitImplId,
        impl_generics: GenericTypeVars,
        trait_impl: Shared<TraitImpl>,
    ) -> Result<(), (Span, FileId)> {
        self.trait_implementations.insert(impl_id, trait_impl.clone());

        // Avoid adding error types to impls since they'll conflict with every other type.
        // We don't need to return an error since we expect an error to already be issued when
        // the error type is created.
        if object_type == Type::Error {
            return Ok(());
        }

        // Replace each generic with a fresh type variable
        let substitutions = impl_generics
            .into_iter()
            .map(|typevar| (typevar.id(), (typevar, self.next_type_variable())))
            .collect();

        let instantiated_object_type = object_type.substitute(&substitutions);

        // Ignoring overlapping `TraitImplKind::Assumed` impls here is perfectly fine.
        // It should never happen since impls are defined at global scope, but even
        // if they were, we should never prevent defining a new impl because a 'where'
        // clause already assumes it exists.
        if let Ok((TraitImplKind::Normal(existing), _)) = self.try_lookup_trait_implementation(
            &instantiated_object_type,
            trait_id,
            &trait_generics,
        ) {
            let existing_impl = self.get_trait_implementation(existing);
            let existing_impl = existing_impl.borrow();
            return Err((existing_impl.ident.span(), existing_impl.file));
        }

        for method in &trait_impl.borrow().methods {
            let method_name = self.function_name(method).to_owned();
            self.add_method(&object_type, method_name, *method, true);
        }

        // The object type is generalized so that a generic impl will apply
        // to any type T, rather than just the generic type named T.
        let generalized_object_type = object_type.generalize_from_substitutions(substitutions);

        let entries = self.trait_implementation_map.entry(trait_id).or_default();
        entries.push((generalized_object_type, TraitImplKind::Normal(impl_id)));
        Ok(())
    }

    /// Search by name for a method on the given struct.
    ///
    /// If `check_type` is true, this will force `lookup_method` to check the type
    /// of each candidate instead of returning only the first candidate if there is exactly one.
    /// This is generally only desired when declaring new methods to check if they overlap any
    /// existing methods.
    ///
    /// Another detail is that this method does not handle auto-dereferencing through `&mut T`.
    /// So if an object is of type `self : &mut T` but a method only accepts `self: T` (or
    /// vice-versa), the call will not be selected. If this is ever implemented into this method,
    /// we can remove the `methods.len() == 1` check and the `check_type` early return.
    pub fn lookup_method(
        &self,
        typ: &Type,
        id: StructId,
        method_name: &str,
        force_type_check: bool,
    ) -> Option<FuncId> {
        let methods = self.struct_methods.get(&id).and_then(|h| h.get(method_name));

        // If there is only one method, just return it immediately.
        // It will still be typechecked later.
        if !force_type_check {
            if let Some(method) = methods.and_then(|m| m.get_unambiguous()) {
                return Some(method);
            }
        }

        self.find_matching_method(typ, methods, method_name)
    }

    /// Select the 1 matching method with an object type matching `typ`
    fn find_matching_method(
        &self,
        typ: &Type,
        methods: Option<&Methods>,
        method_name: &str,
    ) -> Option<FuncId> {
        if let Some(method) = methods.and_then(|m| m.find_matching_method(typ, self)) {
            Some(method)
        } else {
            // Failed to find a match for the type in question, switch to looking at impls
            // for all types `T`, e.g. `impl<T> Foo for T`
            let global_methods =
                self.primitive_methods.get(&TypeMethodKey::Generic)?.get(method_name)?;
            global_methods.find_matching_method(typ, self)
        }
    }

    /// Looks up a given method name on the given primitive type.
    pub fn lookup_primitive_method(&self, typ: &Type, method_name: &str) -> Option<FuncId> {
        let key = get_type_method_key(typ)?;
        let methods = self.primitive_methods.get(&key)?.get(method_name)?;
        self.find_matching_method(typ, Some(methods), method_name)
    }

    pub fn lookup_primitive_trait_method_mut(
        &self,
        typ: &Type,
        method_name: &str,
    ) -> Option<FuncId> {
        let typ = Type::MutableReference(Box::new(typ.clone()));
        self.lookup_primitive_method(&typ, method_name)
    }

    /// Returns what the next trait impl id is expected to be.
    pub fn next_trait_impl_id(&mut self) -> TraitImplId {
        let next_id = self.next_trait_implementation_id;
        self.next_trait_implementation_id += 1;
        TraitImplId(next_id)
    }

    /// Removes all TraitImplKind::Assumed from the list of known impls for the given trait
    pub fn remove_assumed_trait_implementations_for_trait(&mut self, trait_id: TraitId) {
        let entries = self.trait_implementation_map.entry(trait_id).or_default();
        entries.retain(|(_, kind)| matches!(kind, TraitImplKind::Normal(_)));
    }

    /// Tags the given identifier with the selected trait_impl so that monomorphization
    /// can later recover which impl was selected, or alternatively see if it needs to
    /// decide which impl to select (because the impl was Assumed).
    pub fn select_impl_for_expression(&mut self, ident_id: ExprId, trait_impl: TraitImplKind) {
        self.selected_trait_implementations.insert(ident_id, trait_impl);
    }

    /// Retrieves the impl selected for a given ExprId during name resolution.
    pub fn get_selected_impl_for_expression(&self, ident_id: ExprId) -> Option<TraitImplKind> {
        self.selected_trait_implementations.get(&ident_id).cloned()
    }

    /// Retrieves the trait id for a given binary operator.
    /// All binary operators correspond to a trait - although multiple may correspond
    /// to the same trait (such as `==` and `!=`).
    /// `self.infix_operator_traits` is expected to be filled before name resolution,
    /// during definition collection.
    pub fn get_operator_trait_method(&self, operator: BinaryOpKind) -> TraitMethodId {
        let trait_id = self.infix_operator_traits[&operator];

        // Assume that the operator's method to be overloaded is the first method of the trait.
        TraitMethodId { trait_id, method_index: 0 }
    }

    /// Retrieves the trait id for a given unary operator.
    /// Only some unary operators correspond to a trait: `-` and `!`, but for example `*` does not.
    /// `self.prefix_operator_traits` is expected to be filled before name resolution,
    /// during definition collection.
    pub fn get_prefix_operator_trait_method(&self, operator: &UnaryOp) -> Option<TraitMethodId> {
        let trait_id = self.prefix_operator_traits.get(operator)?;

        // Assume that the operator's method to be overloaded is the first method of the trait.
        Some(TraitMethodId { trait_id: *trait_id, method_index: 0 })
    }

    /// Add the given trait as an operator trait if its name matches one of the
    /// operator trait names (Add, Sub, ...).
    pub fn try_add_infix_operator_trait(&mut self, trait_id: TraitId) {
        let the_trait = self.get_trait(trait_id);

        let operator = match the_trait.name.0.contents.as_str() {
            "Add" => BinaryOpKind::Add,
            "Sub" => BinaryOpKind::Subtract,
            "Mul" => BinaryOpKind::Multiply,
            "Div" => BinaryOpKind::Divide,
            "Rem" => BinaryOpKind::Modulo,
            "Eq" => BinaryOpKind::Equal,
            "Ord" => BinaryOpKind::Less,
            "BitAnd" => BinaryOpKind::And,
            "BitOr" => BinaryOpKind::Or,
            "BitXor" => BinaryOpKind::Xor,
            "Shl" => BinaryOpKind::ShiftLeft,
            "Shr" => BinaryOpKind::ShiftRight,
            _ => return,
        };

        self.infix_operator_traits.insert(operator, trait_id);

        // Some operators also require we insert a matching entry for related operators
        match operator {
            BinaryOpKind::Equal => {
                self.infix_operator_traits.insert(BinaryOpKind::NotEqual, trait_id);
            }
            BinaryOpKind::Less => {
                self.infix_operator_traits.insert(BinaryOpKind::LessEqual, trait_id);
                self.infix_operator_traits.insert(BinaryOpKind::Greater, trait_id);
                self.infix_operator_traits.insert(BinaryOpKind::GreaterEqual, trait_id);

                let the_trait = self.get_trait(trait_id);
                self.ordering_type = match &the_trait.methods[0].typ {
                    Type::Forall(_, typ) => match typ.as_ref() {
                        Type::Function(_, return_type, _, _) => Some(return_type.as_ref().clone()),
                        other => unreachable!("Expected function type for `cmp`, found {}", other),
                    },
                    other => unreachable!("Expected Forall type for `cmp`, found {}", other),
                };
            }
            _ => (),
        }
    }

    /// Add the given trait as an operator trait if its name matches one of the
    /// prefix operator trait names (Not or Neg).
    pub fn try_add_prefix_operator_trait(&mut self, trait_id: TraitId) {
        let the_trait = self.get_trait(trait_id);

        let operator = match the_trait.name.0.contents.as_str() {
            "Neg" => UnaryOp::Minus,
            "Not" => UnaryOp::Not,
            _ => return,
        };

        self.prefix_operator_traits.insert(operator, trait_id);
    }

    pub fn is_operator_trait(&self, trait_id: TraitId) -> bool {
        self.infix_operator_traits.values().any(|id| *id == trait_id)
            || self.prefix_operator_traits.values().any(|id| *id == trait_id)
    }

    /// This function is needed when creating a NodeInterner for testing so that calls
    /// to `get_operator_trait` do not panic when the stdlib isn't present.
    #[cfg(test)]
    pub fn populate_dummy_operator_traits(&mut self) {
        let dummy_trait = TraitId(ModuleId::dummy_id());
        self.infix_operator_traits.insert(BinaryOpKind::Add, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::Subtract, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::Multiply, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::Divide, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::Modulo, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::Equal, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::NotEqual, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::Less, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::LessEqual, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::Greater, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::GreaterEqual, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::And, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::Or, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::Xor, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::ShiftLeft, dummy_trait);
        self.infix_operator_traits.insert(BinaryOpKind::ShiftRight, dummy_trait);
        self.prefix_operator_traits.insert(UnaryOp::Minus, dummy_trait);
        self.prefix_operator_traits.insert(UnaryOp::Not, dummy_trait);
    }

    pub(crate) fn ordering_type(&self) -> Type {
        self.ordering_type.clone().expect("Expected ordering_type to be set in the NodeInterner")
    }

    /// Register that `dependent` depends on `dependency`.
    /// This is usually because `dependent` refers to `dependency` in one of its struct fields.
    pub fn add_type_dependency(&mut self, dependent: DependencyId, dependency: StructId) {
        self.add_dependency(dependent, DependencyId::Struct(dependency));
    }

    pub fn add_global_dependency(&mut self, dependent: DependencyId, dependency: GlobalId) {
        self.add_dependency(dependent, DependencyId::Global(dependency));
    }

    pub fn add_function_dependency(&mut self, dependent: DependencyId, dependency: FuncId) {
        self.add_dependency(dependent, DependencyId::Function(dependency));
    }

    pub fn add_type_alias_dependency(&mut self, dependent: DependencyId, dependency: TypeAliasId) {
        self.add_dependency(dependent, DependencyId::Alias(dependency));
    }

    pub fn add_dependency(&mut self, dependent: DependencyId, dependency: DependencyId) {
        let dependent_index = self.get_or_insert_dependency(dependent);
        let dependency_index = self.get_or_insert_dependency(dependency);
        self.dependency_graph.update_edge(dependent_index, dependency_index, ());
    }

    pub fn get_or_insert_dependency(&mut self, id: DependencyId) -> PetGraphIndex {
        if let Some(index) = self.dependency_graph_indices.get(&id) {
            return *index;
        }

        let index = self.dependency_graph.add_node(id);
        self.dependency_graph_indices.insert(id, index);
        index
    }

    pub(crate) fn check_for_dependency_cycles(&self) -> Vec<(CompilationError, FileId)> {
        let strongly_connected_components = tarjan_scc(&self.dependency_graph);
        let mut errors = Vec::new();

        let mut push_error = |item: String, scc: &[_], i, location: Location| {
            let cycle = self.get_cycle_error_string(scc, i);
            let span = location.span;
            let error = ResolverError::DependencyCycle { item, cycle, span };
            errors.push((error.into(), location.file));
        };

        for scc in strongly_connected_components {
            if scc.len() > 1 {
                // If a SCC contains a type, type alias, or global, it must be the only element in the SCC
                for (i, index) in scc.iter().enumerate() {
                    match self.dependency_graph[*index] {
                        DependencyId::Struct(struct_id) => {
                            let struct_type = self.get_struct(struct_id);
                            let struct_type = struct_type.borrow();
                            push_error(struct_type.name.to_string(), &scc, i, struct_type.location);
                            break;
                        }
                        DependencyId::Global(global_id) => {
                            let global = self.get_global(global_id);
                            let name = global.ident.to_string();
                            push_error(name, &scc, i, global.location);
                            break;
                        }
                        DependencyId::Alias(alias_id) => {
                            let alias = self.get_type_alias(alias_id);
                            // If type aliases form a cycle, we have to manually break the cycle
                            // here to prevent infinite recursion in the type checker.
                            alias.borrow_mut().typ = Type::Error;

                            // push_error will borrow the alias so we have to drop the mutable borrow
                            let alias = alias.borrow();
                            push_error(alias.name.to_string(), &scc, i, alias.location);
                            break;
                        }
                        // Mutually recursive functions are allowed
                        DependencyId::Function(_) => (),
                        // Local variables should never be in a dependency cycle, scoping rules
                        // prevents referring to them before they're defined
                        DependencyId::Variable(loc) => unreachable!(
                            "Variable used at location {loc:?} caught in a dependency cycle"
                        ),
                    }
                }
            }
        }

        errors
    }

    /// Build up a string starting from the given item containing each item in the dependency
    /// cycle. The final result will resemble `foo -> bar -> baz -> foo`, always going back to the
    /// element at the given start index.
    fn get_cycle_error_string(&self, scc: &[PetGraphIndex], start_index: usize) -> String {
        let index_to_string = |index: PetGraphIndex| match self.dependency_graph[index] {
            DependencyId::Struct(id) => Cow::Owned(self.get_struct(id).borrow().name.to_string()),
            DependencyId::Function(id) => Cow::Borrowed(self.function_name(&id)),
            DependencyId::Alias(id) => {
                Cow::Owned(self.get_type_alias(id).borrow().name.to_string())
            }
            DependencyId::Global(id) => {
                Cow::Borrowed(self.get_global(id).ident.0.contents.as_ref())
            }
            DependencyId::Variable(loc) => {
                unreachable!("Variable used at location {loc:?} caught in a dependency cycle")
            }
        };

        let mut cycle = index_to_string(scc[start_index]).to_string();

        // Reversing the dependencies here matches the order users would expect for the error message
        for i in (0..scc.len()).rev() {
            cycle += " -> ";
            cycle += &index_to_string(scc[(start_index + i) % scc.len()]);
        }

        cycle
    }

    pub fn push_quoted_type(&mut self, typ: Type) -> QuotedTypeId {
        QuotedTypeId(self.quoted_types.insert(typ))
    }

    pub fn get_quoted_type(&self, id: QuotedTypeId) -> &Type {
        &self.quoted_types[id.0]
    }

    /// Returns the type of an operator (which is always a function), along with its return type.
    pub fn get_infix_operator_type(
        &self,
        lhs: ExprId,
        operator: BinaryOpKind,
        operator_expr: ExprId,
    ) -> (Type, Type) {
        let lhs_type = self.id_type(lhs);
        let args = vec![lhs_type.clone(), lhs_type];

        // If this is a comparison operator, the result is a boolean but
        // the actual method call returns an Ordering
        use crate::ast::BinaryOpKind::*;
        let ret = if matches!(operator, Less | LessEqual | Greater | GreaterEqual) {
            self.ordering_type()
        } else {
            self.id_type(operator_expr)
        };

        let env = Box::new(Type::Unit);
        (Type::Function(args, Box::new(ret.clone()), env, false), ret)
    }

    /// Returns the type of a prefix operator (which is always a function), along with its return type.
    pub fn get_prefix_operator_type(&self, operator_expr: ExprId, rhs: ExprId) -> (Type, Type) {
        let rhs_type = self.id_type(rhs);
        let args = vec![rhs_type];
        let ret = self.id_type(operator_expr);
        let env = Box::new(Type::Unit);
        (Type::Function(args, Box::new(ret.clone()), env, false), ret)
    }

    pub fn is_in_lsp_mode(&self) -> bool {
        self.lsp_mode
    }
}

impl Methods {
    /// Get a single, unambiguous reference to a name if one exists.
    /// If not, there may be multiple methods of the same name for a given
    /// type or there may be no methods at all.
    fn get_unambiguous(&self) -> Option<FuncId> {
        if self.direct.len() == 1 {
            Some(self.direct[0])
        } else if self.direct.is_empty() && self.trait_impl_methods.len() == 1 {
            Some(self.trait_impl_methods[0])
        } else {
            None
        }
    }

    fn add_method(&mut self, method: FuncId, is_trait_method: bool) {
        if is_trait_method {
            self.trait_impl_methods.push(method);
        } else {
            self.direct.push(method);
        }
    }

    /// Iterate through each method, starting with the direct methods
    pub fn iter(&self) -> impl Iterator<Item = FuncId> + '_ {
        self.direct.iter().copied().chain(self.trait_impl_methods.iter().copied())
    }

    /// Select the 1 matching method with an object type matching `typ`
    fn find_matching_method(&self, typ: &Type, interner: &NodeInterner) -> Option<FuncId> {
        // When adding methods we always check they do not overlap, so there should be
        // at most 1 matching method in this list.
        for method in self.iter() {
            match interner.function_meta(&method).typ.instantiate(interner).0 {
                Type::Function(args, _, _, _) => {
                    if let Some(object) = args.first() {
                        let mut bindings = TypeBindings::new();

                        if object.try_unify(typ, &mut bindings).is_ok() {
                            Type::apply_type_bindings(bindings);
                            return Some(method);
                        }
                    }
                }
                Type::Error => (),
                other => unreachable!("Expected function type, found {other}"),
            }
        }
        None
    }
}

/// These are the primitive type variants that we support adding methods to
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
enum TypeMethodKey {
    /// Fields and integers share methods for ease of use. These methods may still
    /// accept only fields or integers, it is just that their names may not clash.
    FieldOrInt,
    Array,
    Slice,
    Bool,
    String,
    FmtString,
    Unit,
    Tuple,
    Function,
    Generic,
    Quoted(QuotedType),
}

fn get_type_method_key(typ: &Type) -> Option<TypeMethodKey> {
    use TypeMethodKey::*;
    let typ = typ.follow_bindings();
    match &typ {
        Type::FieldElement => Some(FieldOrInt),
        Type::Array(_, _) => Some(Array),
        Type::Slice(_) => Some(Slice),
        Type::Integer(_, _) => Some(FieldOrInt),
        Type::TypeVariable(_, TypeVariableKind::IntegerOrField) => Some(FieldOrInt),
        Type::TypeVariable(_, TypeVariableKind::Integer) => Some(FieldOrInt),
        Type::Bool => Some(Bool),
        Type::String(_) => Some(String),
        Type::FmtString(_, _) => Some(FmtString),
        Type::Unit => Some(Unit),
        Type::Tuple(_) => Some(Tuple),
        Type::Function(_, _, _, _) => Some(Function),
        Type::NamedGeneric(_, _, _) => Some(Generic),
        Type::Quoted(quoted) => Some(Quoted(*quoted)),
        Type::MutableReference(element) => get_type_method_key(element),
        Type::Alias(alias, _) => get_type_method_key(&alias.borrow().typ),

        // We do not support adding methods to these types
        Type::TypeVariable(_, _)
        | Type::Forall(_, _)
        | Type::Constant(_)
        | Type::Error
        | Type::Struct(_, _)
        | Type::InfixExpr(..)
        | Type::TraitAsType(..) => None,
    }
}
