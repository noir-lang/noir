use noirc_errors::Location;

use crate::hir::comptime::{errors::IResult, Value};

pub(super) fn array_len(arguments: &[(Value, Location)]) -> IResult<Value> {
    assert_eq!(arguments.len(), 1, "ICE: `array_len` should only receive a single argument");
    match &arguments[0].0 {
        Value::Array(values, _) | Value::Slice(values, _) => Ok(Value::U32(values.len() as u32)),
        // Type checking should prevent this branch being taken.
        _ => unreachable!("ICE: Cannot query length of types other than arrays or slices"),
    }
}
