use std::rc::Rc;

use acvm::FieldElement;
use noirc_errors::Location;

use crate::{
    ast::{BlockExpression, IntegerBitSize, Signedness, UnresolvedTypeData},
    hir::{
        comptime::{
            errors::IResult,
            value::{add_token_spans, ExprValue},
            Interpreter, InterpreterError, Value,
        },
        def_map::ModuleId,
        type_check::generics::TraitGenerics,
    },
    hir_def::{
        function::{FuncMeta, FunctionBody},
        stmt::HirPattern,
    },
    macros_api::{NodeInterner, StructId},
    node_interner::{FuncId, TraitId, TraitImplId},
    parser::NoirParser,
    token::{Token, Tokens},
    QuotedType, Type,
};

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
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<(Value, Location)> {
    check_argument_count(1, &arguments, location)?;

    Ok(arguments.pop().unwrap())
}

pub(crate) fn check_two_arguments(
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<((Value, Location), (Value, Location))> {
    check_argument_count(2, &arguments, location)?;

    let argument2 = arguments.pop().unwrap();
    let argument1 = arguments.pop().unwrap();

    Ok((argument1, argument2))
}

#[allow(clippy::type_complexity)]
pub(crate) fn check_three_arguments(
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<((Value, Location), (Value, Location), (Value, Location))> {
    check_argument_count(3, &arguments, location)?;

    let argument3 = arguments.pop().unwrap();
    let argument2 = arguments.pop().unwrap();
    let argument1 = arguments.pop().unwrap();

    Ok((argument1, argument2, argument3))
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

pub(crate) fn get_expr((value, location): (Value, Location)) -> IResult<ExprValue> {
    match value {
        Value::Expr(expr) => Ok(expr),
        value => type_mismatch(value, Type::Quoted(QuotedType::Expr), location),
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

pub(crate) fn get_quoted((value, location): (Value, Location)) -> IResult<Rc<Vec<Token>>> {
    match value {
        Value::Quoted(tokens) => Ok(tokens),
        value => type_mismatch(value, Type::Quoted(QuotedType::Quoted), location),
    }
}

pub(crate) fn get_unresolved_type(
    (value, location): (Value, Location),
) -> IResult<UnresolvedTypeData> {
    match value {
        Value::UnresolvedType(typ) => Ok(typ),
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

pub(super) fn parse<T>(
    (value, location): (Value, Location),
    parser: impl NoirParser<T>,
    rule: &'static str,
) -> IResult<T> {
    let tokens = get_quoted((value, location))?;
    let quoted = add_token_spans(tokens.clone(), location.span);
    parse_tokens(tokens, quoted, location, parser, rule)
}

pub(super) fn parse_tokens<T>(
    tokens: Rc<Vec<Token>>,
    quoted: Tokens,
    location: Location,
    parser: impl NoirParser<T>,
    rule: &'static str,
) -> IResult<T> {
    parser.parse(quoted).map_err(|mut errors| {
        let error = errors.swap_remove(0);
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
