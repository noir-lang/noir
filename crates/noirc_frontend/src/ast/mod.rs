//! The submodules of this module define the various data types required to
//! represent Noir's Ast. Of particular importance are ExpressionKind and Statement
//! which can be found in expression.rs and statement.rs respectively.
//!
//! Noir's Ast is produced by the parser and taken as input to name resolution,
//! where it is converted into the Hir (defined in the hir_def module).
mod expression;
mod function;
mod statement;
mod structure;
mod traits;

pub use expression::*;
pub use function::*;

use noirc_errors::Span;
pub use statement::*;
pub use structure::*;
pub use traits::*;

use crate::{
    parser::{ParserError, ParserErrorReason},
    token::IntType,
    BinaryTypeOperator, CompTime,
};
use iter_extended::vecmap;

/// The parser parses types as 'UnresolvedType's which
/// require name resolution to resolve any type names used
/// for structs within, but are otherwise identical to Types.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnresolvedType {
    FieldElement(CompTime),
    Array(Option<UnresolvedTypeExpression>, Box<UnresolvedType>), // [4]Witness = Array(4, Witness)
    Integer(CompTime, Signedness, u32),                           // u32 = Integer(unsigned, 32)
    Bool(CompTime),
    Expression(UnresolvedTypeExpression),
    String(Option<UnresolvedTypeExpression>),
    FormatString(UnresolvedTypeExpression, Box<UnresolvedType>),
    Unit,

    /// A Named UnresolvedType can be a struct type or a type variable
    Named(Path, Vec<UnresolvedType>),

    /// &mut T
    MutableReference(Box<UnresolvedType>),

    // Note: Tuples have no visibility, instead each of their elements may have one.
    Tuple(Vec<UnresolvedType>),

    Function(/*args:*/ Vec<UnresolvedType>, /*ret:*/ Box<UnresolvedType>),

    Unspecified, // This is for when the user declares a variable without specifying it's type
    Error,
}

/// The precursor to TypeExpression, this is the type that the parser allows
/// to be used in the length position of an array type. Only constants, variables,
/// and numeric binary operators are allowed here.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnresolvedTypeExpression {
    Variable(Path),
    Constant(u64, Span),
    BinaryOperation(
        Box<UnresolvedTypeExpression>,
        BinaryTypeOperator,
        Box<UnresolvedTypeExpression>,
        Span,
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
            FieldElement(is_const) => write!(f, "{is_const}Field"),
            Array(len, typ) => match len {
                None => write!(f, "[{typ}]"),
                Some(len) => write!(f, "[{typ}; {len}]"),
            },
            Integer(is_const, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{is_const}i{num_bits}"),
                Signedness::Unsigned => write!(f, "{is_const}u{num_bits}"),
            },
            Named(s, args) => {
                let args = vecmap(args, ToString::to_string);
                if args.is_empty() {
                    write!(f, "{s}")
                } else {
                    write!(f, "{}<{}>", s, args.join(", "))
                }
            }
            Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Expression(expression) => expression.fmt(f),
            Bool(is_const) => write!(f, "{is_const}bool"),
            String(len) => match len {
                None => write!(f, "str<_>"),
                Some(len) => write!(f, "str<{len}>"),
            },
            FormatString(len, elements) => write!(f, "fmt<{len}, {elements}"),
            Function(args, ret) => {
                let args = vecmap(args, ToString::to_string);
                write!(f, "fn({}) -> {ret}", args.join(", "))
            }
            MutableReference(element) => write!(f, "&mut {element}"),
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
            UnresolvedTypeExpression::Constant(x, _) => x.fmt(f),
            UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, _) => {
                write!(f, "({lhs} {op} {rhs})")
            }
        }
    }
}

impl UnresolvedType {
    pub fn from_int_token(token: (CompTime, IntType)) -> UnresolvedType {
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
    // This large error size is justified because it improves parsing speeds by around 40% in
    // release mode. See `ParserError` definition for further explanation.
    #[allow(clippy::result_large_err)]
    pub fn from_expr(
        expr: Expression,
        span: Span,
    ) -> Result<UnresolvedTypeExpression, ParserError> {
        Self::from_expr_helper(expr).map_err(|err_expr| {
            ParserError::with_reason(
                ParserErrorReason::InvalidArrayLengthExpression(err_expr),
                span,
            )
        })
    }

    pub fn span(&self) -> Span {
        match self {
            UnresolvedTypeExpression::Variable(path) => path.span(),
            UnresolvedTypeExpression::Constant(_, span) => *span,
            UnresolvedTypeExpression::BinaryOperation(_, _, _, span) => *span,
        }
    }

    fn from_expr_helper(expr: Expression) -> Result<UnresolvedTypeExpression, Expression> {
        match expr.kind {
            ExpressionKind::Literal(Literal::Integer(int)) => match int.try_to_u64() {
                Some(int) => Ok(UnresolvedTypeExpression::Constant(int, expr.span)),
                None => Err(expr),
            },
            ExpressionKind::Variable(path) => Ok(UnresolvedTypeExpression::Variable(path)),
            ExpressionKind::Prefix(prefix) if prefix.operator == UnaryOp::Minus => {
                let lhs = Box::new(UnresolvedTypeExpression::Constant(0, expr.span));
                let rhs = Box::new(UnresolvedTypeExpression::from_expr_helper(prefix.rhs)?);
                let op = BinaryTypeOperator::Subtraction;
                Ok(UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, expr.span))
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
                Ok(UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, expr.span))
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
