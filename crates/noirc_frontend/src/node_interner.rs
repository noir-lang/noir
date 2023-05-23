use std::collections::HashMap;

use arena::{Arena, Index};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span, Spanned};

use crate::ast::Ident;
use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::UnresolvedStruct;
use crate::hir::def_map::{LocalModuleId, ModuleId};
use crate::hir::type_check::TypeCheckError;
use crate::hir::StorageSlot;
use crate::hir_def::stmt::HirLetStatement;
use crate::hir_def::types::{StructType, Type};
use crate::hir_def::{
    expr::HirExpression,
    function::{FuncMeta, HirFunction},
    stmt::HirStatement,
};
use crate::{Shared, TypeBinding, TypeBindings, TypeVariable, TypeVariableId};

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

    /// Map from ExprId (referring to a Function/Method call) to its corresponding TypeBindings,
    /// filled out during type checking from instantiated variables. Used during monomorphization
    /// to map call site types back onto function parameter types, and undo this binding as needed.
    instantiation_bindings: HashMap<ExprId, TypeBindings>,

    /// Remembers the field index a given HirMemberAccess expression was resolved to during type
    /// checking.
    field_indices: HashMap<ExprId, usize>,

    globals: HashMap<StmtId, GlobalInfo>, // NOTE: currently only used for checking repeat globals and restricting their scope to a module

    next_type_variable_id: usize,

    delayed_type_checks: Vec<TypeCheckFn>,

    /// A map from a struct type and method name to a function id for the method.
    struct_methods: HashMap<(StructId, String), FuncId>,

    /// Methods on primitive types defined in the stdlib.
    primitive_methods: HashMap<(TypeMethodKey, String), FuncId>,
}

type TypeCheckFn = Box<dyn FnOnce() -> Result<(), TypeCheckError>>;

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

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct StructId(pub ModuleId);

impl StructId {
    //dummy id for error reporting
    // This can be anything, as the program will ultimately fail
    // after resolution
    pub fn dummy_id() -> StructId {
        StructId(ModuleId { krate: CrateId::dummy_id(), local_id: LocalModuleId::dummy_id() })
    }
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

    pub fn get_rhs(self) -> Option<ExprId> {
        match self {
            DefinitionKind::Function(_) => None,
            DefinitionKind::Global(id) => Some(id),
            DefinitionKind::Local(id) => id,
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
            id_to_location: HashMap::new(),
            definitions: vec![],
            id_to_type: HashMap::new(),
            structs: HashMap::new(),
            instantiation_bindings: HashMap::new(),
            field_indices: HashMap::new(),
            next_type_variable_id: 0,
            globals: HashMap::new(),
            delayed_type_checks: vec![],
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

    pub fn push_empty_struct(&mut self, type_id: StructId, typ: &UnresolvedStruct) {
        self.structs.insert(
            type_id,
            Shared::new(StructType::new(
                type_id,
                typ.struct_def.name.clone(),
                typ.struct_def.span,
                Vec::new(),
                vecmap(&typ.struct_def.generics, |_| {
                    // Temporary type variable ids before the struct is resolved to its actual ids.
                    // This lets us record how many arguments the type expects so that other types
                    // can refer to it with generic arguments before the generic parameters themselves
                    // are resolved.
                    let id = TypeVariableId(0);
                    (id, Shared::new(TypeBinding::Unbound(id)))
                }),
            )),
        );
    }

    pub fn update_struct(&mut self, type_id: StructId, f: impl FnOnce(&mut StructType)) {
        let mut value = self.structs.get_mut(&type_id).unwrap().borrow_mut();
        f(&mut value);
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

    pub fn push_function_definition(&mut self, name: String, func: FuncId) -> DefinitionId {
        self.push_definition(name, false, DefinitionKind::Function(func))
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
        let name_id = self.function_meta(func_id).name.id;
        self.definition_name(name_id)
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

    pub fn definition(&self, id: DefinitionId) -> &DefinitionInfo {
        &self.definitions[id.0]
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
        let binding = TypeBinding::Unbound(self.next_type_variable_id());
        Type::TypeVariable(Shared::new(binding))
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

    pub fn push_delayed_type_check(&mut self, f: TypeCheckFn) {
        self.delayed_type_checks.push(f);
    }

    pub fn take_delayed_type_check_functions(&mut self) -> Vec<TypeCheckFn> {
        std::mem::take(&mut self.delayed_type_checks)
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
    Vec,
}

fn get_type_method_key(typ: &Type) -> Option<TypeMethodKey> {
    use TypeMethodKey::*;
    let typ = typ.follow_bindings();
    match &typ {
        Type::FieldElement(_) => Some(FieldOrInt),
        Type::Array(_, _) => Some(Array),
        Type::Integer(_, _, _) => Some(FieldOrInt),
        Type::PolymorphicInteger(_, _) => Some(FieldOrInt),
        Type::Bool(_) => Some(Bool),
        Type::String(_) => Some(String),
        Type::Unit => Some(Unit),
        Type::Tuple(_) => Some(Tuple),
        Type::Function(_, _) => Some(Function),
        Type::Vec(_) => Some(Vec),

        // We do not support adding methods to these types
        Type::TypeVariable(_)
        | Type::NamedGeneric(_, _)
        | Type::Forall(_, _)
        | Type::Constant(_)
        | Type::Error
        | Type::Struct(_, _) => None,
    }
}
