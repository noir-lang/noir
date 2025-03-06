use std::{borrow::Cow, rc::Rc, vec};

use acvm::FieldElement;
use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;
use strum_macros::Display;

use crate::{
    Kind, QuotedType, Shared, Type, TypeBindings,
    ast::{
        ArrayLiteral, BlockExpression, ConstructorExpression, Expression, ExpressionKind, Ident,
        IntegerBitSize, LValue, Literal, Pattern, Signedness, Statement, StatementKind,
        UnresolvedType, UnresolvedTypeData,
    },
    elaborator::Elaborator,
    hir::{
        def_collector::dc_crate::CompilationError, def_map::ModuleId,
        type_check::generics::TraitGenerics,
    },
    hir_def::expr::{
        HirArrayLiteral, HirConstructorExpression, HirEnumConstructorExpression, HirExpression,
        HirIdent, HirLambda, HirLiteral, ImplKind,
    },
    node_interner::{ExprId, FuncId, NodeInterner, StmtId, TraitId, TraitImplId, TypeId},
    parser::{Item, Parser},
    signed_field::SignedField,
    token::{LocatedToken, Token, Tokens},
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
    U128(u128),
    String(Rc<String>),
    FormatString(Rc<String>, Type),
    CtString(Rc<String>),
    Function(FuncId, Type, Rc<TypeBindings>),

    // Closures also store their original scope (function & module)
    // in case they use functions such as `Quoted::as_type` which require them.
    Closure(Box<Closure>),

    Tuple(Vec<Value>),
    Struct(HashMap<Rc<String>, Value>, Type),
    Enum(/*tag*/ usize, /*args*/ Vec<Value>, Type),
    Pointer(Shared<Value>, /* auto_deref */ bool, /* mutable */ bool),
    Array(Vector<Value>, Type),
    Slice(Vector<Value>, Type),
    Quoted(Rc<Vec<LocatedToken>>),
    StructDefinition(TypeId),
    TraitConstraint(TraitId, TraitGenerics),
    TraitDefinition(TraitId),
    TraitImpl(TraitImplId),
    FunctionDefinition(FuncId),
    ModuleDefinition(ModuleId),
    Type(Type),
    Zeroed(Type),
    Expr(Box<ExprValue>),
    TypedExpr(TypedExpr),
    UnresolvedType(UnresolvedTypeData),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Closure {
    pub lambda: HirLambda,
    pub env: Vec<Value>,
    pub typ: Type,
    pub function_scope: Option<FuncId>,
    pub module_scope: ModuleId,
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
        Value::Expr(Box::new(ExprValue::Expression(expr)))
    }

    pub(crate) fn statement(statement: StatementKind) -> Self {
        Value::Expr(Box::new(ExprValue::Statement(statement)))
    }

    pub(crate) fn lvalue(lvaue: LValue) -> Self {
        Value::Expr(Box::new(ExprValue::LValue(lvaue)))
    }

    pub(crate) fn pattern(pattern: Pattern) -> Self {
        Value::Expr(Box::new(ExprValue::Pattern(pattern)))
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
            Value::U128(_) => {
                Type::Integer(Signedness::Unsigned, IntegerBitSize::HundredTwentyEight)
            }
            Value::String(value) => {
                let length = Type::Constant(value.len().into(), Kind::u32());
                Type::String(Box::new(length))
            }
            Value::FormatString(_, typ) => return Cow::Borrowed(typ),
            Value::Function(_, typ, _) => return Cow::Borrowed(typ),
            Value::Closure(closure) => return Cow::Borrowed(&closure.typ),
            Value::Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| field.get_type().into_owned()))
            }
            Value::Struct(_, typ) => return Cow::Borrowed(typ),
            Value::Enum(_, _, typ) => return Cow::Borrowed(typ),
            Value::Array(_, typ) => return Cow::Borrowed(typ),
            Value::Slice(_, typ) => return Cow::Borrowed(typ),
            Value::Quoted(_) => Type::Quoted(QuotedType::Quoted),
            Value::StructDefinition(_) => Type::Quoted(QuotedType::StructDefinition),
            Value::Pointer(element, auto_deref, mutable) => {
                if *auto_deref {
                    element.borrow().get_type().into_owned()
                } else {
                    let element = element.borrow().get_type().into_owned();
                    Type::Reference(Box::new(element), *mutable)
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
            Value::Field(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::positive(value)))
            }
            Value::I8(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::from_signed(value)))
            }
            Value::I16(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::from_signed(value)))
            }
            Value::I32(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::from_signed(value)))
            }
            Value::I64(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::from_signed(value)))
            }
            Value::U1(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::positive(value)))
            }
            Value::U8(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::positive(value as u128)))
            }
            Value::U16(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::positive(value as u128)))
            }
            Value::U32(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::positive(value)))
            }
            Value::U64(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::positive(value)))
            }
            Value::U128(value) => {
                ExpressionKind::Literal(Literal::Integer(SignedField::positive(value)))
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
                elaborator.interner.push_expr_location(expr_id, location);
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
                    Ok((Ident::new(unwrap_rc(name), location), field))
                })?;

                let typ = match typ.follow_bindings_shallow().as_ref() {
                    Type::DataType(data_type, generics) => {
                        Type::DataType(data_type.clone(), generics.clone())
                    }
                    _ => return Err(InterpreterError::NonStructInConstructor { typ, location }),
                };

                let quoted_type_id = elaborator.interner.push_quoted_type(typ);

                let typ = UnresolvedTypeData::Resolved(quoted_type_id);
                let typ = UnresolvedType { typ, location };
                ExpressionKind::Constructor(Box::new(ConstructorExpression { typ, fields }))
            }
            value @ Value::Enum(..) => {
                let hir = value.into_hir_expression(elaborator.interner, location)?;
                ExpressionKind::Resolved(hir)
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
                let mut tokens_to_parse = unwrap_rc(tokens.clone());
                tokens_to_parse.insert(0, LocatedToken::new(Token::LeftBrace, location));
                tokens_to_parse.push(LocatedToken::new(Token::RightBrace, location));

                let tokens_to_parse = Tokens(tokens_to_parse);

                let parser = Parser::for_tokens(tokens_to_parse);
                return match parser.parse_result(Parser::parse_expression_or_error) {
                    Ok((expr, warnings)) => {
                        for warning in warnings {
                            let warning: CompilationError = warning.into();
                            elaborator.push_err(warning);
                        }

                        Ok(expr)
                    }
                    Err(mut errors) => {
                        let error = Box::new(errors.swap_remove(0));
                        let rule = "an expression";
                        let tokens = tokens_to_string(&tokens, elaborator.interner);
                        Err(InterpreterError::FailedToParseMacro { error, tokens, rule, location })
                    }
                };
            }
            Value::Expr(ref expr) => {
                // We need to do some shenanigans to get around the borrow checker here due to using a boxed value.

                // We first do whatever needs a reference to `expr` to avoid partially moving `self`.
                if matches!(expr.as_ref(), ExprValue::Pattern(_)) {
                    let typ = Type::Quoted(QuotedType::Expr);
                    let value = self.display(elaborator.interner).to_string();
                    return Err(InterpreterError::CannotInlineMacro { typ, value, location });
                }

                // Now drop this references and move `expr` out of `self` so we don't have to clone it.
                let Value::Expr(expr) = self else {
                    unreachable!("Ensured by outer match statement")
                };

                match *expr {
                    ExprValue::Expression(expr) => expr,
                    ExprValue::Statement(statement) => ExpressionKind::Block(BlockExpression {
                        statements: vec![Statement { kind: statement, location }],
                    }),
                    ExprValue::LValue(lvalue) => lvalue.as_expression().kind,
                    ExprValue::Pattern(_) => unreachable!("this case is handled above"),
                }
            }
            Value::TypedExpr(..)
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

        Ok(Expression::new(kind, location))
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
            Value::Field(value) => HirExpression::Literal(HirLiteral::Integer(value.into())),
            Value::I8(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::from_signed(value)))
            }
            Value::I16(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::from_signed(value)))
            }
            Value::I32(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::from_signed(value)))
            }
            Value::I64(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::from_signed(value)))
            }
            Value::U1(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::positive(value)))
            }
            Value::U8(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::positive(value as u128)))
            }
            Value::U16(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::positive(value as u128)))
            }
            Value::U32(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::positive(value)))
            }
            Value::U64(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::positive(value)))
            }
            Value::U128(value) => {
                HirExpression::Literal(HirLiteral::Integer(SignedField::positive(value)))
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
                interner.push_expr_location(expr_id, location);
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
                    Ok((Ident::new(unwrap_rc(name), location), field))
                })?;

                let (r#type, struct_generics) = match typ.follow_bindings() {
                    Type::DataType(def, generics) => (def, generics),
                    _ => return Err(InterpreterError::NonStructInConstructor { typ, location }),
                };

                HirExpression::Constructor(HirConstructorExpression {
                    r#type,
                    struct_generics,
                    fields,
                })
            }
            Value::Enum(variant_index, args, typ) => {
                // Enum constants can have generic types but aren't functions
                let r#type = match typ.unwrap_forall().1.follow_bindings() {
                    Type::DataType(def, _) => def,
                    _ => return Err(InterpreterError::NonEnumInConstructor { typ, location }),
                };

                let arguments =
                    try_vecmap(args, |arg| arg.into_hir_expression(interner, location))?;

                HirExpression::EnumConstructor(HirEnumConstructorExpression {
                    r#type,
                    variant_index,
                    arguments,
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
            Value::Quoted(tokens) => HirExpression::Unquote(Tokens(unwrap_rc(tokens))),
            Value::TypedExpr(TypedExpr::ExprId(expr_id)) => interner.expression(&expr_id),
            // Only convert pointers with auto_deref = true. These are mutable variables
            // and we don't need to wrap them in `&mut`.
            Value::Pointer(element, true, _) => {
                return element.unwrap_or_clone().into_hir_expression(interner, location);
            }
            Value::Closure(closure) => HirExpression::Lambda(closure.lambda.clone()),
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
        interner.push_expr_location(id, location);
        interner.push_expr_type(id, typ);
        Ok(id)
    }

    pub(crate) fn into_tokens(
        self,
        interner: &mut NodeInterner,
        location: Location,
    ) -> IResult<Vec<LocatedToken>> {
        let tokens: Vec<Token> = match self {
            Value::Unit => {
                vec![Token::LeftParen, Token::RightParen]
            }
            Value::Quoted(tokens) => return Ok(unwrap_rc(tokens)),
            Value::Type(typ) => vec![Token::QuotedType(interner.push_quoted_type(typ))],
            Value::Expr(expr) => match *expr {
                ExprValue::Expression(expr) => {
                    vec![Token::InternedExpr(interner.push_expression_kind(expr))]
                }
                ExprValue::Statement(StatementKind::Expression(expr)) => {
                    vec![Token::InternedExpr(interner.push_expression_kind(expr.kind))]
                }
                ExprValue::Statement(statement) => {
                    vec![Token::InternedStatement(interner.push_statement_kind(statement))]
                }
                ExprValue::LValue(lvalue) => {
                    vec![Token::InternedLValue(interner.push_lvalue(lvalue))]
                }
                ExprValue::Pattern(pattern) => {
                    vec![Token::InternedPattern(interner.push_pattern(pattern))]
                }
            },
            Value::UnresolvedType(typ) => {
                vec![Token::InternedUnresolvedTypeData(interner.push_unresolved_type_data(typ))]
            }
            Value::TraitConstraint(trait_id, generics) => {
                let name = Rc::new(interner.get_trait(trait_id).name.0.contents.clone());
                let typ = Type::TraitAsType(trait_id, name, generics);
                vec![Token::QuotedType(interner.push_quoted_type(typ))]
            }
            Value::TypedExpr(TypedExpr::ExprId(expr_id)) => vec![Token::UnquoteMarker(expr_id)],
            Value::U1(bool) => vec![Token::Bool(bool)],
            Value::U8(value) => vec![Token::Int((value as u128).into())],
            Value::U16(value) => vec![Token::Int((value as u128).into())],
            Value::U32(value) => vec![Token::Int((value as u128).into())],
            Value::U64(value) => vec![Token::Int((value as u128).into())],
            Value::I8(value) => {
                if value < 0 {
                    vec![Token::Minus, Token::Int((-value as u128).into())]
                } else {
                    vec![Token::Int((value as u128).into())]
                }
            }
            Value::I16(value) => {
                if value < 0 {
                    vec![Token::Minus, Token::Int((-value as u128).into())]
                } else {
                    vec![Token::Int((value as u128).into())]
                }
            }
            Value::I32(value) => {
                if value < 0 {
                    vec![Token::Minus, Token::Int((-value as u128).into())]
                } else {
                    vec![Token::Int((value as u128).into())]
                }
            }
            Value::I64(value) => {
                if value < 0 {
                    vec![Token::Minus, Token::Int((-value as u128).into())]
                } else {
                    vec![Token::Int((value as u128).into())]
                }
            }
            Value::Field(value) => vec![Token::Int(value)],
            other => vec![Token::UnquoteMarker(other.into_hir_expression(interner, location)?)],
        };
        let tokens = vecmap(tokens, |token| LocatedToken::new(token, location));
        Ok(tokens)
    }

    /// Returns false for non-integral `Value`s.
    pub(crate) fn is_integral(&self) -> bool {
        use Value::*;
        matches!(
            self,
            Field(_) | I8(_) | I16(_) | I32(_) | I64(_) | U8(_) | U16(_) | U32(_) | U64(_)
        )
    }

    pub(crate) fn is_closure(&self) -> bool {
        matches!(self, Value::Closure(..))
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
    tokens: Rc<Vec<LocatedToken>>,
    elaborator: &mut Elaborator,
    parsing_function: F,
    location: Location,
    rule: &'static str,
) -> IResult<T>
where
    F: FnOnce(&mut Parser<'a>) -> T,
{
    let parser = Parser::for_tokens(Tokens(unwrap_rc(tokens.clone())));
    match parser.parse_result(parsing_function) {
        Ok((expr, warnings)) => {
            for warning in warnings {
                let warning: CompilationError = warning.into();
                elaborator.push_err(warning);
            }
            Ok(expr)
        }
        Err(mut errors) => {
            let error = Box::new(errors.swap_remove(0));
            let tokens = tokens_to_string(&tokens, elaborator.interner);
            Err(InterpreterError::FailedToParseMacro { error, tokens, rule, location })
        }
    }
}
