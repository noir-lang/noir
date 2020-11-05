use super::{invert, sub::handle_sub_op};
use crate::{Environment, Evaluator, Object, EvaluatorError};

/// This calls the sub op under the hood
/// Then asserts that the result has an inverse
/// ie a != b => a-b has an inverse => 1/(a-b) * (a-b) = 1
pub fn handle_neq_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Result<Object, EvaluatorError> {
    let result = handle_sub_op(left, right, env, evaluator)?;
    // Add an inversion to ensure that the inverse exists
    let _ = invert(result, env, evaluator);
    Ok(Object::Null)
}
