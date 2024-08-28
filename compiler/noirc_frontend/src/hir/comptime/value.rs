use std::{borrow::Cow, fmt::Display, rc::Rc};

use acvm::{AcirField, FieldElement};
use chumsky::Parser;
use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::{Location, Span};
use strum_macros::Display;

use crate::{
    ast::{
        ArrayLiteral, AssignStatement, BlockExpression, CallExpression, CastExpression,
        ConstrainStatement, ConstructorExpression, ForLoopStatement, ForRange, Ident, IfExpression,
        IndexExpression, InfixExpression, IntegerBitSize, LValue, Lambda, LetStatement,
        MemberAccessExpression, MethodCallExpression, PrefixExpression, Signedness, Statement,
        StatementKind, UnresolvedTypeData,
    },
    hir::{def_map::ModuleId, type_check::generics::TraitGenerics},
    hir_def::{
        expr::{HirArrayLiteral, HirConstructorExpression, HirIdent, HirLambda, ImplKind},
        traits::TraitConstraint,
    },
    macros_api::{
        Expression, ExpressionKind, HirExpression, HirLiteral, Literal, NodeInterner, Path,
        StructId,
    },
    node_interner::{ExprId, FuncId, TraitId, TraitImplId},
    parser::{self, NoirParser, TopLevelStatement},
    token::{SpannedToken, Token, Tokens},
    QuotedType, Shared, Type, TypeBindings,
};
use rustc_hash::FxHashMap as HashMap;

use super::errors::{IResult, InterpreterError};

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
    Function(FuncId, Type, Rc<TypeBindings>),
    Closure(HirLambda, Vec<Value>, Type),
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
    UnresolvedType(UnresolvedTypeData),
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum ExprValue {
    Expression(ExpressionKind),
    Statement(StatementKind),
    LValue(LValue),
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
                let length = Type::Constant(value.len() as u32);
                Type::String(Box::new(length))
            }
            Value::FormatString(_, typ) => return Cow::Borrowed(typ),
            Value::Function(_, typ, _) => return Cow::Borrowed(typ),
            Value::Closure(_, _, typ) => return Cow::Borrowed(typ),
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
            Value::UnresolvedType(_) => Type::Quoted(QuotedType::UnresolvedType),
        })
    }

    pub(crate) fn into_expression(
        self,
        interner: &mut NodeInterner,
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
            Value::String(value) => ExpressionKind::Literal(Literal::Str(unwrap_rc(value))),
            // Format strings are lowered as normal strings since they are already interpolated.
            Value::FormatString(value, _) => {
                ExpressionKind::Literal(Literal::Str(unwrap_rc(value)))
            }
            Value::Function(id, typ, bindings) => {
                let id = interner.function_definition_id(id);
                let impl_kind = ImplKind::NotATraitMethod;
                let ident = HirIdent { location, id, impl_kind };
                let expr_id = interner.push_expr(HirExpression::Ident(ident, None));
                interner.push_expr_location(expr_id, location.span, location.file);
                interner.push_expr_type(expr_id, typ);
                interner.store_instantiation_bindings(expr_id, unwrap_rc(bindings));
                ExpressionKind::Resolved(expr_id)
            }
            Value::Closure(_lambda, _env, _typ) => {
                // TODO: How should a closure's environment be inlined?
                let item = "Returning closures from a comptime fn".into();
                return Err(InterpreterError::Unimplemented { item, location });
            }
            Value::Tuple(fields) => {
                let fields = try_vecmap(fields, |field| field.into_expression(interner, location))?;
                ExpressionKind::Tuple(fields)
            }
            Value::Struct(fields, typ) => {
                let fields = try_vecmap(fields, |(name, field)| {
                    let field = field.into_expression(interner, location)?;
                    Ok((Ident::new(unwrap_rc(name), location.span), field))
                })?;

                let struct_type = match typ.follow_bindings() {
                    Type::Struct(def, _) => Some(def.borrow().id),
                    _ => return Err(InterpreterError::NonStructInConstructor { typ, location }),
                };

                // Since we've provided the struct_type, the path should be ignored.
                let type_name = Path::from_single(String::new(), location.span);
                ExpressionKind::Constructor(Box::new(ConstructorExpression {
                    type_name,
                    fields,
                    struct_type,
                }))
            }
            Value::Array(elements, _) => {
                let elements =
                    try_vecmap(elements, |element| element.into_expression(interner, location))?;
                ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(elements)))
            }
            Value::Slice(elements, _) => {
                let elements =
                    try_vecmap(elements, |element| element.into_expression(interner, location))?;
                ExpressionKind::Literal(Literal::Slice(ArrayLiteral::Standard(elements)))
            }
            Value::Quoted(tokens) => {
                // Wrap the tokens in '{' and '}' so that we can parse statements as well.
                let mut tokens_to_parse = add_token_spans(tokens.clone(), location.span);
                tokens_to_parse.0.insert(0, SpannedToken::new(Token::LeftBrace, location.span));
                tokens_to_parse.0.push(SpannedToken::new(Token::RightBrace, location.span));

                return match parser::expression().parse(tokens_to_parse) {
                    Ok(expr) => Ok(expr),
                    Err(mut errors) => {
                        let error = errors.swap_remove(0);
                        let file = location.file;
                        let rule = "an expression";
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
            Value::Expr(ExprValue::LValue(_))
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
            Value::String(value) => HirExpression::Literal(HirLiteral::Str(unwrap_rc(value))),
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
            Value::Closure(_lambda, _env, _typ) => {
                // TODO: How should a closure's environment be inlined?
                let item = "Returning closures from a comptime fn".into();
                return Err(InterpreterError::Unimplemented { item, location });
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
            Value::Expr(..)
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
            Value::Quoted(tokens) => return Ok(unwrap_rc(tokens)),
            Value::Type(typ) => Token::QuotedType(interner.push_quoted_type(typ)),
            Value::Expr(ExprValue::Expression(expr)) => {
                Token::InternedExpr(interner.push_expression_kind(expr))
            }
            Value::Expr(ExprValue::Statement(statement)) => {
                Token::InternedStatement(interner.push_statement_kind(statement))
            }
            Value::Expr(ExprValue::LValue(lvalue)) => {
                Token::InternedLValue(interner.push_lvalue(lvalue))
            }
            Value::UnresolvedType(typ) => {
                Token::InternedUnresolvedTypeData(interner.push_unresolved_type_data(typ))
            }
            other => Token::UnquoteMarker(other.into_hir_expression(interner, location)?),
        };
        Ok(vec![token])
    }

    /// Converts any unsigned `Value` into a `u128`.
    /// Returns `None` for negative integers.
    pub(crate) fn to_u128(&self) -> Option<u128> {
        match self {
            Self::Field(value) => Some(value.to_u128()),
            Self::I8(value) => (*value >= 0).then_some(*value as u128),
            Self::I16(value) => (*value >= 0).then_some(*value as u128),
            Self::I32(value) => (*value >= 0).then_some(*value as u128),
            Self::I64(value) => (*value >= 0).then_some(*value as u128),
            Self::U8(value) => Some(*value as u128),
            Self::U16(value) => Some(*value as u128),
            Self::U32(value) => Some(*value as u128),
            Self::U64(value) => Some(*value as u128),
            _ => None,
        }
    }

    pub(crate) fn into_top_level_items(
        self,
        location: Location,
        interner: &NodeInterner,
    ) -> IResult<Vec<TopLevelStatement>> {
        match self {
            Value::Quoted(tokens) => parse_tokens(tokens, parser::top_level_items(), location),
            _ => {
                let typ = self.get_type().into_owned();
                let value = self.display(interner).to_string();
                Err(InterpreterError::CannotInlineMacro { value, typ, location })
            }
        }
    }

    pub fn display<'value, 'interner>(
        &'value self,
        interner: &'interner NodeInterner,
    ) -> ValuePrinter<'value, 'interner> {
        ValuePrinter { value: self, interner }
    }
}

/// Unwraps an Rc value without cloning the inner value if the reference count is 1. Clones otherwise.
pub(crate) fn unwrap_rc<T: Clone>(rc: Rc<T>) -> T {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}

fn parse_tokens<T>(
    tokens: Rc<Vec<Token>>,
    parser: impl NoirParser<T>,
    location: Location,
) -> IResult<T> {
    match parser.parse(add_token_spans(tokens.clone(), location.span)) {
        Ok(expr) => Ok(expr),
        Err(mut errors) => {
            let error = errors.swap_remove(0);
            let rule = "an expression";
            let file = location.file;
            Err(InterpreterError::FailedToParseMacro { error, file, tokens, rule })
        }
    }
}

pub(crate) fn add_token_spans(tokens: Rc<Vec<Token>>, span: Span) -> Tokens {
    let tokens = unwrap_rc(tokens);
    Tokens(vecmap(tokens, |token| SpannedToken::new(token, span)))
}

pub struct ValuePrinter<'value, 'interner> {
    value: &'value Value,
    interner: &'interner NodeInterner,
}

impl<'value, 'interner> Display for ValuePrinter<'value, 'interner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Value::Unit => write!(f, "()"),
            Value::Bool(value) => {
                let msg = if *value { "true" } else { "false" };
                write!(f, "{msg}")
            }
            Value::Field(value) => write!(f, "{value}"),
            Value::I8(value) => write!(f, "{value}"),
            Value::I16(value) => write!(f, "{value}"),
            Value::I32(value) => write!(f, "{value}"),
            Value::I64(value) => write!(f, "{value}"),
            Value::U1(value) => write!(f, "{value}"),
            Value::U8(value) => write!(f, "{value}"),
            Value::U16(value) => write!(f, "{value}"),
            Value::U32(value) => write!(f, "{value}"),
            Value::U64(value) => write!(f, "{value}"),
            Value::String(value) => write!(f, "{value}"),
            Value::FormatString(value, _) => write!(f, "{value}"),
            Value::Function(..) => write!(f, "(function)"),
            Value::Closure(_, _, _) => write!(f, "(closure)"),
            Value::Tuple(fields) => {
                let fields = vecmap(fields, |field| field.display(self.interner).to_string());
                write!(f, "({})", fields.join(", "))
            }
            Value::Struct(fields, typ) => {
                let typename = match typ.follow_bindings() {
                    Type::Struct(def, _) => def.borrow().name.to_string(),
                    other => other.to_string(),
                };
                let fields = vecmap(fields, |(name, value)| {
                    format!("{}: {}", name, value.display(self.interner))
                });
                write!(f, "{typename} {{ {} }}", fields.join(", "))
            }
            Value::Pointer(value, _) => write!(f, "&mut {}", value.borrow().display(self.interner)),
            Value::Array(values, _) => {
                let values = vecmap(values, |value| value.display(self.interner).to_string());
                write!(f, "[{}]", values.join(", "))
            }
            Value::Slice(values, _) => {
                let values = vecmap(values, |value| value.display(self.interner).to_string());
                write!(f, "&[{}]", values.join(", "))
            }
            Value::Quoted(tokens) => {
                write!(f, "quote {{")?;
                for token in tokens.iter() {
                    match token {
                        Token::QuotedType(id) => {
                            write!(f, " {}", self.interner.get_quoted_type(*id))?;
                        }
                        other => write!(f, " {other}")?,
                    }
                }
                write!(f, " }}")
            }
            Value::StructDefinition(id) => {
                let def = self.interner.get_struct(*id);
                let def = def.borrow();
                write!(f, "{}", def.name)
            }
            Value::TraitConstraint(trait_id, generics) => {
                let trait_ = self.interner.get_trait(*trait_id);
                write!(f, "{}{generics}", trait_.name)
            }
            Value::TraitDefinition(trait_id) => {
                let trait_ = self.interner.get_trait(*trait_id);
                write!(f, "{}", trait_.name)
            }
            Value::TraitImpl(trait_impl_id) => {
                let trait_impl = self.interner.get_trait_implementation(*trait_impl_id);
                let trait_impl = trait_impl.borrow();

                let generic_string =
                    vecmap(&trait_impl.trait_generics, ToString::to_string).join(", ");
                let generic_string = if generic_string.is_empty() {
                    generic_string
                } else {
                    format!("<{}>", generic_string)
                };

                let where_clause = vecmap(&trait_impl.where_clause, |trait_constraint| {
                    display_trait_constraint(self.interner, trait_constraint)
                });
                let where_clause = where_clause.join(", ");
                let where_clause = if where_clause.is_empty() {
                    where_clause
                } else {
                    format!(" where {}", where_clause)
                };

                write!(
                    f,
                    "impl {}{} for {}{}",
                    trait_impl.ident, generic_string, trait_impl.typ, where_clause
                )
            }
            Value::FunctionDefinition(function_id) => {
                write!(f, "{}", self.interner.function_name(function_id))
            }
            Value::ModuleDefinition(_) => write!(f, "(module)"),
            Value::Zeroed(typ) => write!(f, "(zeroed {typ})"),
            Value::Type(typ) => write!(f, "{}", typ),
            Value::Expr(ExprValue::Expression(expr)) => {
                write!(f, "{}", remove_interned_in_expression_kind(self.interner, expr.clone()))
            }
            Value::Expr(ExprValue::Statement(statement)) => {
                write!(f, "{}", remove_interned_in_statement_kind(self.interner, statement.clone()))
            }
            Value::Expr(ExprValue::LValue(lvalue)) => {
                write!(f, "{}", remove_interned_in_lvalue(self.interner, lvalue.clone()))
            }
            Value::UnresolvedType(typ) => {
                if let UnresolvedTypeData::Interned(id) = typ {
                    let typ = self.interner.get_unresolved_type_data(*id);
                    write!(f, "{}", typ)
                } else {
                    write!(f, "{}", typ)
                }
            }
        }
    }
}

fn display_trait_constraint(interner: &NodeInterner, trait_constraint: &TraitConstraint) -> String {
    let trait_ = interner.get_trait(trait_constraint.trait_id);
    format!("{}: {}{}", trait_constraint.typ, trait_.name, trait_constraint.trait_generics)
}

// Returns a new Expression where all Interned and Resolved expressions have been turned into non-interned ExpressionKind.
fn remove_interned_in_expression(interner: &NodeInterner, expr: Expression) -> Expression {
    Expression { kind: remove_interned_in_expression_kind(interner, expr.kind), span: expr.span }
}

// Returns a new ExpressionKind where all Interned and Resolved expressions have been turned into non-interned ExpressionKind.
fn remove_interned_in_expression_kind(
    interner: &NodeInterner,
    expr: ExpressionKind,
) -> ExpressionKind {
    match expr {
        ExpressionKind::Literal(literal) => {
            ExpressionKind::Literal(remove_interned_in_literal(interner, literal))
        }
        ExpressionKind::Block(block) => {
            let statements =
                vecmap(block.statements, |stmt| remove_interned_in_statement(interner, stmt));
            ExpressionKind::Block(BlockExpression { statements })
        }
        ExpressionKind::Prefix(prefix) => ExpressionKind::Prefix(Box::new(PrefixExpression {
            rhs: remove_interned_in_expression(interner, prefix.rhs),
            ..*prefix
        })),
        ExpressionKind::Index(index) => ExpressionKind::Index(Box::new(IndexExpression {
            collection: remove_interned_in_expression(interner, index.collection),
            index: remove_interned_in_expression(interner, index.index),
        })),
        ExpressionKind::Call(call) => ExpressionKind::Call(Box::new(CallExpression {
            func: Box::new(remove_interned_in_expression(interner, *call.func)),
            arguments: vecmap(call.arguments, |arg| remove_interned_in_expression(interner, arg)),
            ..*call
        })),
        ExpressionKind::MethodCall(call) => {
            ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                object: remove_interned_in_expression(interner, call.object),
                arguments: vecmap(call.arguments, |arg| {
                    remove_interned_in_expression(interner, arg)
                }),
                ..*call
            }))
        }
        ExpressionKind::Constructor(constructor) => {
            ExpressionKind::Constructor(Box::new(ConstructorExpression {
                fields: vecmap(constructor.fields, |(name, expr)| {
                    (name, remove_interned_in_expression(interner, expr))
                }),
                ..*constructor
            }))
        }
        ExpressionKind::MemberAccess(member_access) => {
            ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
                lhs: remove_interned_in_expression(interner, member_access.lhs),
                ..*member_access
            }))
        }
        ExpressionKind::Cast(cast) => ExpressionKind::Cast(Box::new(CastExpression {
            lhs: remove_interned_in_expression(interner, cast.lhs),
            ..*cast
        })),
        ExpressionKind::Infix(infix) => ExpressionKind::Infix(Box::new(InfixExpression {
            lhs: remove_interned_in_expression(interner, infix.lhs),
            rhs: remove_interned_in_expression(interner, infix.rhs),
            ..*infix
        })),
        ExpressionKind::If(if_expr) => ExpressionKind::If(Box::new(IfExpression {
            condition: remove_interned_in_expression(interner, if_expr.condition),
            consequence: remove_interned_in_expression(interner, if_expr.consequence),
            alternative: if_expr
                .alternative
                .map(|alternative| remove_interned_in_expression(interner, alternative)),
        })),
        ExpressionKind::Variable(_) => expr,
        ExpressionKind::Tuple(expressions) => ExpressionKind::Tuple(vecmap(expressions, |expr| {
            remove_interned_in_expression(interner, expr)
        })),
        ExpressionKind::Lambda(lambda) => ExpressionKind::Lambda(Box::new(Lambda {
            body: remove_interned_in_expression(interner, lambda.body),
            ..*lambda
        })),
        ExpressionKind::Parenthesized(expr) => {
            ExpressionKind::Parenthesized(Box::new(remove_interned_in_expression(interner, *expr)))
        }
        ExpressionKind::Quote(_) => expr,
        ExpressionKind::Unquote(expr) => {
            ExpressionKind::Unquote(Box::new(remove_interned_in_expression(interner, *expr)))
        }
        ExpressionKind::Comptime(block, span) => {
            let statements =
                vecmap(block.statements, |stmt| remove_interned_in_statement(interner, stmt));
            ExpressionKind::Comptime(BlockExpression { statements }, span)
        }
        ExpressionKind::Unsafe(block, span) => {
            let statements =
                vecmap(block.statements, |stmt| remove_interned_in_statement(interner, stmt));
            ExpressionKind::Unsafe(BlockExpression { statements }, span)
        }
        ExpressionKind::AsTraitPath(_) => expr,
        ExpressionKind::Resolved(id) => {
            let expr = interner.expression(&id);
            expr.to_display_ast(interner, Span::default()).kind
        }
        ExpressionKind::Interned(id) => {
            let expr = interner.get_expression_kind(id).clone();
            remove_interned_in_expression_kind(interner, expr)
        }
        ExpressionKind::Error => expr,
    }
}

fn remove_interned_in_literal(interner: &NodeInterner, literal: Literal) -> Literal {
    match literal {
        Literal::Array(array_literal) => {
            Literal::Array(remove_interned_in_array_literal(interner, array_literal))
        }
        Literal::Slice(array_literal) => {
            Literal::Array(remove_interned_in_array_literal(interner, array_literal))
        }
        Literal::Bool(_)
        | Literal::Integer(_, _)
        | Literal::Str(_)
        | Literal::RawStr(_, _)
        | Literal::FmtStr(_)
        | Literal::Unit => literal,
    }
}

fn remove_interned_in_array_literal(
    interner: &NodeInterner,
    literal: ArrayLiteral,
) -> ArrayLiteral {
    match literal {
        ArrayLiteral::Standard(expressions) => {
            ArrayLiteral::Standard(vecmap(expressions, |expr| {
                remove_interned_in_expression(interner, expr)
            }))
        }
        ArrayLiteral::Repeated { repeated_element, length } => ArrayLiteral::Repeated {
            repeated_element: Box::new(remove_interned_in_expression(interner, *repeated_element)),
            length: Box::new(remove_interned_in_expression(interner, *length)),
        },
    }
}

// Returns a new Statement where all Interned statements have been turned into non-interned StatementKind.
fn remove_interned_in_statement(interner: &NodeInterner, statement: Statement) -> Statement {
    Statement {
        kind: remove_interned_in_statement_kind(interner, statement.kind),
        span: statement.span,
    }
}

// Returns a new StatementKind where all Interned statements have been turned into non-interned StatementKind.
fn remove_interned_in_statement_kind(
    interner: &NodeInterner,
    statement: StatementKind,
) -> StatementKind {
    match statement {
        StatementKind::Let(let_statement) => StatementKind::Let(LetStatement {
            expression: remove_interned_in_expression(interner, let_statement.expression),
            ..let_statement
        }),
        StatementKind::Constrain(constrain) => StatementKind::Constrain(ConstrainStatement(
            remove_interned_in_expression(interner, constrain.0),
            constrain.1.map(|expr| remove_interned_in_expression(interner, expr)),
            constrain.2,
        )),
        StatementKind::Expression(expr) => {
            StatementKind::Expression(remove_interned_in_expression(interner, expr))
        }
        StatementKind::Assign(assign) => StatementKind::Assign(AssignStatement {
            lvalue: assign.lvalue,
            expression: remove_interned_in_expression(interner, assign.expression),
        }),
        StatementKind::For(for_loop) => StatementKind::For(ForLoopStatement {
            range: match for_loop.range {
                ForRange::Range(from, to) => ForRange::Range(
                    remove_interned_in_expression(interner, from),
                    remove_interned_in_expression(interner, to),
                ),
                ForRange::Array(expr) => {
                    ForRange::Array(remove_interned_in_expression(interner, expr))
                }
            },
            block: remove_interned_in_expression(interner, for_loop.block),
            ..for_loop
        }),
        StatementKind::Comptime(statement) => {
            StatementKind::Comptime(Box::new(remove_interned_in_statement(interner, *statement)))
        }
        StatementKind::Semi(expr) => {
            StatementKind::Semi(remove_interned_in_expression(interner, expr))
        }
        StatementKind::Interned(id) => {
            let statement = interner.get_statement_kind(id).clone();
            remove_interned_in_statement_kind(interner, statement)
        }
        StatementKind::Break | StatementKind::Continue | StatementKind::Error => statement,
    }
}

// Returns a new LValue where all Interned LValues have been turned into LValue.
fn remove_interned_in_lvalue(interner: &NodeInterner, lvalue: LValue) -> LValue {
    match lvalue {
        LValue::Ident(_) => lvalue,
        LValue::MemberAccess { object, field_name, span } => LValue::MemberAccess {
            object: Box::new(remove_interned_in_lvalue(interner, *object)),
            field_name,
            span,
        },
        LValue::Index { array, index, span } => LValue::Index {
            array: Box::new(remove_interned_in_lvalue(interner, *array)),
            index: remove_interned_in_expression(interner, index),
            span,
        },
        LValue::Dereference(lvalue, span) => {
            LValue::Dereference(Box::new(remove_interned_in_lvalue(interner, *lvalue)), span)
        }
        LValue::Interned(id, span) => {
            let lvalue = interner.get_lvalue(id, span);
            remove_interned_in_lvalue(interner, lvalue)
        }
    }
}
