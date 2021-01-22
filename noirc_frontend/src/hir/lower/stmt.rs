use crate::Type;

use super::{ExprId, IdentId, HirInfixExpression, node_interner::StmtId};

#[derive(Debug, Clone)]
pub struct HirLetStatement {
    pub identifier: IdentId,
    pub r#type: Type,
    pub expression: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirConstStatement {
    pub identifier: IdentId,
    pub r#type: Type,
    pub expression: ExprId,
}

#[derive(Debug, Clone)]
#[deprecated = "we will no longer support declaration of public variables"]
pub struct HirPublicStatement {
    pub identifier: IdentId,
    pub r#type: Type, 
    pub expression: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirPrivateStatement {
    pub identifier: IdentId,
    pub r#type: Type, 
    pub expression: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirConstrainStatement(pub HirInfixExpression);
#[derive(Debug, Clone)]
pub struct BinaryStatement {
    pub lhs: ExprId,
    pub r#type: Type, 
    pub expression: ExprId,
}
#[derive(Debug,Clone)]
pub enum HirStatement {
    Let(HirLetStatement),
    Const(HirConstStatement),
    Constrain(HirConstrainStatement),
    Public(HirPublicStatement),
    Private(HirPrivateStatement),
    Block(HirBlockStatement),
    Expression(ExprId),
}

#[derive(Debug, Clone)]
pub struct HirBlockStatement(pub Vec<StmtId>);

impl HirBlockStatement {
    pub fn statements(&self) -> Vec<StmtId> {
        self.0.clone()
    }
}
