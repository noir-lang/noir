use std::collections::HashMap;

use arena::{Arena, Index};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span, Spanned};

use crate::ast::Ident;
use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::{UnresolvedStruct, UnresolvedTrait, UnresolvedTypeAlias};
use crate::hir::def_map::{LocalModuleId, ModuleId};
use crate::hir::StorageSlot;
use crate::hir_def::stmt::HirLetStatement;
use crate::hir_def::traits::Trait;
use crate::hir_def::traits::TraitImpl;
use crate::hir_def::types::{StructType, Type};
use crate::hir_def::{
    expr::HirExpression,
    function::{FuncMeta, HirFunction},
    stmt::HirStatement,
};
use crate::token::{Attributes, SecondaryAttribute};
use crate::{
    ContractFunctionType, FunctionDefinition, Generics, Shared, TypeAliasType, TypeBinding,
    TypeBindings, TypeVariable, TypeVariableId, TypeVariableKind, Visibility,
};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct TraitImplKey {
    pub typ: Type,
    pub trait_id: TraitId,
    // pub generics: Generics - TODO
}

type StructAttributes = Vec<SecondaryAttribute>;

/// The node interner is the central storage location of all nodes in Noir's Hir (the
/// various node types can be found in hir_def). The interner is also used to collect
/// extra information about the Hir, such as the type of each node, information about
/// each definition or struct, etc. Because it is used on the Hir, the NodeInterner is
/// useful in passes where the Hir is used - name resolution, type checking, and
/// monomorphization - and it is not useful afterward.
pub struct NodeInterner {
    nodes: Arena<Node>,
    func_meta: HashMap<FuncId, FuncMeta>,
    function_definition_ids: HashMap<FuncId, DefinitionId>,

    // For a given function ID, this gives the function's modifiers which includes
    // its visibility and whether it is unconstrained, among other information.
    // Unlike func_meta, this map is filled out during definition collection rather than name resolution.
    function_modifiers: HashMap<FuncId, FunctionModifiers>,

    // Contains the source module each function was defined in
    function_modules: HashMap<FuncId, ModuleId>,

    // Map each `Index` to it's own location
    id_to_location: HashMap<Index, Location>,

    // Maps each DefinitionId to a DefinitionInfo.
    definitions: Vec<DefinitionInfo>,

    // Type checking map
    //
    // Notice that we use `Index` as the Key and not an ExprId or IdentId
    // Therefore, If a raw index is passed in, then it is not safe to assume that it will have
    // a Type, as not all Ids have types associated to them.
    // Further note, that an ExprId and an IdentId will never have the same underlying Index
    // Because we use one Arena to store all Definitions/Nodes
    id_to_type: HashMap<Index, Type>,

    // Struct map.
    //
    // Each struct definition is possibly shared across multiple type nodes.
    // It is also mutated through the RefCell during name resolution to append
    // methods from impls to the type.
    structs: HashMap<StructId, Shared<StructType>>,

    struct_attributes: HashMap<StructId, StructAttributes>,
    // Type Aliases map.
    //
    // Map type aliases to the actual type.
    // When resolving types, check against this map to see if a type alias is defined.
    type_aliases: Vec<TypeAliasType>,

    // Trait map.
    //
    // Each trait definition is possibly shared across multiple type nodes.
    // It is also mutated through the RefCell during name resolution to append
    // methods from impls to the type.
    //
    // TODO: We may be able to remove the Shared wrapper once traits are no longer types.
    // We'd just lookup their methods as needed through the NodeInterner.
    traits: HashMap<TraitId, Shared<Trait>>,

    // Trait implementation map
    // For each type that implements a given Trait ( corresponding TraitId), there should be an entry here
    // The purpose for this hashmap is to detect duplication of trait implementations ( if any )
    trait_implementations: HashMap<TraitImplKey, Shared<TraitImpl>>,

    /// Map from ExprId (referring to a Function/Method call) to its corresponding TypeBindings,
    /// filled out during type checking from instantiated variables. Used during monomorphization
    /// to map call site types back onto function parameter types, and undo this binding as needed.
    instantiation_bindings: HashMap<ExprId, TypeBindings>,

    /// Remembers the field index a given HirMemberAccess expression was resolved to during type
    /// checking.
    field_indices: HashMap<ExprId, usize>,

    globals: HashMap<StmtId, GlobalInfo>, // NOTE: currently only used for checking repeat globals and restricting their scope to a module

    next_type_variable_id: usize,

    /// A map from a struct type and method name to a function id for the method.
    struct_methods: HashMap<(StructId, String), FuncId>,

    /// Methods on primitive types defined in the stdlib.
    primitive_methods: HashMap<(TypeMethodKey, String), FuncId>,
}

/// All the information from a function that is filled out during definition collection rather than
/// name resolution. Resultingly, if information about a function is needed during name resolution,
/// this is the only place where it is safe to retrieve it (where all fields are guaranteed to be initialized).
pub struct FunctionModifiers {
    pub name: String,

    /// Whether the function is `pub` or not.
    pub visibility: Visibility,

    pub attributes: Attributes,

    pub is_unconstrained: bool,

    /// This function's type in its contract.
    /// If this function is not in a contract, this is always 'Secret'.
    pub contract_function_type: Option<ContractFunctionType>,

    /// This function's contract visibility.
    /// If this function is internal can only be called by itself.
    /// Will be None if not in contract.
    pub is_internal: Option<bool>,
}

impl FunctionModifiers {
    /// A semi-reasonable set of default FunctionModifiers used for testing.
    #[cfg(test)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            name: String::new(),
            visibility: Visibility::Public,
            attributes: Attributes::empty(),
            is_unconstrained: false,
            is_internal: None,
            contract_function_type: None,
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

impl From<DefinitionId> for Index {
    fn from(id: DefinitionId) -> Self {
        Index::from_raw_parts(id.0, u64::MAX)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct StmtId(Index);

impl StmtId {
    //dummy id for error reporting
    // This can be anything, as the program will ultimately fail
    // after resolution
    pub fn dummy_id() -> StmtId {
        StmtId(Index::from_raw_parts(std::usize::MAX, 0))
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct ExprId(Index);

impl ExprId {
    pub fn empty_block_id() -> ExprId {
        ExprId(Index::from_raw_parts(0, 0))
    }
}
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct FuncId(Index);

impl FuncId {
    //dummy id for error reporting
    // This can be anything, as the program will ultimately fail
    // after resolution
    pub fn dummy_id() -> FuncId {
        FuncId(Index::from_raw_parts(std::usize::MAX, 0))
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

macro_rules! partialeq {
    ($id_type:ty) => {
        impl PartialEq<usize> for &$id_type {
            fn eq(&self, other: &usize) -> bool {
                let (index, _) = self.0.into_raw_parts();
                index == *other
            }
        }
    };
}

into_index!(ExprId);
into_index!(StmtId);

partialeq!(ExprId);
partialeq!(StmtId);

/// A Definition enum specifies anything that we can intern in the NodeInterner
/// We use one Arena for all types that can be interned as that has better cache locality
/// This data structure is never accessed directly, so API wise there is no difference between using
/// Multiple arenas and a single Arena
#[derive(Debug, Clone)]
enum Node {
    Function(HirFunction),
    Statement(HirStatement),
    Expression(HirExpression),
}

#[derive(Debug, Clone)]
pub struct DefinitionInfo {
    pub name: String,
    pub mutable: bool,
    pub kind: DefinitionKind,
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

    Global(ExprId),

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
            DefinitionKind::Global(id) => Some(*id),
            DefinitionKind::Local(id) => *id,
            DefinitionKind::GenericType(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalInfo {
    pub ident: Ident,
    pub local_id: LocalModuleId,

    /// Global definitions have an associated storage slot if they are defined within
    /// a contract. If they're defined elsewhere, this value is None.
    pub storage_slot: Option<StorageSlot>,
}

impl Default for NodeInterner {
    fn default() -> Self {
        let mut interner = NodeInterner {
            nodes: Arena::default(),
            func_meta: HashMap::new(),
            function_definition_ids: HashMap::new(),
            function_modifiers: HashMap::new(),
            function_modules: HashMap::new(),
            id_to_location: HashMap::new(),
            definitions: vec![],
            id_to_type: HashMap::new(),
            structs: HashMap::new(),
            struct_attributes: HashMap::new(),
            type_aliases: Vec::new(),
            traits: HashMap::new(),
            trait_implementations: HashMap::new(),
            instantiation_bindings: HashMap::new(),
            field_indices: HashMap::new(),
            next_type_variable_id: 0,
            globals: HashMap::new(),
            struct_methods: HashMap::new(),
            primitive_methods: HashMap::new(),
        };

        // An empty block expression is used often, we add this into the `node` on startup
        let expr_id = interner.push_expr(HirExpression::empty_block());
        assert_eq!(expr_id, ExprId::empty_block_id());
        interner
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
    pub fn push_expr_type(&mut self, expr_id: &ExprId, typ: Type) {
        self.id_to_type.insert(expr_id.into(), typ);
    }

    pub fn push_empty_trait(&mut self, type_id: TraitId, typ: &UnresolvedTrait) {
        let self_type_typevar_id = self.next_type_variable_id();
        let self_type_typevar = Shared::new(TypeBinding::Unbound(self_type_typevar_id));

        self.traits.insert(
            type_id,
            Shared::new(Trait::new(
                type_id,
                typ.trait_def.name.clone(),
                typ.trait_def.span,
                vecmap(&typ.trait_def.generics, |_| {
                    // Temporary type variable ids before the trait is resolved to its actual ids.
                    // This lets us record how many arguments the type expects so that other types
                    // can refer to it with generic arguments before the generic parameters themselves
                    // are resolved.
                    let id = TypeVariableId(0);
                    (id, Shared::new(TypeBinding::Unbound(id)))
                }),
                self_type_typevar_id,
                self_type_typevar,
            )),
        );
    }

    pub fn new_struct(
        &mut self,
        typ: &UnresolvedStruct,
        krate: CrateId,
        local_id: LocalModuleId,
    ) -> StructId {
        let struct_id = StructId(ModuleId { krate, local_id });
        let name = typ.struct_def.name.clone();

        // Fields will be filled in later
        let no_fields = Vec::new();
        let generics = vecmap(&typ.struct_def.generics, |_| {
            // Temporary type variable ids before the struct is resolved to its actual ids.
            // This lets us record how many arguments the type expects so that other types
            // can refer to it with generic arguments before the generic parameters themselves
            // are resolved.
            let id = TypeVariableId(0);
            (id, Shared::new(TypeBinding::Unbound(id)))
        });

        let new_struct = StructType::new(struct_id, name, typ.struct_def.span, no_fields, generics);
        self.structs.insert(struct_id, Shared::new(new_struct));
        self.struct_attributes.insert(struct_id, typ.struct_def.attributes.clone());
        struct_id
    }

    pub fn push_type_alias(&mut self, typ: &UnresolvedTypeAlias) -> TypeAliasId {
        let type_id = TypeAliasId(self.type_aliases.len());

        self.type_aliases.push(TypeAliasType::new(
            type_id,
            typ.type_alias_def.name.clone(),
            typ.type_alias_def.span,
            Type::Error,
            vecmap(&typ.type_alias_def.generics, |_| {
                let id = TypeVariableId(0);
                (id, Shared::new(TypeBinding::Unbound(id)))
            }),
        ));

        type_id
    }

    pub fn update_struct(&mut self, type_id: StructId, f: impl FnOnce(&mut StructType)) {
        let mut value = self.structs.get_mut(&type_id).unwrap().borrow_mut();
        f(&mut value);
    }

    pub fn update_trait(&mut self, trait_id: TraitId, f: impl FnOnce(&mut Trait)) {
        let mut value = self.traits.get_mut(&trait_id).unwrap().borrow_mut();
        f(&mut value);
    }

    pub fn set_type_alias(&mut self, type_id: TypeAliasId, typ: Type, generics: Generics) {
        let type_alias_type = &mut self.type_aliases[type_id.0];
        type_alias_type.set_type_and_generics(typ, generics);
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

    /// Store the type for an interned Identifier
    pub fn push_definition_type(&mut self, definition_id: DefinitionId, typ: Type) {
        self.id_to_type.insert(definition_id.into(), typ);
    }

    pub fn push_global(
        &mut self,
        stmt_id: StmtId,
        ident: Ident,
        local_id: LocalModuleId,
        storage_slot: Option<StorageSlot>,
    ) {
        self.globals.insert(stmt_id, GlobalInfo { ident, local_id, storage_slot });
    }

    /// Intern an empty global stmt. Used for collecting globals
    pub fn push_empty_global(&mut self) -> StmtId {
        self.push_stmt(HirStatement::Error)
    }

    pub fn update_global(&mut self, stmt_id: StmtId, hir_stmt: HirStatement) {
        let def =
            self.nodes.get_mut(stmt_id.0).expect("ice: all function ids should have definitions");

        let stmt = match def {
            Node::Statement(stmt) => stmt,
            _ => {
                panic!("ice: all global ids should correspond to a statement in the interner")
            }
        };
        *stmt = hir_stmt;
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
        definition: DefinitionKind,
    ) -> DefinitionId {
        let id = DefinitionId(self.definitions.len());
        if let DefinitionKind::Function(func_id) = definition {
            self.function_definition_ids.insert(func_id, id);
        }

        self.definitions.push(DefinitionInfo { name, mutable, kind: definition });
        id
    }

    /// Push a function with the default modifiers and moduleid for testing
    #[cfg(test)]
    pub fn push_test_function_definition(&mut self, name: String) -> FuncId {
        let id = self.push_fn(HirFunction::empty());
        let modifiers = FunctionModifiers::new();
        let module = ModuleId::dummy_id();
        self.push_function_definition(name, id, modifiers, module);
        id
    }

    pub fn push_function(
        &mut self,
        id: FuncId,
        function: &FunctionDefinition,
        module: ModuleId,
    ) -> DefinitionId {
        use ContractFunctionType::*;
        let name = function.name.0.contents.clone();

        // We're filling in contract_function_type and is_internal now, but these will be verified
        // later during name resolution.
        let modifiers = FunctionModifiers {
            name: function.name.0.contents.clone(),
            visibility: if function.is_public { Visibility::Public } else { Visibility::Private },
            attributes: function.attributes.clone(),
            is_unconstrained: function.is_unconstrained,
            contract_function_type: Some(if function.is_open { Open } else { Secret }),
            is_internal: Some(function.is_internal),
        };
        self.push_function_definition(name, id, modifiers, module)
    }

    pub fn push_function_definition(
        &mut self,
        name: String,
        func: FuncId,
        modifiers: FunctionModifiers,
        module: ModuleId,
    ) -> DefinitionId {
        self.function_modifiers.insert(func, modifiers);
        self.function_modules.insert(func, module);
        self.push_definition(name, false, DefinitionKind::Function(func))
    }

    /// Returns the visibility of the given function.
    ///
    /// The underlying function_visibilities map is populated during def collection,
    /// so this function can be called anytime afterward.
    pub fn function_visibility(&self, func: FuncId) -> Visibility {
        self.function_modifiers[&func].visibility
    }

    /// Returns the module this function was defined within
    pub fn function_module(&self, func: FuncId) -> ModuleId {
        self.function_modules[&func]
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
    pub fn function_meta(&self, func_id: &FuncId) -> FuncMeta {
        self.func_meta.get(func_id).cloned().expect("ice: all function ids should have metadata")
    }

    pub fn try_function_meta(&self, func_id: &FuncId) -> Option<FuncMeta> {
        self.func_meta.get(func_id).cloned()
    }

    pub fn function_ident(&self, func_id: &FuncId) -> crate::Ident {
        let name = self.function_name(func_id).to_owned();
        let span = self.function_meta(func_id).name.location.span;
        crate::Ident(Spanned::from(span, name))
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

    /// Returns the interned statement corresponding to `stmt_id`
    pub fn statement(&self, stmt_id: &StmtId) -> HirStatement {
        let def =
            self.nodes.get(stmt_id.0).expect("ice: all statement ids should have definitions");

        match def {
            Node::Statement(stmt) => stmt.clone(),
            _ => panic!("ice: all statement ids should correspond to a statement in the interner"),
        }
    }

    /// Returns the interned let statement corresponding to `stmt_id`
    pub fn let_statement(&self, stmt_id: &StmtId) -> HirLetStatement {
        let def =
            self.nodes.get(stmt_id.0).expect("ice: all statement ids should have definitions");

        match def {
            Node::Statement(hir_stmt) => {
                match hir_stmt {
                    HirStatement::Let(let_stmt) => let_stmt.clone(),
                    _ => panic!("ice: all let statement ids should correspond to a let statement in the interner"),
                }
            },
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

    pub fn expr_location(&self, expr_id: &ExprId) -> Location {
        self.id_location(expr_id)
    }

    pub fn get_struct(&self, id: StructId) -> Shared<StructType> {
        self.structs[&id].clone()
    }

    pub fn get_trait(&self, id: TraitId) -> Shared<Trait> {
        self.traits[&id].clone()
    }

    pub fn get_type_alias(&self, id: TypeAliasId) -> &TypeAliasType {
        &self.type_aliases[id.0]
    }

    pub fn get_global(&self, stmt_id: &StmtId) -> Option<GlobalInfo> {
        self.globals.get(stmt_id).cloned()
    }

    pub fn get_all_globals(&self) -> HashMap<StmtId, GlobalInfo> {
        self.globals.clone()
    }

    /// Returns the type of an item stored in the Interner or Error if it was not found.
    pub fn id_type(&self, index: impl Into<Index>) -> Type {
        self.id_to_type.get(&index.into()).cloned().unwrap_or(Type::Error)
    }

    /// Returns the span of an item stored in the Interner
    pub fn id_location(&self, index: impl Into<Index>) -> Location {
        self.id_to_location.get(&index.into()).copied().unwrap()
    }

    /// Replaces the HirExpression at the given ExprId with a new HirExpression
    pub fn replace_expr(&mut self, id: &ExprId, new: HirExpression) {
        let old = self.nodes.get_mut(id.into()).unwrap();
        *old = Node::Expression(new);
    }

    pub fn next_type_variable_id(&mut self) -> TypeVariableId {
        let id = self.next_type_variable_id;
        self.next_type_variable_id += 1;
        TypeVariableId(id)
    }

    pub fn next_type_variable(&mut self) -> Type {
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

    /// Add a method to a type.
    /// This will panic for non-struct types currently as we do not support methods
    /// for primitives. We could allow this in the future however.
    pub fn add_method(
        &mut self,
        self_type: &Type,
        method_name: String,
        method_id: FuncId,
    ) -> Option<FuncId> {
        match self_type {
            Type::Struct(struct_type, _generics) => {
                let key = (struct_type.borrow().id, method_name);
                self.struct_methods.insert(key, method_id)
            }
            Type::Error => None,

            other => {
                let key = get_type_method_key(self_type).unwrap_or_else(|| {
                    unreachable!("Cannot add a method to the unsupported type '{}'", other)
                });
                self.primitive_methods.insert((key, method_name), method_id)
            }
        }
    }

    pub fn get_trait_implementation(&self, key: &TraitImplKey) -> Option<Shared<TraitImpl>> {
        self.trait_implementations.get(key).cloned()
    }

    pub fn add_trait_implementation(&mut self, key: &TraitImplKey, trait_impl: Shared<TraitImpl>) {
        self.trait_implementations.insert(key.clone(), trait_impl.clone());

        for func_id in &trait_impl.borrow().methods {
            self.add_method(&key.typ, self.function_name(func_id).to_owned(), *func_id);
        }
    }

    /// Search by name for a method on the given struct
    pub fn lookup_method(&self, id: StructId, method_name: &str) -> Option<FuncId> {
        self.struct_methods.get(&(id, method_name.to_owned())).copied()
    }

    /// Looks up a given method name on the given primitive type.
    pub fn lookup_primitive_method(&self, typ: &Type, method_name: &str) -> Option<FuncId> {
        get_type_method_key(typ)
            .and_then(|key| self.primitive_methods.get(&(key, method_name.to_owned())).copied())
    }
}

/// These are the primitive type variants that we support adding methods to
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
enum TypeMethodKey {
    /// Fields and integers share methods for ease of use. These methods may still
    /// accept only fields or integers, it is just that their names may not clash.
    FieldOrInt,
    Array,
    Bool,
    String,
    Unit,
    Tuple,
    Function,
}

fn get_type_method_key(typ: &Type) -> Option<TypeMethodKey> {
    use TypeMethodKey::*;
    let typ = typ.follow_bindings();
    match &typ {
        Type::FieldElement => Some(FieldOrInt),
        Type::Array(_, _) => Some(Array),
        Type::Integer(_, _) => Some(FieldOrInt),
        Type::TypeVariable(_, TypeVariableKind::IntegerOrField) => Some(FieldOrInt),
        Type::Bool => Some(Bool),
        Type::String(_) => Some(String),
        Type::Unit => Some(Unit),
        Type::Tuple(_) => Some(Tuple),
        Type::Function(_, _, _) => Some(Function),
        Type::MutableReference(element) => get_type_method_key(element),

        // We do not support adding methods to these types
        Type::TypeVariable(_, _)
        | Type::NamedGeneric(_, _)
        | Type::Forall(_, _)
        | Type::Constant(_)
        | Type::Error
        | Type::NotConstant
        | Type::Struct(_, _)
        | Type::FmtString(_, _) => None,
    }
}
