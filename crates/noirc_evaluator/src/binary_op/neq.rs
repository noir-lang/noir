use super::{invert, sub::handle_sub_op};
use crate::{object::Array, Evaluator, Object, RuntimeErrorKind};

/// This calls the sub op under the hood
/// Then asserts that the result has an inverse
/// ie a != b => a-b has an inverse => 1/(a-b) * (a-b) = 1
pub fn handle_neq_op(
    left: Object,
    right: Object,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    match (left, right) {
        (Object::Array(left_arr), Object::Array(right_arr)) => {
            Array::not_equal(left_arr, right_arr, evaluator)?;
        }
        (left, right) => {
            let result = handle_sub_op(left, right, evaluator)?;
            // Add an inversion to ensure that the inverse exists
            let _ = invert(result, evaluator);
        }
    }
    Ok(Object::Null)
}
