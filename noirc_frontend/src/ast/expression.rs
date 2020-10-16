use crate::{BlockStatement, Ident, Type};
use crate::token::{Keyword, Token, Attribute};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Ident(String), // an identifer can also produce a value. e.g. let x = y; y is an expression in this case
    If(Box<IfExpression>),
    Literal(Literal),
    Prefix(Box<PrefixExpression>),
    Infix(Box<InfixExpression>),
    Assign(Box<AssignExpression>),
    Index(Box<IndexExpression>),
    Call(NoirPath, Box<CallExpression>), // Make Path Optional and so we only have one call expression
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
    /// Converts an Expression to a u128
    /// The Expression must be a literal integer
    pub fn to_u128(&self) -> u128 {
        let literal = match self {
            Expression::Literal(literal) => literal,
            _ => panic!("Expression is not a Literal, cannot convert to an integer"),
        };

        let integer = match literal {
            Literal::Integer(integer) => integer,
            _ => panic!("Literal is not an integer"),
        };

        *integer as u128
    }

    pub fn identifier(self) -> Option<Ident> {
        match self {
            Expression::Ident(x) => Some(Ident(x)),
            _=> None
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

impl BinaryOp {
    /// Comparator operators return a 0 or 1
    /// When seen in the middle of an infix operator,
    /// they transform the infix expression into a predicate expression
    pub fn is_comparator(&self) -> bool {
        match self {
            BinaryOp::Equal |
            BinaryOp::NotEqual |
            BinaryOp::LessEqual |
            BinaryOp::Less |
            BinaryOp::Greater |
            BinaryOp::GreaterEqual => true, 
            _=> false
        }
    }
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
    Array(ArrayLiteral),
    Bool(bool),
    Integer(i128),
    Str(String),
    Type(Type), // XXX: Possibly replace this with a Type Expression
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
pub struct AssignExpression {
    pub identifier: Ident,
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
    pub attribute : Option<Attribute>, // XXX: Currently we only have one attribute defined. If more attributes are needed per function, we can make this a vector and make attribute definition more expressive
    pub parameters: Vec<(Ident, Type)>,
    pub body: BlockStatement,
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
    pub func_name: Ident,
    pub arguments: Vec<Expression>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexExpression {
    pub collection_name: Ident, // XXX: For now, this will be the name of the array, as we do not support other collections
    pub index: Expression, // XXX: We accept two types of indices, either a normal integer or a constant
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum NoirPath {
    Current,
    External(Vec<Ident>) // These are used for functions, and maybe constants in the future. Example: std::hash -> vec!["std", "hash"]
}

impl From<Vec<Ident>> for NoirPath {
    fn from(path: Vec<Ident>) -> NoirPath {
        if path.len() == 0 {
            NoirPath::Current
        } else {
            NoirPath::External(path)
        }
    }
}

impl Into<Vec<Ident>> for NoirPath {
    fn into(self) -> Vec<Ident> {
        match self {
            NoirPath::Current => Vec::new(),
            NoirPath::External(path) => path
        }
    }
}

impl NoirPath {
    pub fn to_string(&self) -> String {
        let mut string = String::new();

        match self {
            NoirPath::Current => return string,
            NoirPath::External(path) => {
                for ns in path.iter() {
                    string.push_str(&ns.0);
                    string.push_str("::");
                }
                // Remove last `::`
                string.remove(string.len() - 1);
                string.remove(string.len() - 1);
            }
        }

        string
    }
    pub fn len(&self) -> usize {
        match self {
            NoirPath::Current => 0,
            NoirPath::External(path) => path.len()
        }
    }
    pub fn split_first(&self) -> Option<(&Ident, NoirPath)> {

        let path = match self {
            NoirPath::Current => return None,
            NoirPath::External(path) => path
        };

        if let Some((first, rest) ) = path.split_first() {
            return Some((first, rest.to_vec().into()))
        } else {
            return None
        }
    }
}
