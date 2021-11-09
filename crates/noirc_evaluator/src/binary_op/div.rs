use super::{invert, mul::handle_mul_op};
use crate::{Evaluator, Object, RuntimeErrorKind};

/// For a / b . First compute the 1/b and constraint it to be the inverse of b
/// Then multiply this inverse by a
pub fn handle_div_op_default(
    left: Object,
    right: Object,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    let right_inv = invert(right, evaluator)?;
    handle_mul_op(left, right_inv, evaluator)
}

pub fn handle_div_op(
    left: Object,
    right: Object,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    match (left, right)
    {
        (Object::Integer(a), Object::Integer(b))
                =>  Ok(Object::Integer(a.div(Object::Integer(b), evaluator)?)),

        (a,b)   =>   handle_div_op_default(a, b, evaluator)
 
    }
}
