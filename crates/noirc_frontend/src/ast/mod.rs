/// This module contains two Ident structures, due to the fact that an identifier may or may not return a value
/// statement::Ident does not return a value, while Expression::Ident does.
mod expression;
mod function;
mod statement;
mod structure;

pub use expression::*;
pub use function::*;

use noirc_errors::Span;
pub use statement::*;
pub use structure::*;

use crate::{parser::ParserError, token::IntType, BinaryTypeOperator, Comptime};
use iter_extended::vecmap;

/// The parser parses types as 'UnresolvedType's which
/// require name resolution to resolve any typenames used
/// for structs within, but are otherwise identical to Types.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnresolvedType {
    FieldElement(Comptime),
    Array(Option<UnresolvedTypeExpression>, Box<UnresolvedType>), // [4]Witness = Array(4, Witness)
    Integer(Comptime, Signedness, u32),                           // u32 = Integer(unsigned, 32)
    Bool(Comptime),
    Expression(UnresolvedTypeExpression),
    Unit,

    /// A Named UnresolvedType can be a struct type or a type variable
    Named(Path, Vec<UnresolvedType>),

    // Note: Tuples have no visibility, instead each of their elements may have one.
    Tuple(Vec<UnresolvedType>),

    Unspecified, // This is for when the user declares a variable without specifying it's type
    Error,
}

/// The precursor to TypeExpression, this is the type that the parser allows
/// to be used in the length position of an array type. Only constants, variables,
/// and numeric binary operators are allowed here.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnresolvedTypeExpression {
    Variable(Path),
    Constant(u64),
    BinaryOperation(
        Box<UnresolvedTypeExpression>,
        BinaryTypeOperator,
        Box<UnresolvedTypeExpression>,
    ),
}

impl Recoverable for UnresolvedType {
    fn error(_: Span) -> Self {
        UnresolvedType::Error
    }
}

impl std::fmt::Display for UnresolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnresolvedType::*;
        match self {
            FieldElement(is_const) => write!(f, "{}Field", is_const),
            Array(len, typ) => match len {
                None => write!(f, "[{}]", typ),
                Some(expr) => write!(f, "[{}; {}]", typ, expr),
            },
            Integer(is_const, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{}i{}", is_const, num_bits),
                Signedness::Unsigned => write!(f, "{}u{}", is_const, num_bits),
            },
            Named(s, args) => {
                let args = vecmap(args, ToString::to_string);
                if args.is_empty() {
                    write!(f, "{}", s)
                } else {
                    write!(f, "{}<{}>", s, args.join(", "))
                }
            }
            Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Expression(expression) => expression.fmt(f),
            Bool(is_const) => write!(f, "{}bool", is_const),
            Unit => write!(f, "()"),
            Error => write!(f, "error"),
            Unspecified => write!(f, "unspecified"),
        }
    }
}

impl std::fmt::Display for UnresolvedTypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedTypeExpression::Variable(name) => name.fmt(f),
            UnresolvedTypeExpression::Constant(x) => x.fmt(f),
            UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs) => {
                write!(f, "({} {} {})", lhs, op, rhs)
            }
        }
    }
}

impl UnresolvedType {
    pub fn from_int_token(token: (Comptime, IntType)) -> UnresolvedType {
        use {IntType::*, UnresolvedType::Integer};
        match token.1 {
            Signed(num_bits) => Integer(token.0, Signedness::Signed, num_bits),
            Unsigned(num_bits) => Integer(token.0, Signedness::Unsigned, num_bits),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Signedness {
    Unsigned,
    Signed,
}

impl UnresolvedTypeExpression {
    pub fn from_expr(
        expr: Expression,
        span: Span,
    ) -> Result<UnresolvedTypeExpression, ParserError> {
        Self::from_expr_helper(expr).map_err(|err| {
            ParserError::with_reason(
                format!("Expression is invalid in an array-length type: '{}'. Only unsigned integer constants, globals, generics, +, -, *, /, and % may be used in this context.", err),
                span,
            )
        })
    }

    fn from_expr_helper(expr: Expression) -> Result<UnresolvedTypeExpression, Expression> {
        match expr.kind {
            ExpressionKind::Literal(Literal::Integer(int)) => match int.try_to_u64() {
                Some(int) => Ok(UnresolvedTypeExpression::Constant(int)),
                None => Err(expr),
            },
            ExpressionKind::Path(path) => Ok(UnresolvedTypeExpression::Variable(path)),
            ExpressionKind::Ident(name) => {
                let path = Path::from_single(name, expr.span);
                Ok(UnresolvedTypeExpression::Variable(path))
            }
            ExpressionKind::Prefix(prefix) if prefix.operator == UnaryOp::Minus => {
                let lhs = Box::new(UnresolvedTypeExpression::Constant(0));
                let rhs = Box::new(UnresolvedTypeExpression::from_expr_helper(prefix.rhs)?);
                let op = BinaryTypeOperator::Subtraction;
                Ok(UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs))
            }
            ExpressionKind::Infix(infix) if Self::operator_allowed(infix.operator.contents) => {
                let lhs = Box::new(UnresolvedTypeExpression::from_expr_helper(infix.lhs)?);
                let rhs = Box::new(UnresolvedTypeExpression::from_expr_helper(infix.rhs)?);
                let op = match infix.operator.contents {
                    BinaryOpKind::Add => BinaryTypeOperator::Addition,
                    BinaryOpKind::Subtract => BinaryTypeOperator::Subtraction,
                    BinaryOpKind::Multiply => BinaryTypeOperator::Multiplication,
                    BinaryOpKind::Divide => BinaryTypeOperator::Division,
                    BinaryOpKind::Modulo => BinaryTypeOperator::Modulo,
                    _ => unreachable!(), // impossible via operator_allowed check
                };
                Ok(UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs))
            }
            _ => Err(expr),
        }
    }

    fn operator_allowed(op: BinaryOpKind) -> bool {
        matches!(
            op,
            BinaryOpKind::Add
                | BinaryOpKind::Subtract
                | BinaryOpKind::Multiply
                | BinaryOpKind::Divide
                | BinaryOpKind::Modulo
        )
    }
}
