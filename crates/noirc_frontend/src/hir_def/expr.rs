use std::cell::RefCell;
use std::rc::Rc;

use acvm::FieldElement;
use noirc_errors::Span;

use crate::node_interner::{ExprId, FuncId, IdentId, StmtId, TypeId};
use crate::{BinaryOp, BinaryOpKind, Ident, StructType, Type, UnaryOp};

#[derive(Debug, Clone)]
pub enum HirExpression {
    Ident(IdentId),
    Literal(HirLiteral),
    Block(HirBlockExpression),
    Prefix(HirPrefixExpression),
    Infix(HirInfixExpression),
    Index(HirIndexExpression),
    Constructor(HirConstructorExpression),
    MemberAccess(HirMemberAccess),
    Call(HirCallExpression),
    MethodCall(HirMethodCallExpression),
    Cast(HirCastExpression),
    For(HirForExpression),
    If(HirIfExpression),
    Error,
}

impl HirExpression {
    /// Returns an empty block expression
    pub const fn empty_block() -> HirExpression {
        HirExpression::Block(HirBlockExpression(vec![]))
    }
}

#[derive(Debug, Clone)]
pub struct HirForExpression {
    pub identifier: IdentId,
    pub start_range: ExprId,
    pub end_range: ExprId,
    pub block: ExprId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HirBinaryOpKind {
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
    MemberAccess,
    Assign,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HirBinaryOp {
    pub span: Span,
    pub kind: HirBinaryOpKind,
}

impl From<BinaryOpKind> for HirBinaryOpKind {
    fn from(a: BinaryOpKind) -> HirBinaryOpKind {
        match a {
            BinaryOpKind::Add => HirBinaryOpKind::Add,
            BinaryOpKind::Subtract => HirBinaryOpKind::Subtract,
            BinaryOpKind::Multiply => HirBinaryOpKind::Multiply,
            BinaryOpKind::Divide => HirBinaryOpKind::Divide,
            BinaryOpKind::Equal => HirBinaryOpKind::Equal,
            BinaryOpKind::NotEqual => HirBinaryOpKind::NotEqual,
            BinaryOpKind::Less => HirBinaryOpKind::Less,
            BinaryOpKind::LessEqual => HirBinaryOpKind::LessEqual,
            BinaryOpKind::Greater => HirBinaryOpKind::Greater,
            BinaryOpKind::GreaterEqual => HirBinaryOpKind::GreaterEqual,
            BinaryOpKind::And => HirBinaryOpKind::And,
            BinaryOpKind::Or => HirBinaryOpKind::Or,
            BinaryOpKind::Xor => HirBinaryOpKind::Xor,
            BinaryOpKind::Assign => HirBinaryOpKind::Assign,
        }
    }
}
impl From<BinaryOp> for HirBinaryOp {
    fn from(a: BinaryOp) -> HirBinaryOp {
        let kind: HirBinaryOpKind = a.contents.into();

        HirBinaryOp {
            span: a.span(),
            kind,
        }
    }
}

impl HirBinaryOpKind {
    /// Comparator operators return a 0 or 1
    /// When seen in the middle of an infix operator,
    /// they transform the infix expression into a predicate expression
    pub fn is_comparator(&self) -> bool {
        matches!(
            self,
            HirBinaryOpKind::Equal
                | HirBinaryOpKind::NotEqual
                | HirBinaryOpKind::LessEqual
                | HirBinaryOpKind::Less
                | HirBinaryOpKind::Greater
                | HirBinaryOpKind::GreaterEqual
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum HirUnaryOp {
    Minus,
    Not,
}

impl From<UnaryOp> for HirUnaryOp {
    fn from(a: UnaryOp) -> HirUnaryOp {
        match a {
            UnaryOp::Minus => HirUnaryOp::Minus,
            UnaryOp::Not => HirUnaryOp::Not,
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
pub struct HirMemberAccess {
    pub lhs: ExprId,
    // This field is not an IdentId since the rhs of a field
    // access has no corresponding definition
    pub rhs: Ident,
}

#[derive(Debug, Clone)]
pub struct HirIfExpression {
    pub condition: ExprId,
    pub consequence: ExprId,
    pub alternative: Option<ExprId>,
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

/// These nodes are temporary, they're
/// lowered into HirCallExpression nodes
/// after type checking resolves the object
/// type and the method it calls.
#[derive(Debug, Clone)]
pub struct HirMethodCallExpression {
    pub method: Ident,
    pub object: ExprId,
    pub arguments: Vec<ExprId>,
}

impl HirMethodCallExpression {
    pub fn into_function_call(mut self, method_id: FuncId) -> HirExpression {
        let mut arguments = vec![self.object];
        arguments.append(&mut self.arguments);

        HirExpression::Call(HirCallExpression {
            func_id: method_id,
            arguments,
        })
    }

}

#[derive(Debug, Clone)]
pub struct HirConstructorExpression {
    pub type_id: TypeId,
    pub r#type: Rc<RefCell<StructType>>,

    // NOTE: It is tempting to make this a BTreeSet to force ordering of field
    //       names (and thus remove the need to normalize them during type checking)
    //       but doing so would force the order of evaluation of field
    //       arguments to be alphabetical rather than the ordering the user
    //       included in the source code.
    pub fields: Vec<(IdentId, ExprId)>,
}

#[derive(Debug, Clone)]
pub struct HirIndexExpression {
    pub collection_name: IdentId,
    pub index: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirBlockExpression(pub Vec<StmtId>);

impl HirBlockExpression {
    pub fn statements(&self) -> &[StmtId] {
        &self.0
    }
}
