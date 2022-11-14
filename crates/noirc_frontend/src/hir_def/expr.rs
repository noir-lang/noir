use acvm::FieldElement;
use fm::FileId;
use noirc_errors::Location;

use crate::node_interner::{DefinitionId, ExprId, FuncId, NodeInterner, StmtId, StructId};
use crate::{BinaryOp, BinaryOpKind, Ident, Shared, UnaryOp};

use super::types::{StructType, Type};

#[derive(Debug, Clone)]
pub enum HirExpression {
    Ident(HirIdent),
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
    Tuple(Vec<ExprId>),
    Error,
}

impl HirExpression {
    /// Returns an empty block expression
    pub const fn empty_block() -> HirExpression {
        HirExpression::Block(HirBlockExpression(vec![]))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HirIdent {
    pub location: Location,
    pub id: DefinitionId,
}

#[derive(Debug, Clone)]
pub struct HirForExpression {
    pub identifier: HirIdent,
    pub start_range: ExprId,
    pub end_range: ExprId,
    pub block: ExprId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HirBinaryOp {
    pub kind: BinaryOpKind,
    pub location: Location,
}

impl HirBinaryOp {
    pub fn new(op: BinaryOp, file: FileId) -> Self {
        let kind = op.contents;
        let location = Location::new(op.span(), file);
        HirBinaryOp { location, kind }
    }
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Array(Vec<ExprId>),
    Bool(bool),
    Integer(FieldElement),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct HirPrefixExpression {
    pub operator: UnaryOp,
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
pub struct HirCallExpression {
    pub func: ExprId,
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
    pub fn into_function_call(
        mut self,
        func: FuncId,
        func_name: String,
        location: Location,
        interner: &mut NodeInterner,
    ) -> (ExprId, HirExpression) {
        let mut arguments = vec![self.object];
        arguments.append(&mut self.arguments);

        let id = interner.push_function_definition(func_name, func);
        let ident = HirExpression::Ident(HirIdent { location, id });
        let func = interner.push_expr(ident);

        (func, HirExpression::Call(HirCallExpression { func, arguments }))
    }
}

#[derive(Debug, Clone)]
pub struct HirConstructorExpression {
    pub type_id: StructId,
    pub r#type: Shared<StructType>,

    // NOTE: It is tempting to make this a BTreeSet to force ordering of field
    //       names (and thus remove the need to normalize them during type checking)
    //       but doing so would force the order of evaluation of field
    //       arguments to be alphabetical rather than the ordering the user
    //       included in the source code.
    pub fields: Vec<(Ident, ExprId)>,
}

#[derive(Debug, Clone)]
pub struct HirIndexExpression {
    pub collection: ExprId,
    pub index: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirBlockExpression(pub Vec<StmtId>);

impl HirBlockExpression {
    pub fn statements(&self) -> &[StmtId] {
        &self.0
    }
}
