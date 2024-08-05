use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

use acvm::{AcirField, FieldElement};
use chumsky::Parser;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;
use rustc_hash::FxHashMap as HashMap;

use crate::{
    ast::IntegerBitSize,
    hir::comptime::{errors::IResult, value::add_token_spans, InterpreterError, Value},
    macros_api::{NodeInterner, Signedness},
    node_interner::TraitId,
    parser,
    token::Token,
    QuotedType, Shared, Type,
};

use super::Interpreter;

impl<'local, 'context> Interpreter<'local, 'context> {
    pub(super) fn call_builtin(
        &mut self,
        name: &str,
        arguments: Vec<(Value, Location)>,
        return_type: Type,
        location: Location,
    ) -> IResult<Value> {
        let interner = &mut self.elaborator.interner;
        match name {
            "array_len" => array_len(interner, arguments, location),
            "as_slice" => as_slice(interner, arguments, location),
            "is_unconstrained" => Ok(Value::Bool(true)),
            "modulus_be_bits" => modulus_be_bits(interner, arguments, location),
            "modulus_be_bytes" => modulus_be_bytes(interner, arguments, location),
            "modulus_le_bits" => modulus_le_bits(interner, arguments, location),
            "modulus_le_bytes" => modulus_le_bytes(interner, arguments, location),
            "modulus_num_bits" => modulus_num_bits(interner, arguments, location),
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
            "trait_def_as_trait_constraint" => {
                trait_def_as_trait_constraint(interner, arguments, location)
            }
            "trait_def_eq" => trait_def_eq(interner, arguments, location),
            "trait_def_hash" => trait_def_hash(interner, arguments, location),
            "quoted_as_trait_constraint" => quoted_as_trait_constraint(self, arguments, location),
            "quoted_as_type" => quoted_as_type(self, arguments, location),
            "type_as_array" => type_as_array(arguments, return_type, location),
            "type_as_constant" => type_as_constant(arguments, return_type, location),
            "type_as_integer" => type_as_integer(arguments, return_type, location),
            "type_as_slice" => type_as_slice(arguments, return_type, location),
            "type_as_struct" => type_as_struct(arguments, return_type, location),
            "type_as_tuple" => type_as_tuple(arguments, return_type, location),
            "type_eq" => type_eq(arguments, location),
            "type_is_bool" => type_is_bool(arguments, location),
            "type_is_field" => type_is_field(arguments, location),
            "type_of" => type_of(arguments, location),
            "zeroed" => zeroed(return_type),
            _ => {
                let item = format!("Comptime evaluation for builtin function {name}");
                Err(InterpreterError::Unimplemented { item, location })
            }
        }
    }
}

pub(super) fn check_argument_count(
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

pub(super) fn check_one_argument(
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    Ok(arguments.pop().unwrap().0)
}

pub(super) fn check_two_arguments(
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<(Value, Value)> {
    check_argument_count(2, &arguments, location)?;

    let argument2 = arguments.pop().unwrap().0;
    let argument1 = arguments.pop().unwrap().0;

    Ok((argument1, argument2))
}

pub(super) fn check_three_arguments(
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<(Value, Value, Value)> {
    check_argument_count(3, &arguments, location)?;

    let argument3 = arguments.pop().unwrap().0;
    let argument2 = arguments.pop().unwrap().0;
    let argument1 = arguments.pop().unwrap().0;

    Ok((argument1, argument2, argument3))
}

fn failing_constraint<T>(message: impl Into<String>, location: Location) -> IResult<T> {
    Err(InterpreterError::FailingConstraint { message: Some(message.into()), location })
}

pub(super) fn get_array(
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
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

pub(super) fn get_field(value: Value, location: Location) -> IResult<FieldElement> {
    match value {
        Value::Field(value) => Ok(value),
        value => {
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected: Type::FieldElement, actual, location })
        }
    }
}

pub(super) fn get_u32(value: Value, location: Location) -> IResult<u32> {
    match value {
        Value::U32(value) => Ok(value),
        value => {
            let expected = Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

fn get_trait_constraint(value: Value, location: Location) -> IResult<(TraitId, Vec<Type>)> {
    match value {
        Value::TraitConstraint(trait_id, generics) => Ok((trait_id, generics)),
        value => {
            let expected = Type::Quoted(QuotedType::TraitConstraint);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

fn get_trait_def(value: Value, location: Location) -> IResult<TraitId> {
    match value {
        Value::TraitDefinition(id) => Ok(id),
        value => {
            let expected = Type::Quoted(QuotedType::TraitDefinition);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

fn get_type(value: Value, location: Location) -> IResult<Type> {
    match value {
        Value::Type(typ) => Ok(typ),
        value => {
            let expected = Type::Quoted(QuotedType::Type);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

fn get_quoted(value: Value, location: Location) -> IResult<Rc<Vec<Token>>> {
    match value {
        Value::Quoted(tokens) => Ok(tokens),
        value => {
            let expected = Type::Quoted(QuotedType::Quoted);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

fn array_len(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    match argument {
        Value::Array(values, _) | Value::Slice(values, _) => Ok(Value::U32(values.len() as u32)),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Array(type_var.clone(), type_var);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

fn as_slice(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let array = check_one_argument(arguments, location)?;

    match array {
        Value::Array(values, Type::Array(_, typ)) => Ok(Value::Slice(values, Type::Slice(typ))),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Array(type_var.clone(), type_var);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location })
        }
    }
}

fn slice_push_back(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (slice, element) = check_two_arguments(arguments, location)?;

    let (mut values, typ) = get_slice(interner, slice, location)?;
    values.push_back(element);
    Ok(Value::Slice(values, typ))
}

/// fn as_type(self) -> Type
fn struct_def_as_type(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let struct_def = match argument {
        Value::StructDefinition(id) => id,
        value => {
            let expected = Type::Quoted(QuotedType::StructDefinition);
            let actual = value.get_type().into_owned();
            return Err(InterpreterError::TypeMismatch { expected, location, actual });
        }
    };

    let struct_def_rc = interner.get_struct(struct_def);
    let struct_def = struct_def_rc.borrow();

    let generics = vecmap(&struct_def.generics, |generic| {
        Type::NamedGeneric(generic.type_var.clone(), generic.name.clone(), generic.kind.clone())
    });

    drop(struct_def);
    Ok(Value::Type(Type::Struct(struct_def_rc, generics)))
}

/// fn generics(self) -> [Type]
fn struct_def_generics(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let struct_def = match argument {
        Value::StructDefinition(id) => id,
        value => {
            let expected = Type::Quoted(QuotedType::StructDefinition);
            let actual = value.get_type().into_owned();
            return Err(InterpreterError::TypeMismatch { expected, location, actual });
        }
    };

    let struct_def = interner.get_struct(struct_def);
    let struct_def = struct_def.borrow();

    let generics =
        struct_def.generics.iter().map(|generic| Value::Type(generic.clone().as_named_generic()));

    let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Type)));
    Ok(Value::Slice(generics.collect(), typ))
}

/// fn fields(self) -> [(Quoted, Type)]
/// Returns (name, type) pairs of each field of this StructDefinition
fn struct_def_fields(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let struct_def = match argument {
        Value::StructDefinition(id) => id,
        value => {
            let expected = Type::Quoted(QuotedType::StructDefinition);
            let actual = value.get_type().into_owned();
            return Err(InterpreterError::TypeMismatch { expected, location, actual });
        }
    };

    let struct_def = interner.get_struct(struct_def);
    let struct_def = struct_def.borrow();

    let mut fields = im::Vector::new();

    for (name, typ) in struct_def.get_fields_as_written() {
        let name = Value::Quoted(Rc::new(vec![Token::Ident(name)]));
        let typ = Value::Type(typ);
        fields.push_back(Value::Tuple(vec![name, typ]));
    }

    let typ = Type::Slice(Box::new(Type::Tuple(vec![
        Type::Quoted(QuotedType::Quoted),
        Type::Quoted(QuotedType::Type),
    ])));
    Ok(Value::Slice(fields, typ))
}

fn slice_remove(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (slice, index) = check_two_arguments(arguments, location)?;

    let index = get_u32(index, location)? as usize;
    let (mut values, typ) = get_slice(interner, slice, location)?;

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
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (slice, element) = check_two_arguments(arguments, location)?;

    let (mut values, typ) = get_slice(interner, slice, location)?;
    values.push_front(element);
    Ok(Value::Slice(values, typ))
}

fn slice_pop_front(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let (mut values, typ) = get_slice(interner, argument, location)?;
    match values.pop_front() {
        Some(element) => Ok(Value::Tuple(vec![element, Value::Slice(values, typ)])),
        None => failing_constraint("slice_pop_front called on empty slice", location),
    }
}

fn slice_pop_back(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let (mut values, typ) = get_slice(interner, argument, location)?;
    match values.pop_back() {
        Some(element) => Ok(Value::Tuple(vec![Value::Slice(values, typ), element])),
        None => failing_constraint("slice_pop_back called on empty slice", location),
    }
}

fn slice_insert(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (slice, index, element) = check_three_arguments(arguments, location)?;

    let index = get_u32(index, location)? as usize;
    let (mut values, typ) = get_slice(interner, slice, location)?;
    values.insert(index, element);
    Ok(Value::Slice(values, typ))
}

// fn as_trait_constraint(quoted: Quoted) -> TraitConstraint
fn quoted_as_trait_constraint(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let tokens = get_quoted(argument, location)?;
    let quoted = add_token_spans(tokens.clone(), location.span);

    let trait_bound = parser::trait_bound().parse(quoted).map_err(|mut errors| {
        let error = errors.swap_remove(0);
        let rule = "a trait constraint";
        InterpreterError::FailedToParseMacro { error, tokens, rule, file: location.file }
    })?;

    let bound = interpreter
        .elaborate_item(interpreter.current_function, |elaborator| {
            elaborator.resolve_trait_bound(&trait_bound, Type::Unit)
        })
        .ok_or(InterpreterError::FailedToResolveTraitBound { trait_bound, location })?;

    Ok(Value::TraitConstraint(bound.trait_id, bound.trait_generics))
}

// fn as_type(quoted: Quoted) -> Type
fn quoted_as_type(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let tokens = get_quoted(argument, location)?;
    let quoted = add_token_spans(tokens.clone(), location.span);

    let typ = parser::parse_type().parse(quoted).map_err(|mut errors| {
        let error = errors.swap_remove(0);
        let rule = "a type";
        InterpreterError::FailedToParseMacro { error, tokens, rule, file: location.file }
    })?;

    let typ =
        interpreter.elaborate_item(interpreter.current_function, |elab| elab.resolve_type(typ));

    Ok(Value::Type(typ))
}

// fn as_array(self) -> Option<(Type, Type)>
fn type_as_array(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    type_as(arguments, return_type, location, |typ| {
        if let Type::Array(length, array_type) = typ {
            Some(Value::Tuple(vec![Value::Type(*array_type), Value::Type(*length)]))
        } else {
            None
        }
    })
}

// fn as_constant(self) -> Option<u32>
fn type_as_constant(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    type_as(arguments, return_type, location, |typ| {
        if let Type::Constant(n) = typ {
            Some(Value::U32(n))
        } else {
            None
        }
    })
}

// fn as_integer(self) -> Option<(bool, u8)>
fn type_as_integer(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    type_as(arguments, return_type, location, |typ| {
        if let Type::Integer(sign, bits) = typ {
            Some(Value::Tuple(vec![Value::Bool(sign.is_signed()), Value::U8(bits.bit_size())]))
        } else {
            None
        }
    })
}

// fn as_slice(self) -> Option<Type>
fn type_as_slice(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    type_as(arguments, return_type, location, |typ| {
        if let Type::Slice(slice_type) = typ {
            Some(Value::Type(*slice_type))
        } else {
            None
        }
    })
}

// fn as_struct(self) -> Option<(StructDefinition, [Type])>
fn type_as_struct(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    type_as(arguments, return_type, location, |typ| {
        if let Type::Struct(struct_type, generics) = typ {
            Some(Value::Tuple(vec![
                Value::StructDefinition(struct_type.borrow().id),
                Value::Slice(
                    generics.into_iter().map(Value::Type).collect(),
                    Type::Slice(Box::new(Type::Quoted(QuotedType::Type))),
                ),
            ]))
        } else {
            None
        }
    })
}

// fn as_tuple(self) -> Option<[Type]>
fn type_as_tuple(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    type_as(arguments, return_type.clone(), location, |typ| {
        if let Type::Tuple(types) = typ {
            let t = extract_option_generic_type(return_type);

            let Type::Slice(slice_type) = t else {
                panic!("Expected T to be a slice");
            };

            Some(Value::Slice(types.into_iter().map(Value::Type).collect(), *slice_type))
        } else {
            None
        }
    })
}

// Helper function for implementing the `type_as_...` functions.
fn type_as<F>(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
    f: F,
) -> IResult<Value>
where
    F: FnOnce(Type) -> Option<Value>,
{
    let value = check_one_argument(arguments, location)?;
    let typ = get_type(value, location)?;

    let option_value = f(typ);

    option(return_type, option_value)
}

// fn type_eq(_first: Type, _second: Type) -> bool
fn type_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let (self_type, other_type) = check_two_arguments(arguments, location)?;

    Ok(Value::Bool(self_type == other_type))
}

// fn is_bool(self) -> bool
fn type_is_bool(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let value = check_one_argument(arguments, location)?;
    let typ = get_type(value, location)?;

    Ok(Value::Bool(matches!(typ, Type::Bool)))
}

// fn is_field(self) -> bool
fn type_is_field(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let value = check_one_argument(arguments, location)?;
    let typ = get_type(value, location)?;

    Ok(Value::Bool(matches!(typ, Type::FieldElement)))
}

// fn type_of<T>(x: T) -> Type
fn type_of(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let value = check_one_argument(arguments, location)?;
    let typ = value.get_type().into_owned();
    Ok(Value::Type(typ))
}

// fn constraint_hash(constraint: TraitConstraint) -> Field
fn trait_constraint_hash(
    _interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let bound = get_trait_constraint(argument, location)?;

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bound.hash(&mut hasher);
    let hash = hasher.finish();

    Ok(Value::Field((hash as u128).into()))
}

// fn constraint_eq(constraint_a: TraitConstraint, constraint_b: TraitConstraint) -> bool
fn trait_constraint_eq(
    _interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (value_a, value_b) = check_two_arguments(arguments, location)?;

    let constraint_a = get_trait_constraint(value_a, location)?;
    let constraint_b = get_trait_constraint(value_b, location)?;

    Ok(Value::Bool(constraint_a == constraint_b))
}

// fn trait_def_hash(def: TraitDefinition) -> Field
fn trait_def_hash(
    _interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(1, &arguments, location)?;

    let id = get_trait_def(arguments.pop().unwrap().0, location)?;

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    let hash = hasher.finish();

    Ok(Value::Field((hash as u128).into()))
}

// fn trait_def_eq(def_a: TraitDefinition, def_b: TraitDefinition) -> bool
fn trait_def_eq(
    _interner: &mut NodeInterner,
    mut arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(2, &arguments, location)?;

    let id_b = get_trait_def(arguments.pop().unwrap().0, location)?;
    let id_a = get_trait_def(arguments.pop().unwrap().0, location)?;

    Ok(Value::Bool(id_a == id_b))
}

// fn zeroed<T>() -> T
fn zeroed(return_type: Type) -> IResult<Value> {
    match return_type {
        Type::FieldElement => Ok(Value::Field(0u128.into())),
        Type::Array(length_type, elem) => {
            if let Some(length) = length_type.evaluate_to_u32() {
                let element = zeroed(elem.as_ref().clone())?;
                let array = std::iter::repeat(element).take(length as usize).collect();
                Ok(Value::Array(array, Type::Array(length_type, elem)))
            } else {
                // Assume we can resolve the length later
                Ok(Value::Zeroed(Type::Array(length_type, elem)))
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
        Type::String(length_type) => {
            if let Some(length) = length_type.evaluate_to_u32() {
                Ok(Value::String(Rc::new("\0".repeat(length as usize))))
            } else {
                // Assume we can resolve the length later
                Ok(Value::Zeroed(Type::String(length_type)))
            }
        }
        Type::FmtString(length_type, captures) => {
            let length = length_type.evaluate_to_u32();
            let typ = Type::FmtString(length_type, captures);
            if let Some(length) = length {
                Ok(Value::FormatString(Rc::new("\0".repeat(length as usize)), typ))
            } else {
                // Assume we can resolve the length later
                Ok(Value::Zeroed(typ))
            }
        }
        Type::Unit => Ok(Value::Unit),
        Type::Tuple(fields) => Ok(Value::Tuple(try_vecmap(fields, zeroed)?)),
        Type::Struct(struct_type, generics) => {
            let fields = struct_type.borrow().get_fields(&generics);
            let mut values = HashMap::default();

            for (field_name, field_type) in fields {
                let field_value = zeroed(field_type)?;
                values.insert(Rc::new(field_name), field_value);
            }

            let typ = Type::Struct(struct_type, generics);
            Ok(Value::Struct(values, typ))
        }
        Type::Alias(alias, generics) => zeroed(alias.borrow().get_type(&generics)),
        typ @ Type::Function(..) => {
            // Using Value::Zeroed here is probably safer than using FuncId::dummy_id() or similar
            Ok(Value::Zeroed(typ))
        }
        Type::MutableReference(element) => {
            let element = zeroed(*element)?;
            Ok(Value::Pointer(Shared::new(element), false))
        }
        // Optimistically assume we can resolve this type later or that the value is unused
        Type::TypeVariable(_, _)
        | Type::Forall(_, _)
        | Type::Constant(_)
        | Type::InfixExpr(..)
        | Type::Quoted(_)
        | Type::Error
        | Type::TraitAsType(_, _, _)
        | Type::NamedGeneric(_, _, _) => Ok(Value::Zeroed(return_type)),
    }
}

fn modulus_be_bits(
    _interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(0, &arguments, location)?;

    let bits = FieldElement::modulus().to_radix_be(2);
    let bits_vector = bits.into_iter().map(|bit| Value::U1(bit != 0)).collect();

    let int_type = Type::Integer(crate::ast::Signedness::Unsigned, IntegerBitSize::One);
    let typ = Type::Slice(Box::new(int_type));
    Ok(Value::Slice(bits_vector, typ))
}

fn modulus_be_bytes(
    _interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(0, &arguments, location)?;

    let bytes = FieldElement::modulus().to_bytes_be();
    let bytes_vector = bytes.into_iter().map(Value::U8).collect();

    let int_type = Type::Integer(crate::ast::Signedness::Unsigned, IntegerBitSize::Eight);
    let typ = Type::Slice(Box::new(int_type));
    Ok(Value::Slice(bytes_vector, typ))
}

fn modulus_le_bits(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let Value::Slice(bits, typ) = modulus_be_bits(interner, arguments, location)? else {
        unreachable!("modulus_be_bits must return slice")
    };
    let reversed_bits = bits.into_iter().rev().collect();
    Ok(Value::Slice(reversed_bits, typ))
}

fn modulus_le_bytes(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let Value::Slice(bytes, typ) = modulus_be_bytes(interner, arguments, location)? else {
        unreachable!("modulus_be_bytes must return slice")
    };
    let reversed_bytes = bytes.into_iter().rev().collect();
    Ok(Value::Slice(reversed_bytes, typ))
}

fn modulus_num_bits(
    _interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    check_argument_count(0, &arguments, location)?;
    let bits = FieldElement::max_num_bits().into();
    Ok(Value::U64(bits))
}

fn trait_def_as_trait_constraint(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> Result<Value, InterpreterError> {
    let argument = check_one_argument(arguments, location)?;

    let trait_id = get_trait_def(argument, location)?;
    let the_trait = interner.get_trait(trait_id);
    let trait_generics = vecmap(&the_trait.generics, |generic| {
        Type::NamedGeneric(generic.type_var.clone(), generic.name.clone(), generic.kind.clone())
    });

    Ok(Value::TraitConstraint(trait_id, trait_generics))
}

/// Creates a value that holds an `Option`.
/// `option_type` must be a Type referencing the `Option` type.
pub(crate) fn option(option_type: Type, value: Option<Value>) -> IResult<Value> {
    let t = extract_option_generic_type(option_type.clone());

    let (is_some, value) = match value {
        Some(value) => (Value::Bool(true), value),
        None => (Value::Bool(false), zeroed(t)?),
    };

    let mut fields = HashMap::default();
    fields.insert(Rc::new("_is_some".to_string()), is_some);
    fields.insert(Rc::new("_value".to_string()), value);
    Ok(Value::Struct(fields, option_type))
}

/// Given a type, assert that it's an Option<T> and return the Type for T
pub(crate) fn extract_option_generic_type(typ: Type) -> Type {
    let Type::Struct(struct_type, mut generics) = typ else {
        panic!("Expected type to be a struct");
    };

    let struct_type = struct_type.borrow();
    assert_eq!(struct_type.name.0.contents, "Option");

    generics.pop().expect("Expected Option to have a T generic type")
}
