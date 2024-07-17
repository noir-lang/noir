use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

use chumsky::Parser;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::{Location, Span};
use rustc_hash::FxHashMap as HashMap;

use crate::{
    ast::{IntegerBitSize, TraitBound},
    hir::comptime::{errors::IResult, InterpreterError, Value},
    macros_api::{NodeInterner, Path, Signedness, UnresolvedTypeData},
    node_interner::{FuncId, TraitId},
    parser,
    token::{SpannedToken, Token, Tokens},
    QuotedType, Shared, Type,
};

pub(super) fn call_builtin(
    interner: &mut NodeInterner,
    name: &str,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    match name {
        "array_len" => array_len(interner, arguments, location),
        "as_slice" => as_slice(interner, arguments, location),
        "is_unconstrained" => Ok(Value::Bool(true)),
        "slice_insert" => slice_insert(interner, arguments, location),
        "slice_pop_back" => slice_pop_back(interner, arguments, location),
        "slice_pop_front" => slice_pop_front(interner, arguments, location),
        "slice_push_back" => slice_push_back(interner, arguments, location),
        "slice_push_front" => slice_push_front(interner, arguments, location),
        "slice_remove" => slice_remove(interner, arguments, location),
        "struct_def_as_type" => struct_def_as_type(interner, arguments, location),
        "struct_def_fields" => struct_def_fields(interner, arguments, location),
        "struct_def_generics" => struct_def_generics(interner, arguments, location),
        "trait_constraint_eq" => trait_constraint_eq(interner, arguments, location),
        "trait_constraint_hash" => trait_constraint_hash(interner, arguments, location),
        "quoted_as_trait_constraint" => quoted_as_trait_constraint(interner, arguments, location),
        "zeroed" => zeroed(interner, return_type, location),
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

fn failing_constraint<T>(message: impl Into<String>, location: Location) -> IResult<T> {
    let message = Some(Value::String(Rc::new(message.into())));
    Err(InterpreterError::FailingConstraint { message, location })
}

fn get_slice(
    interner: &NodeInterner,
    value: Value,
    location: Location,
) -> IResult<(im::Vector<Value>, Type)> {
    match value {
        Value::Slice(values, typ) => Ok((values, typ)),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Slice(type_var);
            Err(InterpreterError::TypeMismatch { expected, value, location })
        }
    }
}

fn get_u32(value: Value, location: Location) -> IResult<u32> {
    match value {
        Value::U32(value) => Ok(value),
        value => {
            let expected = Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo);
            Err(InterpreterError::TypeMismatch { expected, value, location })
        }
    }
}

fn get_trait_constraint(value: Value, location: Location) -> IResult<TraitBound> {
    match value {
        Value::TraitConstraint(bound) => Ok(bound),
        value => {
            let expected = Type::Quoted(QuotedType::TraitConstraint);
            Err(InterpreterError::TypeMismatch { expected, value, location })
        }
    }
}

fn get_quoted(value: Value, location: Location) -> IResult<Rc<Tokens>> {
    match value {
        Value::Code(tokens) => Ok(tokens),
        value => {
            let expected = Type::Quoted(QuotedType::Quoted);
            Err(InterpreterError::TypeMismatch { expected, value, location })
        }
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
    let (mut values, typ) = get_slice(interner, arguments.pop().unwrap().0, location)?;
    values.push_back(element);
    Ok(Value::Slice(values, typ))
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

fn slice_remove(
    interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(2, &arguments, location)?;

    let index = get_u32(arguments.pop().unwrap().0, location)? as usize;
    let (mut values, typ) = get_slice(interner, arguments.pop().unwrap().0, location)?;

    if values.is_empty() {
        return failing_constraint("slice_remove called on empty slice", location);
    }

    if index >= values.len() {
        let message = format!(
            "slice_remove: index {index} is out of bounds for a slice of length {}",
            values.len()
        );
        return failing_constraint(message, location);
    }

    let element = values.remove(index);
    Ok(Value::Tuple(vec![Value::Slice(values, typ), element]))
}

fn slice_push_front(
    interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(2, &arguments, location)?;

    let (element, _) = arguments.pop().unwrap();
    let (mut values, typ) = get_slice(interner, arguments.pop().unwrap().0, location)?;
    values.push_front(element);
    Ok(Value::Slice(values, typ))
}

fn slice_pop_front(
    interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    let (mut values, typ) = get_slice(interner, arguments.pop().unwrap().0, location)?;
    match values.pop_front() {
        Some(element) => Ok(Value::Tuple(vec![element, Value::Slice(values, typ)])),
        None => failing_constraint("slice_pop_front called on empty slice", location),
    }
}

fn slice_pop_back(
    interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    let (mut values, typ) = get_slice(interner, arguments.pop().unwrap().0, location)?;
    match values.pop_back() {
        Some(element) => Ok(Value::Tuple(vec![Value::Slice(values, typ), element])),
        None => failing_constraint("slice_pop_back called on empty slice", location),
    }
}

fn slice_insert(
    interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(3, &arguments, location)?;

    let (element, _) = arguments.pop().unwrap();
    let index = get_u32(arguments.pop().unwrap().0, location)?;
    let (mut values, typ) = get_slice(interner, arguments.pop().unwrap().0, location)?;
    values.insert(index as usize, element);
    Ok(Value::Slice(values, typ))
}

// fn as_trait_constraint(quoted: Quoted) -> TraitConstraint
fn quoted_as_trait_constraint(
    _interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    let tokens = get_quoted(arguments.pop().unwrap().0, location)?;
    let quoted = tokens.as_ref().clone();

    let trait_bound = parser::trait_bound().parse(quoted).map_err(|mut errors| {
        let error = errors.swap_remove(0);
        let rule = "a trait constraint";
        InterpreterError::FailedToParseMacro { error, tokens, rule, file: location.file }
    })?;

    Ok(Value::TraitConstraint(trait_bound))
}

// fn constraint_hash(constraint: TraitConstraint) -> Field
fn trait_constraint_hash(
    _interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    let bound = get_trait_constraint(arguments.pop().unwrap().0, location)?;

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bound.hash(&mut hasher);
    let hash = hasher.finish();

    Ok(Value::Field((hash as u128).into()))
}

// fn constraint_eq(constraint_a: TraitConstraint, constraint_b: TraitConstraint) -> bool
fn trait_constraint_eq(
    _interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(2, &arguments, location)?;

    let constraint_b = get_trait_constraint(arguments.pop().unwrap().0, location)?;
    let constraint_a = get_trait_constraint(arguments.pop().unwrap().0, location)?;

    Ok(Value::Bool(constraint_a == constraint_b))
}

// fn zeroed<T>() -> T
fn zeroed(_interner: &mut NodeInterner, return_type: Type, location: Location) -> IResult<Value> {
    match return_type {
        Type::FieldElement => Ok(Value::Field(0u128.into())),
        Type::Array(length_type, elem) => {
            if let Some(length) = length_type.evaluate_to_u32() {
                let element = zeroed(_interner, elem.as_ref().clone(), location)?;
                let array = std::iter::repeat(element).take(length as usize).collect();
                Ok(Value::Array(array, Type::Array(length_type, elem)))
            } else {
                todo!()
            }
        }
        Type::Slice(_) => Ok(Value::Slice(im::Vector::new(), return_type)),
        Type::Integer(sign, bits) => match (sign, bits) {
            (Signedness::Unsigned, IntegerBitSize::One) => Ok(Value::U8(0)),
            (Signedness::Unsigned, IntegerBitSize::Eight) => Ok(Value::U8(0)),
            (Signedness::Unsigned, IntegerBitSize::Sixteen) => Ok(Value::U16(0)),
            (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => Ok(Value::U32(0)),
            (Signedness::Unsigned, IntegerBitSize::SixtyFour) => Ok(Value::U64(0)),
            (Signedness::Signed, IntegerBitSize::One) => Ok(Value::I8(0)),
            (Signedness::Signed, IntegerBitSize::Eight) => Ok(Value::I8(0)),
            (Signedness::Signed, IntegerBitSize::Sixteen) => Ok(Value::I16(0)),
            (Signedness::Signed, IntegerBitSize::ThirtyTwo) => Ok(Value::I32(0)),
            (Signedness::Signed, IntegerBitSize::SixtyFour) => Ok(Value::I64(0)),
        },
        Type::Bool => Ok(Value::Bool(false)),
        Type::String(_) => todo!(),
        Type::FmtString(_, _) => todo!(),
        Type::Unit => Ok(Value::Unit),
        Type::Tuple(fields) => {
            Ok(Value::Tuple(try_vecmap(fields, |field| zeroed(_interner, field, location))?))
        }
        Type::Struct(struct_type, generics) => {
            let fields = struct_type.borrow().get_fields(&generics);
            let mut values = HashMap::default();

            for (field_name, field_type) in fields {
                let field_value = zeroed(_interner, field_type, location)?;
                values.insert(Rc::new(field_name), field_value);
            }

            let typ = Type::Struct(struct_type, generics);
            Ok(Value::Struct(values, typ))
        }
        Type::Alias(alias, generics) => {
            zeroed(_interner, alias.borrow().get_type(&generics), location)
        }
        Type::TypeVariable(_, _) => Ok(Value::Zeroed(return_type)),
        Type::TraitAsType(_, _, _) => todo!(),
        Type::NamedGeneric(_, _, _) => todo!(),
        Type::Function(_, _, _) => {
            Ok(Value::Function(FuncId::dummy_id(), Type::Unit, Default::default()))
        }
        Type::MutableReference(element) => {
            let element = zeroed(_interner, *element, location)?;
            Ok(Value::Pointer(Shared::new(element), false))
        }
        Type::Forall(_, _) => todo!(),
        Type::Constant(_) => todo!(),
        Type::Quoted(QuotedType::TraitConstraint) => Ok(Value::TraitConstraint(TraitBound {
            trait_path: Path::from_single(String::new(), Span::default()),
            trait_id: None,
            trait_generics: Vec::new(),
        })),
        Type::Quoted(_) => todo!(),
        Type::Error => todo!(),
    }
}
