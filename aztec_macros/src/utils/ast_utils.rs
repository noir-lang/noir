use noirc_errors::{Span, Spanned};
use noirc_frontend::{
    token::SecondaryAttribute, BinaryOpKind, CallExpression, CastExpression, Expression,
    ExpressionKind, FunctionReturnType, Ident, IndexExpression, InfixExpression, Lambda,
    LetStatement, MemberAccessExpression, MethodCallExpression, Path, Pattern, PrefixExpression,
    Statement, StatementKind, UnaryOp, UnresolvedType, UnresolvedTypeData,
};

//
//             Helper macros for creating noir ast nodes
//
pub fn ident(name: &str) -> Ident {
    Ident::new(name.to_string(), Span::default())
}

pub fn ident_path(name: &str) -> Path {
    Path::from_ident(ident(name))
}

pub fn path(ident: Ident) -> Path {
    Path::from_ident(ident)
}

pub fn expression(kind: ExpressionKind) -> Expression {
    Expression::new(kind, Span::default())
}

pub fn variable(name: &str) -> Expression {
    expression(ExpressionKind::Variable(ident_path(name)))
}

pub fn variable_ident(identifier: Ident) -> Expression {
    expression(ExpressionKind::Variable(path(identifier)))
}

pub fn variable_path(path: Path) -> Expression {
    expression(ExpressionKind::Variable(path))
}

pub fn method_call(
    object: Expression,
    method_name: &str,
    arguments: Vec<Expression>,
) -> Expression {
    expression(ExpressionKind::MethodCall(Box::new(MethodCallExpression {
        object,
        method_name: ident(method_name),
        arguments,
    })))
}

pub fn call(func: Expression, arguments: Vec<Expression>) -> Expression {
    expression(ExpressionKind::Call(Box::new(CallExpression { func: Box::new(func), arguments })))
}

pub fn pattern(name: &str) -> Pattern {
    Pattern::Identifier(ident(name))
}

pub fn mutable(name: &str) -> Pattern {
    Pattern::Mutable(Box::new(pattern(name)), Span::default(), true)
}

pub fn mutable_assignment(name: &str, assigned_to: Expression) -> Statement {
    make_statement(StatementKind::Let(LetStatement {
        pattern: mutable(name),
        r#type: make_type(UnresolvedTypeData::Unspecified),
        expression: assigned_to,
    }))
}

pub fn mutable_reference(variable_name: &str) -> Expression {
    expression(ExpressionKind::Prefix(Box::new(PrefixExpression {
        operator: UnaryOp::MutableReference,
        rhs: variable(variable_name),
    })))
}

pub fn assignment(name: &str, assigned_to: Expression) -> Statement {
    make_statement(StatementKind::Let(LetStatement {
        pattern: pattern(name),
        r#type: make_type(UnresolvedTypeData::Unspecified),
        expression: assigned_to,
    }))
}

pub fn member_access(lhs: &str, rhs: &str) -> Expression {
    expression(ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
        lhs: variable(lhs),
        rhs: ident(rhs),
    })))
}

pub fn return_type(path: Path) -> FunctionReturnType {
    let ty = make_type(UnresolvedTypeData::Named(path, vec![], true));
    FunctionReturnType::Ty(ty)
}

pub fn lambda(parameters: Vec<(Pattern, UnresolvedType)>, body: Expression) -> Expression {
    expression(ExpressionKind::Lambda(Box::new(Lambda {
        parameters,
        return_type: UnresolvedType {
            typ: UnresolvedTypeData::Unspecified,
            span: Some(Span::default()),
        },
        body,
    })))
}

pub fn make_eq(lhs: Expression, rhs: Expression) -> Expression {
    expression(ExpressionKind::Infix(Box::new(InfixExpression {
        lhs,
        rhs,
        operator: Spanned::from(Span::default(), BinaryOpKind::Equal),
    })))
}

pub fn make_statement(kind: StatementKind) -> Statement {
    Statement { span: Span::default(), kind }
}

#[macro_export]
macro_rules! chained_path {
    ( $base:expr ) => {
        {
            ident_path($base)
        }
    };
    ( $base:expr $(, $tail:expr)* ) => {
        {
            let mut base_path = ident_path($base);
            $(
                base_path.segments.push(ident($tail));
            )*
            base_path
        }
    }
}

#[macro_export]
macro_rules! chained_dep {
    ( $base:expr $(, $tail:expr)* ) => {
        {
            let mut base_path = ident_path($base);
            base_path.kind = PathKind::Dep;
            $(
                base_path.segments.push(ident($tail));
            )*
            base_path
        }
    }
}

pub fn cast(lhs: Expression, ty: UnresolvedTypeData) -> Expression {
    expression(ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type: make_type(ty) })))
}

pub fn make_type(typ: UnresolvedTypeData) -> UnresolvedType {
    UnresolvedType { typ, span: Some(Span::default()) }
}

pub fn index_array(array: Ident, index: &str) -> Expression {
    expression(ExpressionKind::Index(Box::new(IndexExpression {
        collection: variable_path(path(array)),
        index: variable(index),
    })))
}

pub fn index_array_variable(array: Expression, index: &str) -> Expression {
    expression(ExpressionKind::Index(Box::new(IndexExpression {
        collection: array,
        index: variable(index),
    })))
}

/// Checks if an attribute is a custom attribute with a specific name
pub fn is_custom_attribute(attr: &SecondaryAttribute, attribute_name: &str) -> bool {
    if let SecondaryAttribute::Custom(custom_attr) = attr {
        custom_attr.as_str() == attribute_name
    } else {
        false
    }
}
