use std::hash::Hash;
use std::marker::Copy;

use fm::FileId;
use noirc_arena::{Arena, Index};
use noirc_errors::{Location, Span};
use petgraph::prelude::DiGraph;
use petgraph::prelude::NodeIndex as PetGraphIndex;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use crate::QuotedType;
use crate::ast::DocComment;
use crate::ast::{
    ExpressionKind, Ident, LValue, Pattern, StatementKind, UnaryOp, UnresolvedTypeData,
    UnresolvedTypeExpression,
};
use crate::graph::CrateId;
use crate::hir::comptime;
use crate::hir::def_collector::dc_crate::{UnresolvedTrait, UnresolvedTypeAlias};
use crate::hir::def_map::{LocalModuleId, ModuleDefId, ModuleId};
use crate::hir::type_check::generics::TraitGenerics;
use crate::hir_def::traits::NamedType;
use crate::locations::AutoImportEntry;
use crate::node_interner::pusher::PushedExpr;
use crate::token::MetaAttribute;
use crate::token::MetaAttributeName;

use crate::Generics;
use crate::TraitAssociatedType;
use crate::ast::{BinaryOpKind, ItemVisibility};
use crate::hir_def::traits::{Trait, TraitConstraint, TraitImpl};
use crate::hir_def::types::{DataType, Kind, Type};
use crate::hir_def::{
    expr::HirExpression,
    function::{FuncMeta, HirFunction},
    stmt::HirStatement,
};
use crate::locations::LocationIndices;
use crate::token::{Attributes, SecondaryAttribute};
use crate::{Shared, TypeAlias, TypeBindings, TypeVariable, TypeVariableId};

mod dependency;
mod function;
mod globals;
mod ids;
mod methods;
mod reexports;
mod trait_impl;

pub mod pusher;

pub use dependency::DependencyId;
use globals::GlobalInfo;
pub use globals::{GlobalId, GlobalValue};
pub use ids::*;
pub use methods::{ImplMethod, Methods};
pub use reexports::Reexport;

#[derive(Debug)]
pub struct ModuleAttributes {
    pub name: String,
    pub location: Location,
    pub parent: Option<LocalModuleId>,
    pub visibility: ItemVisibility,
}

static TOP_LEVEL_MODULE_ATTRIBUTES: ModuleAttributes = ModuleAttributes {
    name: String::new(),
    location: Location::dummy(),
    parent: None,
    visibility: ItemVisibility::Public,
};

type TypeAttributes = Vec<SecondaryAttribute>;

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

    // Struct and Enum map.
    //
    // Each type definition is possibly shared across multiple type nodes.
    // It is also mutated through the RefCell during name resolution to append
    // methods from impls to the type.
    data_types: HashMap<TypeId, Shared<DataType>>,

    type_attributes: HashMap<TypeId, TypeAttributes>,

    // Maps TypeAliasId -> Shared<TypeAlias>
    //
    // Map type aliases to the actual type.
    // When resolving types, check against this map to see if a type alias is defined.
    pub(crate) type_aliases: Vec<Shared<TypeAlias>>,

    /// Each trait associated type. These are tracked so that we can distinguish them
    /// from other types and know that, when directly referenced, they should also
    /// lead to an "ambiguous associated type" error.
    pub(crate) trait_associated_types: Vec<TraitAssociatedType>,

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

    /// The associated types for each trait impl.
    /// This is stored outside of the TraitImpl object since it is required before that object is
    /// created, when resolving the type signature of each method in the impl.
    trait_impl_associated_types: HashMap<TraitImplId, Vec<NamedType>>,

    trait_impl_associated_constants: HashMap<TraitImplId, HashMap<String, (DefinitionId, Type)>>,

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
    pub instantiation_bindings: HashMap<ExprId, TypeBindings>,

    /// Remembers the field index a given HirMemberAccess expression was resolved to during type
    /// checking.
    field_indices: HashMap<ExprId, usize>,

    // Maps GlobalId -> GlobalInfo
    // NOTE: currently only used for checking repeat globals and restricting their scope to a module
    globals: Vec<GlobalInfo>,
    global_attributes: HashMap<GlobalId, Vec<SecondaryAttribute>>,

    next_type_variable_id: std::cell::Cell<usize>,

    /// A map from a type and method name to a function id for the method.
    /// This can resolve to potentially multiple methods if the same method name is
    /// specialized for different generics on the same type. E.g. for `Struct<T>`, we
    /// may have both `impl Struct<u32> { fn foo(){} }` and `impl Struct<u8> { fn foo(){} }`.
    /// If this happens, the returned Vec will have 2 entries and we'll need to further
    /// disambiguate them by checking the type of each function.
    methods: HashMap<TypeMethodKey, HashMap<String, Methods>>,

    // For trait implementation functions, this is their self type and trait they belong to
    func_id_to_trait: HashMap<FuncId, (Type, TraitId)>,

    /// A list of all type aliases that are referenced in the program.
    /// Searched by LSP to resolve [Location]s of [TypeAlias]s
    pub(crate) type_alias_ref: Vec<(TypeAliasId, Location)>,

    /// Stores the [Location] of a [Type] reference
    pub(crate) type_ref_locations: Vec<(Type, Location)>,

    /// In Noir's metaprogramming, a noir type has the type `Type`. When these are spliced
    /// into `quoted` expressions, we preserve the original type by assigning it a unique id
    /// and creating a `Token::QuotedType(id)` from this id. We cannot create a token holding
    /// the actual type since types do not implement Send or Sync.
    quoted_types: Arena<Type>,

    // Interned `ExpressionKind`s during comptime code.
    interned_expression_kinds: Arena<ExpressionKind>,

    // Interned `StatementKind`s during comptime code.
    interned_statement_kinds: Arena<StatementKind>,

    // Interned `UnresolvedTypeData`s during comptime code.
    interned_unresolved_type_data: Arena<UnresolvedTypeData>,

    // Interned `Pattern`s during comptime code.
    interned_patterns: Arena<Pattern>,

    /// Determines whether to run in LSP mode. In LSP mode references are tracked.
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

    // All names (and their definitions) that can be offered for auto_import.
    // The third value in the tuple is the module where the definition is (only for pub use).
    // These include top-level functions, global variables and types, but excludes
    // impl and trait-impl methods.
    pub(crate) auto_import_names: HashMap<String, Vec<AutoImportEntry>>,

    /// Each value currently in scope in the comptime interpreter.
    /// Each element of the Vec represents a scope with every scope together making
    /// up all currently visible definitions. The first scope is always the global scope.
    ///
    /// This is stored in the NodeInterner so that the Elaborator from each crate can
    /// share the same global values.
    pub(crate) comptime_scopes: Vec<HashMap<DefinitionId, comptime::Value>>,

    /// Captures the documentation comments for each module, struct, trait, function, etc.
    pub(crate) doc_comments: HashMap<ReferenceId, Vec<DocComment>>,

    /// A map of ModuleDefId to each module that pub or pub(crate) exports it.
    /// This is used to offer importing the item via one of these exports if
    /// the item is not visible where it's defined.
    pub reexports: HashMap<ModuleDefId, Vec<Reexport>>,

    /// Contains the docs comments of primitive types.
    /// These are defined in `noir_stdlib/src/primitive_docs.nr` using a tag
    /// attribute `#['nargo_primitive_doc]` on private modules.
    pub primitive_docs: HashMap<String, Vec<DocComment>>,

    /// Tracks expressions that encountered errors during elaboration.
    /// Used by the interpreter to skip evaluation of errored expressions.
    pub(crate) exprs_with_errors: HashSet<ExprId>,

    /// Tracks statements that encountered errors during elaboration.
    /// Used by the interpreter to skip evaluation of errored statements.
    pub(crate) stmts_with_errors: HashSet<StmtId>,
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
        trait_generics: TraitGenerics,
    },
}

/// When searching for a trait impl, these are the types of errors we can expect
pub enum ImplSearchErrorKind {
    TypeAnnotationsNeededOnObjectType,
    Nested(Vec<TraitConstraint>),
    MultipleMatching(Vec<String>),
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

    pub fn is_comptime_local(&self) -> bool {
        self.comptime && matches!(self.kind, DefinitionKind::Local(..))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DefinitionKind {
    Function(FuncId),

    Global(GlobalId),

    /// Locals may be defined in let statements or parameters,
    /// in which case they will not have an associated ExprId.
    /// For example a mutable variable can change, so it does
    /// not have a stable defining expression.
    Local(Option<ExprId>),

    /// Generic types in functions (T, U in `fn foo<T, U>(...)` are declared as variables
    /// in scope in case they resolve to numeric generics later.
    NumericGeneric(TypeVariable, Box<Type>),

    AssociatedConstant(TraitImplId, String),
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
            DefinitionKind::NumericGeneric(_, _) => None,
            DefinitionKind::AssociatedConstant(_, _) => None,
        }
    }
}

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
            dependency_graph: DiGraph::new(),
            dependency_graph_indices: HashMap::default(),
            id_to_location: HashMap::default(),
            definitions: vec![],
            id_to_type: HashMap::default(),
            definition_to_type: HashMap::default(),
            data_types: HashMap::default(),
            type_attributes: HashMap::default(),
            type_aliases: Vec::new(),
            trait_associated_types: Vec::new(),
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
            methods: HashMap::default(),
            type_alias_ref: Vec::new(),
            type_ref_locations: Vec::new(),
            quoted_types: Default::default(),
            interned_expression_kinds: Default::default(),
            interned_statement_kinds: Default::default(),
            interned_unresolved_type_data: Default::default(),
            interned_patterns: Default::default(),
            lsp_mode: false,
            location_indices: LocationIndices::default(),
            reference_graph: DiGraph::new(),
            reference_graph_indices: HashMap::default(),
            auto_import_names: HashMap::default(),
            comptime_scopes: vec![HashMap::default()],
            trait_impl_associated_types: HashMap::default(),
            trait_impl_associated_constants: HashMap::default(),
            doc_comments: HashMap::default(),
            reexports: HashMap::default(),
            primitive_docs: HashMap::default(),
            exprs_with_errors: HashSet::default(),
            stmts_with_errors: HashSet::default(),
        }
    }
}

// XXX: Add check that insertions are not overwrites for maps
// XXX: Maybe change push to intern, and remove comments
impl NodeInterner {
    /// Intern a HIR statement with everything needed for it (location).
    pub fn push_stmt_full(&mut self, stmt: HirStatement, location: Location) -> StmtId {
        let id = StmtId(self.nodes.insert(Node::Statement(stmt)));
        self.push_stmt_location(id, location);
        id
    }

    /// Interns a HIR expression, with the location and type information pushed as follow ups.
    pub fn push_expr(&mut self, expr: HirExpression) -> PushedExpr {
        PushedExpr::new(ExprId(self.nodes.insert(Node::Expression(expr))))
    }

    /// Intern an expression with everything needed for it (location & type)
    /// instead of requiring they be pushed later.
    pub fn push_expr_full(&mut self, expr: HirExpression, location: Location, typ: Type) -> ExprId {
        self.push_expr(expr).push_location(self, location).push_type(self, typ)
    }

    /// Stores the span for an interned expression.
    pub fn push_expr_location(&mut self, expr_id: ExprId, location: Location) {
        self.id_to_location.insert(expr_id.into(), location);
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
        associated_types: Generics,
        associated_constant_ids: HashMap<String, DefinitionId>,
    ) {
        let new_trait = Trait {
            id: type_id,
            name: unresolved_trait.trait_def.name.clone(),
            crate_id: unresolved_trait.crate_id,
            location: unresolved_trait.trait_def.name.location(),
            generics,
            visibility: ItemVisibility::Private,
            self_type_typevar: TypeVariable::unbound(self.next_type_variable_id(), Kind::Normal),
            methods: Vec::new(),
            method_ids: unresolved_trait.method_ids.clone(),
            associated_types,
            associated_type_bounds: HashMap::default(),
            trait_bounds: Vec::new(),
            where_clause: Vec::new(),
            all_generics: Vec::new(),
            associated_constant_ids,
        };

        self.traits.insert(type_id, new_trait);
    }

    /// Creates a new struct or enum type with no fields or variants.
    #[allow(clippy::too_many_arguments)]
    pub fn new_type(
        &mut self,
        name: Ident,
        span: Span,
        attributes: Vec<SecondaryAttribute>,
        generics: Generics,
        visibility: ItemVisibility,
        krate: CrateId,
        local_id: LocalModuleId,
        file_id: FileId,
    ) -> TypeId {
        let type_id = TypeId(ModuleId { krate, local_id });

        let location = Location::new(span, file_id);
        let new_type = DataType::new(type_id, name, location, generics, visibility);
        self.data_types.insert(type_id, Shared::new(new_type));
        self.type_attributes.insert(type_id, attributes);
        type_id
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
            typ.type_alias_def.location,
            Type::Error,
            generics,
            typ.type_alias_def.visibility,
            ModuleId { krate: typ.crate_id, local_id: typ.module_id },
        )));

        type_id
    }

    /// Adds [TypeAliasId] and [Location] to the type_alias_ref vector
    /// So that we can later resolve [Location]s type aliases from the LSP requests
    pub fn add_type_alias_ref(&mut self, type_id: TypeAliasId, location: Location) {
        self.type_alias_ref.push((type_id, location));
    }

    pub fn push_trait_associated_type(
        &mut self,
        trait_id: TraitId,
        name: Ident,
    ) -> TraitAssociatedTypeId {
        let id = TraitAssociatedTypeId(self.trait_associated_types.len());
        self.trait_associated_types.push(TraitAssociatedType { id, trait_id, name });
        id
    }

    pub fn update_type(&mut self, type_id: TypeId, f: impl FnOnce(&mut DataType)) {
        let mut value = self.data_types.get_mut(&type_id).unwrap().borrow_mut();
        f(&mut value);
    }

    pub fn update_trait(&mut self, trait_id: TraitId, f: impl FnOnce(&mut Trait)) {
        let value = self.traits.get_mut(&trait_id).unwrap();
        f(value);
    }

    pub fn update_type_attributes(&mut self, type_id: TypeId, f: impl FnOnce(&mut TypeAttributes)) {
        let value = self.type_attributes.get_mut(&type_id).unwrap();
        f(value);
    }

    pub fn set_type_alias(
        &mut self,
        type_id: TypeAliasId,
        typ: Type,
        generics: Generics,
        num_expr: Option<UnresolvedTypeExpression>,
    ) {
        let type_alias_type = &mut self.type_aliases[type_id.0];
        type_alias_type.borrow_mut().set_type_and_generics(typ, generics, num_expr);
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

    /// In LSP mode, take note that the [Type] was referenced at a [Location].
    pub fn push_type_ref_location(&mut self, typ: &Type, location: Location) {
        if !self.is_in_lsp_mode() {
            return;
        }

        self.type_ref_locations.push((typ.clone(), location));
    }

    pub fn definition_count(&self) -> usize {
        self.definitions.len()
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
            self.add_definition_location(ReferenceId::Local(id), location);
        }

        id
    }

    pub fn type_attributes(&self, struct_id: &TypeId) -> &TypeAttributes {
        &self.type_attributes[struct_id]
    }

    pub fn add_module_attributes(&mut self, module_id: ModuleId, attributes: ModuleAttributes) {
        self.module_attributes.insert(module_id, attributes);
    }

    pub fn module_attributes(&self, module_id: ModuleId) -> &ModuleAttributes {
        self.try_module_attributes(module_id).unwrap_or(&TOP_LEVEL_MODULE_ATTRIBUTES)
    }

    pub fn try_module_attributes(&self, module_id: ModuleId) -> Option<&ModuleAttributes> {
        self.module_attributes.get(&module_id)
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

    pub fn push_stmt_location(&mut self, id: StmtId, location: Location) {
        self.id_to_location.insert(id.into(), location);
    }

    pub fn get_type(&self, id: TypeId) -> Shared<DataType> {
        self.data_types[&id].clone()
    }

    pub fn get_type_methods(&self, typ: &Type) -> Option<&HashMap<String, Methods>> {
        get_type_method_key(typ).and_then(|key| self.methods.get(&key))
    }

    pub fn get_trait(&self, id: TraitId) -> &Trait {
        &self.traits[&id]
    }

    pub fn get_trait_associated_type(&self, id: TraitAssociatedTypeId) -> &TraitAssociatedType {
        &self.trait_associated_types[id.0]
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

    /// Returns the type of an item stored in the [NodeInterner], or [Type::Error] if it was not found.
    pub fn id_type(&self, index: impl Into<Index>) -> Type {
        self.try_id_type(index).cloned().unwrap_or(Type::Error)
    }

    pub fn try_id_type(&self, index: impl Into<Index>) -> Option<&Type> {
        self.id_to_type.get(&index.into())
    }

    /// Returns the type of the definition, or [Type::Error] if it was not found.
    pub fn definition_type(&self, id: DefinitionId) -> Type {
        self.definition_to_type.get(&id).cloned().unwrap_or(Type::Error)
    }

    /// Returns the type of the definition, unless it's a function returning an `impl Trait`,
    /// in which case it looks up the type of its body and returns a new function type with
    /// the type fo the body substituted to its return type.
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

    pub fn next_type_variable_with_kind(&self, kind: Kind) -> Type {
        Type::type_variable_with_kind(self, kind)
    }

    /// Remember the [TypeBindings] used during the instantiation of an expression.
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

    pub fn try_get_instantiation_bindings(&self, expr_id: ExprId) -> Option<&TypeBindings> {
        self.instantiation_bindings.get(&expr_id)
    }

    pub fn get_field_index(&self, expr_id: ExprId) -> usize {
        self.field_indices[&expr_id]
    }

    pub fn set_field_index(&mut self, expr_id: ExprId, index: usize) {
        self.field_indices.insert(expr_id, index);
    }

    /// Look up the [DefinitionId] of a [FuncId].
    ///
    /// Panics if it's not found.
    pub fn function_definition_id(&self, function: FuncId) -> DefinitionId {
        self.function_definition_ids[&function]
    }

    /// Returns the definition id and trait id for a given trait or impl function.
    ///
    /// If this is an impl function, the DefinitionId inside the TraitItemId will still
    /// be that of the function in the parent trait.
    pub fn get_trait_item_id(&self, function_id: FuncId) -> Option<TraitItemId> {
        let function = self.function_meta(&function_id);

        match function.trait_impl {
            Some(impl_id) => {
                let trait_id = self.get_trait_implementation(impl_id).borrow().trait_id;
                let the_trait = self.get_trait(trait_id);
                let name = self.definition_name(function.name.id);
                let definition_id = the_trait
                    .find_method_or_constant(name, self)
                    .expect("Expected parent trait to have function from impl");
                Some(TraitItemId { item_id: definition_id, trait_id })
            }
            None => {
                let trait_id = function.trait_id?;
                let definition_id = self.function_definition_id(function_id);
                Some(TraitItemId { item_id: definition_id, trait_id })
            }
        }
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
        trait_id: Option<TraitId>,
    ) -> Option<FuncId> {
        match self_type {
            Type::Error => None,
            Type::Reference(element, _mutable) => {
                self.add_method(element, method_name, method_id, trait_id)
            }
            _ => {
                let key = get_type_method_key(self_type).unwrap_or_else(|| {
                    unreachable!("Cannot add a method to the unsupported type '{}'", self_type)
                });

                if trait_id.is_none() && matches!(self_type, Type::DataType(..)) {
                    let check_self_param = false;
                    if let Some(existing) =
                        self.lookup_direct_method(self_type, &method_name, check_self_param)
                    {
                        return Some(existing);
                    }
                }

                // Only remember the actual type if it's FieldOrInt,
                // so later we can disambiguate on calls like `u32::call`.
                let typ = self_type.clone();
                self.methods
                    .entry(key)
                    .or_default()
                    .entry(method_name)
                    .or_default()
                    .add_method(method_id, typ, trait_id);
                None
            }
        }
    }

    /// Looks up a method that's directly defined in the given type.
    /// If `check_self_param` is `true`, only a method that has a `self` parameter with a type
    /// that unifies with `typ` will be returned.
    pub fn lookup_direct_method(
        &self,
        typ: &Type,
        method_name: &str,
        check_self_param: bool,
    ) -> Option<FuncId> {
        let key = get_type_method_key(typ)?;

        self.methods
            .get(&key)
            .and_then(|h| h.get(method_name))
            .and_then(|methods| methods.find_direct_method(typ, check_self_param, self))
    }

    /// Looks up methods that apply to the given type but are defined in traits.
    pub fn lookup_trait_methods(
        &self,
        typ: &Type,
        method_name: &str,
        has_self_arg: bool,
    ) -> Vec<(FuncId, TraitId)> {
        let key = get_type_method_key(typ);
        if let Some(key) = key {
            self.methods
                .get(&key)
                .and_then(|h| h.get(method_name))
                .map(|methods| methods.find_trait_methods(typ, has_self_arg, self))
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Looks up methods at impls for all types `T`, e.g. `impl<T> Foo for T`
    pub fn lookup_generic_methods(
        &self,
        typ: &Type,
        method_name: &str,
        has_self_arg: bool,
    ) -> Vec<(FuncId, TraitId)> {
        self.methods
            .get(&TypeMethodKey::Generic)
            .and_then(|h| h.get(method_name))
            .map(|methods| methods.find_trait_methods(typ, has_self_arg, self))
            .unwrap_or_default()
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
    pub fn get_operator_trait_method(&self, operator: BinaryOpKind) -> TraitItemId {
        let trait_id = self.infix_operator_traits[&operator];
        let the_trait = self.get_trait(trait_id);
        let func_id = *the_trait.method_ids.values().next().unwrap();
        TraitItemId { trait_id, item_id: self.function_definition_id(func_id) }
    }

    /// Retrieves the trait id for a given unary operator.
    /// Only some unary operators correspond to a trait: `-` and `!`, but for example `*` does not.
    /// `self.prefix_operator_traits` is expected to be filled before name resolution,
    /// during definition collection.
    pub fn get_prefix_operator_trait_method(&self, operator: &UnaryOp) -> Option<TraitItemId> {
        let trait_id = *self.prefix_operator_traits.get(operator)?;
        let the_trait = self.get_trait(trait_id);
        let func_id = *the_trait.method_ids.values().next().unwrap();
        Some(TraitItemId { trait_id, item_id: self.function_definition_id(func_id) })
    }

    /// Add the given trait as an operator trait if its name matches one of the
    /// operator trait names (Add, Sub, ...).
    pub fn try_add_infix_operator_trait(&mut self, trait_id: TraitId) {
        let the_trait = self.get_trait(trait_id);

        let operator = match the_trait.name.as_str() {
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

        let operator = match the_trait.name.as_str() {
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
    #[cfg(any(test, feature = "test_utils"))]
    pub fn populate_dummy_operator_traits(&mut self) {
        // Populate a dummy trait with a single method, as the trait, and its single methods,
        // are looked up during `get_operator_trait`.
        let mut usize_arena = Arena::default();
        let index = usize_arena.insert(0);
        let stdlib = CrateId::Stdlib(0);
        let func_id = self.push_empty_fn();
        // Use a definition ID that won't clash with anything else, and isn't the dummy one
        let definition_id = DefinitionId(usize::MAX - 1);
        self.function_definition_ids.insert(func_id, definition_id);
        let module_id = ModuleId { krate: stdlib, local_id: LocalModuleId::new(index) };
        let trait_id = TraitId(module_id);
        let self_type_typevar = self.next_type_variable_id();
        let mut method_ids: HashMap<String, FuncId> = Default::default();
        method_ids.insert("dummy_method".to_string(), func_id);

        let trait_ = Trait {
            id: trait_id,
            crate_id: stdlib,
            methods: vec![],
            method_ids,
            associated_types: vec![],
            associated_type_bounds: Default::default(),
            name: Ident::new("Dummy".to_string(), Location::dummy()),
            generics: vec![],
            location: Location::dummy(),
            visibility: ItemVisibility::Public,
            self_type_typevar: TypeVariable::unbound(self_type_typevar, Kind::Normal),
            trait_bounds: vec![],
            where_clause: vec![],
            all_generics: vec![],
            associated_constant_ids: Default::default(),
        };
        self.traits.insert(trait_id, trait_);

        let operators = [
            BinaryOpKind::Add,
            BinaryOpKind::Subtract,
            BinaryOpKind::Multiply,
            BinaryOpKind::Divide,
            BinaryOpKind::Modulo,
            BinaryOpKind::Equal,
            BinaryOpKind::NotEqual,
            BinaryOpKind::Less,
            BinaryOpKind::LessEqual,
            BinaryOpKind::Greater,
            BinaryOpKind::GreaterEqual,
            BinaryOpKind::And,
            BinaryOpKind::Or,
            BinaryOpKind::Xor,
            BinaryOpKind::ShiftLeft,
            BinaryOpKind::ShiftRight,
        ];

        // It's fine to use the same trait for all operators, at least in tests
        for operator in operators {
            self.infix_operator_traits.insert(operator, trait_id);
        }
    }

    pub(crate) fn ordering_type(&self) -> Type {
        self.ordering_type.clone().expect("Expected ordering_type to be set in the NodeInterner")
    }

    pub fn push_quoted_type(&mut self, typ: Type) -> QuotedTypeId {
        QuotedTypeId(self.quoted_types.insert(typ))
    }

    pub fn get_quoted_type(&self, id: QuotedTypeId) -> &Type {
        &self.quoted_types[id.0]
    }

    /// Intern a [ExpressionKind].
    pub fn push_expression_kind(&mut self, expr: ExpressionKind) -> InternedExpressionKind {
        InternedExpressionKind(self.interned_expression_kinds.insert(expr))
    }

    /// Get an interned [ExpressionKind] by its [InternedExpressionKind] ID.
    pub fn get_expression_kind(&self, id: InternedExpressionKind) -> &ExpressionKind {
        &self.interned_expression_kinds[id.0]
    }

    /// Intern a [StatementKind].
    pub fn push_statement_kind(&mut self, statement: StatementKind) -> InternedStatementKind {
        InternedStatementKind(self.interned_statement_kinds.insert(statement))
    }

    /// Get an interned [StatementKind] by its [InternedStatementKind] ID.
    pub fn get_statement_kind(&self, id: InternedStatementKind) -> &StatementKind {
        &self.interned_statement_kinds[id.0]
    }

    /// Intern an [LValue] by turning it into an [Expression][crate::ast::Expression] and interning its [ExpressionKind].
    pub fn push_lvalue(&mut self, lvalue: LValue) -> InternedExpressionKind {
        self.push_expression_kind(lvalue.as_expression().kind)
    }

    /// Get an interned [LValue] by its [InternedExpressionKind] ID.
    pub fn get_lvalue(&self, id: InternedExpressionKind, location: Location) -> LValue {
        LValue::from_expression_kind(self.get_expression_kind(id).clone(), location)
            .expect("Called LValue::from_expression with an invalid expression")
    }

    /// Intern a [Pattern].
    pub fn push_pattern(&mut self, pattern: Pattern) -> InternedPattern {
        InternedPattern(self.interned_patterns.insert(pattern))
    }

    /// Get an interned [Pattern] by its [InternedPattern] ID.
    pub fn get_pattern(&self, id: InternedPattern) -> &Pattern {
        &self.interned_patterns[id.0]
    }

    /// Intern a [UnresolvedTypeData].
    pub fn push_unresolved_type_data(
        &mut self,
        typ: UnresolvedTypeData,
    ) -> InternedUnresolvedTypeData {
        InternedUnresolvedTypeData(self.interned_unresolved_type_data.insert(typ))
    }

    pub fn get_unresolved_type_data(&self, id: InternedUnresolvedTypeData) -> &UnresolvedTypeData {
        &self.interned_unresolved_type_data[id.0]
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

    /// Sets the associated types for the given trait impl.
    /// Each type in [`NamedType`] will be wrapped in a [`Type::TypeVariable`] if it's of kind [`Kind::Numeric`].
    pub(crate) fn set_associated_types_for_impl(
        &mut self,
        impl_id: TraitImplId,
        associated_types: Vec<NamedType>,
    ) {
        // Wrap the named generics in type variables to be able to refer them as type variables
        for associated_type in &associated_types {
            let Kind::Numeric(numeric_type) = associated_type.typ.kind() else {
                continue;
            };

            let name = associated_type.name.to_string();
            let definition_id = self.push_definition(
                associated_type.name.to_string(),
                false,
                false,
                DefinitionKind::AssociatedConstant(impl_id, name.clone()),
                associated_type.name.location(),
            );
            self.trait_impl_associated_constants
                .entry(impl_id)
                .or_default()
                .insert(name, (definition_id, *numeric_type));
        }

        self.trait_impl_associated_types.insert(impl_id, associated_types);
    }

    /// Returns the associated types for the given trait impl.
    /// The Type of each [`NamedType`] that is an associated constant is guaranteed to be a [`Type::TypeVariable`].
    pub fn get_associated_types_for_impl(&self, impl_id: TraitImplId) -> &[NamedType] {
        &self.trait_impl_associated_types[&impl_id]
    }

    pub fn find_associated_type_for_impl(
        &self,
        impl_id: TraitImplId,
        type_name: &str,
    ) -> Option<&Type> {
        let types = self.trait_impl_associated_types.get(&impl_id)?;
        types.iter().find(|typ| typ.name.as_str() == type_name).map(|typ| &typ.typ)
    }

    /// Returns the definition id for the associated constant of the given type variable.
    pub fn get_trait_impl_associated_constant(
        &self,
        impl_id: TraitImplId,
        name: &str,
    ) -> Option<&(DefinitionId, Type)> {
        self.trait_impl_associated_constants.get(&impl_id).and_then(|map| map.get(name))
    }

    /// Return a set of TypeBindings to bind types from the parent trait to those from the trait impl.
    pub fn trait_to_impl_bindings(
        &self,
        trait_id: TraitId,
        impl_id: TraitImplId,
        trait_impl_generics: &[Type],
        impl_self_type: Type,
    ) -> TypeBindings {
        let mut bindings = TypeBindings::default();
        let the_trait = self.get_trait(trait_id);
        let trait_generics = the_trait.generics.clone();

        let self_type_var = the_trait.self_type_typevar.clone();
        bindings.insert(
            self_type_var.id(),
            (self_type_var.clone(), self_type_var.kind(), impl_self_type),
        );

        for (trait_generic, trait_impl_generic) in trait_generics.iter().zip(trait_impl_generics) {
            let type_var = trait_generic.type_var.clone();
            bindings.insert(
                type_var.id(),
                (type_var, trait_generic.kind(), trait_impl_generic.clone()),
            );
        }

        // Now that the normal bindings are added, we still need to bind the associated types
        let impl_associated_types = self.get_associated_types_for_impl(impl_id);
        let trait_associated_types = &the_trait.associated_types;

        // `impl_associated_types` may not be in the same order as `trait_associated_types`
        let impl_associated_types = impl_associated_types
            .iter()
            .map(|typ| (typ.name.as_str(), typ))
            .collect::<HashMap<_, _>>();

        for trait_type in trait_associated_types {
            let Some(impl_type) = impl_associated_types.get(trait_type.name.as_str()) else {
                // Impl doesn't have the corresponding associated type - an error should already
                // have been issued beforehand.
                continue;
            };

            let type_variable = trait_type.type_var.clone();
            bindings.insert(
                type_variable.id(),
                (type_variable, trait_type.kind(), impl_type.typ.clone()),
            );
        }

        bindings
    }

    pub fn set_doc_comments(&mut self, id: ReferenceId, doc_comments: Vec<DocComment>) {
        if !doc_comments.is_empty() {
            self.doc_comments.insert(id, doc_comments);
        }
    }

    pub fn doc_comments(&self, id: ReferenceId) -> Option<&Vec<DocComment>> {
        self.doc_comments.get(&id)
    }

    pub fn get_expr_id_from_index(&self, index: impl Into<Index>) -> Option<ExprId> {
        let index = index.into();
        match self.nodes.get(index) {
            Some(Node::Expression(_)) => Some(ExprId(index)),
            _ => None,
        }
    }

    pub fn get_meta_attribute_name(&self, meta: &MetaAttribute) -> Option<String> {
        match &meta.name {
            MetaAttributeName::Path(path) => Some(path.last_name().to_string()),
            MetaAttributeName::Resolved(expr_id) => {
                let HirExpression::Ident(ident, _) = self.expression(expr_id) else {
                    return None;
                };
                self.try_definition(ident.id).map(|def| def.name.clone())
            }
        }
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
    Struct(TypeId),
}

fn get_type_method_key(typ: &Type) -> Option<TypeMethodKey> {
    use TypeMethodKey::*;
    let typ = typ.follow_bindings();
    match &typ {
        Type::FieldElement => Some(FieldOrInt),
        Type::Array(_, _) => Some(Array),
        Type::Slice(_) => Some(Slice),
        Type::Integer(_, _) => Some(FieldOrInt),
        Type::TypeVariable(var) => {
            if var.is_integer() || var.is_integer_or_field() {
                Some(FieldOrInt)
            } else {
                None
            }
        }
        Type::Bool => Some(Bool),
        Type::String(_) => Some(String),
        Type::FmtString(_, _) => Some(FmtString),
        Type::Unit => Some(Unit),
        Type::Tuple(_) => Some(Tuple),
        Type::Function(_, _, _, _) => Some(Function),
        Type::NamedGeneric(_) => Some(Generic),
        Type::Quoted(quoted) => Some(Quoted(*quoted)),
        Type::Reference(element, _) => get_type_method_key(element),
        Type::Alias(alias, _) => get_type_method_key(&alias.borrow().typ),
        Type::DataType(struct_type, _) => Some(Struct(struct_type.borrow().id)),

        // We do not support adding methods to these types
        Type::Forall(_, _)
        | Type::Constant(..)
        | Type::Error
        | Type::InfixExpr(..)
        | Type::CheckedCast { .. }
        | Type::TraitAsType(..) => None,
    }
}
