use crate::{Expression,  InfixExpression, Type};

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
    While(Box<WhileStatement>),
    Let(Box<LetStatement>),
    Const(Box<ConstStatement>),
    Constrain(Box<ConstrainStatement>),
    Public(Box<PublicStatement>),
    Private(Box<PrivateStatement>),
    Block(Box<BlockStatement>),
    Expression(Box<ExpressionStatement>),
    Error,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IfStatement {
    condition: Expression,
    consequence: Statement,
    alternative: Option<Statement>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WhileStatement {
    condition: Expression,
    block: Statement,
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
    pub r#type: Type, // This will always be a Witness
    pub expression: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConstrainStatement(pub InfixExpression);
