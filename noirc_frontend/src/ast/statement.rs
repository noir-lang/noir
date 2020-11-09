use crate::{ExpressionKind, InfixExpression, Type, Expression};
use crate::lexer::token::SpannedToken;
use noirc_errors::{Spanned};

#[derive(PartialOrd, Eq, Ord,  Debug, Clone)]
pub struct Ident(pub Spanned<String>);

impl PartialEq<Ident> for Ident {
    fn eq(&self,other : &Ident) -> bool {
        &self.0.contents == &other.0.contents
    }
}

impl std::hash::Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.contents.hash(state);
    }
}

impl From<Spanned<String>> for Ident {
    fn from(a: Spanned<String>) -> Ident {
        Ident(a)
    }
}

impl From<String> for Ident {
    fn from(a: String) -> Ident {
        Spanned::from_position(Default::default(), Default::default(), a).into()
    }
}

impl From<SpannedToken> for Ident {
    fn from(st : SpannedToken) -> Ident {
        let spanned_str = Spanned::from(st.into_span(), st.token().to_string());
        Ident(spanned_str)
    }
}

impl From<Ident> for Expression {
    fn from(i : Ident) -> Expression {
        Expression{
            span : i.0.span(),
            kind : ExpressionKind::Ident(i.0.contents)
        }
    }
}

impl From<Ident> for ExpressionKind {
    fn from(i : Ident) -> ExpressionKind {
        ExpressionKind::Ident(i.0.contents)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {
    Let(LetStatement),
    Const(ConstStatement),
    Constrain(ConstrainStatement),
    Public(PublicStatement),
    Private(PrivateStatement),
    Block(Box<BlockStatement>),
    Expression(Expression),
}


impl Into<Statement> for Expression {
    fn into(self) -> Statement {
        Statement::Expression(self)
    }
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