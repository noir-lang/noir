use std::rc::Rc;

use acvm::FieldElement;
use noirc_errors::Location;

use crate::{
    ast::{IntegerBitSize, Signedness},
    hir::comptime::{errors::IResult, InterpreterError, Value},
    hir_def::stmt::HirPattern,
    macros_api::NodeInterner,
    node_interner::{FuncId, TraitId},
    token::Token,
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
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    Ok(arguments.pop().unwrap().0)
}

pub(crate) fn check_two_arguments(
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<(Value, Value)> {
    check_argument_count(2, &arguments, location)?;

    let argument2 = arguments.pop().unwrap().0;
    let argument1 = arguments.pop().unwrap().0;

    Ok((argument1, argument2))
}

pub(crate) fn check_three_arguments(
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<(Value, Value, Value)> {
    check_argument_count(3, &arguments, location)?;

    let argument3 = arguments.pop().unwrap().0;
    let argument2 = arguments.pop().unwrap().0;
    let argument1 = arguments.pop().unwrap().0;

    Ok((argument1, argument2, argument3))
}

pub(crate) fn get_array(
    interner: &NodeInterner,
    value: Value,
    location: Location,
) -> IResult<(im::Vector<Value>, Type)> {
    match value {
        Value::Array(values, typ) => Ok((values, typ)),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Array(type_var.clone(), type_var);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

pub(crate) fn get_slice(
    interner: &NodeInterner,
    value: Value,
    location: Location,
) -> IResult<(im::Vector<Value>, Type)> {
    match value {
        Value::Slice(values, typ) => Ok((values, typ)),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Slice(type_var);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

pub(crate) fn get_field(value: Value, location: Location) -> IResult<FieldElement> {
    match value {
        Value::Field(value) => Ok(value),
        value => {
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected: Type::FieldElement, actual, location })
        }
    }
}

pub(crate) fn get_u32(value: Value, location: Location) -> IResult<u32> {
    match value {
        Value::U32(value) => Ok(value),
        value => {
            let expected = Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

pub(crate) fn get_function_def(value: Value, location: Location) -> IResult<FuncId> {
    match value {
        Value::FunctionDefinition(id) => Ok(id),
        value => {
            let expected = Type::Quoted(QuotedType::FunctionDefinition);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

pub(crate) fn get_trait_constraint(
    value: Value,
    location: Location,
) -> IResult<(TraitId, Vec<Type>)> {
    match value {
        Value::TraitConstraint(trait_id, generics) => Ok((trait_id, generics)),
        value => {
            let expected = Type::Quoted(QuotedType::TraitConstraint);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

pub(crate) fn get_trait_def(value: Value, location: Location) -> IResult<TraitId> {
    match value {
        Value::TraitDefinition(id) => Ok(id),
        value => {
            let expected = Type::Quoted(QuotedType::TraitDefinition);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

pub(crate) fn get_type(value: Value, location: Location) -> IResult<Type> {
    match value {
        Value::Type(typ) => Ok(typ),
        value => {
            let expected = Type::Quoted(QuotedType::Type);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

pub(crate) fn get_quoted(value: Value, location: Location) -> IResult<Rc<Vec<Token>>> {
    match value {
        Value::Quoted(tokens) => Ok(tokens),
        value => {
            let expected = Type::Quoted(QuotedType::Quoted);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
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
