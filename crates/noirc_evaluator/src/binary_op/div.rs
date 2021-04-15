use noir_field::FieldElement;

use super::{invert, mul::handle_mul_op};
use crate::{Evaluator, Object, RuntimeErrorKind};

/// For a / b . First compute the 1/b and constraint it to be the inverse of b
/// Then multiply this inverse by a
pub fn handle_div_op<F: FieldElement>(
    left: Object<F>,
    right: Object<F>,
    evaluator: &mut Evaluator<F>,
) -> Result<Object<F>, RuntimeErrorKind> {
    let right_inv = invert(right, evaluator)?;
    handle_mul_op(left, right_inv, evaluator)
}
