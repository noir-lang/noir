use noirc_errors::Location;

use crate::{
    hir::comptime::{errors::IResult, Value},
    Type,
};

pub(super) fn array_len(arguments: &[(Value, Location)]) -> IResult<Value> {
    assert_eq!(arguments.len(), 1, "ICE: `array_len` should only receive a single argument");
    match &arguments[0].0 {
        Value::Array(values, _) | Value::Slice(values, _) => Ok(Value::U32(values.len() as u32)),
        // Type checking should prevent this branch being taken.
        _ => unreachable!("ICE: Cannot query length of types other than arrays or slices"),
    }
}

pub(super) fn as_slice(mut arguments: Vec<(Value, Location)>) -> IResult<Value> {
    assert_eq!(arguments.len(), 1, "ICE: `as_slice` should only receive a single argument");
    let (array, _) = arguments.pop().unwrap();
    match array {
        Value::Array(values, Type::Array(_, typ)) => Ok(Value::Slice(values, Type::Slice(typ))),
        // Type checking should prevent this branch being taken.
        _ => unreachable!("ICE: Cannot convert types other than arrays into slices"),
    }
}
