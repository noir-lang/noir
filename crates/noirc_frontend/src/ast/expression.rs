use crate::token::{Attribute, Token};
use crate::{Ident, Path, Statement, Type};
use noir_field::FieldElement;
use noirc_errors::{Span, Spanned};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpressionKind<F> {
    Ident(String),
    Literal(Literal<F>),
    Block(BlockExpression<F>),
    Prefix(Box<PrefixExpression<F>>),
    Index(Box<IndexExpression<F>>),
    Call(Box<CallExpression<F>>),
    Cast(Box<CastExpression<F>>),
    Infix(Box<InfixExpression<F>>),
    Predicate(Box<InfixExpression<F>>),
    For(Box<ForExpression<F>>),
    If(Box<IfExpression<F>>),
    Path(Path),
}

impl<F: FieldElement> ExpressionKind<F> {
    pub fn into_path(self) -> Option<Path> {
        match self {
            ExpressionKind::Path(path) => Some(path),
            _ => None,
        }
    }

    pub fn into_infix(self) -> Option<InfixExpression<F>> {
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

    fn as_integer(&self) -> Option<F> {
        let literal = match self {
            ExpressionKind::Literal(literal) => literal,
            _ => return None,
        };

        match literal {
            Literal::Integer(integer) => Some(*integer),
            _ => None,
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
            _ => None,
        }
    }
}

#[derive(Debug, Eq, Clone)]
pub struct Expression<F> {
    pub kind: ExpressionKind<F>,
    pub span: Span,
}

// This is important for tests. Two expressions are the same, iff their Kind is the same
// We are ignoring Span
impl<F: PartialEq> PartialEq<Expression<F>> for Expression<F> {
    fn eq(&self, rhs: &Expression<F>) -> bool {
        self.kind == rhs.kind
    }
}

impl<F: FieldElement> Expression<F> {
    pub fn into_ident(self) -> Option<Ident> {
        let identifier = match self.kind {
            ExpressionKind::Ident(x) => x,
            _ => return None,
        };

        let ident = Ident(Spanned::from(self.span, identifier));
        Some(ident)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ForExpression<F> {
    pub identifier: Ident,
    pub start_range: Expression<F>,
    pub end_range: Expression<F>,
    pub block: BlockExpression<F>,
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
        matches!(
            self,
            BinaryOpKind::Equal
                | BinaryOpKind::NotEqual
                | BinaryOpKind::LessEqual
                | BinaryOpKind::Less
                | BinaryOpKind::Greater
                | BinaryOpKind::GreaterEqual
        )
    }

    pub fn as_string(&self) -> &str {
        match self {
            BinaryOpKind::Add => "+",
            BinaryOpKind::Subtract => "-",
            BinaryOpKind::Multiply => "*",
            BinaryOpKind::Divide => "/",
            BinaryOpKind::Equal => "==",
            BinaryOpKind::NotEqual => "!=",
            BinaryOpKind::Less => "<",
            BinaryOpKind::LessEqual => "<=",
            BinaryOpKind::Greater => ">",
            BinaryOpKind::GreaterEqual => ">=",
            BinaryOpKind::And => "&",
            BinaryOpKind::Or => "|",
            BinaryOpKind::Xor => "^",
            BinaryOpKind::Assign => "=",
        }
    }
}

impl<F: FieldElement> From<&Token<F>> for Option<BinaryOpKind> {
    fn from(token: &Token<F>) -> Option<BinaryOpKind> {
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
            _ => return None,
        };
        Some(op)
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
    pub fn from<F: FieldElement>(token: &Token<F>) -> Option<UnaryOp> {
        match token {
            Token::Minus => Some(UnaryOp::Minus),
            Token::Bang => Some(UnaryOp::Not),
            _ => None,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal<F> {
    Array(ArrayLiteral<F>),
    Bool(bool),
    Integer(F),
    Str(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrefixExpression<F> {
    pub operator: UnaryOp,
    pub rhs: Expression<F>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InfixExpression<F> {
    pub lhs: Expression<F>,
    pub operator: BinaryOp,
    pub rhs: Expression<F>,
}

// This is an infix expression with 'as' as the binary operator
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CastExpression<F> {
    pub lhs: Expression<F>,
    pub r#type: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IfExpression<F> {
    pub condition: Expression<F>,
    pub consequence: BlockExpression<F>,
    pub alternative: Option<BlockExpression<F>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDefinition<F> {
    pub name: Ident,
    pub attribute: Option<Attribute>, // XXX: Currently we only have one attribute defined. If more attributes are needed per function, we can make this a vector and make attribute definition more expressive
    pub parameters: Vec<(Ident, Type)>,
    pub body: BlockExpression<F>,
    pub span: Span,
    pub return_type: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayLiteral<F> {
    pub length: u128, // XXX: Maybe allow field element, so that the user can define the length using a constant
    pub r#type: Type,
    pub contents: Vec<Expression<F>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallExpression<F> {
    pub func_name: Path,
    pub arguments: Vec<Expression<F>>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexExpression<F> {
    pub collection_name: Ident, // XXX: For now, this will be the name of the array, as we do not support other collections
    pub index: Expression<F>, // XXX: We accept two types of indices, either a normal integer or a constant
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockExpression<F>(pub Vec<Statement<F>>);

impl<F: FieldElement> BlockExpression<F> {
    pub fn pop(&mut self) -> Option<Statement<F>> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
