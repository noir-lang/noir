use std::rc::Rc;

use noirc_errors::Location;

use crate::{
    hir::comptime::{errors::IResult, InterpreterError, Value},
    macros_api::NodeInterner,
    token::{SpannedToken, Token, Tokens},
    QuotedType, Type,
};

pub(super) fn call_builtin(
    interner: &mut NodeInterner,
    name: &str,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    match name {
        "array_len" => array_len(interner, arguments, location),
        "as_slice" => as_slice(interner, arguments, location),
        "slice_push_back" => slice_push_back(interner, arguments, location),
        "struct_def_as_type" => struct_def_as_type(interner, arguments, location),
        "struct_def_generics" => struct_def_generics(interner, arguments, location),
        "struct_def_fields" => struct_def_fields(interner, arguments, location),
        _ => {
            let item = format!("Comptime evaluation for builtin function {name}");
            Err(InterpreterError::Unimplemented { item, location })
        }
    }
}

fn check_argument_count(
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

fn array_len(
    interner: &NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    match arguments.pop().unwrap().0 {
        Value::Array(values, _) | Value::Slice(values, _) => Ok(Value::U32(values.len() as u32)),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Array(type_var.clone(), type_var);
            Err(InterpreterError::TypeMismatch { expected, value, location })
        }
    }
}

fn as_slice(
    interner: &NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    let (array, _) = arguments.pop().unwrap();
    match array {
        Value::Array(values, Type::Array(_, typ)) => Ok(Value::Slice(values, Type::Slice(typ))),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Array(type_var.clone(), type_var);
            Err(InterpreterError::TypeMismatch { expected, value, location })
        }
    }
}

fn slice_push_back(
    interner: &NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(2, &arguments, location)?;

    let (element, _) = arguments.pop().unwrap();
    let (slice, _) = arguments.pop().unwrap();
    match slice {
        Value::Slice(mut values, typ) => {
            values.push_back(element);
            Ok(Value::Slice(values, typ))
        }
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Slice(type_var);
            Err(InterpreterError::TypeMismatch { expected, value, location })
        }
    }
}

/// fn as_type(self) -> Quoted
fn struct_def_as_type(
    interner: &NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    let (struct_def, span) = match arguments.pop().unwrap() {
        (Value::StructDefinition(id), location) => (id, location.span),
        value => {
            let expected = Type::Quoted(QuotedType::StructDefinition);
            return Err(InterpreterError::TypeMismatch { expected, location, value: value.0 });
        }
    };

    let struct_def = interner.get_struct(struct_def);
    let struct_def = struct_def.borrow();
    let make_token = |name| SpannedToken::new(Token::Ident(name), span);

    let mut tokens = vec![make_token(struct_def.name.to_string())];

    for (i, generic) in struct_def.generics.iter().enumerate() {
        if i != 0 {
            tokens.push(SpannedToken::new(Token::Comma, span));
        }
        tokens.push(make_token(generic.type_var.borrow().to_string()));
    }

    Ok(Value::Code(Rc::new(Tokens(tokens))))
}

/// fn generics(self) -> [Quoted]
fn struct_def_generics(
    interner: &NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    let (struct_def, span) = match arguments.pop().unwrap() {
        (Value::StructDefinition(id), location) => (id, location.span),
        value => {
            let expected = Type::Quoted(QuotedType::StructDefinition);
            return Err(InterpreterError::TypeMismatch { expected, location, value: value.0 });
        }
    };

    let struct_def = interner.get_struct(struct_def);
    let struct_def = struct_def.borrow();

    let generics = struct_def.generics.iter().map(|generic| {
        let name = SpannedToken::new(Token::Ident(generic.type_var.borrow().to_string()), span);
        Value::Code(Rc::new(Tokens(vec![name])))
    });

    let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Quoted)));
    Ok(Value::Slice(generics.collect(), typ))
}

/// fn fields(self) -> [(Quoted, Quoted)]
/// Returns (name, type) pairs of each field of this StructDefinition
fn struct_def_fields(
    interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    let (struct_def, span) = match arguments.pop().unwrap() {
        (Value::StructDefinition(id), location) => (id, location.span),
        value => {
            let expected = Type::Quoted(QuotedType::StructDefinition);
            return Err(InterpreterError::TypeMismatch { expected, location, value: value.0 });
        }
    };

    let struct_def = interner.get_struct(struct_def);
    let struct_def = struct_def.borrow();

    let make_token = |name| SpannedToken::new(Token::Ident(name), span);
    let make_quoted = |tokens| Value::Code(Rc::new(Tokens(tokens)));

    let mut fields = im::Vector::new();

    for (name, typ) in struct_def.get_fields_as_written() {
        let name = make_quoted(vec![make_token(name)]);
        let id = interner.push_quoted_type(typ);
        let typ = SpannedToken::new(Token::QuotedType(id), span);
        let typ = Value::Code(Rc::new(Tokens(vec![typ])));
        fields.push_back(Value::Tuple(vec![name, typ]));
    }

    let typ = Type::Slice(Box::new(Type::Tuple(vec![
        Type::Quoted(QuotedType::Quoted),
        Type::Quoted(QuotedType::Quoted),
    ])));
    Ok(Value::Slice(fields, typ))
}
