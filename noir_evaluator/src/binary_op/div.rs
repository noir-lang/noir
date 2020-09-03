use super::{invert, mul::handle_mul_op};
use crate::{Environment, Evaluator, Polynomial};

/// For a / b . First compute the 1/b and constraint it to be the inverse of b
/// Then multiply this inverse by a
pub fn handle_div_op(
    left: Polynomial,
    right: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    let right_inv = invert(right, env, evaluator);
    handle_mul_op(left, right_inv, env, evaluator)
}
