use std::{borrow::Cow, fmt::Display, rc::Rc};

use acvm::{AcirField, FieldElement};
use chumsky::Parser;
use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;

use crate::{
    ast::{ArrayLiteral, ConstructorExpression, Ident, IntegerBitSize, Signedness},
    hir_def::expr::{HirArrayLiteral, HirConstructorExpression, HirIdent, HirLambda, ImplKind},
    macros_api::{
        Expression, ExpressionKind, HirExpression, HirLiteral, Literal, NodeInterner, Path,
        StructId,
    },
    node_interner::{ExprId, FuncId},
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
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    String(Rc<String>),
    Function(FuncId, Type, Rc<TypeBindings>),
    Closure(HirLambda, Vec<Value>, Type),
    Tuple(Vec<Value>),
    Struct(HashMap<Rc<String>, Value>, Type),
    Pointer(Shared<Value>),
    Array(Vector<Value>, Type),
    Slice(Vector<Value>, Type),
    Code(Rc<Tokens>),
    StructDefinition(StructId),
}

impl Value {
    pub(crate) fn get_type(&self) -> Cow<Type> {
        Cow::Owned(match self {
            Value::Unit => Type::Unit,
            Value::Bool(_) => Type::Bool,
            Value::Field(_) => Type::FieldElement,
            Value::I8(_) => Type::Integer(Signedness::Signed, IntegerBitSize::Eight),
            Value::I16(_) => Type::Integer(Signedness::Signed, IntegerBitSize::Sixteen),
            Value::I32(_) => Type::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo),
            Value::I64(_) => Type::Integer(Signedness::Signed, IntegerBitSize::SixtyFour),
            Value::U8(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight),
            Value::U16(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::Sixteen),
            Value::U32(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo),
            Value::U64(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour),
            Value::String(value) => {
                let length = Type::Constant(value.len() as u32);
                Type::String(Box::new(length))
            }
            Value::Function(_, typ, _) => return Cow::Borrowed(typ),
            Value::Closure(_, _, typ) => return Cow::Borrowed(typ),
            Value::Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| field.get_type().into_owned()))
            }
            Value::Struct(_, typ) => return Cow::Borrowed(typ),
            Value::Array(_, typ) => return Cow::Borrowed(typ),
            Value::Slice(_, typ) => return Cow::Borrowed(typ),
            Value::Code(_) => Type::Quoted(QuotedType::Quoted),
            Value::StructDefinition(_) => Type::Quoted(QuotedType::StructDefinition),
            Value::Pointer(element) => {
                let element = element.borrow().get_type().into_owned();
                Type::MutableReference(Box::new(element))
            }
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
            Value::Code(tokens) => {
                // Wrap the tokens in '{' and '}' so that we can parse statements as well.
                let mut tokens_to_parse = tokens.as_ref().clone();
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
            Value::Pointer(_) | Value::StructDefinition(_) => {
                return Err(InterpreterError::CannotInlineMacro { value: self, location })
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
            Value::Code(block) => HirExpression::Unquote(unwrap_rc(block)),
            Value::Pointer(_) | Value::StructDefinition(_) => {
                return Err(InterpreterError::CannotInlineMacro { value: self, location })
            }
        };

        let id = interner.push_expr(expression);
        interner.push_expr_location(id, location.span, location.file);
        interner.push_expr_type(id, typ);
        Ok(id)
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
    ) -> IResult<Vec<TopLevelStatement>> {
        match self {
            Value::Code(tokens) => parse_tokens(tokens, parser::top_level_items(), location.file),
            value => Err(InterpreterError::CannotInlineMacro { value, location }),
        }
    }
}

/// Unwraps an Rc value without cloning the inner value if the reference count is 1. Clones otherwise.
pub(crate) fn unwrap_rc<T: Clone>(rc: Rc<T>) -> T {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}

fn parse_tokens<T>(tokens: Rc<Tokens>, parser: impl NoirParser<T>, file: fm::FileId) -> IResult<T> {
    match parser.parse(tokens.as_ref().clone()) {
        Ok(expr) => Ok(expr),
        Err(mut errors) => {
            let error = errors.swap_remove(0);
            let rule = "an expression";
            Err(InterpreterError::FailedToParseMacro { error, file, tokens, rule })
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
            Value::U8(value) => write!(f, "{value}"),
            Value::U16(value) => write!(f, "{value}"),
            Value::U32(value) => write!(f, "{value}"),
            Value::U64(value) => write!(f, "{value}"),
            Value::String(value) => write!(f, "{value}"),
            Value::Function(..) => write!(f, "(function)"),
            Value::Closure(_, _, _) => write!(f, "(closure)"),
            Value::Tuple(fields) => {
                let fields = vecmap(fields, ToString::to_string);
                write!(f, "({})", fields.join(", "))
            }
            Value::Struct(fields, typ) => {
                let typename = match typ.follow_bindings() {
                    Type::Struct(def, _) => def.borrow().name.to_string(),
                    other => other.to_string(),
                };
                let fields = vecmap(fields, |(name, value)| format!("{}: {}", name, value));
                write!(f, "{typename} {{ {} }}", fields.join(", "))
            }
            Value::Pointer(value) => write!(f, "&mut {}", value.borrow()),
            Value::Array(values, _) => {
                let values = vecmap(values, ToString::to_string);
                write!(f, "[{}]", values.join(", "))
            }
            Value::Slice(values, _) => {
                let values = vecmap(values, ToString::to_string);
                write!(f, "&[{}]", values.join(", "))
            }
            Value::Code(tokens) => {
                write!(f, "quote {{")?;
                for token in tokens.0.iter() {
                    write!(f, " {token}")?;
                }
                write!(f, " }}")
            }
            Value::StructDefinition(_) => write!(f, "(struct definition)"),
        }
    }
}
