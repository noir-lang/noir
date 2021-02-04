use crate::{Ident, Path, Statement, Type};
use crate::token::{Token, Attribute};
use noirc_errors::{Spanned, Span};
use noir_field::FieldElement;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpressionKind {
    Ident(String),
    Literal(Literal),
    Block(BlockExpression),
    Prefix(Box<PrefixExpression>),
    Index(Box<IndexExpression>),
    Call(Box<CallExpression>),
    Cast(Box<CastExpression>),
    Infix(Box<InfixExpression>),
    Predicate(Box<InfixExpression>),
    For(Box<ForExpression>),
    If(Box<IfExpression>),
    Path(Path),
}

impl ExpressionKind {
    pub fn into_path(self) -> Option<Path> {
        match self {
            ExpressionKind::Path(path) => Some(path),
            _=> None
        }
    }

    pub fn into_infix(self) -> Option<InfixExpression> {
        match self {
            ExpressionKind::Infix(infix) => Some(*infix),
            ExpressionKind::Predicate(infix) => Some(*infix),
            _ => None,
        }
    }

    /// Returns true if the expression is a literal integer
    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }

    fn as_integer(&self) -> Option<FieldElement> {
        let literal = match self {
            ExpressionKind::Literal(literal) => literal,
            _ => return None,
        };

        match literal {
            Literal::Integer(integer) => Some(*integer),
            _=> None
        }
    }

    /// Returns true if the expression can be used in a range expression
    /// Currently we only support Identifiers and constants literals
    pub fn can_be_used_range(&self) -> bool {
        self.is_identifier() || self.is_integer()
    }

    /// Returns true if the expression is an identifier
    fn is_identifier(&self) -> bool {
        self.as_identifier().is_some()
    }

    fn as_identifier(&self) -> Option<String> {
        match self {
            ExpressionKind::Ident(x) => Some(x.clone()),
            _=> None
        }
    }


}

#[derive(Debug, Eq, Clone)]
pub struct Expression {
    pub kind : ExpressionKind,
    pub span : Span,
}

// This is important for tests. Two expressions are the same, iff their Kind is the same
// We are ignoring Span
impl PartialEq<Expression> for Expression {
    fn eq(&self, rhs: &Expression) -> bool {
        self.kind == rhs.kind
    }
}

impl Expression {
    pub fn into_ident(self) -> Option<Ident> {
        let identifier = match self.kind {
            ExpressionKind::Ident(x) => x,
            _=>return None
        };

        let ident = Ident(Spanned::from(self.span, identifier));
        return Some(ident)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ForExpression{
    pub identifier: Ident,
    pub start_range: Expression,
    pub end_range: Expression,
    pub block: BlockExpression,
}

pub type BinaryOp = Spanned<BinaryOpKind>;

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
pub enum BinaryOpKind {
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
    // Assign is the only binary operator which cannot be used in a constrain statement
    Assign,
}

impl BinaryOpKind {
    /// Comparator operators return a 0 or 1
    /// When seen in the middle of an infix operator,
    /// they transform the infix expression into a predicate expression
    pub fn is_comparator(&self) -> bool {
        match self {
            BinaryOpKind::Equal |
            BinaryOpKind::NotEqual |
            BinaryOpKind::LessEqual |
            BinaryOpKind::Less |
            BinaryOpKind::Greater |
            BinaryOpKind::GreaterEqual => true, 
            _=> false
        }
    }
}

impl From<&Token> for Option<BinaryOpKind> {
    fn from(token: &Token) -> Option<BinaryOpKind> {
        let op = match token {
            Token::Plus => BinaryOpKind::Add,
            Token::Ampersand => BinaryOpKind::And,
            Token::Caret => BinaryOpKind::Xor,
            Token::Pipe => BinaryOpKind::Or,
            Token::Minus => BinaryOpKind::Subtract,
            Token::Star => BinaryOpKind::Multiply,
            Token::Slash => BinaryOpKind::Divide,
            Token::Equal => BinaryOpKind::Equal,
            Token::NotEqual => BinaryOpKind::NotEqual,
            Token::Less => BinaryOpKind::Less,
            Token::LessEqual => BinaryOpKind::LessEqual,
            Token::Greater => BinaryOpKind::Greater,
            Token::GreaterEqual => BinaryOpKind::GreaterEqual,
            Token::Assign => BinaryOpKind::Assign,
            _ => return None
        };
        return Some(op)
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Copy, Clone)]
pub enum UnaryOp {
    Minus,
    Not,
}

impl UnaryOp {
    /// Converts a token to a unary operator
    /// If you want the parser to recognise another Token as being a prefix operator, it is defined here
    pub fn from(token: &Token) -> Option<UnaryOp> {
        match token {
            Token::Minus => Some(UnaryOp::Minus),
            Token::Bang => Some(UnaryOp::Not),
            _ => None
        }
    }

}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
    Array(ArrayLiteral),
    Bool(bool),
    Integer(FieldElement),
    Str(String),
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
pub struct IfExpression {
    pub condition: Expression,
    pub consequence: BlockExpression,
    pub alternative: Option<BlockExpression>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDefinition {
    pub name: Ident,
    pub attribute : Option<Attribute>, // XXX: Currently we only have one attribute defined. If more attributes are needed per function, we can make this a vector and make attribute definition more expressive
    pub parameters: Vec<(Ident, Type)>,
    pub body: BlockExpression,
    pub span : Span,
    pub return_type : Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayLiteral {
    pub length: u128, // XXX: Maybe allow field element, so that the user can define the length using a constant
    pub r#type: Type,
    pub contents: Vec<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallExpression {
    pub func_name: Path,
    pub arguments: Vec<Expression>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexExpression {
    pub collection_name: Ident, // XXX: For now, this will be the name of the array, as we do not support other collections
    pub index: Expression, // XXX: We accept two types of indices, either a normal integer or a constant
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockExpression(pub Vec<Statement>);

impl BlockExpression {
    pub fn pop(&mut self) -> Option<Statement> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
