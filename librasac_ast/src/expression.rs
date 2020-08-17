use crate::{BlockStatement, Ident, Type};
use librasac_lexer::token::Token;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Ident(String),
    If(Box<IfExpression>),
    Literal(Literal),
    Prefix(Box<PrefixExpression>),
    Infix(Box<InfixExpression>),
    Call(Box<CallExpression>),
}

impl Expression {
    /// Returns an infix if the expression is an InfixExpression
    /// Returns None otherwise
    pub fn infix(self) -> Option<InfixExpression> {
        match self {
            Expression::Infix(infix) => Some(*infix),
            _ => None,
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
pub enum BinaryOp {
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
    Assign,
}

impl From<Token> for BinaryOp {
    fn from(token: Token) -> BinaryOp {
        match token {
            Token::Plus => BinaryOp::Add,
            Token::Ampersand => BinaryOp::And,
            Token::Pipe => BinaryOp::Or,
            Token::Minus => BinaryOp::Subtract,
            Token::Star => BinaryOp::Multiply,
            Token::Slash => BinaryOp::Divide,
            Token::Equal => BinaryOp::Equal,
            Token::NotEqual => BinaryOp::NotEqual,
            Token::Less => BinaryOp::Less,
            Token::LessEqual => BinaryOp::LessEqual,
            Token::Greater => BinaryOp::Greater,
            Token::GreaterEqual => BinaryOp::GreaterEqual,
            Token::Assign => BinaryOp::Assign,
            _ => panic!(
                "The token:  \" {} \"does not seem to be a binary operation ",
                token
            ),
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
    Bool(bool),
    Integer(i128),
    Str(String),
    Func(FunctionLiteral),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IfExpression {
    pub condition: Expression,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrefixExpression {
    pub operator: UnaryOp,
    pub rhs: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InfixExpression {
    pub lhs: Expression,
    pub operator: BinaryOp,
    pub rhs: Expression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
// Function definition
// fn add(x, y) {x+y}
pub struct FunctionDefinition {
    pub name: Ident,
    pub func: FunctionLiteral,
}
#[derive(Debug, PartialEq, Eq, Clone)]
// Function definition literal
// let add = fn(x, y) {x+y}
// XXX: This will be implemented later because it requires the Evaluators object system to accommodate for FunctionLiterals ontop of Polynomials
pub struct FunctionLiteral {
    pub parameters: Vec<(Ident, Type)>,
    pub body: BlockStatement,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallExpression {
    pub func_name: Ident,
    pub arguments: Vec<Expression>,
}
