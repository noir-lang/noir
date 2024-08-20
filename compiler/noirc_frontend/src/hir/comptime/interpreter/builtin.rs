use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

use acvm::{AcirField, FieldElement};
use builtin_helpers::{
    check_argument_count, check_function_not_yet_resolved, check_one_argument,
    check_three_arguments, check_two_arguments, get_expr, get_function_def, get_module, get_quoted,
    get_slice, get_struct, get_trait_constraint, get_trait_def, get_trait_impl, get_tuple,
    get_type, get_u32, hir_pattern_to_tokens, mutate_func_meta_type, parse, parse_tokens,
    replace_func_meta_parameters, replace_func_meta_return_type,
};
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;
use rustc_hash::FxHashMap as HashMap;

use crate::{
    ast::{
        ArrayLiteral, ExpressionKind, FunctionKind, FunctionReturnType, IntegerBitSize, Literal,
        UnaryOp, UnresolvedType, UnresolvedTypeData, Visibility,
    },
    hir::comptime::{errors::IResult, value::add_token_spans, InterpreterError, Value},
    hir_def::function::FunctionBody,
    macros_api::{ModuleDefId, NodeInterner, Signedness},
    node_interner::{DefinitionKind, TraitImplKind},
    parser::{self},
    token::{SpannedToken, Token},
    QuotedType, Shared, Type,
};

use self::builtin_helpers::{get_array, get_u8};
use super::Interpreter;

pub(crate) mod builtin_helpers;

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
            "array_as_str_unchecked" => array_as_str_unchecked(interner, arguments, location),
            "array_len" => array_len(interner, arguments, location),
            "as_slice" => as_slice(interner, arguments, location),
            "expr_as_array" => expr_as_array(arguments, return_type, location),
            "expr_as_binary_op" => expr_as_binary_op(arguments, return_type, location),
            "expr_as_bool" => expr_as_bool(arguments, return_type, location),
            "expr_as_function_call" => expr_as_function_call(arguments, return_type, location),
            "expr_as_if" => expr_as_if(arguments, return_type, location),
            "expr_as_index" => expr_as_index(arguments, return_type, location),
            "expr_as_integer" => expr_as_integer(arguments, return_type, location),
            "expr_as_member_access" => expr_as_member_access(arguments, return_type, location),
            "expr_as_repeated_element_array" => {
                expr_as_repeated_element_array(arguments, return_type, location)
            }
            "expr_as_repeated_element_slice" => {
                expr_as_repeated_element_slice(arguments, return_type, location)
            }
            "expr_as_slice" => expr_as_slice(arguments, return_type, location),
            "expr_as_tuple" => expr_as_tuple(arguments, return_type, location),
            "expr_as_unary_op" => expr_as_unary_op(arguments, return_type, location),
            "is_unconstrained" => Ok(Value::Bool(true)),
            "function_def_name" => function_def_name(interner, arguments, location),
            "function_def_parameters" => function_def_parameters(interner, arguments, location),
            "function_def_return_type" => function_def_return_type(interner, arguments, location),
            "function_def_set_body" => function_def_set_body(self, arguments, location),
            "function_def_set_parameters" => function_def_set_parameters(self, arguments, location),
            "function_def_set_return_type" => {
                function_def_set_return_type(self, arguments, location)
            }
            "module_functions" => module_functions(self, arguments, location),
            "module_is_contract" => module_is_contract(self, arguments, location),
            "module_name" => module_name(interner, arguments, location),
            "modulus_be_bits" => modulus_be_bits(interner, arguments, location),
            "modulus_be_bytes" => modulus_be_bytes(interner, arguments, location),
            "modulus_le_bits" => modulus_le_bits(interner, arguments, location),
            "modulus_le_bytes" => modulus_le_bytes(interner, arguments, location),
            "modulus_num_bits" => modulus_num_bits(interner, arguments, location),
            "quoted_as_expr" => quoted_as_expr(arguments, return_type, location),
            "quoted_as_module" => quoted_as_module(self, arguments, return_type, location),
            "quoted_as_trait_constraint" => quoted_as_trait_constraint(self, arguments, location),
            "quoted_as_type" => quoted_as_type(self, arguments, location),
            "quoted_eq" => quoted_eq(arguments, location),
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
            "trait_impl_methods" => trait_impl_methods(interner, arguments, location),
            "trait_impl_trait_generic_args" => {
                trait_impl_trait_generic_args(interner, arguments, location)
            }
            "type_as_array" => type_as_array(arguments, return_type, location),
            "type_as_constant" => type_as_constant(arguments, return_type, location),
            "type_as_integer" => type_as_integer(arguments, return_type, location),
            "type_as_slice" => type_as_slice(arguments, return_type, location),
            "type_as_struct" => type_as_struct(arguments, return_type, location),
            "type_as_tuple" => type_as_tuple(arguments, return_type, location),
            "type_eq" => type_eq(arguments, location),
            "type_get_trait_impl" => {
                type_get_trait_impl(interner, arguments, return_type, location)
            }
            "type_implements" => type_implements(interner, arguments, location),
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

fn failing_constraint<T>(message: impl Into<String>, location: Location) -> IResult<T> {
    Err(InterpreterError::FailingConstraint { message: Some(message.into()), location })
}

fn array_len(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (argument, argument_location) = check_one_argument(arguments, location)?;

    match argument {
        Value::Array(values, _) | Value::Slice(values, _) => Ok(Value::U32(values.len() as u32)),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Array(type_var.clone(), type_var);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location: argument_location })
        }
    }
}

fn array_as_str_unchecked(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let array = get_array(interner, argument)?.0;
    let string_bytes = try_vecmap(array, |byte| get_u8((byte, location)))?;
    let string = String::from_utf8_lossy(&string_bytes).into_owned();
    Ok(Value::String(Rc::new(string)))
}

fn as_slice(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (array, array_location) = check_one_argument(arguments, location)?;

    match array {
        Value::Array(values, Type::Array(_, typ)) => Ok(Value::Slice(values, Type::Slice(typ))),
        value => {
            let type_var = Box::new(interner.next_type_variable());
            let expected = Type::Array(type_var.clone(), type_var);
            let actual = value.get_type().into_owned();
            Err(InterpreterError::TypeMismatch { expected, actual, location: array_location })
        }
    }
}

fn slice_push_back(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (slice, (element, _)) = check_two_arguments(arguments, location)?;

    let (mut values, typ) = get_slice(interner, slice)?;
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
    let struct_id = get_struct(argument)?;
    let struct_def_rc = interner.get_struct(struct_id);
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
    let struct_id = get_struct(argument)?;
    let struct_def = interner.get_struct(struct_id);
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
    let struct_id = get_struct(argument)?;
    let struct_def = interner.get_struct(struct_id);
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

    let (mut values, typ) = get_slice(interner, slice)?;
    let index = get_u32(index)? as usize;

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
    let (slice, (element, _)) = check_two_arguments(arguments, location)?;

    let (mut values, typ) = get_slice(interner, slice)?;
    values.push_front(element);
    Ok(Value::Slice(values, typ))
}

fn slice_pop_front(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let (mut values, typ) = get_slice(interner, argument)?;
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

    let (mut values, typ) = get_slice(interner, argument)?;
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
    let (slice, index, (element, _)) = check_three_arguments(arguments, location)?;

    let (mut values, typ) = get_slice(interner, slice)?;
    let index = get_u32(index)? as usize;
    values.insert(index, element);
    Ok(Value::Slice(values, typ))
}

// fn as_expr(quoted: Quoted) -> Option<Expr>
fn quoted_as_expr(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let expr = parse(argument, parser::expression(), "an expression").ok();
    let value = expr.map(|expr| Value::Expr(expr.kind));

    option(return_type, value)
}

// fn as_module(quoted: Quoted) -> Option<Module>
fn quoted_as_module(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let path = parse(argument, parser::path_no_turbofish(), "a path").ok();
    let option_value = path.and_then(|path| {
        let module = interpreter.elaborate_item(interpreter.current_function, |elaborator| {
            elaborator.resolve_module_by_path(path)
        });
        module.map(Value::ModuleDefinition)
    });

    option(return_type, option_value)
}

// fn as_trait_constraint(quoted: Quoted) -> TraitConstraint
fn quoted_as_trait_constraint(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;
    let trait_bound = parse(argument, parser::trait_bound(), "a trait constraint")?;
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
    let typ = parse(argument, parser::parse_type(), "a type")?;
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
    let typ = get_type(value)?;

    let option_value = f(typ);

    option(return_type, option_value)
}

// fn type_eq(_first: Type, _second: Type) -> bool
fn type_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let (self_type, other_type) = check_two_arguments(arguments, location)?;

    let self_type = get_type(self_type)?;
    let other_type = get_type(other_type)?;

    Ok(Value::Bool(self_type == other_type))
}

// fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl>
fn type_get_trait_impl(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let (typ, constraint) = check_two_arguments(arguments, location)?;

    let typ = get_type(typ)?;
    let (trait_id, generics) = get_trait_constraint(constraint)?;

    let option_value = match interner.try_lookup_trait_implementation(&typ, trait_id, &generics) {
        Ok((TraitImplKind::Normal(trait_impl_id), _)) => Some(Value::TraitImpl(trait_impl_id)),
        _ => None,
    };

    option(return_type, option_value)
}

// fn implements(self, constraint: TraitConstraint) -> bool
fn type_implements(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (typ, constraint) = check_two_arguments(arguments, location)?;

    let typ = get_type(typ)?;
    let (trait_id, generics) = get_trait_constraint(constraint)?;

    let implements = interner.try_lookup_trait_implementation(&typ, trait_id, &generics).is_ok();
    Ok(Value::Bool(implements))
}

// fn is_bool(self) -> bool
fn type_is_bool(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let value = check_one_argument(arguments, location)?;
    let typ = get_type(value)?;

    Ok(Value::Bool(matches!(typ, Type::Bool)))
}

// fn is_field(self) -> bool
fn type_is_field(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let value = check_one_argument(arguments, location)?;
    let typ = get_type(value)?;

    Ok(Value::Bool(matches!(typ, Type::FieldElement)))
}

// fn type_of<T>(x: T) -> Type
fn type_of(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let (value, _) = check_one_argument(arguments, location)?;
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

    let bound = get_trait_constraint(argument)?;

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

    let constraint_a = get_trait_constraint(value_a)?;
    let constraint_b = get_trait_constraint(value_b)?;

    Ok(Value::Bool(constraint_a == constraint_b))
}

// fn trait_def_hash(def: TraitDefinition) -> Field
fn trait_def_hash(
    _interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let id = get_trait_def(argument)?;

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    let hash = hasher.finish();

    Ok(Value::Field((hash as u128).into()))
}

// fn trait_def_eq(def_a: TraitDefinition, def_b: TraitDefinition) -> bool
fn trait_def_eq(
    _interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (id_a, id_b) = check_two_arguments(arguments, location)?;

    let id_a = get_trait_def(id_a)?;
    let id_b = get_trait_def(id_b)?;

    Ok(Value::Bool(id_a == id_b))
}

// fn methods(self) -> [FunctionDefinition]
fn trait_impl_methods(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let trait_impl_id = get_trait_impl(argument)?;
    let trait_impl = interner.get_trait_implementation(trait_impl_id);
    let trait_impl = trait_impl.borrow();
    let methods =
        trait_impl.methods.iter().map(|func_id| Value::FunctionDefinition(*func_id)).collect();
    let slice_type = Type::Slice(Box::new(Type::Quoted(QuotedType::FunctionDefinition)));

    Ok(Value::Slice(methods, slice_type))
}

// fn trait_generic_args(self) -> [Type]
fn trait_impl_trait_generic_args(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let trait_impl_id = get_trait_impl(argument)?;
    let trait_impl = interner.get_trait_implementation(trait_impl_id);
    let trait_impl = trait_impl.borrow();
    let trait_generics = trait_impl.trait_generics.iter().map(|t| Value::Type(t.clone())).collect();
    let slice_type = Type::Slice(Box::new(Type::Quoted(QuotedType::Type)));

    Ok(Value::Slice(trait_generics, slice_type))
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

// fn as_array(self) -> Option<[Expr]>
fn expr_as_array(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type, location, |expr| {
        if let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(exprs))) = expr {
            let exprs = exprs.into_iter().map(|expr| Value::Expr(expr.kind)).collect();
            let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Expr)));
            Some(Value::Slice(exprs, typ))
        } else {
            None
        }
    })
}

// fn as_binary_op(self) -> Option<(Expr, BinaryOp, Expr)>
fn expr_as_binary_op(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type.clone(), location, |expr| {
        if let ExpressionKind::Infix(infix_expr) = expr {
            let option_type = extract_option_generic_type(return_type);
            let Type::Tuple(mut tuple_types) = option_type else {
                panic!("Expected the return type option generic arg to be a tuple");
            };
            assert_eq!(tuple_types.len(), 3);

            tuple_types.pop().unwrap();
            let binary_op_type = tuple_types.pop().unwrap();

            // For the op value we use the enum member index, which should match noir_stdlib/src/meta/op.nr
            let binary_op_value = infix_expr.operator.contents as u128;

            let mut fields = HashMap::default();
            fields.insert(Rc::new("op".to_string()), Value::Field(binary_op_value.into()));

            let unary_op = Value::Struct(fields, binary_op_type);
            let lhs = Value::Expr(infix_expr.lhs.kind);
            let rhs = Value::Expr(infix_expr.rhs.kind);
            Some(Value::Tuple(vec![lhs, unary_op, rhs]))
        } else {
            None
        }
    })
}

// fn as_bool(self) -> Option<bool>
fn expr_as_bool(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type, location, |expr| {
        if let ExpressionKind::Literal(Literal::Bool(bool)) = expr {
            Some(Value::Bool(bool))
        } else {
            None
        }
    })
}

// fn as_function_call(self) -> Option<(Expr, [Expr])>
fn expr_as_function_call(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type, location, |expr| {
        if let ExpressionKind::Call(call_expression) = expr {
            let function = Value::Expr(call_expression.func.kind);
            let arguments = call_expression.arguments.into_iter();
            let arguments = arguments.map(|argument| Value::Expr(argument.kind)).collect();
            let arguments =
                Value::Slice(arguments, Type::Slice(Box::new(Type::Quoted(QuotedType::Expr))));
            Some(Value::Tuple(vec![function, arguments]))
        } else {
            None
        }
    })
}

// fn as_if(self) -> Option<(Expr, Expr, Option<Expr>)>
fn expr_as_if(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type.clone(), location, |expr| {
        if let ExpressionKind::If(if_expr) = expr {
            // Get the type of `Option<Expr>`
            let option_type = extract_option_generic_type(return_type.clone());
            let Type::Tuple(option_types) = option_type else {
                panic!("Expected the return type option generic arg to be a tuple");
            };
            assert_eq!(option_types.len(), 3);
            let alternative_option_type = option_types[2].clone();

            let alternative =
                option(alternative_option_type, if_expr.alternative.map(|e| Value::Expr(e.kind)));

            Some(Value::Tuple(vec![
                Value::Expr(if_expr.condition.kind),
                Value::Expr(if_expr.consequence.kind),
                alternative.ok()?,
            ]))
        } else {
            None
        }
    })
}

// fn as_index(self) -> Option<Expr>
fn expr_as_index(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type, location, |expr| {
        if let ExpressionKind::Index(index_expr) = expr {
            Some(Value::Tuple(vec![
                Value::Expr(index_expr.collection.kind),
                Value::Expr(index_expr.index.kind),
            ]))
        } else {
            None
        }
    })
}

// fn as_integer(self) -> Option<(Field, bool)>
fn expr_as_integer(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type.clone(), location, |expr| {
        if let ExpressionKind::Literal(Literal::Integer(field, sign)) = expr {
            Some(Value::Tuple(vec![Value::Field(field), Value::Bool(sign)]))
        } else {
            None
        }
    })
}

// fn as_member_access(self) -> Option<(Expr, Quoted)>
fn expr_as_member_access(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type, location, |expr| {
        if let ExpressionKind::MemberAccess(member_access) = expr {
            let tokens = Rc::new(vec![Token::Ident(member_access.rhs.0.contents.clone())]);
            Some(Value::Tuple(vec![Value::Expr(member_access.lhs.kind), Value::Quoted(tokens)]))
        } else {
            None
        }
    })
}

// fn as_repeated_element_array(self) -> Option<(Expr, Expr)>
fn expr_as_repeated_element_array(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type, location, |expr| {
        if let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Repeated {
            repeated_element,
            length,
        })) = expr
        {
            Some(Value::Tuple(vec![Value::Expr(repeated_element.kind), Value::Expr(length.kind)]))
        } else {
            None
        }
    })
}

// fn as_repeated_element_slice(self) -> Option<(Expr, Expr)>
fn expr_as_repeated_element_slice(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type, location, |expr| {
        if let ExpressionKind::Literal(Literal::Slice(ArrayLiteral::Repeated {
            repeated_element,
            length,
        })) = expr
        {
            Some(Value::Tuple(vec![Value::Expr(repeated_element.kind), Value::Expr(length.kind)]))
        } else {
            None
        }
    })
}

// fn as_slice(self) -> Option<[Expr]>
fn expr_as_slice(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type, location, |expr| {
        if let ExpressionKind::Literal(Literal::Slice(ArrayLiteral::Standard(exprs))) = expr {
            let exprs = exprs.into_iter().map(|expr| Value::Expr(expr.kind)).collect();
            let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Expr)));
            Some(Value::Slice(exprs, typ))
        } else {
            None
        }
    })
}

// fn as_tuple(self) -> Option<[Expr]>
fn expr_as_tuple(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type, location, |expr| {
        if let ExpressionKind::Tuple(expressions) = expr {
            let expressions = expressions.into_iter().map(|expr| Value::Expr(expr.kind)).collect();
            let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Expr)));
            Some(Value::Slice(expressions, typ))
        } else {
            None
        }
    })
}

// fn as_unary_op(self) -> Option<(UnaryOp, Expr)>
fn expr_as_unary_op(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(arguments, return_type.clone(), location, |expr| {
        if let ExpressionKind::Prefix(prefix_expr) = expr {
            let option_type = extract_option_generic_type(return_type);
            let Type::Tuple(mut tuple_types) = option_type else {
                panic!("Expected the return type option generic arg to be a tuple");
            };
            assert_eq!(tuple_types.len(), 2);

            tuple_types.pop().unwrap();
            let unary_op_type = tuple_types.pop().unwrap();

            // These values should match the values used in noir_stdlib/src/meta/op.nr
            let unary_op_value: u128 = match prefix_expr.operator {
                UnaryOp::Minus => 0,
                UnaryOp::Not => 1,
                UnaryOp::MutableReference => 2,
                UnaryOp::Dereference { .. } => 3,
            };

            let mut fields = HashMap::default();
            fields.insert(Rc::new("op".to_string()), Value::Field(unary_op_value.into()));

            let unary_op = Value::Struct(fields, unary_op_type);
            let rhs = Value::Expr(prefix_expr.rhs.kind);
            Some(Value::Tuple(vec![unary_op, rhs]))
        } else {
            None
        }
    })
}

// Helper function for implementing the `expr_as_...` functions.
fn expr_as<F>(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
    f: F,
) -> IResult<Value>
where
    F: FnOnce(ExpressionKind) -> Option<Value>,
{
    let self_argument = check_one_argument(arguments, location)?;
    let mut expression_kind = get_expr(self_argument)?;
    while let ExpressionKind::Parenthesized(expression) = expression_kind {
        expression_kind = expression.kind;
    }

    let option_value = f(expression_kind);
    option(return_type, option_value)
}

// fn name(self) -> Quoted
fn function_def_name(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let func_id = get_function_def(self_argument)?;
    let name = interner.function_name(&func_id).to_string();
    let tokens = Rc::new(vec![Token::Ident(name)]);
    Ok(Value::Quoted(tokens))
}

// fn parameters(self) -> [(Quoted, Type)]
fn function_def_parameters(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let func_id = get_function_def(self_argument)?;
    let func_meta = interner.function_meta(&func_id);

    let parameters = func_meta
        .parameters
        .iter()
        .map(|(hir_pattern, typ, _visibility)| {
            let name = Value::Quoted(Rc::new(hir_pattern_to_tokens(interner, hir_pattern)));
            let typ = Value::Type(typ.clone());
            Value::Tuple(vec![name, typ])
        })
        .collect();

    let typ = Type::Slice(Box::new(Type::Tuple(vec![
        Type::Quoted(QuotedType::Quoted),
        Type::Quoted(QuotedType::Type),
    ])));

    Ok(Value::Slice(parameters, typ))
}

// fn return_type(self) -> Type
fn function_def_return_type(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let func_id = get_function_def(self_argument)?;
    let func_meta = interner.function_meta(&func_id);

    Ok(Value::Type(func_meta.return_type().follow_bindings()))
}

// fn set_body(self, body: Quoted)
fn function_def_set_body(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, body_argument) = check_two_arguments(arguments, location)?;
    let body_argument_location = body_argument.1;

    let func_id = get_function_def(self_argument)?;
    check_function_not_yet_resolved(interpreter, func_id, location)?;

    let body_tokens = get_quoted(body_argument)?;
    let mut body_quoted = add_token_spans(body_tokens.clone(), body_argument_location.span);

    // Surround the body in `{ ... }` so we can parse it as a block
    body_quoted.0.insert(0, SpannedToken::new(Token::LeftBrace, location.span));
    body_quoted.0.push(SpannedToken::new(Token::RightBrace, location.span));

    let body = parse_tokens(
        body_tokens,
        body_quoted,
        body_argument_location,
        parser::block(parser::fresh_statement()),
        "a block",
    )?;

    let func_meta = interpreter.elaborator.interner.function_meta_mut(&func_id);
    func_meta.has_body = true;
    func_meta.function_body = FunctionBody::Unresolved(FunctionKind::Normal, body, location.span);

    Ok(Value::Unit)
}

// fn set_parameters(self, parameters: [(Quoted, Type)])
fn function_def_set_parameters(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, parameters_argument) = check_two_arguments(arguments, location)?;
    let parameters_argument_location = parameters_argument.1;

    let func_id = get_function_def(self_argument)?;
    check_function_not_yet_resolved(interpreter, func_id, location)?;

    let (input_parameters, _type) =
        get_slice(interpreter.elaborator.interner, parameters_argument)?;

    // What follows is very similar to what happens in Elaborator::define_function_meta
    let mut parameters = Vec::new();
    let mut parameter_types = Vec::new();
    let mut parameter_idents = Vec::new();

    for input_parameter in input_parameters {
        let mut tuple = get_tuple(
            interpreter.elaborator.interner,
            (input_parameter, parameters_argument_location),
        )?;
        let parameter_type = get_type((tuple.pop().unwrap(), parameters_argument_location))?;
        let parameter_pattern = parse(
            (tuple.pop().unwrap(), parameters_argument_location),
            parser::pattern(),
            "a pattern",
        )?;

        let hir_pattern = interpreter.elaborate_item(Some(func_id), |elaborator| {
            elaborator.elaborate_pattern_and_store_ids(
                parameter_pattern,
                parameter_type.clone(),
                DefinitionKind::Local(None),
                &mut parameter_idents,
                None,
            )
        });

        parameters.push((hir_pattern, parameter_type.clone(), Visibility::Private));
        parameter_types.push(parameter_type);
    }

    mutate_func_meta_type(interpreter.elaborator.interner, func_id, |func_meta| {
        func_meta.parameters = parameters.into();
        func_meta.parameter_idents = parameter_idents;
        replace_func_meta_parameters(&mut func_meta.typ, parameter_types);
    });

    Ok(Value::Unit)
}

// fn set_return_type(self, return_type: Type)
fn function_def_set_return_type(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, return_type_argument) = check_two_arguments(arguments, location)?;
    let return_type = get_type(return_type_argument)?;

    let func_id = get_function_def(self_argument)?;
    check_function_not_yet_resolved(interpreter, func_id, location)?;

    let quoted_type_id = interpreter.elaborator.interner.push_quoted_type(return_type.clone());

    mutate_func_meta_type(interpreter.elaborator.interner, func_id, |func_meta| {
        func_meta.return_type = FunctionReturnType::Ty(UnresolvedType {
            typ: UnresolvedTypeData::Resolved(quoted_type_id),
            span: location.span,
        });
        replace_func_meta_return_type(&mut func_meta.typ, return_type);
    });

    Ok(Value::Unit)
}

// fn functions(self) -> [FunctionDefinition]
fn module_functions(
    interpreter: &Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let module_id = get_module(self_argument)?;
    let module_data = interpreter.elaborator.get_module(module_id);
    let func_ids = module_data
        .value_definitions()
        .filter_map(|module_def_id| {
            if let ModuleDefId::FunctionId(func_id) = module_def_id {
                Some(Value::FunctionDefinition(func_id))
            } else {
                None
            }
        })
        .collect();

    let slice_type = Type::Slice(Box::new(Type::Quoted(QuotedType::FunctionDefinition)));
    Ok(Value::Slice(func_ids, slice_type))
}

// fn is_contract(self) -> bool
fn module_is_contract(
    interpreter: &Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let module_id = get_module(self_argument)?;
    Ok(Value::Bool(interpreter.elaborator.module_is_contract(module_id)))
}

// fn name(self) -> Quoted
fn module_name(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let module_id = get_module(self_argument)?;
    let name = &interner.module_attributes(&module_id).name;
    let tokens = Rc::new(vec![Token::Ident(name.clone())]);
    Ok(Value::Quoted(tokens))
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

// fn quoted_eq(_first: Quoted, _second: Quoted) -> bool
fn quoted_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let (self_value, other_value) = check_two_arguments(arguments, location)?;

    let self_quoted = get_quoted(self_value)?;
    let other_quoted = get_quoted(other_value)?;

    Ok(Value::Bool(self_quoted == other_quoted))
}

fn trait_def_as_trait_constraint(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> Result<Value, InterpreterError> {
    let argument = check_one_argument(arguments, location)?;

    let trait_id = get_trait_def(argument)?;
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
