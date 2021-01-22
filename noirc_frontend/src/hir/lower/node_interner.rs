use std::collections::HashMap;

use arena::{Arena, Index};
use noirc_errors::Span;

use crate::{Ident, Type};

use super::{HirExpression, function::{FuncMeta, HirFunction}, stmt::HirStatement};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct IdentId(Index);

#[derive(Debug, Clone, Copy)]
pub struct StmtId(Index);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct ExprId(Index);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct FuncId(Index);

macro_rules! into_index {
    ($id_type:ty) => {
        impl Into<Index> for $id_type {
            fn into(self) -> Index {
                self.0
            }
        }
        
    };
}

into_index!(ExprId);
into_index!(StmtId);
into_index!(IdentId);

/// A Definition enum specifies anything that we can intern in the NodeInterner
/// We use one Arena for all types that can be interned as that has better cache locality
/// This data structure is never accessed directly, so API wise there is no difference between using
/// Multiple arenas and a single Arena
/// XXX: Possibly rename this to Node and `NodeInterner` to `NodeInterner`
#[derive(Debug)]
enum Node {
    Function(HirFunction),
    Ident(Ident),
    Statement(HirStatement),
    Expression(HirExpression),
}

#[derive(Debug)]
pub struct NodeInterner{

    nodes : Arena<Node>,
    func_meta : HashMap<FuncId, FuncMeta>,

    // Maps for span
    // Each encountered variable has it's own span 
    // We therefore give each variable, it's own IdentId
    //
    // Maps IdentId to it's definition
    // For `let x = EXPR` x will point to itself
    id_to_defs : HashMap<IdentId, IdentId>,
    // Map each IdentId to it's own span
    id_to_span : HashMap<IdentId, Span>,
    // Map each IdentId to it's name
    // This is a string right now, but once Strings are interned
    // In the lexer, this will be a SymbolId
    id_to_name : HashMap<IdentId, String>,

    // Type checking map
    //
    // Notice that we use `Index` as the Key and not an ExprId or IdentId
    // Therefore, If a raw index is passed in, then it is not safe to assume that it will have 
    // a Type, as not all Ids have types associated to them.
    // Further note, that an ExprId and an IdentId will never have the same underlying Index
    // Because we use one Arena to store all Definitions/Nodes
    id_to_type : HashMap<Index, Type>,
}

impl Default for NodeInterner {
    fn default() -> Self {
        NodeInterner {
            nodes : Arena::default(),
            func_meta : HashMap::new(),
            id_to_defs : HashMap::new(),
            id_to_span : HashMap::new(),
            id_to_name : HashMap::new(),
            id_to_type : HashMap::new(),
        }
    }
}

impl NodeInterner {
    pub fn push_stmt(&mut self, stmt : HirStatement) -> StmtId {
        StmtId(self.nodes.insert(Node::Statement(stmt)))
    }

    pub fn push_expr(&mut self, expr : HirExpression) -> ExprId {
        ExprId(self.nodes.insert(Node::Expression(expr)))
    }

    pub fn push_fn(&mut self, func : HirFunction) -> FuncId {
        FuncId(self.nodes.insert(Node::Function(func)))
    }

    // Type Checker
    pub fn push_expr_type(&mut self, expr_id : ExprId, typ : Type) {
        self.id_to_type.insert(expr_id.into(), typ);
    }
    pub fn push_ident_type(&mut self, ident_id : IdentId, typ : Type) {
        self.id_to_type.insert(ident_id.into(), typ);
    }
  
    pub fn push_empty_fn(&mut self) -> FuncId {
        self.push_fn(HirFunction::empty())
    }
    pub fn update_fn(&mut self, func_id : FuncId, hir_func : HirFunction) {
        let def = self.nodes.get_mut(func_id.0).expect("ice: all function ids should have definitions");

        let func = match def {
            Node::Function(func) => func,
            _=> panic!("ice: all function ids should correspond to a function in the interner")
        };
        *func = hir_func;
    }

    pub fn push_fn_meta(&mut self, func_data : FuncMeta, func_id : FuncId) {
        self.func_meta.insert(func_id, func_data);
    }

    pub fn push_ident(&mut self, ident : Ident) -> IdentId {
        let span = ident.0.span();
        let name = ident.0.contents.clone();

        let id = IdentId(self.nodes.insert(Node::Ident(ident)));
        
        self.id_to_span.insert(id, span);
        
        // XXX: Once Strings are interned name will also be an Id
        self.id_to_name.insert(id, name);

        // Note: These three maps are not invariant under their length
        // consider the case that we only ever inserted functions
        // the last two maps would be empty, while the first would be non-empty.

        id
    }
    pub fn linked_id_to_def(&mut self, ident : IdentId, def : IdentId) {
        self.id_to_defs.insert(ident, def);
    }
    /// Find the IdentifierId which declared/defined this IdentifierId
    pub fn ident_def(&self, ident : &IdentId) -> Option<IdentId> {
        self.id_to_defs.get(ident).copied()
    }

    // Cloning Hir structures is cheap, so we return owned structures 
    pub fn function(&self, func_id : FuncId) -> HirFunction {
        let def = self.nodes.get(func_id.0).expect("ice: all function ids should have definitions");

        match def {
            Node::Function(func) => return func.clone(),
            _=> panic!("ice: all function ids should correspond to a function in the interner")
        }
    }
    pub fn function_meta(&self, func_id : FuncId) -> FuncMeta {
        self.func_meta.get(&func_id).cloned().expect("ice: all function ids should have metadata")
    }
    pub fn statement(&self, stmt_id : StmtId) -> HirStatement {
        let def = self.nodes.get(stmt_id.0).expect("ice: all statement ids should have definitions");

        match def {
            Node::Statement(stmt) => return stmt.clone(),
            _=> panic!("ice: all statement ids should correspond to a statement in the interner")
        }
    }

    pub fn expression(&self, expr_id : ExprId) -> HirExpression {
        let def = self.nodes.get(expr_id.0).expect("ice: all expression ids should have definitions");

        match def {
            Node::Expression(expr) => return expr.clone(),
            _=> panic!("ice: all expression ids should correspond to a statement in the interner")
        }
    }
    pub fn ident(&self, ident_id : IdentId) -> Ident {
        let def = self.nodes.get(ident_id.0).expect("ice: all ident ids should have definitions");

        match def {
            Node::Ident(ident) => return ident.clone(),
            _=> panic!("ice: all expression ids should correspond to a statement in the interner")
        }
    }
    
    // XXX: This is needed as Witnesses in Evaluator require a name at the moment
    pub fn id_name(&self, ident_id : IdentId) -> String {
        self.id_to_name.get(&ident_id).expect("ice: all ident ids should have names").clone()
    }

    // Why can we unwrap here?
    // If the compiler is correct, it will not ask for a an Id of an object 
    // which does not have a type. This will cause a panic.
    // Since type checking always comes after resolution.
    // If resolution is correct, we will always assign types to Identifiers before we use them.
    // The same would go for Expressions.
    pub fn id_type(&self, index : Index) -> Type {
        self.id_to_type.get(&index).cloned().unwrap()
    }
}