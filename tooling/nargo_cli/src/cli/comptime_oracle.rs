use std::rc::Rc;

use acvm::acir::brillig::{ForeignCallParam, ForeignCallResult};
use acvm::pwg::ForeignCallWaitInfo;
use acvm::{AcirField, FieldElement};
use noirc_errors::Location;
use noirc_frontend::ast::IntegerBitSize;
use noirc_frontend::hir::comptime::{ComptimeOracleExecutor, InterpreterError, Value};
use noirc_frontend::shared::Signedness;
use noirc_frontend::{Shared, Type};
use rustc_hash::FxHashMap as HashMap;

use nargo::foreign_calls::{DefaultForeignCallBuilder, ForeignCallExecutor};

/// Bridges comptime `Value` to the existing `ForeignCallExecutor<FieldElement>` infrastructure.
pub(crate) struct ComptimeForeignCallExecutor {
    executor: Box<dyn ForeignCallExecutor<FieldElement>>,
}

impl ComptimeForeignCallExecutor {
    pub(crate) fn new() -> Self {
        let executor = DefaultForeignCallBuilder::default()
            .with_output(std::io::stdout())
            .build::<FieldElement>();
        Self { executor: Box::new(executor) }
    }
}

impl ComptimeOracleExecutor for ComptimeForeignCallExecutor {
    fn execute_oracle(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
        return_type: &Type,
        location: Location,
    ) -> Result<Value, InterpreterError> {
        let inputs: Vec<ForeignCallParam<FieldElement>> =
            arguments.iter().map(value_to_foreign_call_param).collect();

        let foreign_call = ForeignCallWaitInfo { function: name.to_string(), inputs };

        let result = self.executor.execute(&foreign_call).map_err(|err| {
            InterpreterError::Unimplemented {
                item: format!("Oracle '{name}' failed: {err}"),
                location,
            }
        })?;

        foreign_call_result_to_value(&result, return_type, location)
    }
}

/// Flatten a comptime `Value` into a `ForeignCallParam<FieldElement>`.
fn value_to_foreign_call_param(value: &Value) -> ForeignCallParam<FieldElement> {
    match value {
        Value::Integer(int) => ForeignCallParam::Single(int.as_field()),
        Value::Bool(b) => {
            ForeignCallParam::Single(if *b { FieldElement::one() } else { FieldElement::zero() })
        }
        Value::Unit => ForeignCallParam::Array(vec![]),
        Value::String(bytes) => {
            let fields: Vec<_> = bytes.iter().map(|b| FieldElement::from(u128::from(*b))).collect();
            ForeignCallParam::Array(fields)
        }
        Value::Array(values, _) | Value::Vector(values, _) => {
            let fields: Vec<_> = values.iter().flat_map(flatten_value_to_fields).collect();
            ForeignCallParam::Array(fields)
        }
        Value::Tuple(values) => {
            let fields: Vec<_> =
                values.iter().flat_map(|v| flatten_value_to_fields(&v.borrow())).collect();
            ForeignCallParam::Array(fields)
        }
        Value::Struct(field_map, typ) => {
            let fields = get_ordered_struct_fields(field_map, typ);
            let flat: Vec<_> = fields.iter().flat_map(flatten_value_to_fields).collect();
            ForeignCallParam::Array(flat)
        }
        _ => ForeignCallParam::Single(FieldElement::zero()),
    }
}

/// Recursively flatten a `Value` into a sequence of `FieldElement`.
fn flatten_value_to_fields(value: &Value) -> Vec<FieldElement> {
    match value {
        Value::Integer(int) => vec![int.as_field()],
        Value::Bool(b) => {
            vec![if *b { FieldElement::one() } else { FieldElement::zero() }]
        }
        Value::Unit => vec![],
        Value::String(bytes) => bytes.iter().map(|b| FieldElement::from(u128::from(*b))).collect(),
        Value::Array(values, _) | Value::Vector(values, _) => {
            values.iter().flat_map(flatten_value_to_fields).collect()
        }
        Value::Tuple(values) => {
            values.iter().flat_map(|v| flatten_value_to_fields(&v.borrow())).collect()
        }
        Value::Struct(field_map, typ) => {
            let fields = get_ordered_struct_fields(field_map, typ);
            fields.iter().flat_map(flatten_value_to_fields).collect()
        }
        _ => vec![FieldElement::zero()],
    }
}

/// Get struct field values in declaration order (matching how Brillig serializes them).
fn get_ordered_struct_fields(
    field_map: &HashMap<Rc<String>, Shared<Value>>,
    typ: &Type,
) -> Vec<Value> {
    match typ.follow_bindings() {
        Type::DataType(def, generics) => {
            let def = def.borrow();
            if let Some(fields) = def.get_fields(&generics) {
                fields
                    .iter()
                    .map(|(name, _, _)| {
                        field_map
                            .get(&Rc::new(name.clone()))
                            .map_or(Value::Unit, |v| v.borrow().clone())
                    })
                    .collect()
            } else {
                field_map.values().map(|v| v.borrow().clone()).collect()
            }
        }
        _ => field_map.values().map(|v| v.borrow().clone()).collect(),
    }
}

/// Reconstruct a comptime `Value` from a `ForeignCallResult`, guided by the expected return type.
fn foreign_call_result_to_value(
    result: &ForeignCallResult<FieldElement>,
    return_type: &Type,
    location: Location,
) -> Result<Value, InterpreterError> {
    let return_type = return_type.follow_bindings();

    if matches!(return_type, Type::Unit) {
        return Ok(Value::Unit);
    }

    // Collect all field elements from the result into a flat sequence.
    let flat_fields: Vec<FieldElement> = result
        .values
        .iter()
        .flat_map(|param| match param {
            ForeignCallParam::Single(f) => vec![*f],
            ForeignCallParam::Array(arr) => arr.clone(),
        })
        .collect();

    // For vectors, we also need the array lengths from the result.
    let array_lengths: Vec<usize> = result
        .values
        .iter()
        .map(|param| match param {
            ForeignCallParam::Single(_) => 1,
            ForeignCallParam::Array(arr) => arr.len(),
        })
        .collect();

    let mut cursor = 0;
    let mut array_idx = 0;
    unflatten_value(
        &flat_fields,
        &array_lengths,
        &mut cursor,
        &mut array_idx,
        &return_type,
        location,
    )
}

/// Recursively reconstruct a `Value` from flat field elements, guided by the expected type.
fn unflatten_value(
    fields: &[FieldElement],
    array_lengths: &[usize],
    cursor: &mut usize,
    array_idx: &mut usize,
    typ: &Type,
    location: Location,
) -> Result<Value, InterpreterError> {
    match typ {
        Type::Unit => Ok(Value::Unit),
        Type::Bool => {
            let field = take_field(fields, cursor, location)?;
            Ok(Value::Bool(!field.is_zero()))
        }
        Type::FieldElement => {
            let field = take_field(fields, cursor, location)?;
            Ok(Value::field(field))
        }
        Type::Integer(signedness, bit_size) => {
            let field = take_field(fields, cursor, location)?;
            Ok(field_to_integer_value(field, *signedness, *bit_size))
        }
        Type::String(length) => {
            let len = length.evaluate_to_u32(location).expect("Could not evaluate string length")
                as usize;
            let mut bytes = Vec::with_capacity(len);
            for _ in 0..len {
                let field = take_field(fields, cursor, location)?;
                let byte = field.try_into_u128().expect("String byte should fit in u128") as u8;
                bytes.push(byte);
            }
            Ok(Value::String(Rc::new(bytes)))
        }
        Type::Array(element_typ, length) => {
            let len =
                length.evaluate_to_u32(location).expect("Could not evaluate array length") as usize;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                let value = unflatten_value(
                    fields,
                    array_lengths,
                    cursor,
                    array_idx,
                    element_typ,
                    location,
                )?;
                values.push(value);
            }
            Ok(Value::Array(values.into(), typ.clone()))
        }
        Type::Vector(element_typ) => {
            let elem_field_count = type_field_count(element_typ);
            let total_fields = if *array_idx < array_lengths.len() {
                let len = array_lengths[*array_idx];
                *array_idx += 1;
                len
            } else {
                0
            };
            let elem_count =
                if elem_field_count > 0 { total_fields / elem_field_count } else { total_fields };

            let mut values = Vec::with_capacity(elem_count);
            for _ in 0..elem_count {
                let value = unflatten_value(
                    fields,
                    array_lengths,
                    cursor,
                    array_idx,
                    element_typ,
                    location,
                )?;
                values.push(value);
            }
            Ok(Value::Vector(values.into(), typ.clone()))
        }
        Type::Tuple(types) => {
            let mut values = Vec::with_capacity(types.len());
            for elem_type in types {
                let value =
                    unflatten_value(fields, array_lengths, cursor, array_idx, elem_type, location)?;
                values.push(Shared::new(value));
            }
            Ok(Value::Tuple(values))
        }
        Type::DataType(def, generics) => {
            let def_borrowed = def.borrow();
            if let Some(struct_fields) = def_borrowed.get_fields(generics) {
                let mut field_map = HashMap::default();
                for (name, field_type, _) in &struct_fields {
                    let value = unflatten_value(
                        fields,
                        array_lengths,
                        cursor,
                        array_idx,
                        field_type,
                        location,
                    )?;
                    field_map.insert(Rc::new(name.clone()), Shared::new(value));
                }
                drop(def_borrowed);
                Ok(Value::Struct(field_map, typ.clone()))
            } else {
                drop(def_borrowed);
                Err(InterpreterError::Unimplemented {
                    item: "Oracle return value with enum type".to_string(),
                    location,
                })
            }
        }
        Type::Alias(alias, generics) => {
            let resolved = alias.borrow().get_type(generics);
            unflatten_value(fields, array_lengths, cursor, array_idx, &resolved, location)
        }
        _ => Err(InterpreterError::Unimplemented {
            item: format!("Oracle return type {typ}"),
            location,
        }),
    }
}

/// Take a single field element from the flat sequence.
fn take_field(
    fields: &[FieldElement],
    cursor: &mut usize,
    location: Location,
) -> Result<FieldElement, InterpreterError> {
    if *cursor < fields.len() {
        let field = fields[*cursor];
        *cursor += 1;
        Ok(field)
    } else {
        Err(InterpreterError::Unimplemented {
            item: "Oracle returned fewer values than expected".to_string(),
            location,
        })
    }
}

/// Convert a field element to an integer `Value` based on signedness and bit size.
fn field_to_integer_value(
    field: FieldElement,
    signedness: Signedness,
    bit_size: IntegerBitSize,
) -> Value {
    let raw = field.try_into_u128().unwrap_or(0);
    match (signedness, bit_size) {
        (Signedness::Unsigned, IntegerBitSize::Eight) => Value::u8(raw as u8),
        (Signedness::Unsigned, IntegerBitSize::Sixteen) => Value::u16(raw as u16),
        (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => Value::u32(raw as u32),
        (Signedness::Unsigned, IntegerBitSize::SixtyFour) => Value::u64(raw as u64),
        (Signedness::Unsigned, IntegerBitSize::HundredTwentyEight) => Value::u128(raw),
        (Signedness::Signed, IntegerBitSize::Eight) => Value::i8(raw as u8 as i8),
        (Signedness::Signed, IntegerBitSize::Sixteen) => Value::i16(raw as u16 as i16),
        (Signedness::Signed, IntegerBitSize::ThirtyTwo) => Value::i32(raw as u32 as i32),
        (Signedness::Signed, IntegerBitSize::SixtyFour) => Value::i64(raw as u64 as i64),
        _ => Value::field(field),
    }
}

/// Estimate how many field elements a type takes when flattened.
fn type_field_count(typ: &Type) -> usize {
    match typ {
        Type::Unit => 0,
        Type::Bool | Type::FieldElement => 1,
        Type::Integer(..) => 1,
        Type::String(length) => {
            length.evaluate_to_u32(Location::dummy()).map(|l| l as usize).unwrap_or(0)
        }
        Type::Array(elem, length) => {
            let elem_count = type_field_count(elem);
            let len = length.evaluate_to_u32(Location::dummy()).map(|l| l as usize).unwrap_or(0);
            elem_count * len
        }
        Type::Tuple(types) => types.iter().map(type_field_count).sum(),
        // Dynamic-length types take a variable number of fields.
        Type::Vector(_) => 1,
        _ => 1,
    }
}
