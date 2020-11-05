use super::{invert, mul::handle_mul_op};
use crate::{Environment, Evaluator, Object, EvaluatorError};

/// For a / b . First compute the 1/b and constraint it to be the inverse of b
/// Then multiply this inverse by a
pub fn handle_div_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Result<Object, EvaluatorError> {
    let right_inv = invert(right, env, evaluator)?;
    handle_mul_op(left, right_inv, env, evaluator)
}
