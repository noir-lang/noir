use std::rc::Rc;

use acvm::{AcirField, FieldElement};
use builtin_helpers::{
    block_expression_to_value, byte_array_type, check_argument_count,
    check_function_not_yet_resolved, check_one_argument, check_three_arguments,
    check_two_arguments, get_bool, get_expr, get_field, get_format_string, get_function_def,
    get_module, get_quoted, get_slice, get_struct, get_trait_constraint, get_trait_def,
    get_trait_impl, get_tuple, get_type, get_typed_expr, get_u32, get_unresolved_type,
    has_named_attribute, hir_pattern_to_tokens, mutate_func_meta_type, parse, quote_ident,
    replace_func_meta_parameters, replace_func_meta_return_type,
};
use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::{Location, Span};
use num_bigint::BigUint;
use rustc_hash::FxHashMap as HashMap;

use crate::{
    ast::{
        ArrayLiteral, BlockExpression, ConstrainKind, Expression, ExpressionKind, ForRange,
        FunctionKind, FunctionReturnType, Ident, IntegerBitSize, ItemVisibility, LValue, Literal,
        Pattern, Signedness, Statement, StatementKind, UnaryOp, UnresolvedType, UnresolvedTypeData,
        Visibility,
    },
    elaborator::Elaborator,
    hir::{
        comptime::{
            errors::IResult,
            value::{ExprValue, TypedExpr},
            InterpreterError, Value,
        },
        def_collector::dc_crate::CollectedItems,
        def_map::ModuleDefId,
    },
    hir_def::{
        self,
        expr::{HirExpression, HirLiteral},
        function::FunctionBody,
    },
    node_interner::{DefinitionKind, NodeInterner, TraitImplKind},
    parser::{Parser, StatementOrExpressionOrLValue},
    token::{Attribute, Token},
    Kind, QuotedType, ResolvedGeneric, Shared, Type, TypeVariable,
};

use self::builtin_helpers::{eq_item, get_array, get_ctstring, get_str, get_u8, hash_item, lex};
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
        let call_stack = &self.elaborator.interpreter_call_stack;
        match name {
            "apply_range_constraint" => {
                self.call_foreign("range", arguments, return_type, location)
            }
            "array_as_str_unchecked" => array_as_str_unchecked(interner, arguments, location),
            "array_len" => array_len(interner, arguments, location),
            "array_refcount" => Ok(Value::U32(0)),
            "assert_constant" => Ok(Value::Bool(true)),
            "as_slice" => as_slice(interner, arguments, location),
            "ctstring_eq" => ctstring_eq(arguments, location),
            "ctstring_hash" => ctstring_hash(arguments, location),
            "derive_pedersen_generators" => {
                derive_generators(interner, arguments, return_type, location)
            }
            "expr_as_array" => expr_as_array(interner, arguments, return_type, location),
            "expr_as_assert" => expr_as_assert(interner, arguments, return_type, location),
            "expr_as_assert_eq" => expr_as_assert_eq(interner, arguments, return_type, location),
            "expr_as_assign" => expr_as_assign(interner, arguments, return_type, location),
            "expr_as_binary_op" => expr_as_binary_op(interner, arguments, return_type, location),
            "expr_as_block" => expr_as_block(interner, arguments, return_type, location),
            "expr_as_bool" => expr_as_bool(interner, arguments, return_type, location),
            "expr_as_cast" => expr_as_cast(interner, arguments, return_type, location),
            "expr_as_comptime" => expr_as_comptime(interner, arguments, return_type, location),
            "expr_as_constructor" => {
                expr_as_constructor(interner, arguments, return_type, location)
            }
            "expr_as_for" => expr_as_for(interner, arguments, return_type, location),
            "expr_as_for_range" => expr_as_for_range(interner, arguments, return_type, location),
            "expr_as_function_call" => {
                expr_as_function_call(interner, arguments, return_type, location)
            }
            "expr_as_if" => expr_as_if(interner, arguments, return_type, location),
            "expr_as_index" => expr_as_index(interner, arguments, return_type, location),
            "expr_as_integer" => expr_as_integer(interner, arguments, return_type, location),
            "expr_as_lambda" => expr_as_lambda(interner, arguments, return_type, location),
            "expr_as_let" => expr_as_let(interner, arguments, return_type, location),
            "expr_as_member_access" => {
                expr_as_member_access(interner, arguments, return_type, location)
            }
            "expr_as_method_call" => {
                expr_as_method_call(interner, arguments, return_type, location)
            }
            "expr_as_repeated_element_array" => {
                expr_as_repeated_element_array(interner, arguments, return_type, location)
            }
            "expr_as_repeated_element_slice" => {
                expr_as_repeated_element_slice(interner, arguments, return_type, location)
            }
            "expr_as_slice" => expr_as_slice(interner, arguments, return_type, location),
            "expr_as_tuple" => expr_as_tuple(interner, arguments, return_type, location),
            "expr_as_unary_op" => expr_as_unary_op(interner, arguments, return_type, location),
            "expr_as_unsafe" => expr_as_unsafe(interner, arguments, return_type, location),
            "expr_has_semicolon" => expr_has_semicolon(interner, arguments, location),
            "expr_is_break" => expr_is_break(interner, arguments, location),
            "expr_is_continue" => expr_is_continue(interner, arguments, location),
            "expr_resolve" => expr_resolve(self, arguments, location),
            "is_unconstrained" => Ok(Value::Bool(true)),
            "field_less_than" => field_less_than(arguments, location),
            "fmtstr_as_ctstring" => fmtstr_as_ctstring(interner, arguments, location),
            "fmtstr_quoted_contents" => fmtstr_quoted_contents(interner, arguments, location),
            "fresh_type_variable" => fresh_type_variable(interner),
            "function_def_add_attribute" => function_def_add_attribute(self, arguments, location),
            "function_def_body" => function_def_body(interner, arguments, location),
            "function_def_eq" => function_def_eq(arguments, location),
            "function_def_has_named_attribute" => {
                function_def_has_named_attribute(interner, arguments, location)
            }
            "function_def_hash" => function_def_hash(arguments, location),
            "function_def_is_unconstrained" => {
                function_def_is_unconstrained(interner, arguments, location)
            }
            "function_def_module" => function_def_module(interner, arguments, location),
            "function_def_name" => function_def_name(interner, arguments, location),
            "function_def_parameters" => function_def_parameters(interner, arguments, location),
            "function_def_return_type" => function_def_return_type(interner, arguments, location),
            "function_def_set_body" => function_def_set_body(self, arguments, location),
            "function_def_set_parameters" => function_def_set_parameters(self, arguments, location),
            "function_def_set_return_type" => {
                function_def_set_return_type(self, arguments, location)
            }
            "function_def_set_return_public" => {
                function_def_set_return_public(self, arguments, location)
            }
            "function_def_set_return_data" => {
                function_def_set_return_data(self, arguments, location)
            }
            "function_def_set_unconstrained" => {
                function_def_set_unconstrained(self, arguments, location)
            }
            "module_add_item" => module_add_item(self, arguments, location),
            "module_eq" => module_eq(arguments, location),
            "module_functions" => module_functions(self, arguments, location),
            "module_has_named_attribute" => module_has_named_attribute(self, arguments, location),
            "module_hash" => module_hash(arguments, location),
            "module_is_contract" => module_is_contract(self, arguments, location),
            "module_name" => module_name(interner, arguments, location),
            "module_structs" => module_structs(self, arguments, location),
            "modulus_be_bits" => modulus_be_bits(arguments, location),
            "modulus_be_bytes" => modulus_be_bytes(arguments, location),
            "modulus_le_bits" => modulus_le_bits(arguments, location),
            "modulus_le_bytes" => modulus_le_bytes(arguments, location),
            "modulus_num_bits" => modulus_num_bits(arguments, location),
            "quoted_as_expr" => quoted_as_expr(self.elaborator, arguments, return_type, location),
            "quoted_as_module" => quoted_as_module(self, arguments, return_type, location),
            "quoted_as_trait_constraint" => quoted_as_trait_constraint(self, arguments, location),
            "quoted_as_type" => quoted_as_type(self, arguments, location),
            "quoted_eq" => quoted_eq(arguments, location),
            "quoted_hash" => quoted_hash(arguments, location),
            "quoted_tokens" => quoted_tokens(arguments, location),
            "slice_insert" => slice_insert(interner, arguments, location),
            "slice_pop_back" => slice_pop_back(interner, arguments, location, call_stack),
            "slice_pop_front" => slice_pop_front(interner, arguments, location, call_stack),
            "slice_push_back" => slice_push_back(interner, arguments, location),
            "slice_push_front" => slice_push_front(interner, arguments, location),
            "slice_refcount" => Ok(Value::U32(0)),
            "slice_remove" => slice_remove(interner, arguments, location, call_stack),
            "str_as_bytes" => str_as_bytes(interner, arguments, location),
            "str_as_ctstring" => str_as_ctstring(interner, arguments, location),
            "struct_def_add_attribute" => struct_def_add_attribute(interner, arguments, location),
            "struct_def_add_generic" => struct_def_add_generic(interner, arguments, location),
            "struct_def_as_type" => struct_def_as_type(interner, arguments, location),
            "struct_def_eq" => struct_def_eq(arguments, location),
            "struct_def_fields" => struct_def_fields(interner, arguments, location, call_stack),
            "struct_def_fields_as_written" => {
                struct_def_fields_as_written(interner, arguments, location)
            }
            "struct_def_generics" => struct_def_generics(interner, arguments, location),
            "struct_def_has_named_attribute" => {
                struct_def_has_named_attribute(interner, arguments, location)
            }
            "struct_def_hash" => struct_def_hash(arguments, location),
            "struct_def_module" => struct_def_module(self, arguments, location),
            "struct_def_name" => struct_def_name(interner, arguments, location),
            "struct_def_set_fields" => struct_def_set_fields(interner, arguments, location),
            "to_be_radix" => to_be_radix(arguments, return_type, location),
            "to_le_radix" => to_le_radix(arguments, return_type, location),
            "to_be_bits" => to_be_bits(arguments, return_type, location),
            "to_le_bits" => to_le_bits(arguments, return_type, location),
            "trait_constraint_eq" => trait_constraint_eq(arguments, location),
            "trait_constraint_hash" => trait_constraint_hash(arguments, location),
            "trait_def_as_trait_constraint" => {
                trait_def_as_trait_constraint(interner, arguments, location)
            }
            "trait_def_eq" => trait_def_eq(arguments, location),
            "trait_def_hash" => trait_def_hash(arguments, location),
            "trait_impl_methods" => trait_impl_methods(interner, arguments, location),
            "trait_impl_trait_generic_args" => {
                trait_impl_trait_generic_args(interner, arguments, location)
            }
            "type_as_array" => type_as_array(arguments, return_type, location),
            "type_as_constant" => type_as_constant(arguments, return_type, location),
            "type_as_integer" => type_as_integer(arguments, return_type, location),
            "type_as_mutable_reference" => {
                type_as_mutable_reference(arguments, return_type, location)
            }
            "type_as_slice" => type_as_slice(arguments, return_type, location),
            "type_as_str" => type_as_str(arguments, return_type, location),
            "type_as_struct" => type_as_struct(arguments, return_type, location),
            "type_as_tuple" => type_as_tuple(arguments, return_type, location),
            "type_eq" => type_eq(arguments, location),
            "type_get_trait_impl" => {
                type_get_trait_impl(interner, arguments, return_type, location)
            }
            "type_hash" => type_hash(arguments, location),
            "type_implements" => type_implements(interner, arguments, location),
            "type_is_bool" => type_is_bool(arguments, location),
            "type_is_field" => type_is_field(arguments, location),
            "type_is_unit" => type_is_unit(arguments, location),
            "type_of" => type_of(arguments, location),
            "typed_expr_as_function_definition" => {
                typed_expr_as_function_definition(interner, arguments, return_type, location)
            }
            "typed_expr_get_type" => {
                typed_expr_get_type(interner, arguments, return_type, location)
            }
            "unresolved_type_as_mutable_reference" => {
                unresolved_type_as_mutable_reference(interner, arguments, return_type, location)
            }
            "unresolved_type_as_slice" => {
                unresolved_type_as_slice(interner, arguments, return_type, location)
            }
            "unresolved_type_is_bool" => unresolved_type_is_bool(interner, arguments, location),
            "unresolved_type_is_field" => unresolved_type_is_field(interner, arguments, location),
            "unresolved_type_is_unit" => unresolved_type_is_unit(interner, arguments, location),
            "zeroed" => zeroed(return_type, location.span),
            _ => {
                let item = format!("Comptime evaluation for builtin function '{name}'");
                Err(InterpreterError::Unimplemented { item, location })
            }
        }
    }
}

fn failing_constraint<T>(
    message: impl Into<String>,
    location: Location,
    call_stack: &im::Vector<Location>,
) -> IResult<T> {
    Err(InterpreterError::FailingConstraint {
        message: Some(message.into()),
        location,
        call_stack: call_stack.clone(),
    })
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

fn str_as_bytes(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let string = check_one_argument(arguments, location)?;
    let string = get_str(interner, string)?;

    let bytes: im::Vector<Value> = string.bytes().map(Value::U8).collect();
    let byte_array_type = byte_array_type(bytes.len());
    Ok(Value::Array(bytes, byte_array_type))
}

// fn str_as_ctstring(self) -> CtString
fn str_as_ctstring(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let string = get_str(interner, self_argument)?;
    Ok(Value::CtString(string))
}

// fn add_attribute<let N: u32>(self, attribute: str<N>)
fn struct_def_add_attribute(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, attribute) = check_two_arguments(arguments, location)?;
    let attribute_location = attribute.1;
    let attribute = get_str(interner, attribute)?;
    let attribute = format!("#[{}]", attribute);
    let mut parser = Parser::for_str(&attribute);
    let Some((Attribute::Secondary(attribute), _span)) = parser.parse_attribute() else {
        return Err(InterpreterError::InvalidAttribute {
            attribute: attribute.to_string(),
            location: attribute_location,
        });
    };

    let struct_id = get_struct(self_argument)?;
    interner.update_struct_attributes(struct_id, |attributes| {
        attributes.push(attribute);
    });

    Ok(Value::Unit)
}

// fn add_generic<let N: u32>(self, generic_name: str<N>)
fn struct_def_add_generic(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, generic) = check_two_arguments(arguments, location)?;
    let generic_location = generic.1;
    let generic = get_str(interner, generic)?;

    let mut tokens = lex(&generic);
    if tokens.len() != 1 {
        return Err(InterpreterError::GenericNameShouldBeAnIdent {
            name: generic,
            location: generic_location,
        });
    }

    let Token::Ident(generic_name) = tokens.remove(0) else {
        return Err(InterpreterError::GenericNameShouldBeAnIdent {
            name: generic,
            location: generic_location,
        });
    };

    let struct_id = get_struct(self_argument)?;
    let the_struct = interner.get_struct(struct_id);
    let mut the_struct = the_struct.borrow_mut();
    let name = Rc::new(generic_name);

    for generic in &the_struct.generics {
        if generic.name == name {
            return Err(InterpreterError::DuplicateGeneric {
                name,
                struct_name: the_struct.name.to_string(),
                existing_location: Location::new(generic.span, the_struct.location.file),
                duplicate_location: generic_location,
            });
        }
    }

    let type_var_kind = Kind::Normal;
    let type_var = TypeVariable::unbound(interner.next_type_variable_id(), type_var_kind);
    let span = generic_location.span;
    let typ = Type::NamedGeneric(type_var.clone(), name.clone());
    let new_generic = ResolvedGeneric { name, type_var, span };
    the_struct.generics.push(new_generic);

    Ok(Value::Type(typ))
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
        Type::NamedGeneric(generic.type_var.clone(), generic.name.clone())
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

fn struct_def_hash(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    hash_item(arguments, location, get_struct)
}

fn struct_def_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    eq_item(arguments, location, get_struct)
}

// fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
fn struct_def_has_named_attribute(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, name) = check_two_arguments(arguments, location)?;
    let struct_id = get_struct(self_argument)?;

    let name = get_str(interner, name)?;

    Ok(Value::Bool(has_named_attribute(&name, interner.struct_attributes(&struct_id))))
}

/// fn fields(self, generic_args: [Type]) -> [(Quoted, Type)]
/// Returns (name, type) pairs of each field of this StructDefinition.
/// Applies the given generic arguments to each field.
fn struct_def_fields(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
    call_stack: &im::Vector<Location>,
) -> IResult<Value> {
    let (typ, generic_args) = check_two_arguments(arguments, location)?;
    let struct_id = get_struct(typ)?;
    let struct_def = interner.get_struct(struct_id);
    let struct_def = struct_def.borrow();

    let args_location = generic_args.1;
    let generic_args = get_slice(interner, generic_args)?.0;
    let generic_args = try_vecmap(generic_args, |arg| get_type((arg, args_location)))?;

    let actual = generic_args.len();
    let expected = struct_def.generics.len();
    if actual != expected {
        let s = if expected == 1 { "" } else { "s" };
        let was_were = if actual == 1 { "was" } else { "were" };
        let message = Some(format!("`StructDefinition::fields` expected {expected} generic{s} for `{}` but {actual} {was_were} given", struct_def.name));
        let location = args_location;
        let call_stack = call_stack.clone();
        return Err(InterpreterError::FailingConstraint { message, location, call_stack });
    }

    let mut fields = im::Vector::new();

    for (field_name, field_type) in struct_def.get_fields(&generic_args) {
        let name = Value::Quoted(Rc::new(vec![Token::Ident(field_name)]));
        fields.push_back(Value::Tuple(vec![name, Value::Type(field_type)]));
    }

    let typ = Type::Slice(Box::new(Type::Tuple(vec![
        Type::Quoted(QuotedType::Quoted),
        Type::Quoted(QuotedType::Type),
    ])));
    Ok(Value::Slice(fields, typ))
}

/// fn fields_as_written(self) -> [(Quoted, Type)]
/// Returns (name, type) pairs of each field of this StructDefinition.
///
/// Note that any generic arguments won't be applied: if you need them to be, use `fields`.
fn struct_def_fields_as_written(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;
    let struct_id = get_struct(argument)?;
    let struct_def = interner.get_struct(struct_id);
    let struct_def = struct_def.borrow();

    let mut fields = im::Vector::new();

    for field in struct_def.get_fields_as_written() {
        let name = Value::Quoted(Rc::new(vec![Token::Ident(field.name.to_string())]));
        let typ = Value::Type(field.typ);
        fields.push_back(Value::Tuple(vec![name, typ]));
    }

    let typ = Type::Slice(Box::new(Type::Tuple(vec![
        Type::Quoted(QuotedType::Quoted),
        Type::Quoted(QuotedType::Type),
    ])));
    Ok(Value::Slice(fields, typ))
}

// fn module(self) -> Module
fn struct_def_module(
    interpreter: &Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let struct_id = get_struct(self_argument)?;
    let parent = struct_id.parent_module_id(interpreter.elaborator.def_maps);
    Ok(Value::ModuleDefinition(parent))
}

// fn name(self) -> Quoted
fn struct_def_name(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let struct_id = get_struct(self_argument)?;
    let the_struct = interner.get_struct(struct_id);

    let name = Token::Ident(the_struct.borrow().name.to_string());
    Ok(Value::Quoted(Rc::new(vec![name])))
}

/// fn set_fields(self, new_fields: [(Quoted, Type)]) {}
/// Returns (name, type) pairs of each field of this StructDefinition
fn struct_def_set_fields(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (the_struct, fields) = check_two_arguments(arguments, location)?;
    let struct_id = get_struct(the_struct)?;

    let struct_def = interner.get_struct(struct_id);
    let mut struct_def = struct_def.borrow_mut();

    let field_location = fields.1;
    let fields = get_slice(interner, fields)?.0;

    let new_fields = fields
        .into_iter()
        .flat_map(|field_pair| get_tuple(interner, (field_pair, field_location)))
        .enumerate()
        .map(|(index, mut field_pair)| {
            if field_pair.len() == 2 {
                let typ = field_pair.pop().unwrap();
                let name_value = field_pair.pop().unwrap();

                let name_tokens = get_quoted((name_value.clone(), field_location))?;
                let typ = get_type((typ, field_location))?;

                match name_tokens.first() {
                    Some(Token::Ident(name)) if name_tokens.len() == 1 => {
                        Ok(hir_def::types::StructField {
                            visibility: ItemVisibility::Public,
                            name: Ident::new(name.clone(), field_location.span),
                            typ,
                        })
                    }
                    _ => {
                        let value = name_value.display(interner).to_string();
                        let location = field_location;
                        Err(InterpreterError::ExpectedIdentForStructField {
                            value,
                            index,
                            location,
                        })
                    }
                }
            } else {
                let type_var = interner.next_type_variable();
                let expected = Type::Tuple(vec![type_var.clone(), type_var]);

                let actual =
                    Type::Tuple(vecmap(&field_pair, |value| value.get_type().into_owned()));

                Err(InterpreterError::TypeMismatch { expected, actual, location })
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    struct_def.set_fields(new_fields);
    Ok(Value::Unit)
}

fn slice_remove(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
    call_stack: &im::Vector<Location>,
) -> IResult<Value> {
    let (slice, index) = check_two_arguments(arguments, location)?;

    let (mut values, typ) = get_slice(interner, slice)?;
    let index = get_u32(index)? as usize;

    if values.is_empty() {
        return failing_constraint("slice_remove called on empty slice", location, call_stack);
    }

    if index >= values.len() {
        let message = format!(
            "slice_remove: index {index} is out of bounds for a slice of length {}",
            values.len()
        );
        return failing_constraint(message, location, call_stack);
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
    call_stack: &im::Vector<Location>,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let (mut values, typ) = get_slice(interner, argument)?;
    match values.pop_front() {
        Some(element) => Ok(Value::Tuple(vec![element, Value::Slice(values, typ)])),
        None => failing_constraint("slice_pop_front called on empty slice", location, call_stack),
    }
}

fn slice_pop_back(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
    call_stack: &im::Vector<Location>,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let (mut values, typ) = get_slice(interner, argument)?;
    match values.pop_back() {
        Some(element) => Ok(Value::Tuple(vec![Value::Slice(values, typ), element])),
        None => failing_constraint("slice_pop_back called on empty slice", location, call_stack),
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
    elaborator: &mut Elaborator,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let result = parse(
        elaborator,
        argument,
        Parser::parse_statement_or_expression_or_lvalue,
        "an expression",
    );

    let value =
        result.ok().map(
            |statement_or_expression_or_lvalue| match statement_or_expression_or_lvalue {
                StatementOrExpressionOrLValue::Expression(expr) => Value::expression(expr.kind),
                StatementOrExpressionOrLValue::Statement(statement) => {
                    Value::statement(statement.kind)
                }
                StatementOrExpressionOrLValue::LValue(lvalue) => Value::lvalue(lvalue),
            },
        );

    option(return_type, value, location.span)
}

// fn as_module(quoted: Quoted) -> Option<Module>
fn quoted_as_module(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let path =
        parse(interpreter.elaborator, argument, Parser::parse_path_no_turbofish_or_error, "a path")
            .ok();
    let option_value = path.and_then(|path| {
        let module = interpreter
            .elaborate_in_function(interpreter.current_function, |elaborator| {
                elaborator.resolve_module_by_path(path)
            });
        module.map(Value::ModuleDefinition)
    });

    option(return_type, option_value, location.span)
}

// fn as_trait_constraint(quoted: Quoted) -> TraitConstraint
fn quoted_as_trait_constraint(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;
    let trait_bound = parse(
        interpreter.elaborator,
        argument,
        Parser::parse_trait_bound_or_error,
        "a trait constraint",
    )?;
    let bound = interpreter
        .elaborate_in_function(interpreter.current_function, |elaborator| {
            elaborator.resolve_trait_bound(&trait_bound)
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
    let typ = parse(interpreter.elaborator, argument, Parser::parse_type_or_error, "a type")?;
    let typ = interpreter
        .elaborate_in_function(interpreter.current_function, |elab| elab.resolve_type(typ));
    Ok(Value::Type(typ))
}

// fn tokens(quoted: Quoted) -> [Quoted]
fn quoted_tokens(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;
    let value = get_quoted(argument)?;

    Ok(Value::Slice(
        value.iter().map(|token| Value::Quoted(Rc::new(vec![token.clone()]))).collect(),
        Type::Slice(Box::new(Type::Quoted(QuotedType::Quoted))),
    ))
}

fn to_be_bits(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let value = check_one_argument(arguments, location)?;
    let radix = (Value::U32(2), value.1);
    to_be_radix(vec![value, radix], return_type, location)
}

fn to_le_bits(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let value = check_one_argument(arguments, location)?;
    let radix = (Value::U32(2), value.1);
    to_le_radix(vec![value, radix], return_type, location)
}

fn to_be_radix(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let le_radix_limbs = to_le_radix(arguments, return_type, location)?;

    let Value::Array(limbs, typ) = le_radix_limbs else {
        unreachable!("`to_le_radix` should always return an array");
    };
    let be_radix_limbs = limbs.into_iter().rev().collect();

    Ok(Value::Array(be_radix_limbs, typ))
}

fn to_le_radix(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let (value, radix) = check_two_arguments(arguments, location)?;

    let value = get_field(value)?;
    let radix = get_u32(radix)?;
    let (limb_count, element_type) = if let Type::Array(length, element_type) = return_type {
        if let Type::Constant(limb_count, kind) = *length {
            if kind.unifies(&Kind::u32()) {
                (limb_count, element_type)
            } else {
                return Err(InterpreterError::TypeAnnotationsNeededForMethodCall { location });
            }
        } else {
            return Err(InterpreterError::TypeAnnotationsNeededForMethodCall { location });
        }
    } else {
        return Err(InterpreterError::TypeAnnotationsNeededForMethodCall { location });
    };

    let return_type_is_bits =
        *element_type == Type::Integer(Signedness::Unsigned, IntegerBitSize::One);

    // Decompose the integer into its radix digits in little endian form.
    let decomposed_integer = compute_to_radix_le(value, radix);
    let decomposed_integer = vecmap(0..limb_count.to_u128() as usize, |i| {
        let digit = match decomposed_integer.get(i) {
            Some(digit) => *digit,
            None => 0,
        };
        // The only built-ins that use these either return `[u1; N]` or `[u8; N]`
        if return_type_is_bits {
            Value::U1(digit != 0)
        } else {
            Value::U8(digit)
        }
    });

    let result_type = Type::Array(
        Box::new(Type::Constant(decomposed_integer.len().into(), Kind::u32())),
        element_type,
    );

    Ok(Value::Array(decomposed_integer.into(), result_type))
}

fn compute_to_radix_le(field: FieldElement, radix: u32) -> Vec<u8> {
    let bit_size = u32::BITS - (radix - 1).leading_zeros();
    let radix_big = BigUint::from(radix);
    assert_eq!(BigUint::from(2u128).pow(bit_size), radix_big, "ICE: Radix must be a power of 2");
    let big_integer = BigUint::from_bytes_be(&field.to_be_bytes());

    // Decompose the integer into its radix digits in little endian form.
    big_integer.to_radix_le(radix)
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
    type_as_or_err(arguments, return_type, location, |typ| {
        // Prefer to use `evaluate_to_u32` over matching on `Type::Constant`
        // since arithmetic generics may be `Type::InfixExpr`s which evaluate to
        // constants but are not actually the `Type::Constant` variant.
        match typ.evaluate_to_u32(location.span) {
            Ok(constant) => Ok(Some(Value::U32(constant))),
            Err(err) => {
                // Evaluating to a non-constant returns 'None' in user code
                if err.is_non_constant_evaluated() {
                    Ok(None)
                } else {
                    let err = Some(Box::new(err));
                    Err(InterpreterError::NonIntegerArrayLength { typ, err, location })
                }
            }
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

// fn as_mutable_reference(self) -> Option<Type>
fn type_as_mutable_reference(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    type_as(arguments, return_type, location, |typ| {
        if let Type::MutableReference(typ) = typ {
            Some(Value::Type(*typ))
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

// fn as_str(self) -> Option<Type>
fn type_as_str(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    type_as(arguments, return_type, location, |typ| {
        if let Type::String(n) = typ {
            Some(Value::Type(*n))
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

fn type_as<F>(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
    f: F,
) -> IResult<Value>
where
    F: FnOnce(Type) -> Option<Value>,
{
    type_as_or_err(arguments, return_type, location, |x| Ok(f(x)))
}

// Helper function for implementing the `type_as_...` functions.
fn type_as_or_err<F>(
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
    f: F,
) -> IResult<Value>
where
    F: FnOnce(Type) -> IResult<Option<Value>>,
{
    let value = check_one_argument(arguments, location)?;
    let typ = get_type(value)?.follow_bindings();

    let option_value = f(typ)?;

    option(return_type, option_value, location.span)
}

// fn type_eq(_first: Type, _second: Type) -> bool
fn type_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    eq_item(arguments, location, get_type)
}

// fn type_hash(_t: Type) -> Field
fn type_hash(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    hash_item(arguments, location, get_type)
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

    let option_value = match interner.lookup_trait_implementation(
        &typ,
        trait_id,
        &generics.ordered,
        &generics.named,
    ) {
        Ok(TraitImplKind::Normal(trait_impl_id)) => Some(Value::TraitImpl(trait_impl_id)),
        _ => None,
    };

    option(return_type, option_value, location.span)
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

    let implements = interner
        .lookup_trait_implementation(&typ, trait_id, &generics.ordered, &generics.named)
        .is_ok();
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

// fn is_unit(self) -> bool
fn type_is_unit(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let value = check_one_argument(arguments, location)?;
    let typ = get_type(value)?;

    Ok(Value::Bool(matches!(typ, Type::Unit)))
}

// fn type_of<T>(x: T) -> Type
fn type_of(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let (value, _) = check_one_argument(arguments, location)?;
    let typ = value.get_type().into_owned();
    Ok(Value::Type(typ))
}

// fn constraint_hash(constraint: TraitConstraint) -> Field
fn trait_constraint_hash(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    hash_item(arguments, location, get_trait_constraint)
}

// fn constraint_eq(constraint_a: TraitConstraint, constraint_b: TraitConstraint) -> bool
fn trait_constraint_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    eq_item(arguments, location, get_trait_constraint)
}

// fn trait_def_hash(def: TraitDefinition) -> Field
fn trait_def_hash(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    hash_item(arguments, location, get_trait_def)
}

// fn trait_def_eq(def_a: TraitDefinition, def_b: TraitDefinition) -> bool
fn trait_def_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    eq_item(arguments, location, get_trait_def)
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

// fn as_function_definition(self) -> Option<FunctionDefinition>
fn typed_expr_as_function_definition(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let typed_expr = get_typed_expr(self_argument)?;
    let option_value = if let TypedExpr::ExprId(expr_id) = typed_expr {
        let func_id = interner.lookup_function_from_expr(&expr_id);
        func_id.map(Value::FunctionDefinition)
    } else {
        None
    };
    option(return_type, option_value, location.span)
}

// fn get_type(self) -> Option<Type>
fn typed_expr_get_type(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let typed_expr = get_typed_expr(self_argument)?;
    let option_value = if let TypedExpr::ExprId(expr_id) = typed_expr {
        let typ = interner.id_type(expr_id);
        if typ == Type::Error {
            None
        } else {
            Some(Value::Type(typ))
        }
    } else {
        None
    };
    option(return_type, option_value, location.span)
}

// fn as_mutable_reference(self) -> Option<UnresolvedType>
fn unresolved_type_as_mutable_reference(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    unresolved_type_as(interner, arguments, return_type, location, |typ| {
        if let UnresolvedTypeData::MutableReference(typ) = typ {
            Some(Value::UnresolvedType(typ.typ))
        } else {
            None
        }
    })
}

// fn as_slice(self) -> Option<UnresolvedType>
fn unresolved_type_as_slice(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    unresolved_type_as(interner, arguments, return_type, location, |typ| {
        if let UnresolvedTypeData::Slice(typ) = typ {
            Some(Value::UnresolvedType(typ.typ))
        } else {
            None
        }
    })
}

// fn is_bool(self) -> bool
fn unresolved_type_is_bool(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let typ = get_unresolved_type(interner, self_argument)?;
    Ok(Value::Bool(matches!(typ, UnresolvedTypeData::Bool)))
}

// fn is_field(self) -> bool
fn unresolved_type_is_field(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let typ = get_unresolved_type(interner, self_argument)?;
    Ok(Value::Bool(matches!(typ, UnresolvedTypeData::FieldElement)))
}

// fn is_unit(self) -> bool
fn unresolved_type_is_unit(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let typ = get_unresolved_type(interner, self_argument)?;
    Ok(Value::Bool(matches!(typ, UnresolvedTypeData::Unit)))
}

// Helper function for implementing the `unresolved_type_as_...` functions.
fn unresolved_type_as<F>(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
    f: F,
) -> IResult<Value>
where
    F: FnOnce(UnresolvedTypeData) -> Option<Value>,
{
    let value = check_one_argument(arguments, location)?;
    let typ = get_unresolved_type(interner, value)?;

    let option_value = f(typ);

    option(return_type, option_value, location.span)
}

// fn zeroed<T>() -> T
fn zeroed(return_type: Type, span: Span) -> IResult<Value> {
    match return_type {
        Type::FieldElement => Ok(Value::Field(0u128.into())),
        Type::Array(length_type, elem) => {
            if let Ok(length) = length_type.evaluate_to_u32(span) {
                let element = zeroed(elem.as_ref().clone(), span)?;
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
            if let Ok(length) = length_type.evaluate_to_u32(span) {
                Ok(Value::String(Rc::new("\0".repeat(length as usize))))
            } else {
                // Assume we can resolve the length later
                Ok(Value::Zeroed(Type::String(length_type)))
            }
        }
        Type::FmtString(length_type, captures) => {
            let length = length_type.evaluate_to_u32(span);
            let typ = Type::FmtString(length_type, captures);
            if let Ok(length) = length {
                Ok(Value::FormatString(Rc::new("\0".repeat(length as usize)), typ))
            } else {
                // Assume we can resolve the length later
                Ok(Value::Zeroed(typ))
            }
        }
        Type::Unit => Ok(Value::Unit),
        Type::Tuple(fields) => Ok(Value::Tuple(try_vecmap(fields, |field| zeroed(field, span))?)),
        Type::Struct(struct_type, generics) => {
            let fields = struct_type.borrow().get_fields(&generics);
            let mut values = HashMap::default();

            for (field_name, field_type) in fields {
                let field_value = zeroed(field_type, span)?;
                values.insert(Rc::new(field_name), field_value);
            }

            let typ = Type::Struct(struct_type, generics);
            Ok(Value::Struct(values, typ))
        }
        Type::Alias(alias, generics) => zeroed(alias.borrow().get_type(&generics), span),
        Type::CheckedCast { to, .. } => zeroed(*to, span),
        typ @ Type::Function(..) => {
            // Using Value::Zeroed here is probably safer than using FuncId::dummy_id() or similar
            Ok(Value::Zeroed(typ))
        }
        Type::MutableReference(element) => {
            let element = zeroed(*element, span)?;
            Ok(Value::Pointer(Shared::new(element), false))
        }
        // Optimistically assume we can resolve this type later or that the value is unused
        Type::TypeVariable(_)
        | Type::Forall(_, _)
        | Type::Constant(..)
        | Type::InfixExpr(..)
        | Type::Quoted(_)
        | Type::Error
        | Type::TraitAsType(..)
        | Type::NamedGeneric(_, _) => Ok(Value::Zeroed(return_type)),
    }
}

// fn as_array(self) -> Option<[Expr]>
fn expr_as_array(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Literal(Literal::Array(
            ArrayLiteral::Standard(exprs),
        ))) = expr
        {
            let exprs = exprs.into_iter().map(|expr| Value::expression(expr.kind)).collect();
            let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Expr)));
            Some(Value::Slice(exprs, typ))
        } else {
            None
        }
    })
}

// fn as_assert(self) -> Option<(Expr, Option<Expr>)>
fn expr_as_assert(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type.clone(), location, |expr| {
        if let ExprValue::Statement(StatementKind::Constrain(mut constrain)) = expr {
            if constrain.kind == ConstrainKind::Assert
                && !constrain.arguments.is_empty()
                && constrain.arguments.len() <= 2
            {
                let (message, predicate) = if constrain.arguments.len() == 1 {
                    (None, constrain.arguments.pop().unwrap())
                } else {
                    (Some(constrain.arguments.pop().unwrap()), constrain.arguments.pop().unwrap())
                };
                let predicate = Value::expression(predicate.kind);

                let option_type = extract_option_generic_type(return_type);
                let Type::Tuple(mut tuple_types) = option_type else {
                    panic!("Expected the return type option generic arg to be a tuple");
                };
                assert_eq!(tuple_types.len(), 2);

                let option_type = tuple_types.pop().unwrap();
                let message = message.map(|msg| Value::expression(msg.kind));
                let message = option(option_type, message, location.span).ok()?;

                Some(Value::Tuple(vec![predicate, message]))
            } else {
                None
            }
        } else {
            None
        }
    })
}

// fn as_assert_eq(self) -> Option<(Expr, Expr, Option<Expr>)>
fn expr_as_assert_eq(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type.clone(), location, |expr| {
        if let ExprValue::Statement(StatementKind::Constrain(mut constrain)) = expr {
            if constrain.kind == ConstrainKind::AssertEq
                && constrain.arguments.len() >= 2
                && constrain.arguments.len() <= 3
            {
                let (message, rhs, lhs) = if constrain.arguments.len() == 2 {
                    (None, constrain.arguments.pop().unwrap(), constrain.arguments.pop().unwrap())
                } else {
                    (
                        Some(constrain.arguments.pop().unwrap()),
                        constrain.arguments.pop().unwrap(),
                        constrain.arguments.pop().unwrap(),
                    )
                };

                let lhs = Value::expression(lhs.kind);
                let rhs = Value::expression(rhs.kind);

                let option_type = extract_option_generic_type(return_type);
                let Type::Tuple(mut tuple_types) = option_type else {
                    panic!("Expected the return type option generic arg to be a tuple");
                };
                assert_eq!(tuple_types.len(), 3);

                let option_type = tuple_types.pop().unwrap();
                let message = message.map(|message| Value::expression(message.kind));
                let message = option(option_type, message, location.span).ok()?;

                Some(Value::Tuple(vec![lhs, rhs, message]))
            } else {
                None
            }
        } else {
            None
        }
    })
}

// fn as_assign(self) -> Option<(Expr, Expr)>
fn expr_as_assign(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Statement(StatementKind::Assign(assign)) = expr {
            let lhs = Value::lvalue(assign.lvalue);
            let rhs = Value::expression(assign.expression.kind);
            Some(Value::Tuple(vec![lhs, rhs]))
        } else {
            None
        }
    })
}

// fn as_binary_op(self) -> Option<(Expr, BinaryOp, Expr)>
fn expr_as_binary_op(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type.clone(), location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Infix(infix_expr)) = expr {
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
            let lhs = Value::expression(infix_expr.lhs.kind);
            let rhs = Value::expression(infix_expr.rhs.kind);
            Some(Value::Tuple(vec![lhs, unary_op, rhs]))
        } else {
            None
        }
    })
}

// fn as_block(self) -> Option<[Expr]>
fn expr_as_block(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Block(block_expr)) = expr {
            Some(block_expression_to_value(block_expr))
        } else {
            None
        }
    })
}

// fn as_bool(self) -> Option<bool>
fn expr_as_bool(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Literal(Literal::Bool(bool))) = expr {
            Some(Value::Bool(bool))
        } else {
            None
        }
    })
}

// fn as_cast(self) -> Option<(Expr, UnresolvedType)>
fn expr_as_cast(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Cast(cast)) = expr {
            let lhs = Value::expression(cast.lhs.kind);
            let typ = Value::UnresolvedType(cast.r#type.typ);
            Some(Value::Tuple(vec![lhs, typ]))
        } else {
            None
        }
    })
}

// fn as_comptime(self) -> Option<[Expr]>
fn expr_as_comptime(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    use ExpressionKind::Block;

    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Comptime(block_expr, _)) = expr {
            Some(block_expression_to_value(block_expr))
        } else if let ExprValue::Statement(StatementKind::Comptime(statement)) = expr {
            let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Expr)));

            // comptime { ... } as a statement wraps a block expression,
            // and in that case we return the block expression statements
            // (comptime as a statement can also be comptime for, but in that case we'll
            // return the for statement as a single expression)
            if let StatementKind::Expression(Expression { kind: Block(block), .. }) = statement.kind
            {
                Some(block_expression_to_value(block))
            } else {
                let mut elements = Vector::new();
                elements.push_back(Value::statement(statement.kind));
                Some(Value::Slice(elements, typ))
            }
        } else {
            None
        }
    })
}

// fn as_constructor(self) -> Option<(Quoted, [(Quoted, Expr)])>
fn expr_as_constructor(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let expr_value = get_expr(interner, self_argument)?;
    let expr_value = unwrap_expr_value(interner, expr_value);

    let option_value =
        if let ExprValue::Expression(ExpressionKind::Constructor(constructor)) = expr_value {
            let typ = Value::UnresolvedType(constructor.typ.typ);
            let fields = constructor.fields.into_iter();
            let fields = fields.map(|(name, value)| {
                Value::Tuple(vec![quote_ident(&name), Value::expression(value.kind)])
            });
            let fields = fields.collect();
            let fields_type = Type::Slice(Box::new(Type::Tuple(vec![
                Type::Quoted(QuotedType::Quoted),
                Type::Quoted(QuotedType::Expr),
            ])));
            let fields = Value::Slice(fields, fields_type);
            Some(Value::Tuple(vec![typ, fields]))
        } else {
            None
        };

    option(return_type, option_value, location.span)
}

// fn as_for(self) -> Option<(Quoted, Expr, Expr)>
fn expr_as_for(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Statement(StatementKind::For(for_statement)) = expr {
            if let ForRange::Array(array) = for_statement.range {
                let identifier =
                    Value::Quoted(Rc::new(vec![Token::Ident(for_statement.identifier.0.contents)]));
                let array = Value::expression(array.kind);
                let body = Value::expression(for_statement.block.kind);
                Some(Value::Tuple(vec![identifier, array, body]))
            } else {
                None
            }
        } else {
            None
        }
    })
}

// fn as_for_range(self) -> Option<(Quoted, Expr, Expr, Expr)>
fn expr_as_for_range(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Statement(StatementKind::For(for_statement)) = expr {
            if let ForRange::Range(bounds) = for_statement.range {
                let (from, to) = bounds.into_half_open();
                let identifier =
                    Value::Quoted(Rc::new(vec![Token::Ident(for_statement.identifier.0.contents)]));
                let from = Value::expression(from.kind);
                let to = Value::expression(to.kind);
                let body = Value::expression(for_statement.block.kind);
                Some(Value::Tuple(vec![identifier, from, to, body]))
            } else {
                None
            }
        } else {
            None
        }
    })
}

// fn as_function_call(self) -> Option<(Expr, [Expr])>
fn expr_as_function_call(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Call(call_expression)) = expr {
            let function = Value::expression(call_expression.func.kind);
            let arguments = call_expression.arguments.into_iter();
            let arguments = arguments.map(|argument| Value::expression(argument.kind)).collect();
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
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type.clone(), location, |expr| {
        if let ExprValue::Expression(ExpressionKind::If(if_expr)) = expr {
            // Get the type of `Option<Expr>`
            let option_type = extract_option_generic_type(return_type.clone());
            let Type::Tuple(option_types) = option_type else {
                panic!("Expected the return type option generic arg to be a tuple");
            };
            assert_eq!(option_types.len(), 3);
            let alternative_option_type = option_types[2].clone();

            let alternative = option(
                alternative_option_type,
                if_expr.alternative.map(|e| Value::expression(e.kind)),
                location.span,
            );

            Some(Value::Tuple(vec![
                Value::expression(if_expr.condition.kind),
                Value::expression(if_expr.consequence.kind),
                alternative.ok()?,
            ]))
        } else {
            None
        }
    })
}

// fn as_index(self) -> Option<Expr>
fn expr_as_index(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Index(index_expr)) = expr {
            Some(Value::Tuple(vec![
                Value::expression(index_expr.collection.kind),
                Value::expression(index_expr.index.kind),
            ]))
        } else {
            None
        }
    })
}

// fn as_integer(self) -> Option<(Field, bool)>
fn expr_as_integer(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type.clone(), location, |expr| match expr {
        ExprValue::Expression(ExpressionKind::Literal(Literal::Integer(field, sign))) => {
            Some(Value::Tuple(vec![Value::Field(field), Value::Bool(sign)]))
        }
        ExprValue::Expression(ExpressionKind::Resolved(id)) => {
            if let HirExpression::Literal(HirLiteral::Integer(field, sign)) =
                interner.expression(&id)
            {
                Some(Value::Tuple(vec![Value::Field(field), Value::Bool(sign)]))
            } else {
                None
            }
        }
        _ => None,
    })
}

// fn as_lambda(self) -> Option<([(Expr, Option<UnresolvedType>)], Option<UnresolvedType>, Expr)>
fn expr_as_lambda(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type.clone(), location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Lambda(lambda)) = expr {
            // ([(Expr, Option<UnresolvedType>)], Option<UnresolvedType>, Expr)
            let option_type = extract_option_generic_type(return_type);
            let Type::Tuple(mut tuple_types) = option_type else {
                panic!("Expected the return type option generic arg to be a tuple");
            };
            assert_eq!(tuple_types.len(), 3);

            // Expr
            tuple_types.pop().unwrap();

            // Option<UnresolvedType>
            let option_unresolved_type = tuple_types.pop().unwrap();

            let parameters = lambda
                .parameters
                .into_iter()
                .map(|(pattern, typ)| {
                    let pattern = Value::pattern(pattern);
                    let typ = if let UnresolvedTypeData::Unspecified = typ.typ {
                        None
                    } else {
                        Some(Value::UnresolvedType(typ.typ))
                    };
                    let typ = option(option_unresolved_type.clone(), typ, location.span).unwrap();
                    Value::Tuple(vec![pattern, typ])
                })
                .collect();
            let parameters = Value::Slice(
                parameters,
                Type::Slice(Box::new(Type::Tuple(vec![
                    Type::Quoted(QuotedType::Expr),
                    Type::Quoted(QuotedType::UnresolvedType),
                ]))),
            );

            let return_type = lambda.return_type.typ;
            let return_type = if let UnresolvedTypeData::Unspecified = return_type {
                None
            } else {
                Some(return_type)
            };
            let return_type = return_type.map(Value::UnresolvedType);
            let return_type = option(option_unresolved_type, return_type, location.span).ok()?;

            let body = Value::expression(lambda.body.kind);

            Some(Value::Tuple(vec![parameters, return_type, body]))
        } else {
            None
        }
    })
}

// fn as_let(self) -> Option<(Expr, Option<UnresolvedType>, Expr)>
fn expr_as_let(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type.clone(), location, |expr| match expr {
        ExprValue::Statement(StatementKind::Let(let_statement)) => {
            let option_type = extract_option_generic_type(return_type);
            let Type::Tuple(mut tuple_types) = option_type else {
                panic!("Expected the return type option generic arg to be a tuple");
            };
            assert_eq!(tuple_types.len(), 3);
            tuple_types.pop().unwrap();
            let option_type = tuple_types.pop().unwrap();

            let typ = if let_statement.r#type.typ == UnresolvedTypeData::Unspecified {
                None
            } else {
                Some(Value::UnresolvedType(let_statement.r#type.typ))
            };

            let typ = option(option_type, typ, location.span).ok()?;

            Some(Value::Tuple(vec![
                Value::pattern(let_statement.pattern),
                typ,
                Value::expression(let_statement.expression.kind),
            ]))
        }
        _ => None,
    })
}

// fn as_member_access(self) -> Option<(Expr, Quoted)>
fn expr_as_member_access(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| match expr {
        ExprValue::Expression(ExpressionKind::MemberAccess(member_access)) => {
            Some(Value::Tuple(vec![
                Value::expression(member_access.lhs.kind),
                quote_ident(&member_access.rhs),
            ]))
        }
        ExprValue::LValue(crate::ast::LValue::MemberAccess { object, field_name, span: _ }) => {
            Some(Value::Tuple(vec![Value::lvalue(*object), quote_ident(&field_name)]))
        }
        _ => None,
    })
}

// fn as_method_call(self) -> Option<(Expr, Quoted, [UnresolvedType], [Expr])>
fn expr_as_method_call(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::MethodCall(method_call)) = expr {
            let object = Value::expression(method_call.object.kind);

            let name = quote_ident(&method_call.method_name);

            let generics = method_call.generics.unwrap_or_default().into_iter();
            let generics = generics.map(|generic| Value::UnresolvedType(generic.typ)).collect();
            let generics = Value::Slice(
                generics,
                Type::Slice(Box::new(Type::Quoted(QuotedType::UnresolvedType))),
            );

            let arguments = method_call.arguments.into_iter();
            let arguments = arguments.map(|argument| Value::expression(argument.kind)).collect();
            let arguments =
                Value::Slice(arguments, Type::Slice(Box::new(Type::Quoted(QuotedType::Expr))));

            Some(Value::Tuple(vec![object, name, generics, arguments]))
        } else {
            None
        }
    })
}

// fn as_repeated_element_array(self) -> Option<(Expr, Expr)>
fn expr_as_repeated_element_array(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Literal(Literal::Array(
            ArrayLiteral::Repeated { repeated_element, length },
        ))) = expr
        {
            Some(Value::Tuple(vec![
                Value::expression(repeated_element.kind),
                Value::expression(length.kind),
            ]))
        } else {
            None
        }
    })
}

// fn as_repeated_element_slice(self) -> Option<(Expr, Expr)>
fn expr_as_repeated_element_slice(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Literal(Literal::Slice(
            ArrayLiteral::Repeated { repeated_element, length },
        ))) = expr
        {
            Some(Value::Tuple(vec![
                Value::expression(repeated_element.kind),
                Value::expression(length.kind),
            ]))
        } else {
            None
        }
    })
}

// fn as_slice(self) -> Option<[Expr]>
fn expr_as_slice(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Literal(Literal::Slice(
            ArrayLiteral::Standard(exprs),
        ))) = expr
        {
            let exprs = exprs.into_iter().map(|expr| Value::expression(expr.kind)).collect();
            let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Expr)));
            Some(Value::Slice(exprs, typ))
        } else {
            None
        }
    })
}

// fn as_tuple(self) -> Option<[Expr]>
fn expr_as_tuple(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Tuple(expressions)) = expr {
            let expressions =
                expressions.into_iter().map(|expr| Value::expression(expr.kind)).collect();
            let typ = Type::Slice(Box::new(Type::Quoted(QuotedType::Expr)));
            Some(Value::Slice(expressions, typ))
        } else {
            None
        }
    })
}

// fn as_unary_op(self) -> Option<(UnaryOp, Expr)>
fn expr_as_unary_op(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type.clone(), location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Prefix(prefix_expr)) = expr {
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
            let rhs = Value::expression(prefix_expr.rhs.kind);
            Some(Value::Tuple(vec![unary_op, rhs]))
        } else {
            None
        }
    })
}

// fn as_unsafe(self) -> Option<[Expr]>
fn expr_as_unsafe(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    expr_as(interner, arguments, return_type, location, |expr| {
        if let ExprValue::Expression(ExpressionKind::Unsafe(block_expr, _)) = expr {
            Some(block_expression_to_value(block_expr))
        } else {
            None
        }
    })
}

// fn as_has_semicolon(self) -> bool
fn expr_has_semicolon(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let expr_value = get_expr(interner, self_argument)?;
    Ok(Value::Bool(matches!(expr_value, ExprValue::Statement(StatementKind::Semi(..)))))
}

// fn is_break(self) -> bool
fn expr_is_break(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let expr_value = get_expr(interner, self_argument)?;
    Ok(Value::Bool(matches!(expr_value, ExprValue::Statement(StatementKind::Break))))
}

// fn is_continue(self) -> bool
fn expr_is_continue(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let expr_value = get_expr(interner, self_argument)?;
    Ok(Value::Bool(matches!(expr_value, ExprValue::Statement(StatementKind::Continue))))
}

// Helper function for implementing the `expr_as_...` functions.
fn expr_as<F>(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
    f: F,
) -> IResult<Value>
where
    F: FnOnce(ExprValue) -> Option<Value>,
{
    let self_argument = check_one_argument(arguments, location)?;
    let expr_value = get_expr(interner, self_argument)?;
    let expr_value = unwrap_expr_value(interner, expr_value);

    let option_value = f(expr_value);
    option(return_type, option_value, location.span)
}

// fn resolve(self, in_function: Option<FunctionDefinition>) -> TypedExpr
fn expr_resolve(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, func) = check_two_arguments(arguments, location)?;
    let self_argument_location = self_argument.1;
    let expr_value = get_expr(interpreter.elaborator.interner, self_argument)?;
    let expr_value = unwrap_expr_value(interpreter.elaborator.interner, expr_value);

    let Value::Struct(fields, _) = func.0 else {
        panic!("Expected second argument to be a struct");
    };

    let is_some = fields.get(&Rc::new("_is_some".to_string())).unwrap();
    let Value::Bool(is_some) = is_some else {
        panic!("Expected is_some to be a boolean");
    };

    let function_to_resolve_in = if *is_some {
        let value = fields.get(&Rc::new("_value".to_string())).unwrap();
        let Value::FunctionDefinition(func_id) = value else {
            panic!("Expected option value to be a FunctionDefinition");
        };
        Some(*func_id)
    } else {
        interpreter.current_function
    };

    interpreter.elaborate_in_function(function_to_resolve_in, |elaborator| match expr_value {
        ExprValue::Expression(expression_kind) => {
            let expr = Expression { kind: expression_kind, span: self_argument_location.span };
            let (expr_id, _) = elaborator.elaborate_expression(expr);
            Ok(Value::TypedExpr(TypedExpr::ExprId(expr_id)))
        }
        ExprValue::Statement(statement_kind) => {
            let statement = Statement { kind: statement_kind, span: self_argument_location.span };
            let (stmt_id, _) = elaborator.elaborate_statement(statement);
            Ok(Value::TypedExpr(TypedExpr::StmtId(stmt_id)))
        }
        ExprValue::LValue(lvalue) => {
            let expr = lvalue.as_expression();
            let (expr_id, _) = elaborator.elaborate_expression(expr);
            Ok(Value::TypedExpr(TypedExpr::ExprId(expr_id)))
        }
        ExprValue::Pattern(pattern) => {
            if let Some(expression) = pattern.try_as_expression(elaborator.interner) {
                let (expr_id, _) = elaborator.elaborate_expression(expression);
                Ok(Value::TypedExpr(TypedExpr::ExprId(expr_id)))
            } else {
                let expression = Value::pattern(pattern).display(elaborator.interner).to_string();
                let location = self_argument_location;
                Err(InterpreterError::CannotResolveExpression { location, expression })
            }
        }
    })
}

fn unwrap_expr_value(interner: &NodeInterner, mut expr_value: ExprValue) -> ExprValue {
    loop {
        match expr_value {
            ExprValue::Expression(ExpressionKind::Parenthesized(expression)) => {
                expr_value = ExprValue::Expression(expression.kind);
            }
            ExprValue::Statement(StatementKind::Expression(expression))
            | ExprValue::Statement(StatementKind::Semi(expression)) => {
                expr_value = ExprValue::Expression(expression.kind);
            }
            ExprValue::Expression(ExpressionKind::Interned(id)) => {
                expr_value = ExprValue::Expression(interner.get_expression_kind(id).clone());
            }
            ExprValue::Statement(StatementKind::Interned(id)) => {
                expr_value = ExprValue::Statement(interner.get_statement_kind(id).clone());
            }
            ExprValue::LValue(LValue::Interned(id, span)) => {
                expr_value = ExprValue::LValue(interner.get_lvalue(id, span).clone());
            }
            ExprValue::Pattern(Pattern::Interned(id, _)) => {
                expr_value = ExprValue::Pattern(interner.get_pattern(id).clone());
            }
            _ => break,
        }
    }
    expr_value
}

// fn fmtstr_as_ctstring(self) -> CtString
fn fmtstr_as_ctstring(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let (string, _) = get_format_string(interner, self_argument)?;
    Ok(Value::CtString(string))
}

// fn quoted_contents(self) -> Quoted
fn fmtstr_quoted_contents(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let (string, _) = get_format_string(interner, self_argument)?;
    let tokens = lex(&string);
    Ok(Value::Quoted(Rc::new(tokens)))
}

// fn fresh_type_variable() -> Type
fn fresh_type_variable(interner: &NodeInterner) -> IResult<Value> {
    Ok(Value::Type(interner.next_type_variable_with_kind(Kind::Any)))
}

// fn add_attribute<let N: u32>(self, attribute: str<N>)
fn function_def_add_attribute(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, attribute) = check_two_arguments(arguments, location)?;
    let attribute_location = attribute.1;
    let attribute = get_str(interpreter.elaborator.interner, attribute)?;
    let attribute = format!("#[{}]", attribute);
    let mut parser = Parser::for_str(&attribute);
    let Some((attribute, _span)) = parser.parse_attribute() else {
        return Err(InterpreterError::InvalidAttribute {
            attribute: attribute.to_string(),
            location: attribute_location,
        });
    };

    let func_id = get_function_def(self_argument)?;
    check_function_not_yet_resolved(interpreter, func_id, location)?;

    let function_modifiers = interpreter.elaborator.interner.function_modifiers_mut(&func_id);

    match &attribute {
        Attribute::Function(attribute) => {
            function_modifiers.attributes.set_function(attribute.clone());
        }
        Attribute::Secondary(attribute) => {
            function_modifiers.attributes.secondary.push(attribute.clone());
        }
    }

    Ok(Value::Unit)
}

// fn body(self) -> Expr
fn function_def_body(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let func_id = get_function_def(self_argument)?;
    let func_meta = interner.function_meta(&func_id);
    if let FunctionBody::Unresolved(_, block_expr, _) = &func_meta.function_body {
        Ok(Value::expression(ExpressionKind::Block(block_expr.clone())))
    } else {
        Err(InterpreterError::FunctionAlreadyResolved { location })
    }
}

// fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
fn function_def_has_named_attribute(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, name) = check_two_arguments(arguments, location)?;
    let func_id = get_function_def(self_argument)?;

    let name = &*get_str(interner, name)?;

    let modifiers = interner.function_modifiers(&func_id);
    if let Some(attribute) = modifiers.attributes.function() {
        if name == attribute.name() {
            return Ok(Value::Bool(true));
        }
    }

    Ok(Value::Bool(has_named_attribute(name, &modifiers.attributes.secondary)))
}

fn function_def_hash(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    hash_item(arguments, location, get_function_def)
}

fn function_def_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    eq_item(arguments, location, get_function_def)
}

// fn is_unconstrained(self) -> bool
fn function_def_is_unconstrained(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let func_id = get_function_def(self_argument)?;
    let is_unconstrained = interner.function_modifiers(&func_id).is_unconstrained;
    Ok(Value::Bool(is_unconstrained))
}

// fn module(self) -> Module
fn function_def_module(
    interner: &NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let func_id = get_function_def(self_argument)?;
    let module = interner.function_module(func_id);
    Ok(Value::ModuleDefinition(module))
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

// fn set_body(self, body: Expr)
fn function_def_set_body(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, body_argument) = check_two_arguments(arguments, location)?;
    let body_location = body_argument.1;

    let func_id = get_function_def(self_argument)?;
    check_function_not_yet_resolved(interpreter, func_id, location)?;

    let body_argument = get_expr(interpreter.elaborator.interner, body_argument)?;
    let statement_kind = match body_argument {
        ExprValue::Expression(expression_kind) => StatementKind::Expression(Expression {
            kind: expression_kind,
            span: body_location.span,
        }),
        ExprValue::Statement(statement_kind) => statement_kind,
        ExprValue::LValue(lvalue) => StatementKind::Expression(lvalue.as_expression()),
        ExprValue::Pattern(pattern) => {
            if let Some(expression) = pattern.try_as_expression(interpreter.elaborator.interner) {
                StatementKind::Expression(expression)
            } else {
                let expression =
                    Value::pattern(pattern).display(interpreter.elaborator.interner).to_string();
                let location = body_location;
                return Err(InterpreterError::CannotSetFunctionBody { location, expression });
            }
        }
    };

    let statement = Statement { kind: statement_kind, span: body_location.span };
    let body = BlockExpression { statements: vec![statement] };

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
            interpreter.elaborator,
            (tuple.pop().unwrap(), parameters_argument_location),
            Parser::parse_pattern_or_error,
            "a pattern",
        )?;

        let hir_pattern = interpreter.elaborate_in_function(Some(func_id), |elaborator| {
            elaborator.elaborate_pattern_and_store_ids(
                parameter_pattern,
                parameter_type.clone(),
                DefinitionKind::Local(None),
                &mut parameter_idents,
                true, // warn_if_unused
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

// fn set_return_public(self, public: bool)
fn function_def_set_return_public(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, public) = check_two_arguments(arguments, location)?;

    let func_id = get_function_def(self_argument)?;
    check_function_not_yet_resolved(interpreter, func_id, location)?;

    let public = get_bool(public)?;

    let func_meta = interpreter.elaborator.interner.function_meta_mut(&func_id);
    func_meta.return_visibility = if public { Visibility::Public } else { Visibility::Private };

    Ok(Value::Unit)
}

// fn set_return_data(self)
fn function_def_set_return_data(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;

    let func_id = get_function_def(self_argument)?;
    check_function_not_yet_resolved(interpreter, func_id, location)?;

    let func_meta = interpreter.elaborator.interner.function_meta_mut(&func_id);
    func_meta.return_visibility = Visibility::ReturnData;

    Ok(Value::Unit)
}

// fn set_unconstrained(self, value: bool)
fn function_def_set_unconstrained(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, unconstrained) = check_two_arguments(arguments, location)?;

    let func_id = get_function_def(self_argument)?;
    check_function_not_yet_resolved(interpreter, func_id, location)?;

    let unconstrained = get_bool(unconstrained)?;

    let modifiers = interpreter.elaborator.interner.function_modifiers_mut(&func_id);
    modifiers.is_unconstrained = unconstrained;

    Ok(Value::Unit)
}

// fn add_item(self, item: Quoted)
fn module_add_item(
    interpreter: &mut Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, item) = check_two_arguments(arguments, location)?;
    let module_id = get_module(self_argument)?;

    let parser = Parser::parse_top_level_items;
    let top_level_statements = parse(interpreter.elaborator, item, parser, "a top-level item")?;

    let module_data = interpreter.elaborator.get_module(module_id);
    interpreter.elaborate_in_module(module_id, module_data.location.file, |elaborator| {
        let mut generated_items = CollectedItems::default();

        for top_level_statement in top_level_statements {
            elaborator.add_item(top_level_statement, &mut generated_items, location);
        }

        if !generated_items.is_empty() {
            elaborator.elaborate_items(generated_items);
        }
    });

    Ok(Value::Unit)
}

fn module_hash(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    hash_item(arguments, location, get_module)
}

fn module_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    eq_item(arguments, location, get_module)
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
        .definitions()
        .definitions()
        .iter()
        .filter_map(|module_def_id| {
            if let ModuleDefId::FunctionId(func_id) = module_def_id {
                Some(Value::FunctionDefinition(*func_id))
            } else {
                None
            }
        })
        .collect();

    let slice_type = Type::Slice(Box::new(Type::Quoted(QuotedType::FunctionDefinition)));
    Ok(Value::Slice(func_ids, slice_type))
}

// fn structs(self) -> [StructDefinition]
fn module_structs(
    interpreter: &Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let self_argument = check_one_argument(arguments, location)?;
    let module_id = get_module(self_argument)?;
    let module_data = interpreter.elaborator.get_module(module_id);
    let struct_ids = module_data
        .definitions()
        .definitions()
        .iter()
        .filter_map(|module_def_id| {
            if let ModuleDefId::TypeId(id) = module_def_id {
                Some(Value::StructDefinition(*id))
            } else {
                None
            }
        })
        .collect();

    let slice_type = Type::Slice(Box::new(Type::Quoted(QuotedType::StructDefinition)));
    Ok(Value::Slice(struct_ids, slice_type))
}

// fn has_named_attribute<let N: u32>(self, name: str<N>) -> bool {}
fn module_has_named_attribute(
    interpreter: &Interpreter,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let (self_argument, name) = check_two_arguments(arguments, location)?;
    let module_id = get_module(self_argument)?;
    let module_data = interpreter.elaborator.get_module(module_id);

    let name = get_str(interpreter.elaborator.interner, name)?;

    Ok(Value::Bool(has_named_attribute(&name, &module_data.attributes)))
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

fn modulus_be_bits(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    check_argument_count(0, &arguments, location)?;

    let bits = FieldElement::modulus().to_radix_be(2);
    let bits_vector = bits.into_iter().map(|bit| Value::U1(bit != 0)).collect();

    let int_type = Type::Integer(crate::ast::Signedness::Unsigned, IntegerBitSize::One);
    let typ = Type::Slice(Box::new(int_type));
    Ok(Value::Slice(bits_vector, typ))
}

fn modulus_be_bytes(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    check_argument_count(0, &arguments, location)?;

    let bytes = FieldElement::modulus().to_bytes_be();
    let bytes_vector = bytes.into_iter().map(Value::U8).collect();

    let int_type = Type::Integer(crate::ast::Signedness::Unsigned, IntegerBitSize::Eight);
    let typ = Type::Slice(Box::new(int_type));
    Ok(Value::Slice(bytes_vector, typ))
}

fn modulus_le_bits(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let Value::Slice(bits, typ) = modulus_be_bits(arguments, location)? else {
        unreachable!("modulus_be_bits must return slice")
    };
    let reversed_bits = bits.into_iter().rev().collect();
    Ok(Value::Slice(reversed_bits, typ))
}

fn modulus_le_bytes(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let Value::Slice(bytes, typ) = modulus_be_bytes(arguments, location)? else {
        unreachable!("modulus_be_bytes must return slice")
    };
    let reversed_bytes = bytes.into_iter().rev().collect();
    Ok(Value::Slice(reversed_bytes, typ))
}

fn modulus_num_bits(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    check_argument_count(0, &arguments, location)?;
    let bits = FieldElement::max_num_bits().into();
    Ok(Value::U64(bits))
}

// fn quoted_eq(_first: Quoted, _second: Quoted) -> bool
fn quoted_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    eq_item(arguments, location, get_quoted)
}

fn quoted_hash(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    hash_item(arguments, location, get_quoted)
}

fn trait_def_as_trait_constraint(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    location: Location,
) -> IResult<Value> {
    let argument = check_one_argument(arguments, location)?;

    let trait_id = get_trait_def(argument)?;
    let constraint = interner.get_trait(trait_id).as_constraint(location.span);

    Ok(Value::TraitConstraint(trait_id, constraint.trait_bound.trait_generics))
}

/// Creates a value that holds an `Option`.
/// `option_type` must be a Type referencing the `Option` type.
pub(crate) fn option(option_type: Type, value: Option<Value>, span: Span) -> IResult<Value> {
    let t = extract_option_generic_type(option_type.clone());

    let (is_some, value) = match value {
        Some(value) => (Value::Bool(true), value),
        None => (Value::Bool(false), zeroed(t, span)?),
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

fn ctstring_eq(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    eq_item(arguments, location, get_ctstring)
}

fn ctstring_hash(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    hash_item(arguments, location, get_ctstring)
}

fn derive_generators(
    interner: &mut NodeInterner,
    arguments: Vec<(Value, Location)>,
    return_type: Type,
    location: Location,
) -> IResult<Value> {
    let (domain_separator_string, starting_index) = check_two_arguments(arguments, location)?;

    let domain_separator_location = domain_separator_string.1;
    let (domain_separator_string, _) = get_array(interner, domain_separator_string)?;
    let starting_index = get_u32(starting_index)?;

    let domain_separator_string =
        try_vecmap(domain_separator_string, |byte| get_u8((byte, domain_separator_location)))?;

    let (size, elements) = match return_type.clone() {
        Type::Array(size, elements) => (size, elements),
        _ => panic!("ICE: Should only have an array return type"),
    };

    let num_generators = size.evaluate_to_u32(location.span).map_err(|err| {
        let err = Box::new(err);
        InterpreterError::UnknownArrayLength { length: *size, err, location }
    })?;

    let generators = bn254_blackbox_solver::derive_generators(
        &domain_separator_string,
        num_generators,
        starting_index,
    );

    let is_infinite = FieldElement::zero();
    let x_field_name: Rc<String> = Rc::new("x".to_owned());
    let y_field_name: Rc<String> = Rc::new("y".to_owned());
    let is_infinite_field_name: Rc<String> = Rc::new("is_infinite".to_owned());
    let mut results = Vector::new();
    for gen in generators {
        let x_big: BigUint = gen.x.into();
        let x = FieldElement::from_be_bytes_reduce(&x_big.to_bytes_be());
        let y_big: BigUint = gen.y.into();
        let y = FieldElement::from_be_bytes_reduce(&y_big.to_bytes_be());
        let mut embedded_curve_point_fields = HashMap::default();
        embedded_curve_point_fields.insert(x_field_name.clone(), Value::Field(x));
        embedded_curve_point_fields.insert(y_field_name.clone(), Value::Field(y));
        embedded_curve_point_fields
            .insert(is_infinite_field_name.clone(), Value::Field(is_infinite));
        let embedded_curve_point_struct =
            Value::Struct(embedded_curve_point_fields, *elements.clone());
        results.push_back(embedded_curve_point_struct);
    }

    Ok(Value::Array(results, return_type))
}

fn field_less_than(arguments: Vec<(Value, Location)>, location: Location) -> IResult<Value> {
    let (lhs, rhs) = check_two_arguments(arguments, location)?;

    let lhs = get_field(lhs)?;
    let rhs = get_field(rhs)?;

    Ok(Value::Bool(lhs < rhs))
}
