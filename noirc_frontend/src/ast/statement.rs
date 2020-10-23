use crate::{Expression, InfixExpression, Type};

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Clone)]
pub struct Ident(pub String);

impl From<String> for Ident {
    fn from(a: String) -> Ident {
        Ident(a)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {
    If(Box<IfStatement>),
    Let(LetStatement),
    Const(ConstStatement),
    Constrain(ConstrainStatement),
    Public(PublicStatement),
    Private(PrivateStatement),
    Block(Box<BlockStatement>),
    Expression(ExpressionStatement),
}


impl Into<Statement> for Expression {
    fn into(self) -> Statement {
        let expr_stmt = ExpressionStatement(self);
        Statement::Expression(expr_stmt)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImportStatement {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockStatement(pub Vec<Statement>);

impl BlockStatement {
    pub fn pop(&mut self) -> Option<Statement> {
        self.0.pop()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExpressionStatement(pub Expression);

#[derive(Debug, PartialEq, Eq, Clone)]
// This will be used for structs and maybe closures(if we decide to have them)
pub struct LetStatement {
    pub identifier: Ident,
    pub r#type: Type,
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstStatement {
    pub identifier: Ident,
    pub r#type: Type, // This will always be a Literal FieldElement
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublicStatement {
    pub identifier: Ident,
    pub r#type: Type, // This will always be a Literal FieldElement
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrivateStatement {
    pub identifier: Ident,
    pub r#type: Type, 
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstrainStatement(pub InfixExpression);