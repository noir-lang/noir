use std::hash::Hash;
use std::{hash::Hasher, rc::Rc};

use acvm::FieldElement;
use iter_extended::try_vecmap;
use noirc_errors::Location;

use crate::hir::comptime::display::tokens_to_string;
use crate::hir::comptime::value::add_token_spans;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::{
    ast::{
        BlockExpression, ExpressionKind, Ident, IntegerBitSize, LValue, Pattern, Signedness,
        StatementKind, UnresolvedTypeData,
    },
    hir::{
        comptime::{
            errors::IResult,
            value::{ExprValue, TypedExpr},
            Interpreter, InterpreterError, Value,
        },
        def_map::ModuleId,
        type_check::generics::TraitGenerics,
    },
    hir_def::{
        function::{FuncMeta, FunctionBody},
        stmt::HirPattern,
    },
    node_interner::{FuncId, NodeInterner, StructId, TraitId, TraitImplId},
    token::{SecondaryAttribute, Token, Tokens},
    QuotedType, Type,
};
use crate::{Kind, Shared, StructType};
use rustc_hash::FxHashMap as HashMap;

pub(crate) fn check_argument_count(
    expected: usize,
    arguments: &[(Value, Location)],
    location: Location,
) -> IResult<()> {
    if arguments.len() == expected {
        Ok(())
    } else {
        let actual = arguments.len();
        Err(InterpreterError::ArgumentCountMismatch { expected, actual, location })
    }
}

pub(crate) fn check_one_argument(
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<(Value, Location)> {
    let [arg1] = check_arguments(arguments, location)?;

    Ok(arg1)
}

pub(crate) fn check_two_arguments(
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<((Value, Location), (Value, Location))> {
    let [arg1, arg2] = check_arguments(arguments, location)?;

    Ok((arg1, arg2))
}

#[allow(clippy::type_complexity)]
pub(crate) fn check_three_arguments(
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<((Value, Location), (Value, Location), (Value, Location))> {
    let [arg1, arg2, arg3] = check_arguments(arguments, location)?;

    Ok((arg1, arg2, arg3))
}

#[allow(clippy::type_complexity)]
pub(crate) fn check_arguments<const N: usize>(
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<[(Value, Location); N]> {
    check_argument_count(N, &arguments, location)?;
    Ok(arguments.try_into().expect("checked arg count"))
}

pub(crate) fn get_array(
    interner: &NodeInterner,
    (value, location): (Value, Location),
) -> IResult<(im::Vector<Value>, Type)> {
    match value {
        Value::Array(values, typ) => Ok((values, typ)),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Array(type_var.clone(), type_var);
            type_mismatch(value, expected, location)
        }
    }
}

/// Get the fields if the value is a `Value::Struct`, otherwise report that a struct type
/// with `name` is expected. Returns the `Type` but doesn't verify that it's called `name`.
pub(crate) fn get_struct_fields(
    name: &str,
    (value, location): (Value, Location),
) -> IResult<(HashMap<Rc<String>, Value>, Type)> {
    match value {
        Value::Struct(fields, typ) => Ok((fields, typ)),
        _ => {
            let expected = StructType::new(
                StructId::dummy_id(),
                Ident::new(name.to_string(), location.span),
                location,
                Vec::new(),
                Vec::new(),
            );
            let expected = Type::Struct(Shared::new(expected), Vec::new());
            type_mismatch(value, expected, location)
        }
    }
}

/// Get a specific field of a struct and apply a decoder function on it.
pub(crate) fn get_struct_field<T>(
    field_name: &str,
    struct_fields: &HashMap<Rc<String>, Value>,
    struct_type: &Type,
    location: Location,
    f: impl Fn((Value, Location)) -> IResult<T>,
) -> IResult<T> {
    let key = Rc::new(field_name.to_string());
    let Some(value) = struct_fields.get(&key) else {
        return Err(InterpreterError::ExpectedStructToHaveField {
            typ: struct_type.clone(),
            field_name: Rc::into_inner(key).unwrap(),
            location,
        });
    };
    f((value.clone(), location))
}

pub(crate) fn get_bool((value, location): (Value, Location)) -> IResult<bool> {
    match value {
        Value::Bool(value) => Ok(value),
        value => type_mismatch(value, Type::Bool, location),
    }
}

pub(crate) fn get_slice(
    interner: &NodeInterner,
    (value, location): (Value, Location),
) -> IResult<(im::Vector<Value>, Type)> {
    match value {
        Value::Slice(values, typ) => Ok((values, typ)),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Slice(type_var);
            type_mismatch(value, expected, location)
        }
    }
}

/// Interpret the input as a slice, then map each element.
/// Returns the values in the slice and the original type.
pub(crate) fn get_slice_map<T>(
    interner: &NodeInterner,
    (value, location): (Value, Location),
    f: impl Fn((Value, Location)) -> IResult<T>,
) -> IResult<(Vec<T>, Type)> {
    let (values, typ) = get_slice(interner, (value, location))?;
    let values = try_vecmap(values, |value| f((value, location)))?;
    Ok((values, typ))
}

/// Interpret the input as an array, then map each element.
/// Returns the values in the array and the original array type.
pub(crate) fn get_array_map<T>(
    interner: &NodeInterner,
    (value, location): (Value, Location),
    f: impl Fn((Value, Location)) -> IResult<T>,
) -> IResult<(Vec<T>, Type)> {
    let (values, typ) = get_array(interner, (value, location))?;
    let values = try_vecmap(values, |value| f((value, location)))?;
    Ok((values, typ))
}

/// Get an array and convert it to a fixed size.
/// Returns the values in the array and the original array type.
pub(crate) fn get_fixed_array_map<T, const N: usize>(
    interner: &NodeInterner,
    (value, location): (Value, Location),
    f: impl Fn((Value, Location)) -> IResult<T>,
) -> IResult<([T; N], Type)> {
    let (values, typ) = get_array_map(interner, (value, location), f)?;

    values.try_into().map(|v| (v, typ.clone())).map_err(|_| {
        // Assuming that `values.len()` corresponds to `typ`.
        let Type::Array(_, ref elem) = typ else {
            unreachable!("get_array_map checked it was an array")
        };
        let expected = Type::Array(Box::new(Type::Constant(N.into(), Kind::u32())), elem.clone());
        InterpreterError::TypeMismatch { expected, actual: typ, location }
    })
}

pub(crate) fn get_str(
    interner: &NodeInterner,
    (value, location): (Value, Location),
) -> IResult<Rc<String>> {
    match value {
        Value::String(string) => Ok(string),
        value => {
            let expected = Type::String(Box::new(interner.next_type_variable()));
            type_mismatch(value, expected, location)
        }
    }
}

pub(crate) fn get_ctstring((value, location): (Value, Location)) -> IResult<Rc<String>> {
    match value {
        Value::CtString(string) => Ok(string),
        value => type_mismatch(value, Type::Quoted(QuotedType::CtString), location),
    }
}

pub(crate) fn get_tuple(
    interner: &NodeInterner,
    (value, location): (Value, Location),
) -> IResult<Vec<Value>> {
    match value {
        Value::Tuple(values) => Ok(values),
        value => {
            let type_var = interner.next_type_variable();
            let expected = Type::Tuple(vec![type_var]);
            type_mismatch(value, expected, location)
        }
    }
}

pub(crate) fn get_field((value, location): (Value, Location)) -> IResult<FieldElement> {
    match value {
        Value::Field(value) => Ok(value),
        value => type_mismatch(value, Type::FieldElement, location),
    }
}

pub(crate) fn get_u8((value, location): (Value, Location)) -> IResult<u8> {
    match value {
        Value::U8(value) => Ok(value),
        value => {
            let expected = Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight);
            type_mismatch(value, expected, location)
        }
    }
}

pub(crate) fn get_u32((value, location): (Value, Location)) -> IResult<u32> {
    match value {
        Value::U32(value) => Ok(value),
        value => {
            let expected = Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo);
            type_mismatch(value, expected, location)
        }
    }
}

pub(crate) fn get_u64((value, location): (Value, Location)) -> IResult<u64> {
    match value {
        Value::U64(value) => Ok(value),
        value => {
            let expected = Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour);
            type_mismatch(value, expected, location)
        }
    }
}

pub(crate) fn get_expr(
    interner: &NodeInterner,
    (value, location): (Value, Location),
) -> IResult<ExprValue> {
    match value {
        Value::Expr(expr) => match expr {
            ExprValue::Expression(ExpressionKind::Interned(id)) => {
                Ok(ExprValue::Expression(interner.get_expression_kind(id).clone()))
            }
            ExprValue::Statement(StatementKind::Interned(id)) => {
                Ok(ExprValue::Statement(interner.get_statement_kind(id).clone()))
            }
            ExprValue::LValue(LValue::Interned(id, _)) => {
                Ok(ExprValue::LValue(interner.get_lvalue(id, location.span).clone()))
            }
            ExprValue::Pattern(Pattern::Interned(id, _)) => {
                Ok(ExprValue::Pattern(interner.get_pattern(id).clone()))
            }
            _ => Ok(expr),
        },
        value => type_mismatch(value, Type::Quoted(QuotedType::Expr), location),
    }
}

pub(crate) fn get_format_string(
    interner: &NodeInterner,
    (value, location): (Value, Location),
) -> IResult<(Rc<String>, Type)> {
    match value {
        Value::FormatString(value, typ) => Ok((value, typ)),
        value => {
            let n = Box::new(interner.next_type_variable());
            let e = Box::new(interner.next_type_variable());
            type_mismatch(value, Type::FmtString(n, e), location)
        }
    }
}

pub(crate) fn get_function_def((value, location): (Value, Location)) -> IResult<FuncId> {
    match value {
        Value::FunctionDefinition(id) => Ok(id),
        value => type_mismatch(value, Type::Quoted(QuotedType::FunctionDefinition), location),
    }
}

pub(crate) fn get_module((value, location): (Value, Location)) -> IResult<ModuleId> {
    match value {
        Value::ModuleDefinition(module_id) => Ok(module_id),
        value => type_mismatch(value, Type::Quoted(QuotedType::Module), location),
    }
}

pub(crate) fn get_struct((value, location): (Value, Location)) -> IResult<StructId> {
    match value {
        Value::StructDefinition(id) => Ok(id),
        _ => type_mismatch(value, Type::Quoted(QuotedType::StructDefinition), location),
    }
}

pub(crate) fn get_trait_constraint(
    (value, location): (Value, Location),
) -> IResult<(TraitId, TraitGenerics)> {
    match value {
        Value::TraitConstraint(trait_id, generics) => Ok((trait_id, generics)),
        value => type_mismatch(value, Type::Quoted(QuotedType::TraitConstraint), location),
    }
}

pub(crate) fn get_trait_def((value, location): (Value, Location)) -> IResult<TraitId> {
    match value {
        Value::TraitDefinition(id) => Ok(id),
        value => type_mismatch(value, Type::Quoted(QuotedType::TraitDefinition), location),
    }
}

pub(crate) fn get_trait_impl((value, location): (Value, Location)) -> IResult<TraitImplId> {
    match value {
        Value::TraitImpl(id) => Ok(id),
        value => type_mismatch(value, Type::Quoted(QuotedType::TraitImpl), location),
    }
}

pub(crate) fn get_type((value, location): (Value, Location)) -> IResult<Type> {
    match value {
        Value::Type(typ) => Ok(typ),
        value => type_mismatch(value, Type::Quoted(QuotedType::Type), location),
    }
}

pub(crate) fn get_typed_expr((value, location): (Value, Location)) -> IResult<TypedExpr> {
    match value {
        Value::TypedExpr(typed_expr) => Ok(typed_expr),
        value => type_mismatch(value, Type::Quoted(QuotedType::TypedExpr), location),
    }
}

pub(crate) fn get_quoted((value, location): (Value, Location)) -> IResult<Rc<Vec<Token>>> {
    match value {
        Value::Quoted(tokens) => Ok(tokens),
        value => type_mismatch(value, Type::Quoted(QuotedType::Quoted), location),
    }
}

pub(crate) fn get_unresolved_type(
    interner: &NodeInterner,
    (value, location): (Value, Location),
) -> IResult<UnresolvedTypeData> {
    match value {
        Value::UnresolvedType(typ) => {
            if let UnresolvedTypeData::Interned(id) = typ {
                let typ = interner.get_unresolved_type_data(id).clone();
                Ok(typ)
            } else {
                Ok(typ)
            }
        }
        value => type_mismatch(value, Type::Quoted(QuotedType::UnresolvedType), location),
    }
}

fn type_mismatch<T>(value: Value, expected: Type, location: Location) -> IResult<T> {
    let actual = value.get_type().into_owned();
    Err(InterpreterError::TypeMismatch { expected, actual, location })
}

pub(crate) fn hir_pattern_to_tokens(
    interner: &NodeInterner,
    hir_pattern: &HirPattern,
) -> Vec<Token> {
    let mut tokens = Vec::new();
    gather_hir_pattern_tokens(interner, hir_pattern, &mut tokens);
    tokens
}

fn gather_hir_pattern_tokens(
    interner: &NodeInterner,
    hir_pattern: &HirPattern,
    tokens: &mut Vec<Token>,
) {
    match hir_pattern {
        HirPattern::Identifier(hir_ident) => {
            let name = interner.definition_name(hir_ident.id).to_string();
            tokens.push(Token::Ident(name));
        }
        HirPattern::Mutable(pattern, _) => {
            tokens.push(Token::Keyword(crate::token::Keyword::Mut));
            gather_hir_pattern_tokens(interner, pattern, tokens);
        }
        HirPattern::Tuple(patterns, _) => {
            tokens.push(Token::LeftParen);
            for (index, pattern) in patterns.iter().enumerate() {
                if index != 0 {
                    tokens.push(Token::Comma);
                }
                gather_hir_pattern_tokens(interner, pattern, tokens);
            }
            tokens.push(Token::RightParen);
        }
        HirPattern::Struct(typ, fields, _) => {
            let Type::Struct(struct_type, _) = typ.follow_bindings() else {
                panic!("Expected type to be a struct");
            };

            let name = struct_type.borrow().name.to_string();
            tokens.push(Token::Ident(name));

            tokens.push(Token::LeftBrace);
            for (index, (field_name, pattern)) in fields.iter().enumerate() {
                if index != 0 {
                    tokens.push(Token::Comma);
                }

                let field_name = &field_name.0.contents;
                tokens.push(Token::Ident(field_name.to_string()));

                // If we have a pattern like `Foo { x }`, that's internally represented as `Foo { x: x }` so
                // here we check if the field name is the same as the pattern and, if so, omit the `: x` part.
                let field_name_is_same_as_pattern = if let HirPattern::Identifier(pattern) = pattern
                {
                    let pattern_name = interner.definition_name(pattern.id);
                    field_name == pattern_name
                } else {
                    false
                };

                if !field_name_is_same_as_pattern {
                    tokens.push(Token::Colon);
                    gather_hir_pattern_tokens(interner, pattern, tokens);
                }
            }
            tokens.push(Token::RightBrace);
        }
    }
}

pub(super) fn check_function_not_yet_resolved(
    interpreter: &Interpreter,
    func_id: FuncId,
    location: Location,
) -> Result<(), InterpreterError> {
    let func_meta = interpreter.elaborator.interner.function_meta(&func_id);
    match func_meta.function_body {
        FunctionBody::Unresolved(_, _, _) => Ok(()),
        FunctionBody::Resolving | FunctionBody::Resolved => {
            Err(InterpreterError::FunctionAlreadyResolved { location })
        }
    }
}

pub(super) fn lex(input: &str) -> Vec<Token> {
    let (tokens, _) = Lexer::lex(input);
    let mut tokens: Vec<_> = tokens.0.into_iter().map(|token| token.into_token()).collect();
    if let Some(Token::EOF) = tokens.last() {
        tokens.pop();
    }
    tokens
}

pub(super) fn parse<'a, T, F>(
    interner: &NodeInterner,
    (value, location): (Value, Location),
    parser: F,
    rule: &'static str,
) -> IResult<T>
where
    F: FnOnce(&mut Parser<'a>) -> T,
{
    let tokens = get_quoted((value, location))?;
    let quoted = add_token_spans(tokens.clone(), location.span);
    parse_tokens(tokens, quoted, interner, location, parser, rule)
}

pub(super) fn parse_tokens<'a, T, F>(
    tokens: Rc<Vec<Token>>,
    quoted: Tokens,
    interner: &NodeInterner,
    location: Location,
    parsing_function: F,
    rule: &'static str,
) -> IResult<T>
where
    F: FnOnce(&mut Parser<'a>) -> T,
{
    Parser::for_tokens(quoted).parse_result(parsing_function).map_err(|mut errors| {
        let error = errors.swap_remove(0);
        let tokens = tokens_to_string(tokens, interner);
        InterpreterError::FailedToParseMacro { error, tokens, rule, file: location.file }
    })
}

pub(super) fn mutate_func_meta_type<F>(interner: &mut NodeInterner, func_id: FuncId, f: F)
where
    F: FnOnce(&mut FuncMeta),
{
    let (name_id, function_type) = {
        let func_meta = interner.function_meta_mut(&func_id);
        f(func_meta);
        (func_meta.name.id, func_meta.typ.clone())
    };

    interner.push_definition_type(name_id, function_type);
}

pub(super) fn replace_func_meta_parameters(typ: &mut Type, parameter_types: Vec<Type>) {
    match typ {
        Type::Function(parameters, _, _, _) => {
            *parameters = parameter_types;
        }
        Type::Forall(_, typ) => replace_func_meta_parameters(typ, parameter_types),
        _ => {}
    }
}

pub(super) fn replace_func_meta_return_type(typ: &mut Type, return_type: Type) {
    match typ {
        Type::Function(_, ret, _, _) => {
            *ret = Box::new(return_type);
        }
        Type::Forall(_, typ) => replace_func_meta_return_type(typ, return_type),
        _ => {}
    }
}

pub(super) fn block_expression_to_value(block_expr: BlockExpression) -> Value {
    let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Expr)));
    let statements = block_expr.statements.into_iter();
    let statements = statements.map(|statement| Value::statement(statement.kind)).collect();

    Value::Slice(statements, typ)
}

pub(super) fn has_named_attribute(name: &str, attributes: &[SecondaryAttribute]) -> bool {
    for attribute in attributes {
        if let Some(attribute_name) = attribute.name() {
            if name == attribute_name {
                return true;
            }
        }
    }

    false
}

pub(super) fn quote_ident(ident: &Ident) -> Value {
    Value::Quoted(ident_to_tokens(ident))
}

pub(super) fn ident_to_tokens(ident: &Ident) -> Rc<Vec<Token>> {
    Rc::new(vec![Token::Ident(ident.0.contents.clone())])
}

pub(super) fn hash_item<T: Hash>(
    arguments: Vec<(Value, Location)>,
    location: Location,
    get_item: impl FnOnce((Value, Location)) -> IResult<T>,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;
    let item = get_item(argument)?;

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    item.hash(&mut hasher);
    let hash = hasher.finish();
    Ok(Value::Field((hash as u128).into()))
}

pub(super) fn eq_item<T: Eq>(
    arguments: Vec<(Value, Location)>,
    location: Location,
    mut get_item: impl FnMut((Value, Location)) -> IResult<T>,
) -> IResult<Value> {
    let (self_arg, other_arg) = check_two_arguments(arguments, location)?;
    let self_arg = get_item(self_arg)?;
    let other_arg = get_item(other_arg)?;
    Ok(Value::Bool(self_arg == other_arg))
}

/// Type to be used in `Value::Array(<values>, <array-type>)`.
pub(crate) fn byte_array_type(len: usize) -> Type {
    Type::Array(
        Box::new(Type::Constant(len.into(), Kind::u32())),
        Box::new(Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight)),
    )
}

/// Type to be used in `Value::Slice(<values>, <slice-type>)`.
pub(crate) fn byte_slice_type() -> Type {
    Type::Slice(Box::new(Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight)))
}

/// Create a `Value::Array` from bytes.
pub(crate) fn to_byte_array(values: &[u8]) -> Value {
    Value::Array(values.iter().copied().map(Value::U8).collect(), byte_array_type(values.len()))
}

/// Create a `Value::Slice` from bytes.
pub(crate) fn to_byte_slice(values: &[u8]) -> Value {
    Value::Slice(values.iter().copied().map(Value::U8).collect(), byte_slice_type())
}

/// Create a `Value::Array` from fields.
pub(crate) fn to_field_array(values: &[FieldElement]) -> Value {
    let typ = Type::Array(
        Box::new(Type::Constant(values.len().into(), Kind::u32())),
        Box::new(Type::FieldElement),
    );
    Value::Array(values.iter().copied().map(Value::Field).collect(), typ)
}

/// Create a `Value::Struct` from fields and the expected return type.
pub(crate) fn to_struct(
    fields: impl IntoIterator<Item = (&'static str, Value)>,
    typ: Type,
) -> Value {
    let fields = fields.into_iter().map(|(k, v)| (Rc::new(k.to_string()), v)).collect();
    Value::Struct(fields, typ)
}
