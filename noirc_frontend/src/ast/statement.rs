use crate::{Expression, ExpressionKind, InfixExpression, Type, hir::crate_def_map::ModuleDefId};
use crate::lexer::token::SpannedToken;
use noirc_errors::{Span, Spanned};

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
    pub path: Path,
    pub alias: Option<Ident>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PathKind {
    Crate, 
    Dep, 
    Plain
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Path{
    pub segments : Vec<Ident>,
    pub kind : PathKind,
}

impl Path {
    pub fn span(&self)-> Span {
        let mut span = Span::default();
        for segment in self.segments.iter() {
            span.merge(segment.0.span());
        }
        span
    }

    pub fn last_segment(&self) -> Ident {
        assert!(!self.segments.is_empty());
        self.segments.last().unwrap().clone()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockStatement(pub Vec<Statement>);

impl BlockStatement {
    pub fn pop(&mut self) -> Option<Statement> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
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