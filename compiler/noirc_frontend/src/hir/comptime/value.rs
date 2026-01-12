//! Defines the [Value] type, representing a compile-time value, used by the
//! comptime interpreter when evaluating code.
use std::{borrow::Cow, rc::Rc, vec};

use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;
use strum_macros::Display;

use crate::{
    Kind, QuotedType, Shared, Type, TypeBindings, TypeVariable,
    ast::{
        ArrayLiteral, BlockExpression, CallExpression, ConstructorExpression, Expression,
        ExpressionKind, Ident, IntegerBitSize, LValue, LetStatement, Literal, Path, PathKind,
        PathSegment, Pattern, Statement, StatementKind, UnresolvedType, UnresolvedTypeData,
    },
    elaborator::Elaborator,
    hir::{
        comptime::interpreter::builtin_helpers::fragments_to_string,
        def_collector::dc_crate::CompilationError, def_map::ModuleId,
        type_check::generics::TraitGenerics,
    },
    hir_def::expr::{
        HirArrayLiteral, HirConstructorExpression, HirEnumConstructorExpression, HirExpression,
        HirIdent, HirLambda, HirLiteral, ImplKind,
    },
    node_interner::{ExprId, FuncId, NodeInterner, StmtId, TraitId, TraitImplId, TypeId},
    parser::{Item, Parser},
    shared::Signedness,
    signed_field::SignedField,
    token::{FmtStrFragment, IntegerTypeSuffix, LocatedToken, Token, Tokens},
};
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use super::{
    display::tokens_to_string,
    errors::{IResult, InterpreterError},
};

/// A value representing the result of evaluating a Noir expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    Field(SignedField),
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
    FormatString(Rc<Vec<FormatStringFragment>>, Type, u32 /* length */),
    CtString(Rc<String>),
    Function(FuncId, Type, Rc<TypeBindings>),

    /// Closures also store their original scope (function & module)
    /// in case they use functions such as `Quoted::as_type` which require them.
    Closure(Box<Closure>),

    /// Tuple elements are automatically shared to support projection into a tuple:
    /// `let elem = &mut tuple.0` should mutate the original element.
    Tuple(Vec<Shared<Value>>),

    /// Struct elements are automatically shared to support projection:
    /// `let elem = &mut my_struct.field` should mutate the original element.
    Struct(StructFields, Type),

    Enum(/*tag*/ usize, /*args*/ Vec<Value>, Type),
    Pointer(Shared<Value>, /* auto_deref */ bool, /* mutable */ bool),
    Array(Vector<Value>, Type),
    Vector(Vector<Value>, Type),
    Quoted(Rc<Vec<LocatedToken>>),
    TypeDefinition(TypeId),
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
pub enum FormatStringFragment {
    String(String),
    Value { name: String, value: Value },
}

pub(super) type StructFields = HashMap<Rc<String>, Shared<Value>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Closure {
    pub lambda: HirLambda,
    pub env: Vec<Value>,
    pub typ: Type,
    pub function_scope: Option<FuncId>,
    pub module_scope: ModuleId,
    /// The type bindings where the closure was created.
    /// This is needed because when the closure is interpreted, those type bindings
    /// need to be restored.
    pub bindings: HashMap<TypeVariable, (Type, Kind)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
#[allow(clippy::large_enum_variant)] // Tested shrinking in https://github.com/noir-lang/noir/pull/8746 with minimal memory impact
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

    pub(crate) fn lvalue(lvalue: LValue) -> Self {
        Value::Expr(Box::new(ExprValue::LValue(lvalue)))
    }

    pub(crate) fn pattern(pattern: Pattern) -> Self {
        Value::Expr(Box::new(ExprValue::Pattern(pattern)))
    }

    /// Retrieves the type of this value. Types can always be determined from the value,
    /// in cases where it would be ambiguous, Values store the type directly.
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
                let length: u32 = value
                    .len()
                    .try_into()
                    .expect("ICE: Value::get_type: value.len() is expected to fit into a u32");
                Type::String(Box::new(length.into()))
            }
            Value::FormatString(_, typ, _) => return Cow::Borrowed(typ),
            Value::Function(_, typ, _) => return Cow::Borrowed(typ),
            Value::Closure(closure) => return Cow::Borrowed(&closure.typ),
            Value::Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| field.borrow().get_type().into_owned()))
            }
            Value::Struct(_, typ) => return Cow::Borrowed(typ),
            Value::Enum(_, _, typ) => return Cow::Borrowed(typ),
            Value::Array(_, typ) => return Cow::Borrowed(typ),
            Value::Vector(_, typ) => return Cow::Borrowed(typ),
            Value::Quoted(_) => Type::Quoted(QuotedType::Quoted),
            Value::TypeDefinition(_) => Type::Quoted(QuotedType::TypeDefinition),
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

    /// Lowers this value into a runtime expression.
    ///
    /// For literals this is often simple, e.g. `Value::I8(3)` translates to `3`, but not
    /// all values are valid to lower. Lowering quoted code will simply return the quoted code (after
    /// parsing), this is how macros are implemented.
    pub(crate) fn into_expression(
        self,
        elaborator: &mut Elaborator,
        location: Location,
    ) -> IResult<Expression> {
        let kind = match self {
            Value::Unit => ExpressionKind::Literal(Literal::Unit),
            Value::Bool(value) => ExpressionKind::Literal(Literal::Bool(value)),
            Value::Field(value) => {
                ExpressionKind::Literal(Literal::Integer(value, Some(IntegerTypeSuffix::Field)))
            }
            Value::I8(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::from_signed(value),
                Some(IntegerTypeSuffix::I8),
            )),
            Value::I16(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::from_signed(value),
                Some(IntegerTypeSuffix::I16),
            )),
            Value::I32(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::from_signed(value),
                Some(IntegerTypeSuffix::I32),
            )),
            Value::I64(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::from_signed(value),
                Some(IntegerTypeSuffix::I64),
            )),
            Value::U1(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::positive(value),
                Some(IntegerTypeSuffix::U1),
            )),
            Value::U8(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::positive(u128::from(value)),
                Some(IntegerTypeSuffix::U8),
            )),
            Value::U16(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::positive(u128::from(value)),
                Some(IntegerTypeSuffix::U16),
            )),
            Value::U32(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::positive(value),
                Some(IntegerTypeSuffix::U32),
            )),
            Value::U64(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::positive(value),
                Some(IntegerTypeSuffix::U64),
            )),
            Value::U128(value) => ExpressionKind::Literal(Literal::Integer(
                SignedField::positive(value),
                Some(IntegerTypeSuffix::U128),
            )),
            Value::String(value) => ExpressionKind::Literal(Literal::Str(unwrap_rc(value))),
            Value::CtString(value) => {
                // Lower to `std::append::Append::append(CtString::new(), <contents>)`
                let ident = |name: &str| Ident::new(name.to_string(), location);
                let segment = |name: &str| PathSegment::from(ident(name));
                let path = |segments| Path {
                    segments,
                    location,
                    kind: PathKind::Plain,
                    kind_location: location,
                };
                let call = |path, arguments| {
                    ExpressionKind::Call(Box::new(CallExpression {
                        func: Box::new(Expression {
                            kind: ExpressionKind::Variable(path),
                            location,
                        }),
                        arguments,
                        is_macro_call: false,
                    }))
                };

                let ct_string_new = path(vec![segment("CtString"), segment("new")]);
                let ct_string_new = call(ct_string_new, vec![]);
                let ct_string_new = Expression { kind: ct_string_new, location };
                let contents = Literal::Str(unwrap_rc(value));
                let contents = Expression { kind: ExpressionKind::Literal(contents), location };
                let append = path(vec![
                    segment("std"),
                    segment("append"),
                    segment("Append"),
                    segment("append"),
                ]);
                call(append, vec![ct_string_new, contents])
            }
            Value::FormatString(fragments, _, length) => {
                // When turning a format string into an expression we could either:
                // 1. Create a single string literal with all interpolations resolved
                // 2. Create a format string literal
                // The problem with 1 is that the type of the value ends up being different
                // than the type of the value itself (a `fmtstr` in this case, which is also what
                // `get_type` returns).
                // In order to implement 2, and to preserve the type, we need to create
                // a format string with interpolated values. These values are referenced by
                // name, so we end up returning a block with `let` statements with names
                // that reference those values, with a final format string as the resulting
                // block expression.
                let mut statements = Vec::new();
                let mut new_fragments = Vec::with_capacity(fragments.len());
                let mut has_values = false;
                let mut seen_names: HashSet<String> = HashSet::default();
                for fragment in fragments.iter() {
                    let new_fragment = match fragment {
                        FormatStringFragment::String(string) => {
                            FmtStrFragment::String(string.clone())
                        }
                        FormatStringFragment::Value { name, value } => {
                            // A name might be interpolated multiple times. In that case it will always
                            // have the same value: we just need one `let` for it.
                            if !seen_names.insert(name.clone()) {
                                continue;
                            }

                            has_values = true;

                            let expression = value.clone().into_expression(elaborator, location)?;
                            let let_statement = LetStatement {
                                pattern: Pattern::Identifier(Ident::new(name.clone(), location)),
                                r#type: None,
                                expression,
                                attributes: Vec::new(),
                                comptime: false,
                                is_global_let: false,
                            };
                            let statement =
                                Statement { kind: StatementKind::Let(let_statement), location };
                            statements.push(statement);
                            FmtStrFragment::Interpolation(name.clone(), location)
                        }
                    };
                    new_fragments.push(new_fragment);
                }
                let fmtstr = ExpressionKind::Literal(Literal::FmtStr(new_fragments, length));
                if has_values {
                    statements.push(Statement {
                        kind: StatementKind::Expression(Expression { kind: fmtstr, location }),
                        location,
                    });
                    ExpressionKind::Block(BlockExpression { statements })
                } else {
                    fmtstr
                }
            }
            Value::Function(id, typ, bindings) => {
                let id = elaborator.interner.function_definition_id(id);
                let impl_kind = ImplKind::NotATraitMethod;
                let ident = HirIdent { location, id, impl_kind };
                let expr_id = elaborator.interner.push_expr_full(
                    HirExpression::Ident(ident, None),
                    location,
                    typ,
                );
                elaborator.interner.store_instantiation_bindings(expr_id, unwrap_rc(bindings));
                ExpressionKind::Resolved(expr_id)
            }
            Value::Tuple(fields) => {
                let fields = try_vecmap(fields, |field| {
                    field.unwrap_or_clone().into_expression(elaborator, location)
                })?;
                ExpressionKind::Tuple(fields)
            }
            Value::Struct(fields, typ) => {
                let fields = try_vecmap(fields, |(name, field)| {
                    let field = field.unwrap_or_clone().into_expression(elaborator, location)?;
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
            Value::Vector(elements, _) => {
                let elements =
                    try_vecmap(elements, |element| element.into_expression(elaborator, location))?;
                ExpressionKind::Literal(Literal::Vector(ArrayLiteral::Standard(elements)))
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
                    Err(errors) => {
                        let error = errors
                            .into_iter()
                            .find(|error| !error.is_warning())
                            .expect("there is at least one error");
                        let error = Box::new(error);
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
            | Value::TypeDefinition(_)
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

    /// Lowers this compile-time value into a HIR expression. This is similar to
    /// [Self::into_expression] but is used in some cases in the monomorphizer where
    /// code must already be in HIR.
    pub(crate) fn into_hir_expression(
        self,
        interner: &mut NodeInterner,
        location: Location,
    ) -> IResult<ExprId> {
        let typ = self.get_type().into_owned();
        let expression = match self {
            Value::Unit => HirExpression::Literal(HirLiteral::Unit),
            Value::Bool(value) => HirExpression::Literal(HirLiteral::Bool(value)),
            Value::Field(value) => HirExpression::Literal(HirLiteral::Integer(value)),
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
            Value::U8(value) => HirExpression::Literal(HirLiteral::Integer(SignedField::positive(
                u128::from(value),
            ))),
            Value::U16(value) => HirExpression::Literal(HirLiteral::Integer(
                SignedField::positive(u128::from(value)),
            )),
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
            Value::FormatString(fragments, _typ, length) => {
                let mut captures = Vec::new();
                let mut new_fragments = Vec::with_capacity(fragments.len());
                for fragment in fragments.iter() {
                    match fragment {
                        FormatStringFragment::String(string) => {
                            new_fragments.push(FmtStrFragment::String(string.clone()));
                        }
                        FormatStringFragment::Value { name, value } => {
                            let expr_id = value.clone().into_hir_expression(interner, location)?;
                            captures.push(expr_id);
                            new_fragments
                                .push(FmtStrFragment::Interpolation(name.clone(), location));
                        }
                    }
                }
                HirExpression::Literal(HirLiteral::FmtStr(new_fragments, captures, length))
            }
            Value::Function(id, typ, bindings) => {
                let id = interner.function_definition_id(id);
                let impl_kind = ImplKind::NotATraitMethod;
                let ident = HirIdent { location, id, impl_kind };
                let expr_id =
                    interner.push_expr_full(HirExpression::Ident(ident, None), location, typ);
                interner.store_instantiation_bindings(expr_id, unwrap_rc(bindings));
                return Ok(expr_id);
            }
            Value::Tuple(fields) => {
                let fields = try_vecmap(fields, |field| {
                    field.unwrap_or_clone().into_hir_expression(interner, location)
                })?;
                HirExpression::Tuple(fields)
            }
            Value::Struct(fields, typ) => {
                let fields = try_vecmap(fields, |(name, field)| {
                    let field = field.unwrap_or_clone().into_hir_expression(interner, location)?;
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
            Value::Vector(elements, _) => {
                let elements = try_vecmap(elements, |element| {
                    element.into_hir_expression(interner, location)
                })?;
                HirExpression::Literal(HirLiteral::Vector(HirArrayLiteral::Standard(elements)))
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
            | Value::TypeDefinition(_)
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

        let id = interner.push_expr_full(expression, location, typ);
        Ok(id)
    }

    /// Attempt to convert this value into a Vec of tokens representing this value if it appeared
    /// in source code. For example, `Value::Unit` is `vec!['(', ')']`. This is used for splicing
    /// values into quoted values when `$` is used within a `quote {  }` expression. Since `Quoted`
    /// code is represented as tokens, we need to convert the value into tokens.
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
                let name = Rc::new(interner.get_trait(trait_id).name.to_string());
                let typ = Type::TraitAsType(trait_id, name, generics);
                vec![Token::QuotedType(interner.push_quoted_type(typ))]
            }
            Value::TypedExpr(TypedExpr::ExprId(expr_id)) => vec![Token::UnquoteMarker(expr_id)],
            Value::Bool(bool) => vec![Token::Bool(bool)],
            Value::U1(bool) => {
                vec![Token::Int(u128::from(bool).into(), Some(IntegerTypeSuffix::U1))]
            }
            Value::U8(value) => {
                vec![Token::Int(u128::from(value).into(), Some(IntegerTypeSuffix::U8))]
            }
            Value::U16(value) => {
                vec![Token::Int(u128::from(value).into(), Some(IntegerTypeSuffix::U16))]
            }
            Value::U32(value) => {
                vec![Token::Int(u128::from(value).into(), Some(IntegerTypeSuffix::U32))]
            }
            Value::U64(value) => {
                vec![Token::Int(u128::from(value).into(), Some(IntegerTypeSuffix::U64))]
            }
            Value::U128(value) => {
                vec![Token::Int(value.into(), Some(IntegerTypeSuffix::U128))]
            }
            Value::I8(value) => {
                if value < 0 {
                    vec![
                        Token::Minus,
                        Token::Int(
                            u128::from(value.unsigned_abs()).into(),
                            Some(IntegerTypeSuffix::I8),
                        ),
                    ]
                } else {
                    vec![Token::Int((value as u128).into(), Some(IntegerTypeSuffix::I8))]
                }
            }
            Value::I16(value) => {
                if value < 0 {
                    vec![
                        Token::Minus,
                        Token::Int(
                            u128::from(value.unsigned_abs()).into(),
                            Some(IntegerTypeSuffix::I16),
                        ),
                    ]
                } else {
                    vec![Token::Int((value as u128).into(), Some(IntegerTypeSuffix::I16))]
                }
            }
            Value::I32(value) => {
                if value < 0 {
                    vec![
                        Token::Minus,
                        Token::Int(
                            u128::from(value.unsigned_abs()).into(),
                            Some(IntegerTypeSuffix::I32),
                        ),
                    ]
                } else {
                    vec![Token::Int((value as u128).into(), Some(IntegerTypeSuffix::I32))]
                }
            }
            Value::I64(value) => {
                if value < 0 {
                    vec![
                        Token::Minus,
                        Token::Int(
                            u128::from(value.unsigned_abs()).into(),
                            Some(IntegerTypeSuffix::I64),
                        ),
                    ]
                } else {
                    vec![Token::Int((value as u128).into(), Some(IntegerTypeSuffix::I64))]
                }
            }
            Value::Field(value) => {
                if value.is_negative() {
                    vec![Token::Minus, Token::Int(value.absolute_value(), None)]
                } else {
                    vec![Token::Int(value.absolute_value(), None)]
                }
            }
            Value::String(value) | Value::CtString(value) => {
                vec![Token::Str(unwrap_rc(value))]
            }
            Value::FormatString(fragments, _, _) => {
                // When a fmtstr is unquoted, we turn it into a normal string by evaluating the interpolations
                let string = fragments_to_string(&fragments, interner);
                vec![Token::Str(string)]
            }
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
            Field(_)
                | I8(_)
                | I16(_)
                | I32(_)
                | I64(_)
                | U1(_)
                | U8(_)
                | U16(_)
                | U32(_)
                | U64(_)
                | U128(_)
        )
    }

    pub(crate) fn is_zero(&self) -> bool {
        use Value::*;
        match self {
            Field(value) => value.is_zero(),
            I8(value) => *value == 0,
            I16(value) => *value == 0,
            I32(value) => *value == 0,
            I64(value) => *value == 0,
            U1(value) => !*value,
            U8(value) => *value == 0,
            U16(value) => *value == 0,
            U32(value) => *value == 0,
            U64(value) => *value == 0,
            U128(value) => *value == 0,
            _ => false,
        }
    }

    pub(crate) fn contains_function_or_closure(&self) -> bool {
        match self {
            Value::Function(..) => true,
            Value::Closure(..) => true,
            Value::Array(values, _) => {
                values.iter().any(|value| value.contains_function_or_closure())
            }
            Value::Vector(values, _) => {
                values.iter().any(|value| value.contains_function_or_closure())
            }
            Value::Tuple(values) => {
                values.iter().any(|value| value.borrow().contains_function_or_closure())
            }
            Value::Struct(fields, _) => {
                fields.values().any(|value| value.borrow().contains_function_or_closure())
            }
            Value::Enum(_, values, _) => {
                values.iter().any(|value| value.contains_function_or_closure())
            }
            Value::Pointer(shared, _, _) => shared.borrow().contains_function_or_closure(),
            Value::Unit
            | Value::Bool(_)
            | Value::Field(_)
            | Value::I8(_)
            | Value::I16(_)
            | Value::I32(_)
            | Value::I64(_)
            | Value::U1(_)
            | Value::U8(_)
            | Value::U16(_)
            | Value::U32(_)
            | Value::U64(_)
            | Value::U128(_)
            | Value::String(_)
            | Value::FormatString(_, _, _)
            | Value::CtString(_)
            | Value::Quoted(_)
            | Value::TypeDefinition(_)
            | Value::TraitConstraint(_, _)
            | Value::TraitDefinition(_)
            | Value::TraitImpl(_)
            | Value::FunctionDefinition(_)
            | Value::ModuleDefinition(_)
            | Value::Type(_)
            | Value::Zeroed(_)
            | Value::Expr(_)
            | Value::TypedExpr(_)
            | Value::UnresolvedType(_) => false,
        }
    }

    /// Converts any integral `Value` into a `SignedField`.
    /// Returns `None` for non-integral `Value`s and negative numbers.
    pub(crate) fn to_non_negative_signed_field(&self) -> Option<SignedField> {
        let value = self.to_signed_field()?;
        value.is_positive().then_some(value)
    }

    /// Converts any integral `Value` into a `SignedField`.
    /// Returns `None` for non-integral `Value`s
    pub(crate) fn to_signed_field(&self) -> Option<SignedField> {
        match self {
            Value::Field(value) => Some(value.into()),
            Value::I8(value) => Some(value.into()),
            Value::I16(value) => Some(value.into()),
            Value::I32(value) => Some(value.into()),
            Value::I64(value) => Some(value.into()),
            Value::U1(value) => Some(value.into()),
            Value::U8(value) => Some(value.into()),
            Value::U16(value) => Some(value.into()),
            Value::U32(value) => Some(value.into()),
            Value::U64(value) => Some(value.into()),
            Value::U128(value) => Some(value.into()),
            _ => None,
        }
    }

    /// Similar to [Self::into_expression] or [Self::into_hir_expression] but for converting
    /// into top-level item(s). Unlike those other methods, most expressions are invalid
    /// as top-level items (e.g. a lone `3` is not a valid top-level statement). As a result,
    /// this method is significantly simpler because we only have to parse `Quoted` values
    /// into top level items.
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

    pub fn is_negative(&self) -> bool {
        match self {
            Value::I8(x) => *x < 0,
            Value::I16(x) => *x < 0,
            Value::I32(x) => *x < 0,
            Value::I64(x) => *x < 0,
            _ => false, // Unsigned or Field types are never negative
        }
    }

    /// Structs and tuples store references to their fields internally which need to be manually
    /// changed when moving them.
    ///
    /// All references are shared by default but when we have `let mut foo = Struct { .. }` in
    /// code, we don't want moving it: `let mut bar = foo;` to refer to the same references.
    /// This function will copy them so that mutating the fields of `foo` will not mutate `bar`.
    pub(crate) fn move_struct(self) -> Value {
        match self {
            Value::Tuple(fields) => Value::Tuple(vecmap(fields, |field| {
                Shared::new(field.unwrap_or_clone().move_struct())
            })),
            Value::Struct(fields, typ) => {
                let fields = fields.into_iter().map(|(name, field)| {
                    (name, Shared::new(field.unwrap_or_clone().move_struct()))
                });
                Value::Struct(fields.collect(), typ)
            }
            other => other,
        }
    }
}

/// Unwraps an Rc value without cloning the inner value if the reference count is 1. Clones otherwise.
pub(crate) fn unwrap_rc<T: Clone>(rc: Rc<T>) -> T {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}

/// Helper to parse the given tokens using the given parse function.
///
/// If they fail to parse, [InterpreterError::FailedToParseMacro] is returned.
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
        Err(errors) => {
            let error = errors.into_iter().find(|error| !error.is_warning()).unwrap();
            let error = Box::new(error);
            let tokens = tokens_to_string(&tokens, elaborator.interner);
            Err(InterpreterError::FailedToParseMacro { error, tokens, rule, location })
        }
    }
}
