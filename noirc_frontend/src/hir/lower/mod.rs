use node_interner::StmtId;
use noir_field::FieldElement;

use crate::{BinaryOpKind, Type, UnaryOp};
pub mod stmt;
pub mod node_interner;
pub mod function;
pub mod resolver;
mod errors;

use self::node_interner::{ExprId, FuncId, IdentId};

/// A CrateItem is just a root level module
/// Each crate has exactly one.
/// We pass this to the Evaluator and it uses this to evaluate the crate.
/// We cannot give this to the type checker however, because the Type checker needs to check the types
/// of all functions, even unused ones. The type checker instead iterates all interned functions after they are resolved. 
/// This is used in the Evaluator and is populated in the resolution phase
#[derive(Debug, Clone)]
pub struct CrateItem {
    /// If a crate is a library this will be none
    /// If it is a binary, then this will be populated with the Id of the main function
    entry : Option<FuncId>,

    // This definition has no reference to it's children
    // because at this stage, the module should be resolved 
    // and functions point directly to other items

    // This definition has non reference to parent, 
    // because the root module has no parent

    // This method also has no reference to other functions because
    // once we go to the entry point, we will follow it to see what 
    // to execute next.
    //
    // At this stage, the other functions which were in the module 
    // have been resolved and are no longer needed.
}



#[derive(Debug, Clone)]
pub enum HirExpression {
    Ident(IdentId), 
    Literal(HirLiteral),
    Prefix(HirPrefixExpression),
    Infix(HirInfixExpression),
    Index(HirIndexExpression),
    Call(HirCallExpression),
    Cast(HirCastExpression),
    Predicate(HirInfixExpression),
    For(HirForExpression),
    If(IfExpression),
}

#[derive(Debug, Clone)]
pub struct HirForExpression{
    pub identifier: IdentId,
    pub start_range: ExprId,
    pub end_range: ExprId,
    pub block: StmtId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HirBinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Xor,
    Assign,
}

impl From<BinaryOpKind> for HirBinaryOp {
    fn from(a : BinaryOpKind) -> HirBinaryOp {
        match a {
            BinaryOpKind::Add => HirBinaryOp::Add,
            BinaryOpKind::Subtract => HirBinaryOp::Subtract,
            BinaryOpKind::Multiply => HirBinaryOp::Multiply,
            BinaryOpKind::Divide => HirBinaryOp::Divide,
            BinaryOpKind::Equal => HirBinaryOp::Equal,
            BinaryOpKind::NotEqual => HirBinaryOp::NotEqual,
            BinaryOpKind::Less => HirBinaryOp::Less,
            BinaryOpKind::LessEqual => HirBinaryOp::LessEqual,
            BinaryOpKind::Greater => HirBinaryOp::Greater,
            BinaryOpKind::GreaterEqual => HirBinaryOp::GreaterEqual,
            BinaryOpKind::And => HirBinaryOp::And,
            BinaryOpKind::Or => HirBinaryOp::Or,
            BinaryOpKind::Xor => HirBinaryOp::Xor,
            BinaryOpKind::Assign => HirBinaryOp::Assign,
        }
    }
}

impl HirBinaryOp {
    /// Comparator operators return a 0 or 1
    /// When seen in the middle of an infix operator,
    /// they transform the infix expression into a predicate expression
    pub fn is_comparator(&self) -> bool {
        match self {
            HirBinaryOp::Equal |
            HirBinaryOp::NotEqual |
            HirBinaryOp::LessEqual |
            HirBinaryOp::Less |
            HirBinaryOp::Greater |
            HirBinaryOp::GreaterEqual => true, 
            _=> false
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum HirUnaryOp {
    Minus,
    Not,
}

impl From<UnaryOp> for HirUnaryOp {
    fn from(a : UnaryOp) -> HirUnaryOp {
        match a {
            UnaryOp::Minus => HirUnaryOp::Minus,
            UnaryOp::Not => HirUnaryOp::Not
        }
    }
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Array(HirArrayLiteral),
    Bool(bool),
    Integer(FieldElement),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct HirPrefixExpression {
    pub operator: HirUnaryOp,
    pub rhs: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirInfixExpression {
    pub lhs: ExprId,
    pub operator: HirBinaryOp,
    pub rhs: ExprId,
}

#[derive(Debug, Clone)]
pub struct IfExpression {
    pub condition: StmtId,
    pub consequence: StmtId,
    pub alternative: Option<StmtId>,
}

#[derive(Debug, Clone)]
pub struct HirCastExpression {
    pub lhs: ExprId,
    pub r#type: Type,
}
#[derive(Debug, Clone)]
pub struct HirArrayLiteral {
    pub length: u128,
    pub r#type: Type,
    pub contents: Vec<ExprId>,
}

#[derive(Debug, Clone)]
pub struct HirCallExpression {
    pub func_id: FuncId,
    pub arguments: Vec<ExprId>,
}
#[derive(Debug, Clone)]
pub struct HirIndexExpression {
    pub collection_name: IdentId,
    pub index: ExprId,
}