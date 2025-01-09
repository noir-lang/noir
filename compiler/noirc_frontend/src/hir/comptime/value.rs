use std::{borrow::Cow, rc::Rc, vec};

use acvm::FieldElement;
use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::{Location, Span};
use strum_macros::Display;

use crate::{
    ast::{
        ArrayLiteral, BlockExpression, ConstructorExpression, Expression, ExpressionKind, Ident,
        IntegerBitSize, LValue, Literal, Path, Pattern, Signedness, Statement, StatementKind,
        UnresolvedType, UnresolvedTypeData,
    },
    elaborator::Elaborator,
    hir::{def_map::ModuleId, type_check::generics::TraitGenerics},
    hir_def::expr::{
        HirArrayLiteral, HirConstructorExpression, HirExpression, HirIdent, HirLambda, HirLiteral,
        ImplKind,
    },
    node_interner::{ExprId, FuncId, NodeInterner, StmtId, StructId, TraitId, TraitImplId},
    parser::{Item, Parser},
    token::{SpannedToken, Token, Tokens},
    Kind, QuotedType, Shared, Type, TypeBindings,
};
use rustc_hash::FxHashMap as HashMap;

use super::{
    display::tokens_to_string,
    errors::{IResult, InterpreterError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    Field(FieldElement),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U1(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    String(Rc<String>),
    FormatString(Rc<String>, Type),
    CtString(Rc<String>),
    Function(FuncId, Type, Rc<TypeBindings>),

    // Closures also store their original scope (function & module)
    // in case they use functions such as `Quoted::as_type` which require them.
    Closure(HirLambda, Vec<Value>, Type, Option<FuncId>, ModuleId),

    Tuple(Vec<Value>),
    Struct(HashMap<Rc<String>, Value>, Type),
    Pointer(Shared<Value>, /* auto_deref */ bool),
    Array(Vector<Value>, Type),
    Slice(Vector<Value>, Type),
    /// Quoted tokens don't have spans because otherwise inserting them in the middle of other
    /// tokens can cause larger spans to be before lesser spans, causing an assert. They may also
    /// be inserted into separate files entirely.
    Quoted(Rc<Vec<Token>>),
    StructDefinition(StructId),
    TraitConstraint(TraitId, TraitGenerics),
    TraitDefinition(TraitId),
    TraitImpl(TraitImplId),
    FunctionDefinition(FuncId),
    ModuleDefinition(ModuleId),
    Type(Type),
    Zeroed(Type),
    Expr(ExprValue),
    TypedExpr(TypedExpr),
    UnresolvedType(UnresolvedTypeData),
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum ExprValue {
    Expression(ExpressionKind),
    Statement(StatementKind),
    LValue(LValue),
    Pattern(Pattern),
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum TypedExpr {
    ExprId(ExprId),
    StmtId(StmtId),
}

impl Value {
    pub(crate) fn expression(expr: ExpressionKind) -> Self {
        Value::Expr(ExprValue::Expression(expr))
    }

    pub(crate) fn statement(statement: StatementKind) -> Self {
        Value::Expr(ExprValue::Statement(statement))
    }

    pub(crate) fn lvalue(lvaue: LValue) -> Self {
        Value::Expr(ExprValue::LValue(lvaue))
    }

    pub(crate) fn pattern(pattern: Pattern) -> Self {
        Value::Expr(ExprValue::Pattern(pattern))
    }

    pub(crate) fn get_type(&self) -> Cow<Type> {
        Cow::Owned(match self {
            Value::Unit => Type::Unit,
            Value::Bool(_) => Type::Bool,
            Value::Field(_) => Type::FieldElement,
            Value::I8(_) => Type::Integer(Signedness::Signed, IntegerBitSize::Eight),
            Value::I16(_) => Type::Integer(Signedness::Signed, IntegerBitSize::Sixteen),
            Value::I32(_) => Type::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo),
            Value::I64(_) => Type::Integer(Signedness::Signed, IntegerBitSize::SixtyFour),
            Value::U1(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::One),
            Value::U8(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight),
            Value::U16(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::Sixteen),
            Value::U32(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo),
            Value::U64(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour),
            Value::String(value) => {
                let length = Type::Constant(value.len().into(), Kind::u32());
                Type::String(Box::new(length))
            }
            Value::FormatString(_, typ) => return Cow::Borrowed(typ),
            Value::Function(_, typ, _) => return Cow::Borrowed(typ),
            Value::Closure(_, _, typ, ..) => return Cow::Borrowed(typ),
            Value::Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| field.get_type().into_owned()))
            }
            Value::Struct(_, typ) => return Cow::Borrowed(typ),
            Value::Array(_, typ) => return Cow::Borrowed(typ),
            Value::Slice(_, typ) => return Cow::Borrowed(typ),
            Value::Quoted(_) => Type::Quoted(QuotedType::Quoted),
            Value::StructDefinition(_) => Type::Quoted(QuotedType::StructDefinition),
            Value::Pointer(element, auto_deref) => {
                if *auto_deref {
                    element.borrow().get_type().into_owned()
                } else {
                    let element = element.borrow().get_type().into_owned();
                    Type::MutableReference(Box::new(element))
                }
            }
            Value::TraitConstraint { .. } => Type::Quoted(QuotedType::TraitConstraint),
            Value::TraitDefinition(_) => Type::Quoted(QuotedType::TraitDefinition),
            Value::TraitImpl(_) => Type::Quoted(QuotedType::TraitImpl),
            Value::FunctionDefinition(_) => Type::Quoted(QuotedType::FunctionDefinition),
            Value::ModuleDefinition(_) => Type::Quoted(QuotedType::Module),
            Value::Type(_) => Type::Quoted(QuotedType::Type),
            Value::Zeroed(typ) => return Cow::Borrowed(typ),
            Value::Expr(_) => Type::Quoted(QuotedType::Expr),
            Value::TypedExpr(_) => Type::Quoted(QuotedType::TypedExpr),
            Value::UnresolvedType(_) => Type::Quoted(QuotedType::UnresolvedType),
            Value::CtString(_) => Type::Quoted(QuotedType::CtString),
        })
    }

    pub(crate) fn into_expression(
        self,
        elaborator: &mut Elaborator,
        location: Location,
    ) -> IResult<Expression> {
        let kind = match self {
            Value::Unit => ExpressionKind::Literal(Literal::Unit),
            Value::Bool(value) => ExpressionKind::Literal(Literal::Bool(value)),
            Value::Field(value) => ExpressionKind::Literal(Literal::Integer(value, false)),
            Value::I8(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                ExpressionKind::Literal(Literal::Integer(value, negative))
            }
            Value::I16(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                ExpressionKind::Literal(Literal::Integer(value, negative))
            }
            Value::I32(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                ExpressionKind::Literal(Literal::Integer(value, negative))
            }
            Value::I64(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                ExpressionKind::Literal(Literal::Integer(value, negative))
            }
            Value::U1(value) => {
                ExpressionKind::Literal(Literal::Integer((value as u128).into(), false))
            }
            Value::U8(value) => {
                ExpressionKind::Literal(Literal::Integer((value as u128).into(), false))
            }
            Value::U16(value) => {
                ExpressionKind::Literal(Literal::Integer((value as u128).into(), false))
            }
            Value::U32(value) => {
                ExpressionKind::Literal(Literal::Integer((value as u128).into(), false))
            }
            Value::U64(value) => {
                ExpressionKind::Literal(Literal::Integer((value as u128).into(), false))
            }
            Value::String(value) | Value::CtString(value) => {
                ExpressionKind::Literal(Literal::Str(unwrap_rc(value)))
            }
            // Format strings are lowered as normal strings since they are already interpolated.
            Value::FormatString(value, _) => {
                ExpressionKind::Literal(Literal::Str(unwrap_rc(value)))
            }
            Value::Function(id, typ, bindings) => {
                let id = elaborator.interner.function_definition_id(id);
                let impl_kind = ImplKind::NotATraitMethod;
                let ident = HirIdent { location, id, impl_kind };
                let expr_id = elaborator.interner.push_expr(HirExpression::Ident(ident, None));
                elaborator.interner.push_expr_location(expr_id, location.span, location.file);
                elaborator.interner.push_expr_type(expr_id, typ);
                elaborator.interner.store_instantiation_bindings(expr_id, unwrap_rc(bindings));
                ExpressionKind::Resolved(expr_id)
            }
            Value::Tuple(fields) => {
                let fields =
                    try_vecmap(fields, |field| field.into_expression(elaborator, location))?;
                ExpressionKind::Tuple(fields)
            }
            Value::Struct(fields, typ) => {
                let fields = try_vecmap(fields, |(name, field)| {
                    let field = field.into_expression(elaborator, location)?;
                    Ok((Ident::new(unwrap_rc(name), location.span), field))
                })?;

                let struct_type = match typ.follow_bindings() {
                    Type::Struct(def, _) => Some(def.borrow().id),
                    _ => return Err(InterpreterError::NonStructInConstructor { typ, location }),
                };

                // Since we've provided the struct_type, the path should be ignored.
                let type_name = Path::from_single(String::new(), location.span);
                ExpressionKind::Constructor(Box::new(ConstructorExpression {
                    typ: UnresolvedType::from_path(type_name),
                    fields,
                    struct_type,
                }))
            }
            Value::Array(elements, _) => {
                let elements =
                    try_vecmap(elements, |element| element.into_expression(elaborator, location))?;
                ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(elements)))
            }
            Value::Slice(elements, _) => {
                let elements =
                    try_vecmap(elements, |element| element.into_expression(elaborator, location))?;
                ExpressionKind::Literal(Literal::Slice(ArrayLiteral::Standard(elements)))
            }
            Value::Quoted(tokens) => {
                // Wrap the tokens in '{' and '}' so that we can parse statements as well.
                let mut tokens_to_parse = add_token_spans(tokens.clone(), location.span);
                tokens_to_parse.0.insert(0, SpannedToken::new(Token::LeftBrace, location.span));
                tokens_to_parse.0.push(SpannedToken::new(Token::RightBrace, location.span));

                let parser = Parser::for_tokens(tokens_to_parse);
                return match parser.parse_result(Parser::parse_expression_or_error) {
                    Ok((expr, warnings)) => {
                        for warning in warnings {
                            elaborator.errors.push((warning.into(), location.file));
                        }

                        Ok(expr)
                    }
                    Err(mut errors) => {
                        let error = errors.swap_remove(0);
                        let file = location.file;
                        let rule = "an expression";
                        let tokens = tokens_to_string(tokens, elaborator.interner);
                        Err(InterpreterError::FailedToParseMacro { error, file, tokens, rule })
                    }
                };
            }
            Value::Expr(ExprValue::Expression(expr)) => expr,
            Value::Expr(ExprValue::Statement(statement)) => {
                ExpressionKind::Block(BlockExpression {
                    statements: vec![Statement { kind: statement, span: location.span }],
                })
            }
            Value::Expr(ExprValue::LValue(lvalue)) => lvalue.as_expression().kind,
            Value::Expr(ExprValue::Pattern(_))
            | Value::TypedExpr(..)
            | Value::Pointer(..)
            | Value::StructDefinition(_)
            | Value::TraitConstraint(..)
            | Value::TraitDefinition(_)
            | Value::TraitImpl(_)
            | Value::FunctionDefinition(_)
            | Value::Zeroed(_)
            | Value::Type(_)
            | Value::UnresolvedType(_)
            | Value::Closure(..)
            | Value::ModuleDefinition(_) => {
                let typ = self.get_type().into_owned();
                let value = self.display(elaborator.interner).to_string();
                return Err(InterpreterError::CannotInlineMacro { typ, value, location });
            }
        };

        Ok(Expression::new(kind, location.span))
    }

    pub(crate) fn into_hir_expression(
        self,
        interner: &mut NodeInterner,
        location: Location,
    ) -> IResult<ExprId> {
        let typ = self.get_type().into_owned();

        let expression = match self {
            Value::Unit => HirExpression::Literal(HirLiteral::Unit),
            Value::Bool(value) => HirExpression::Literal(HirLiteral::Bool(value)),
            Value::Field(value) => HirExpression::Literal(HirLiteral::Integer(value, false)),
            Value::I8(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            }
            Value::I16(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            }
            Value::I32(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            }
            Value::I64(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            }
            Value::U1(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            }
            Value::U8(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            }
            Value::U16(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            }
            Value::U32(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            }
            Value::U64(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            }
            Value::String(value) | Value::CtString(value) => {
                HirExpression::Literal(HirLiteral::Str(unwrap_rc(value)))
            }
            // Format strings are lowered as normal strings since they are already interpolated.
            Value::FormatString(value, _) => {
                HirExpression::Literal(HirLiteral::Str(unwrap_rc(value)))
            }
            Value::Function(id, typ, bindings) => {
                let id = interner.function_definition_id(id);
                let impl_kind = ImplKind::NotATraitMethod;
                let ident = HirIdent { location, id, impl_kind };
                let expr_id = interner.push_expr(HirExpression::Ident(ident, None));
                interner.push_expr_location(expr_id, location.span, location.file);
                interner.push_expr_type(expr_id, typ);
                interner.store_instantiation_bindings(expr_id, unwrap_rc(bindings));
                return Ok(expr_id);
            }
            Value::Tuple(fields) => {
                let fields =
                    try_vecmap(fields, |field| field.into_hir_expression(interner, location))?;
                HirExpression::Tuple(fields)
            }
            Value::Struct(fields, typ) => {
                let fields = try_vecmap(fields, |(name, field)| {
                    let field = field.into_hir_expression(interner, location)?;
                    Ok((Ident::new(unwrap_rc(name), location.span), field))
                })?;

                let (r#type, struct_generics) = match typ.follow_bindings() {
                    Type::Struct(def, generics) => (def, generics),
                    _ => return Err(InterpreterError::NonStructInConstructor { typ, location }),
                };

                HirExpression::Constructor(HirConstructorExpression {
                    r#type,
                    struct_generics,
                    fields,
                })
            }
            Value::Array(elements, _) => {
                let elements = try_vecmap(elements, |element| {
                    element.into_hir_expression(interner, location)
                })?;
                HirExpression::Literal(HirLiteral::Array(HirArrayLiteral::Standard(elements)))
            }
            Value::Slice(elements, _) => {
                let elements = try_vecmap(elements, |element| {
                    element.into_hir_expression(interner, location)
                })?;
                HirExpression::Literal(HirLiteral::Slice(HirArrayLiteral::Standard(elements)))
            }
            Value::Quoted(tokens) => HirExpression::Unquote(add_token_spans(tokens, location.span)),
            Value::TypedExpr(TypedExpr::ExprId(expr_id)) => interner.expression(&expr_id),
            // Only convert pointers with auto_deref = true. These are mutable variables
            // and we don't need to wrap them in `&mut`.
            Value::Pointer(element, true) => {
                return element.unwrap_or_clone().into_hir_expression(interner, location);
            }
            Value::Closure(hir_lambda, _args, _typ, _opt_func_id, _module_id) => {
                HirExpression::Lambda(hir_lambda)
            }
            Value::TypedExpr(TypedExpr::StmtId(..))
            | Value::Expr(..)
            | Value::Pointer(..)
            | Value::StructDefinition(_)
            | Value::TraitConstraint(..)
            | Value::TraitDefinition(_)
            | Value::TraitImpl(_)
            | Value::FunctionDefinition(_)
            | Value::Zeroed(_)
            | Value::Type(_)
            | Value::UnresolvedType(_)
            | Value::ModuleDefinition(_) => {
                let typ = self.get_type().into_owned();
                let value = self.display(interner).to_string();
                return Err(InterpreterError::CannotInlineMacro { value, typ, location });
            }
        };

        let id = interner.push_expr(expression);
        interner.push_expr_location(id, location.span, location.file);
        interner.push_expr_type(id, typ);
        Ok(id)
    }

    pub(crate) fn into_tokens(
        self,
        interner: &mut NodeInterner,
        location: Location,
    ) -> IResult<Vec<Token>> {
        let token = match self {
            Value::Unit => {
                return Ok(vec![Token::LeftParen, Token::RightParen]);
            }
            Value::Quoted(tokens) => return Ok(unwrap_rc(tokens)),
            Value::Type(typ) => Token::QuotedType(interner.push_quoted_type(typ)),
            Value::Expr(ExprValue::Expression(expr)) => {
                Token::InternedExpr(interner.push_expression_kind(expr))
            }
            Value::Expr(ExprValue::Statement(StatementKind::Expression(expr))) => {
                Token::InternedExpr(interner.push_expression_kind(expr.kind))
            }
            Value::Expr(ExprValue::Statement(statement)) => {
                Token::InternedStatement(interner.push_statement_kind(statement))
            }
            Value::Expr(ExprValue::LValue(lvalue)) => {
                Token::InternedLValue(interner.push_lvalue(lvalue))
            }
            Value::Expr(ExprValue::Pattern(pattern)) => {
                Token::InternedPattern(interner.push_pattern(pattern))
            }
            Value::UnresolvedType(typ) => {
                Token::InternedUnresolvedTypeData(interner.push_unresolved_type_data(typ))
            }
            Value::U1(bool) => Token::Bool(bool),
            Value::U8(value) => Token::Int((value as u128).into()),
            Value::U16(value) => Token::Int((value as u128).into()),
            Value::U32(value) => Token::Int((value as u128).into()),
            Value::U64(value) => Token::Int((value as u128).into()),
            Value::I8(value) => {
                if value < 0 {
                    return Ok(vec![Token::Minus, Token::Int((-value as u128).into())]);
                } else {
                    Token::Int((value as u128).into())
                }
            }
            Value::I16(value) => {
                if value < 0 {
                    return Ok(vec![Token::Minus, Token::Int((-value as u128).into())]);
                } else {
                    Token::Int((value as u128).into())
                }
            }
            Value::I32(value) => {
                if value < 0 {
                    return Ok(vec![Token::Minus, Token::Int((-value as u128).into())]);
                } else {
                    Token::Int((value as u128).into())
                }
            }
            Value::I64(value) => {
                if value < 0 {
                    return Ok(vec![Token::Minus, Token::Int((-value as u128).into())]);
                } else {
                    Token::Int((value as u128).into())
                }
            }
            Value::Field(value) => Token::Int(value),
            other => Token::UnquoteMarker(other.into_hir_expression(interner, location)?),
        };
        Ok(vec![token])
    }

    /// Returns false for non-integral `Value`s.
    pub(crate) fn is_integral(&self) -> bool {
        use Value::*;
        matches!(
            self,
            Field(_) | I8(_) | I16(_) | I32(_) | I64(_) | U8(_) | U16(_) | U32(_) | U64(_)
        )
    }

    /// Converts any non-negative `Value` into a `FieldElement`.
    /// Returns `None` for negative integers and non-integral `Value`s.
    pub(crate) fn to_field_element(&self) -> Option<FieldElement> {
        match self {
            Self::Field(value) => Some(*value),
            Self::I8(value) => (*value >= 0).then_some((*value as u128).into()),
            Self::I16(value) => (*value >= 0).then_some((*value as u128).into()),
            Self::I32(value) => (*value >= 0).then_some((*value as u128).into()),
            Self::I64(value) => (*value >= 0).then_some((*value as u128).into()),
            Self::U8(value) => Some((*value as u128).into()),
            Self::U16(value) => Some((*value as u128).into()),
            Self::U32(value) => Some((*value as u128).into()),
            Self::U64(value) => Some((*value as u128).into()),
            _ => None,
        }
    }

    pub(crate) fn into_top_level_items(
        self,
        location: Location,
        elaborator: &mut Elaborator,
    ) -> IResult<Vec<Item>> {
        let parser = Parser::parse_top_level_items;
        match self {
            Value::Quoted(tokens) => {
                parse_tokens(tokens, elaborator, parser, location, "top-level item")
            }
            _ => {
                let typ = self.get_type().into_owned();
                let value = self.display(elaborator.interner).to_string();
                Err(InterpreterError::CannotInlineMacro { value, typ, location })
            }
        }
    }
}

/// Unwraps an Rc value without cloning the inner value if the reference count is 1. Clones otherwise.
pub(crate) fn unwrap_rc<T: Clone>(rc: Rc<T>) -> T {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}

fn parse_tokens<'a, T, F>(
    tokens: Rc<Vec<Token>>,
    elaborator: &mut Elaborator,
    parsing_function: F,
    location: Location,
    rule: &'static str,
) -> IResult<T>
where
    F: FnOnce(&mut Parser<'a>) -> T,
{
    let parser = Parser::for_tokens(add_token_spans(tokens.clone(), location.span));
    match parser.parse_result(parsing_function) {
        Ok((expr, warnings)) => {
            for warning in warnings {
                elaborator.errors.push((warning.into(), location.file));
            }
            Ok(expr)
        }
        Err(mut errors) => {
            let error = errors.swap_remove(0);
            let file = location.file;
            let tokens = tokens_to_string(tokens, elaborator.interner);
            Err(InterpreterError::FailedToParseMacro { error, file, tokens, rule })
        }
    }
}

pub(crate) fn add_token_spans(tokens: Rc<Vec<Token>>, span: Span) -> Tokens {
    let tokens = unwrap_rc(tokens);
    Tokens(vecmap(tokens, |token| SpannedToken::new(token, span)))
}
