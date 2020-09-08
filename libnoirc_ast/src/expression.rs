use crate::{BlockStatement, Ident, Type};
use libnoirc_lexer::token::{Keyword, Token};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Ident(String), // an identifer can also produce a value. e.g. let x = y; y is an expression in this case
    If(Box<IfExpression>),
    Literal(Literal),
    Prefix(Box<PrefixExpression>),
    Infix(Box<InfixExpression>),
    Call(Box<CallExpression>),
    Cast(Box<CastExpression>),

    Predicate(Box<InfixExpression>),
}

impl Expression {
    pub fn infix(self) -> Option<InfixExpression> {
        match self {
            Expression::Infix(infix) => Some(*infix),
            Expression::Predicate(infix) => Some(*infix),
            _ => None,
        }
    }
    pub fn r#type(self) -> Option<Type> {
        match self {
            Expression::Literal(literal) => match literal {
                Literal::Type(typ) => return Some(typ),
                _ => return None,
            },
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
    Xor,
    As,
    // This is the only binary operator which cannot be used in a constrain statement
    Assign,
}

impl From<Token> for BinaryOp {
    fn from(token: Token) -> BinaryOp {
        match token {
            Token::Plus => BinaryOp::Add,
            Token::Ampersand => BinaryOp::And,
            Token::Caret => BinaryOp::Xor,
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
            Token::Keyword(Keyword::As) => BinaryOp::As,
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
    Type(Type),
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

// This is an infix expression with 'as' as the binary operator
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CastExpression {
    pub lhs: Expression,
    pub r#type: Type,
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
