use std::collections::HashMap;

use arena::{Arena, Index};
use noirc_errors::Span;

use crate::{Ident, Type};

use crate::hir::lower::{
    function::{FuncMeta, HirFunction},
    stmt::HirStatement,
    HirExpression,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct IdentId(Index);

impl IdentId {
    //dummy id for error reporting
    pub fn dummy_id() -> IdentId {
        IdentId(Index::from_raw_parts(std::usize::MAX, 0))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StmtId(Index);

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

macro_rules! into_index {
    ($id_type:ty) => {
        impl Into<Index> for $id_type {
            fn into(self) -> Index {
                self.0
            }
        }
        impl Into<Index> for &$id_type {
            fn into(self) -> Index {
                self.0
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
into_index!(IdentId);

partialeq!(ExprId);
partialeq!(IdentId);
partialeq!(StmtId);

/// A Definition enum specifies anything that we can intern in the NodeInterner
/// We use one Arena for all types that can be interned as that has better cache locality
/// This data structure is never accessed directly, so API wise there is no difference between using
/// Multiple arenas and a single Arena
#[derive(Debug, Clone)]
enum Node {
    Function(HirFunction),
    Ident(Ident),
    Statement(HirStatement),
    Expression(HirExpression),
}

#[derive(Debug, Clone)]
pub struct NodeInterner {
    nodes: Arena<Node>,
    func_meta: HashMap<FuncId, FuncMeta>,

    // Maps for span
    // Each encountered variable has it's own span
    // We therefore give each variable, it's own IdentId
    //
    // Maps IdentId to it's definition
    // For `let x = EXPR` x will point to itself as a definition
    ident_to_defs: HashMap<IdentId, IdentId>,
    // Map each `Index` to it's own span
    id_to_span: HashMap<Index, Span>,
    // Map each IdentId to it's name
    // This is a string right now, but once Strings are interned
    // In the lexer, this will be a SymbolId
    ident_to_name: HashMap<IdentId, String>,

    // Type checking map
    //
    // Notice that we use `Index` as the Key and not an ExprId or IdentId
    // Therefore, If a raw index is passed in, then it is not safe to assume that it will have
    // a Type, as not all Ids have types associated to them.
    // Further note, that an ExprId and an IdentId will never have the same underlying Index
    // Because we use one Arena to store all Definitions/Nodes
    id_to_type: HashMap<Index, Type>,
}

impl Default for NodeInterner {
    fn default() -> Self {
        let mut interner = NodeInterner {
            nodes: Arena::default(),
            func_meta: HashMap::new(),
            ident_to_defs: HashMap::new(),
            id_to_span: HashMap::new(),
            ident_to_name: HashMap::new(),
            id_to_type: HashMap::new(),
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
    pub fn push_expr_span(&mut self, expr_id: ExprId, span: Span) {
        self.id_to_span.insert(expr_id.into(), span);
    }
    /// Interns a HIR Function.
    pub fn push_fn(&mut self, func: HirFunction) -> FuncId {
        FuncId(self.nodes.insert(Node::Function(func)))
    }

    /// Store the type for an interned expression
    pub fn push_expr_type(&mut self, expr_id: &ExprId, typ: Type) {
        self.id_to_type.insert(expr_id.into(), typ);
    }

    /// Modify the type of an expression.
    ///
    /// This is used specifically for SemiExpressions.
    /// We type check them as regular expressions which implicitly interns
    /// the type of the expression. This function is then used to change
    /// it's type to Unit for the type checker.
    pub fn make_expr_type_unit(&mut self, expr_id: &ExprId) {
        self.id_to_type.insert(expr_id.into(), Type::Unit);
    }
    /// Store the type for an interned Identifier
    pub fn push_ident_type(&mut self, ident_id: &IdentId, typ: Type) {
        self.id_to_type.insert(ident_id.into(), typ);
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
        let def = self
            .nodes
            .get_mut(func_id.0)
            .expect("ice: all function ids should have definitions");

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

    /// Interns an Identifier
    pub fn push_ident(&mut self, ident: Ident) -> IdentId {
        let span = ident.0.span();
        let name = ident.0.contents.clone();

        let id = IdentId(self.nodes.insert(Node::Ident(ident)));

        self.id_to_span.insert(id.into(), span);

        // XXX: Once Strings are interned name will also be an Id
        self.ident_to_name.insert(id, name);

        // Note: These three maps are not invariant under their length
        // consider the case that we only ever inserted functions
        // the last two maps would be empty, while the first would be non-empty.

        id
    }
    /// Links the Identifier to the Identifier which defined it.
    pub fn linked_ident_to_def(&mut self, ident: IdentId, def: IdentId) {
        self.ident_to_defs.insert(ident, def);
    }
    /// Finds the IdentifierId which declared/defined this IdentifierId
    ///
    /// Example:
    ///
    /// priv z = x + a;
    /// priv k = z + b;
    ///
    /// Notice z is used twice. Each `z` has a unique IdentId
    /// However, the first `z` is the one that defines it.
    /// This function would return the Identifier of the first `z`
    pub fn ident_def(&self, ident: &IdentId) -> Option<IdentId> {
        self.ident_to_defs.get(ident).copied()
    }

    /// Returns the interned HIR function corresponding to `func_id`
    //
    // Cloning Hir structures is cheap, so we return owned structures
    pub fn function(&self, func_id: &FuncId) -> HirFunction {
        let def = self
            .nodes
            .get(func_id.0)
            .expect("ice: all function ids should have definitions");

        match def {
            Node::Function(func) => return func.clone(),
            _ => panic!("ice: all function ids should correspond to a function in the interner"),
        }
    }
    /// Returns the interned meta data corresponding to `func_id`
    pub fn function_meta(&self, func_id: &FuncId) -> FuncMeta {
        self.func_meta
            .get(func_id)
            .cloned()
            .expect("ice: all function ids should have metadata")
    }

    /// Returns the interned statement corresponding to `stmt_id`
    pub fn statement(&self, stmt_id: &StmtId) -> HirStatement {
        let def = self
            .nodes
            .get(stmt_id.0)
            .expect("ice: all statement ids should have definitions");

        match def {
            Node::Statement(stmt) => return stmt.clone(),
            _ => panic!("ice: all statement ids should correspond to a statement in the interner"),
        }
    }
    /// Returns the interned expression corresponding to `expr_id`
    pub fn expression(&self, expr_id: &ExprId) -> HirExpression {
        let def = self
            .nodes
            .get(expr_id.0)
            .expect("ice: all expression ids should have definitions");

        match def {
            Node::Expression(expr) => return expr.clone(),
            _ => {
                panic!("ice: all expression ids should correspond to a expression in the interner")
            }
        }
    }
    /// Returns the interned identifier corresponding to `ident_id`
    pub fn ident(&self, ident_id: &IdentId) -> Ident {
        let def = self
            .nodes
            .get(ident_id.0)
            .expect("ice: all ident ids should have definitions");

        match def {
            Node::Ident(ident) => return ident.clone(),
            _ => panic!("ice: all expression ids should correspond to a statement in the interner"),
        }
    }

    /// Returns the Identifier as a String
    ///
    /// This is needed as the Environment needs to map variable names to witness indices
    pub fn ident_name(&self, ident_id: &IdentId) -> String {
        self.ident_to_name
            .get(ident_id)
            .expect("ice: all ident ids should have names. This indicates a bug in the Resolver.")
            .clone()
    }

    /// Returns the span of an identifier
    pub fn ident_span(&self, ident_id: &IdentId) -> Span {
        self.id_span(ident_id)
    }
    /// Returns the span of an expression
    pub fn expr_span(&self, expr_id: &ExprId) -> Span {
        self.id_span(expr_id)
    }

    /// Returns the type of an item stored in the Interner.
    //
    // Why can we unwrap here?
    // If the compiler is correct, it will not ask for a an Id of an object
    // which does not have a type. This will cause a panic.
    // Since type checking always comes after resolution.
    // If resolution is correct, we will always assign types to Identifiers before we use them.
    // The same would go for Expressions.
    pub fn id_type(&self, index: impl Into<Index>) -> Type {
        self.id_to_type.get(&index.into()).cloned().unwrap()
    }
    /// Returns the span of an item stored in the Interner
    pub fn id_span(&self, index: impl Into<Index>) -> Span {
        self.id_to_span.get(&index.into()).copied().unwrap()
    }
}
