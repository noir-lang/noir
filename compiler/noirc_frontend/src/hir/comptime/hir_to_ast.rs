use iter_extended::vecmap;
use noirc_errors::{Span, Spanned};

use crate::ast::{
    ArrayLiteral, AssignStatement, BlockExpression, CallExpression, CastExpression, ConstrainKind,
    ConstructorExpression, ExpressionKind, ForLoopStatement, ForRange, Ident, IfExpression,
    IndexExpression, InfixExpression, LValue, Lambda, LetStatement, Literal,
    MemberAccessExpression, MethodCallExpression, Path, Pattern, PrefixExpression, UnresolvedType,
    UnresolvedTypeData, UnresolvedTypeExpression,
};
use crate::ast::{ConstrainStatement, Expression, Statement, StatementKind};
use crate::hir_def::expr::{HirArrayLiteral, HirBlockExpression, HirExpression, HirIdent};
use crate::hir_def::stmt::{HirLValue, HirPattern, HirStatement};
use crate::hir_def::types::Type;
use crate::macros_api::HirLiteral;
use crate::node_interner::{ExprId, NodeInterner, StmtId};

// TODO:
// - Full path for idents & types
// - Assert/AssertEq information lost
// - The type name span is lost in constructor patterns & expressions
// - All type spans are lost
// - Type::TypeVariable has no equivalent in the Ast

impl StmtId {
    #[allow(unused)]
    fn to_ast(self, interner: &NodeInterner) -> Statement {
        let statement = interner.statement(&self);
        let span = interner.statement_span(self);

        let kind = match statement {
            HirStatement::Let(let_stmt) => {
                let pattern = let_stmt.pattern.into_ast(interner);
                let r#type = interner.id_type(let_stmt.expression).to_ast();
                let expression = let_stmt.expression.to_ast(interner);
                StatementKind::Let(LetStatement {
                    pattern,
                    r#type,
                    expression,
                    comptime: false,
                    attributes: Vec::new(),
                })
            }
            HirStatement::Constrain(constrain) => {
                let expr = constrain.0.to_ast(interner);
                let message = constrain.2.map(|message| message.to_ast(interner));

                // TODO: Find difference in usage between Assert & AssertEq
                StatementKind::Constrain(ConstrainStatement(expr, message, ConstrainKind::Assert))
            }
            HirStatement::Assign(assign) => StatementKind::Assign(AssignStatement {
                lvalue: assign.lvalue.into_ast(interner),
                expression: assign.expression.to_ast(interner),
            }),
            HirStatement::For(for_stmt) => StatementKind::For(ForLoopStatement {
                identifier: for_stmt.identifier.to_ast(interner),
                range: ForRange::Range(
                    for_stmt.start_range.to_ast(interner),
                    for_stmt.end_range.to_ast(interner),
                ),
                block: for_stmt.block.to_ast(interner),
                span,
            }),
            HirStatement::Break => StatementKind::Break,
            HirStatement::Continue => StatementKind::Continue,
            HirStatement::Expression(expr) => StatementKind::Expression(expr.to_ast(interner)),
            HirStatement::Semi(expr) => StatementKind::Semi(expr.to_ast(interner)),
            HirStatement::Error => StatementKind::Error,
            HirStatement::Comptime(statement) => {
                StatementKind::Comptime(Box::new(statement.to_ast(interner)))
            }
        };

        Statement { kind, span }
    }
}

impl ExprId {
    #[allow(unused)]
    fn to_ast(self, interner: &NodeInterner) -> Expression {
        let expression = interner.expression(&self);
        let span = interner.expr_span(&self);

        let kind = match expression {
            HirExpression::Ident(ident) => {
                let path = Path::from_ident(ident.to_ast(interner));
                ExpressionKind::Variable(path)
            }
            HirExpression::Literal(HirLiteral::Array(array)) => {
                let array = array.into_ast(interner, span);
                ExpressionKind::Literal(Literal::Array(array))
            }
            HirExpression::Literal(HirLiteral::Slice(array)) => {
                let array = array.into_ast(interner, span);
                ExpressionKind::Literal(Literal::Slice(array))
            }
            HirExpression::Literal(HirLiteral::Bool(value)) => {
                ExpressionKind::Literal(Literal::Bool(value))
            }
            HirExpression::Literal(HirLiteral::Integer(value, sign)) => {
                ExpressionKind::Literal(Literal::Integer(value, sign))
            }
            HirExpression::Literal(HirLiteral::Str(string)) => {
                ExpressionKind::Literal(Literal::Str(string))
            }
            HirExpression::Literal(HirLiteral::FmtStr(string, _exprs)) => {
                // TODO: Is throwing away the exprs here valid?
                ExpressionKind::Literal(Literal::FmtStr(string))
            }
            HirExpression::Literal(HirLiteral::Unit) => ExpressionKind::Literal(Literal::Unit),
            HirExpression::Block(expr) => ExpressionKind::Block(expr.into_ast(interner)),
            HirExpression::Prefix(prefix) => ExpressionKind::Prefix(Box::new(PrefixExpression {
                operator: prefix.operator,
                rhs: prefix.rhs.to_ast(interner),
            })),
            HirExpression::Infix(infix) => ExpressionKind::Infix(Box::new(InfixExpression {
                lhs: infix.lhs.to_ast(interner),
                operator: Spanned::from(infix.operator.location.span, infix.operator.kind),
                rhs: infix.rhs.to_ast(interner),
            })),
            HirExpression::Index(index) => ExpressionKind::Index(Box::new(IndexExpression {
                collection: index.collection.to_ast(interner),
                index: index.index.to_ast(interner),
            })),
            HirExpression::Constructor(constructor) => {
                let type_name = constructor.r#type.borrow().name.to_string();
                let type_name = Path::from_single(type_name, span);
                let fields =
                    vecmap(constructor.fields, |(name, expr)| (name, expr.to_ast(interner)));

                ExpressionKind::Constructor(Box::new(ConstructorExpression { type_name, fields }))
            }
            HirExpression::MemberAccess(access) => {
                ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
                    lhs: access.lhs.to_ast(interner),
                    rhs: access.rhs,
                }))
            }
            HirExpression::Call(call) => {
                let func = Box::new(call.func.to_ast(interner));
                let arguments = vecmap(call.arguments, |arg| arg.to_ast(interner));
                ExpressionKind::Call(Box::new(CallExpression { func, arguments }))
            }
            HirExpression::MethodCall(method_call) => {
                ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                    object: method_call.object.to_ast(interner),
                    method_name: method_call.method,
                    arguments: vecmap(method_call.arguments, |arg| arg.to_ast(interner)),
                }))
            }
            HirExpression::Cast(cast) => {
                let lhs = cast.lhs.to_ast(interner);
                let r#type = cast.r#type.to_ast();
                ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type }))
            }
            HirExpression::If(if_expr) => ExpressionKind::If(Box::new(IfExpression {
                condition: if_expr.condition.to_ast(interner),
                consequence: if_expr.consequence.to_ast(interner),
                alternative: if_expr.alternative.map(|expr| expr.to_ast(interner)),
            })),
            HirExpression::Tuple(fields) => {
                ExpressionKind::Tuple(vecmap(fields, |field| field.to_ast(interner)))
            }
            HirExpression::Lambda(lambda) => {
                let parameters = vecmap(lambda.parameters, |(pattern, typ)| {
                    (pattern.into_ast(interner), typ.to_ast())
                });
                let return_type = lambda.return_type.to_ast();
                let body = lambda.body.to_ast(interner);
                ExpressionKind::Lambda(Box::new(Lambda { parameters, return_type, body }))
            }
            HirExpression::Error => ExpressionKind::Error,
            HirExpression::Comptime(block) => ExpressionKind::Comptime(block.into_ast(interner)),
            HirExpression::Quote(block) => ExpressionKind::Quote(block),

            // A macro was evaluated here!
            HirExpression::Unquote(block) => ExpressionKind::Block(block),
        };

        Expression::new(kind, span)
    }
}

impl HirPattern {
    fn into_ast(self, interner: &NodeInterner) -> Pattern {
        match self {
            HirPattern::Identifier(ident) => Pattern::Identifier(ident.to_ast(interner)),
            HirPattern::Mutable(pattern, location) => {
                let pattern = Box::new(pattern.into_ast(interner));
                Pattern::Mutable(pattern, location.span, false)
            }
            HirPattern::Tuple(patterns, location) => {
                let patterns = vecmap(patterns, |pattern| pattern.into_ast(interner));
                Pattern::Tuple(patterns, location.span)
            }
            HirPattern::Struct(typ, patterns, location) => {
                let patterns =
                    vecmap(patterns, |(name, pattern)| (name, pattern.into_ast(interner)));
                let name = match typ.follow_bindings() {
                    Type::Struct(struct_def, _) => {
                        let struct_def = struct_def.borrow();
                        struct_def.name.0.contents.clone()
                    }
                    // This pass shouldn't error so if the type isn't a struct we just get a string
                    // representation of any other type and use that. We're relying on name
                    // resolution to fail later when this Ast is re-converted to Hir.
                    other => other.to_string(),
                };
                // The name span is lost here
                let path = Path::from_single(name, location.span);
                Pattern::Struct(path, patterns, location.span)
            }
        }
    }
}

impl HirIdent {
    fn to_ast(&self, interner: &NodeInterner) -> Ident {
        let name = interner.definition_name(self.id).to_owned();
        Ident(Spanned::from(self.location.span, name))
    }
}

impl Type {
    fn to_ast(&self) -> UnresolvedType {
        let typ = match self {
            Type::FieldElement => UnresolvedTypeData::FieldElement,
            Type::Array(length, element) => {
                let length = length.to_type_expression();
                let element = Box::new(element.to_ast());
                UnresolvedTypeData::Array(length, element)
            }
            Type::Slice(element) => {
                let element = Box::new(element.to_ast());
                UnresolvedTypeData::Slice(element)
            }
            Type::Integer(sign, bit_size) => UnresolvedTypeData::Integer(*sign, *bit_size),
            Type::Bool => UnresolvedTypeData::Bool,
            Type::String(length) => {
                let length = length.to_type_expression();
                UnresolvedTypeData::String(Some(length))
            }
            Type::FmtString(length, element) => {
                let length = length.to_type_expression();
                let element = Box::new(element.to_ast());
                UnresolvedTypeData::FormatString(length, element)
            }
            Type::Unit => UnresolvedTypeData::Unit,
            Type::Tuple(fields) => {
                let fields = vecmap(fields, |field| field.to_ast());
                UnresolvedTypeData::Tuple(fields)
            }
            Type::Struct(def, generics) => {
                let struct_def = def.borrow();
                let generics = vecmap(generics, |generic| generic.to_ast());
                let name = Path::from_ident(struct_def.name.clone());
                UnresolvedTypeData::Named(name, generics, false)
            }
            Type::Alias(type_def, generics) => {
                // Keep the alias name instead of expanding this in case the
                // alias' definition was changed
                let type_def = type_def.borrow();
                let generics = vecmap(generics, |generic| generic.to_ast());
                let name = Path::from_ident(type_def.name.clone());
                UnresolvedTypeData::Named(name, generics, false)
            }
            Type::TypeVariable(_, _) => todo!("Convert Type::TypeVariable Hir -> Ast"),
            Type::TraitAsType(_, name, generics) => {
                let generics = vecmap(generics, |generic| generic.to_ast());
                let name = Path::from_single(name.as_ref().clone(), Span::default());
                UnresolvedTypeData::TraitAsType(name, generics)
            }
            Type::NamedGeneric(_, name) => {
                let name = Path::from_single(name.as_ref().clone(), Span::default());
                UnresolvedTypeData::TraitAsType(name, Vec::new())
            }
            Type::Function(args, ret, env) => {
                let args = vecmap(args, |arg| arg.to_ast());
                let ret = Box::new(ret.to_ast());
                let env = Box::new(env.to_ast());
                UnresolvedTypeData::Function(args, ret, env)
            }
            Type::MutableReference(element) => {
                let element = Box::new(element.to_ast());
                UnresolvedTypeData::MutableReference(element)
            }
            // Type::Forall is only for generic functions which don't store a type
            // in their Ast so they don't need to call to_ast for their Forall type.
            // Since there is no UnresolvedTypeData equivalent for Type::Forall, we use
            // this to ignore this case since it shouldn't be needed anyway.
            Type::Forall(_, typ) => return typ.to_ast(),
            Type::Constant(_) => panic!("Type::Constant where a type was expected: {self:?}"),
            Type::Code => UnresolvedTypeData::Code,
            Type::Error => UnresolvedTypeData::Error,
        };

        UnresolvedType { typ, span: None }
    }

    fn to_type_expression(&self) -> UnresolvedTypeExpression {
        let span = Span::default();

        match self.follow_bindings() {
            Type::Constant(length) => UnresolvedTypeExpression::Constant(length, span),
            Type::NamedGeneric(_, name) => {
                let path = Path::from_single(name.as_ref().clone(), span);
                UnresolvedTypeExpression::Variable(path)
            }
            // TODO: This should be turned into a proper error.
            other => panic!("Cannot represent {other:?} as type expression"),
        }
    }
}

impl HirLValue {
    fn into_ast(self, interner: &NodeInterner) -> LValue {
        match self {
            HirLValue::Ident(ident, _) => LValue::Ident(ident.to_ast(interner)),
            HirLValue::MemberAccess { object, field_name, field_index: _, typ: _, location } => {
                let object = Box::new(object.into_ast(interner));
                LValue::MemberAccess { object, field_name, span: location.span }
            }
            HirLValue::Index { array, index, typ: _, location } => {
                let array = Box::new(array.into_ast(interner));
                let index = index.to_ast(interner);
                LValue::Index { array, index, span: location.span }
            }
            HirLValue::Dereference { lvalue, element_type: _, location } => {
                let lvalue = Box::new(lvalue.into_ast(interner));
                LValue::Dereference(lvalue, location.span)
            }
        }
    }
}

impl HirArrayLiteral {
    fn into_ast(self, interner: &NodeInterner, span: Span) -> ArrayLiteral {
        match self {
            HirArrayLiteral::Standard(elements) => {
                ArrayLiteral::Standard(vecmap(elements, |element| element.to_ast(interner)))
            }
            HirArrayLiteral::Repeated { repeated_element, length } => {
                let repeated_element = Box::new(repeated_element.to_ast(interner));
                let length = match length {
                    Type::Constant(length) => {
                        let literal = Literal::Integer((length as u128).into(), false);
                        let kind = ExpressionKind::Literal(literal);
                        Box::new(Expression::new(kind, span))
                    }
                    other => panic!("Cannot convert non-constant type for repeated array literal from Hir -> Ast: {other:?}"),
                };
                ArrayLiteral::Repeated { repeated_element, length }
            }
        }
    }
}

impl HirBlockExpression {
    fn into_ast(self, interner: &NodeInterner) -> BlockExpression {
        let statements = vecmap(self.statements, |statement| statement.to_ast(interner));
        BlockExpression { statements }
    }
}
