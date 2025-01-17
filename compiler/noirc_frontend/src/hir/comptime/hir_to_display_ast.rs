use iter_extended::vecmap;
use noirc_errors::{Span, Spanned};

use crate::ast::{
    ArrayLiteral, AssignStatement, BlockExpression, CallExpression, CastExpression, ConstrainKind,
    ConstructorExpression, ExpressionKind, ForLoopStatement, ForRange, GenericTypeArgs, Ident,
    IfExpression, IndexExpression, InfixExpression, LValue, Lambda, Literal,
    MemberAccessExpression, MethodCallExpression, Path, PathSegment, Pattern, PrefixExpression,
    UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression,
};
use crate::ast::{ConstrainStatement, Expression, Statement, StatementKind};
use crate::hir_def::expr::{
    HirArrayLiteral, HirBlockExpression, HirExpression, HirIdent, HirLiteral,
};
use crate::hir_def::stmt::{HirLValue, HirPattern, HirStatement};
use crate::hir_def::types::{Type, TypeBinding};
use crate::node_interner::{ExprId, NodeInterner, StmtId};

// TODO:
// - Full path for idents & types
// - Assert/AssertEq information lost
// - The type name span is lost in constructor patterns & expressions
// - All type spans are lost
// - Type::TypeVariable has no equivalent in the Ast

impl HirStatement {
    pub fn to_display_ast(&self, interner: &NodeInterner, span: Span) -> Statement {
        let kind = match self {
            HirStatement::Let(let_stmt) => {
                let pattern = let_stmt.pattern.to_display_ast(interner);
                let r#type = interner.id_type(let_stmt.expression).to_display_ast();
                let expression = let_stmt.expression.to_display_ast(interner);
                StatementKind::new_let(pattern, r#type, expression, let_stmt.attributes.clone())
            }
            HirStatement::Constrain(constrain) => {
                let expr = constrain.0.to_display_ast(interner);
                let mut arguments = vec![expr];
                if let Some(message) = constrain.2 {
                    arguments.push(message.to_display_ast(interner));
                }

                // TODO: Find difference in usage between Assert & AssertEq
                StatementKind::Constrain(ConstrainStatement {
                    kind: ConstrainKind::Assert,
                    arguments,
                    span,
                })
            }
            HirStatement::Assign(assign) => StatementKind::Assign(AssignStatement {
                lvalue: assign.lvalue.to_display_ast(interner),
                expression: assign.expression.to_display_ast(interner),
            }),
            HirStatement::For(for_stmt) => StatementKind::For(ForLoopStatement {
                identifier: for_stmt.identifier.to_display_ast(interner),
                range: ForRange::range(
                    for_stmt.start_range.to_display_ast(interner),
                    for_stmt.end_range.to_display_ast(interner),
                ),
                block: for_stmt.block.to_display_ast(interner),
                span,
            }),
            HirStatement::Loop(block) => StatementKind::Loop(block.to_display_ast(interner)),
            HirStatement::Break => StatementKind::Break,
            HirStatement::Continue => StatementKind::Continue,
            HirStatement::Expression(expr) => {
                StatementKind::Expression(expr.to_display_ast(interner))
            }
            HirStatement::Semi(expr) => StatementKind::Semi(expr.to_display_ast(interner)),
            HirStatement::Error => StatementKind::Error,
            HirStatement::Comptime(statement) => {
                StatementKind::Comptime(Box::new(statement.to_display_ast(interner)))
            }
        };

        Statement { kind, span }
    }
}

impl StmtId {
    /// Convert to AST for display (some details lost)
    pub fn to_display_ast(self, interner: &NodeInterner) -> Statement {
        let statement = interner.statement(&self);
        let span = interner.statement_span(self);

        statement.to_display_ast(interner, span)
    }
}

impl HirExpression {
    /// Convert to AST for display (some details lost)
    pub fn to_display_ast(&self, interner: &NodeInterner, span: Span) -> Expression {
        let kind = match self {
            HirExpression::Ident(ident, generics) => {
                let ident = ident.to_display_ast(interner);
                let segment = PathSegment {
                    ident,
                    generics: generics.as_ref().map(|option| {
                        option.iter().map(|generic| generic.to_display_ast()).collect()
                    }),
                    span,
                };

                let path =
                    Path { segments: vec![segment], kind: crate::ast::PathKind::Plain, span };

                ExpressionKind::Variable(path)
            }
            HirExpression::Literal(HirLiteral::Array(array)) => {
                let array = array.to_display_ast(interner, span);
                ExpressionKind::Literal(Literal::Array(array))
            }
            HirExpression::Literal(HirLiteral::Slice(array)) => {
                let array = array.to_display_ast(interner, span);
                ExpressionKind::Literal(Literal::Slice(array))
            }
            HirExpression::Literal(HirLiteral::Bool(value)) => {
                ExpressionKind::Literal(Literal::Bool(*value))
            }
            HirExpression::Literal(HirLiteral::Integer(value, sign)) => {
                ExpressionKind::Literal(Literal::Integer(*value, *sign))
            }
            HirExpression::Literal(HirLiteral::Str(string)) => {
                ExpressionKind::Literal(Literal::Str(string.clone()))
            }
            HirExpression::Literal(HirLiteral::FmtStr(fragments, _exprs, length)) => {
                // TODO: Is throwing away the exprs here valid?
                ExpressionKind::Literal(Literal::FmtStr(fragments.clone(), *length))
            }
            HirExpression::Literal(HirLiteral::Unit) => ExpressionKind::Literal(Literal::Unit),
            HirExpression::Block(expr) => ExpressionKind::Block(expr.to_display_ast(interner)),
            HirExpression::Prefix(prefix) => ExpressionKind::Prefix(Box::new(PrefixExpression {
                operator: prefix.operator,
                rhs: prefix.rhs.to_display_ast(interner),
            })),
            HirExpression::Infix(infix) => ExpressionKind::Infix(Box::new(InfixExpression {
                lhs: infix.lhs.to_display_ast(interner),
                operator: Spanned::from(infix.operator.location.span, infix.operator.kind),
                rhs: infix.rhs.to_display_ast(interner),
            })),
            HirExpression::Index(index) => ExpressionKind::Index(Box::new(IndexExpression {
                collection: index.collection.to_display_ast(interner),
                index: index.index.to_display_ast(interner),
            })),
            HirExpression::Constructor(constructor) => {
                let type_name = constructor.r#type.borrow().name.to_string();
                let type_name = Path::from_single(type_name, span);
                let fields = vecmap(constructor.fields.clone(), |(name, expr): (Ident, ExprId)| {
                    (name, expr.to_display_ast(interner))
                });
                let struct_type = None;

                ExpressionKind::Constructor(Box::new(ConstructorExpression {
                    typ: UnresolvedType::from_path(type_name),
                    fields,
                    struct_type,
                }))
            }
            HirExpression::MemberAccess(access) => {
                ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
                    lhs: access.lhs.to_display_ast(interner),
                    rhs: access.rhs.clone(),
                }))
            }
            HirExpression::Call(call) => {
                let func = Box::new(call.func.to_display_ast(interner));
                let arguments = vecmap(call.arguments.clone(), |arg| arg.to_display_ast(interner));
                let is_macro_call = false;
                ExpressionKind::Call(Box::new(CallExpression { func, arguments, is_macro_call }))
            }
            HirExpression::MethodCall(method_call) => {
                ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                    object: method_call.object.to_display_ast(interner),
                    method_name: method_call.method.clone(),
                    arguments: vecmap(method_call.arguments.clone(), |arg| {
                        arg.to_display_ast(interner)
                    }),
                    generics: method_call.generics.clone().map(|option| {
                        option.iter().map(|generic| generic.to_display_ast()).collect()
                    }),
                    is_macro_call: false,
                }))
            }
            HirExpression::Cast(cast) => {
                let lhs = cast.lhs.to_display_ast(interner);
                let r#type = cast.r#type.to_display_ast();
                ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type }))
            }
            HirExpression::If(if_expr) => ExpressionKind::If(Box::new(IfExpression {
                condition: if_expr.condition.to_display_ast(interner),
                consequence: if_expr.consequence.to_display_ast(interner),
                alternative: if_expr.alternative.map(|expr| expr.to_display_ast(interner)),
            })),
            HirExpression::Tuple(fields) => {
                ExpressionKind::Tuple(vecmap(fields, |field| field.to_display_ast(interner)))
            }
            HirExpression::Lambda(lambda) => {
                let parameters = vecmap(lambda.parameters.clone(), |(pattern, typ)| {
                    (pattern.to_display_ast(interner), typ.to_display_ast())
                });
                let return_type = lambda.return_type.to_display_ast();
                let body = lambda.body.to_display_ast(interner);
                ExpressionKind::Lambda(Box::new(Lambda { parameters, return_type, body }))
            }
            HirExpression::Error => ExpressionKind::Error,
            HirExpression::Comptime(block) => {
                ExpressionKind::Comptime(block.to_display_ast(interner), span)
            }
            HirExpression::Unsafe(block) => {
                ExpressionKind::Unsafe(block.to_display_ast(interner), span)
            }
            HirExpression::Quote(block) => ExpressionKind::Quote(block.clone()),

            // A macro was evaluated here: return the quoted result
            HirExpression::Unquote(block) => ExpressionKind::Quote(block.clone()),
        };

        Expression::new(kind, span)
    }
}

impl ExprId {
    /// Convert to AST for display (some details lost)
    pub fn to_display_ast(self, interner: &NodeInterner) -> Expression {
        let expression = interner.expression(&self);
        // TODO: empty 0 span
        let span = interner.try_expr_span(&self).unwrap_or_else(|| Span::empty(0));
        expression.to_display_ast(interner, span)
    }
}

impl HirPattern {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner) -> Pattern {
        match self {
            HirPattern::Identifier(ident) => Pattern::Identifier(ident.to_display_ast(interner)),
            HirPattern::Mutable(pattern, location) => {
                let pattern = Box::new(pattern.to_display_ast(interner));
                Pattern::Mutable(pattern, location.span, false)
            }
            HirPattern::Tuple(patterns, location) => {
                let patterns = vecmap(patterns, |pattern| pattern.to_display_ast(interner));
                Pattern::Tuple(patterns, location.span)
            }
            HirPattern::Struct(typ, patterns, location) => {
                let patterns = vecmap(patterns, |(name, pattern)| {
                    (name.clone(), pattern.to_display_ast(interner))
                });
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
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner) -> Ident {
        let name = interner.definition_name(self.id).to_owned();
        Ident(Spanned::from(self.location.span, name))
    }
}

impl Type {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self) -> UnresolvedType {
        let typ = match self {
            Type::FieldElement => UnresolvedTypeData::FieldElement,
            Type::Array(length, element) => {
                let length = length.to_type_expression();
                let element = Box::new(element.to_display_ast());
                UnresolvedTypeData::Array(length, element)
            }
            Type::Slice(element) => {
                let element = Box::new(element.to_display_ast());
                UnresolvedTypeData::Slice(element)
            }
            Type::Integer(sign, bit_size) => UnresolvedTypeData::Integer(*sign, *bit_size),
            Type::Bool => UnresolvedTypeData::Bool,
            Type::String(length) => {
                let length = length.to_type_expression();
                UnresolvedTypeData::String(length)
            }
            Type::FmtString(length, element) => {
                let length = length.to_type_expression();
                let element = Box::new(element.to_display_ast());
                UnresolvedTypeData::FormatString(length, element)
            }
            Type::Unit => UnresolvedTypeData::Unit,
            Type::Tuple(fields) => {
                let fields = vecmap(fields, |field| field.to_display_ast());
                UnresolvedTypeData::Tuple(fields)
            }
            Type::Struct(def, generics) => {
                let struct_def = def.borrow();
                let ordered_args = vecmap(generics, |generic| generic.to_display_ast());
                let generics =
                    GenericTypeArgs { ordered_args, named_args: Vec::new(), kinds: Vec::new() };
                let name = Path::from_ident(struct_def.name.clone());
                UnresolvedTypeData::Named(name, generics, false)
            }
            Type::Alias(type_def, generics) => {
                // Keep the alias name instead of expanding this in case the
                // alias' definition was changed
                let type_def = type_def.borrow();
                let ordered_args = vecmap(generics, |generic| generic.to_display_ast());
                let generics =
                    GenericTypeArgs { ordered_args, named_args: Vec::new(), kinds: Vec::new() };
                let name = Path::from_ident(type_def.name.clone());
                UnresolvedTypeData::Named(name, generics, false)
            }
            Type::TypeVariable(binding) => match &*binding.borrow() {
                TypeBinding::Bound(typ) => return typ.to_display_ast(),
                TypeBinding::Unbound(id, type_var_kind) => {
                    let name = format!("var_{:?}_{}", type_var_kind, id);
                    let path = Path::from_single(name, Span::empty(0));
                    let expression = UnresolvedTypeExpression::Variable(path);
                    UnresolvedTypeData::Expression(expression)
                }
            },
            Type::TraitAsType(_, name, generics) => {
                let ordered_args = vecmap(&generics.ordered, |generic| generic.to_display_ast());
                let named_args = vecmap(&generics.named, |named_type| {
                    (named_type.name.clone(), named_type.typ.to_display_ast())
                });
                let generics = GenericTypeArgs { ordered_args, named_args, kinds: Vec::new() };
                let name = Path::from_single(name.as_ref().clone(), Span::default());
                UnresolvedTypeData::TraitAsType(name, generics)
            }
            Type::NamedGeneric(_var, name) => {
                let name = Path::from_single(name.as_ref().clone(), Span::default());
                UnresolvedTypeData::Named(name, GenericTypeArgs::default(), true)
            }
            Type::CheckedCast { to, .. } => to.to_display_ast().typ,
            Type::Function(args, ret, env, unconstrained) => {
                let args = vecmap(args, |arg| arg.to_display_ast());
                let ret = Box::new(ret.to_display_ast());
                let env = Box::new(env.to_display_ast());
                UnresolvedTypeData::Function(args, ret, env, *unconstrained)
            }
            Type::MutableReference(element) => {
                let element = Box::new(element.to_display_ast());
                UnresolvedTypeData::MutableReference(element)
            }
            // Type::Forall is only for generic functions which don't store a type
            // in their Ast so they don't need to call to_display_ast for their Forall type.
            // Since there is no UnresolvedTypeData equivalent for Type::Forall, we use
            // this to ignore this case since it shouldn't be needed anyway.
            Type::Forall(_, typ) => return typ.to_display_ast(),
            Type::Constant(..) => panic!("Type::Constant where a type was expected: {self:?}"),
            Type::Quoted(quoted_type) => UnresolvedTypeData::Quoted(*quoted_type),
            Type::Error => UnresolvedTypeData::Error,
            Type::InfixExpr(lhs, op, rhs, _) => {
                let lhs = Box::new(lhs.to_type_expression());
                let rhs = Box::new(rhs.to_type_expression());
                let span = Span::default();
                let expr = UnresolvedTypeExpression::BinaryOperation(lhs, *op, rhs, span);
                UnresolvedTypeData::Expression(expr)
            }
        };

        UnresolvedType { typ, span: Span::default() }
    }

    /// Convert to AST for display (some details lost)
    fn to_type_expression(&self) -> UnresolvedTypeExpression {
        let span = Span::default();

        match self.follow_bindings() {
            Type::Constant(length, _kind) => UnresolvedTypeExpression::Constant(length, span),
            Type::NamedGeneric(_var, name) => {
                let path = Path::from_single(name.as_ref().clone(), span);
                UnresolvedTypeExpression::Variable(path)
            }
            // TODO: This should be turned into a proper error.
            other => panic!("Cannot represent {other:?} as type expression"),
        }
    }
}

impl HirLValue {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner) -> LValue {
        match self {
            HirLValue::Ident(ident, _) => LValue::Ident(ident.to_display_ast(interner)),
            HirLValue::MemberAccess { object, field_name, field_index: _, typ: _, location } => {
                let object = Box::new(object.to_display_ast(interner));
                LValue::MemberAccess { object, field_name: field_name.clone(), span: location.span }
            }
            HirLValue::Index { array, index, typ: _, location } => {
                let array = Box::new(array.to_display_ast(interner));
                let index = index.to_display_ast(interner);
                LValue::Index { array, index, span: location.span }
            }
            HirLValue::Dereference { lvalue, element_type: _, location } => {
                let lvalue = Box::new(lvalue.to_display_ast(interner));
                LValue::Dereference(lvalue, location.span)
            }
        }
    }
}

impl HirArrayLiteral {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner, span: Span) -> ArrayLiteral {
        match self {
            HirArrayLiteral::Standard(elements) => {
                ArrayLiteral::Standard(vecmap(elements, |element| element.to_display_ast(interner)))
            }
            HirArrayLiteral::Repeated { repeated_element, length } => {
                let repeated_element = Box::new(repeated_element.to_display_ast(interner));
                let length = match length {
                    Type::Constant(length, _kind) => {
                        let literal = Literal::Integer(*length, false);
                        let expr_kind = ExpressionKind::Literal(literal);
                        Box::new(Expression::new(expr_kind, span))
                    }
                    other => panic!("Cannot convert non-constant type for repeated array literal from Hir -> Ast: {other:?}"),
                };
                ArrayLiteral::Repeated { repeated_element, length }
            }
        }
    }
}

impl HirBlockExpression {
    /// Convert to AST for display (some details lost)
    fn to_display_ast(&self, interner: &NodeInterner) -> BlockExpression {
        let statements =
            vecmap(self.statements.clone(), |statement| statement.to_display_ast(interner));
        BlockExpression { statements }
    }
}
