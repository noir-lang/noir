use super::add::handle_add_op;
use crate::{object::Array, Evaluator, Object, RuntimeErrorKind};

/// This calls the add op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_sub_op(
    left: Object,
    right: Object,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    let negated_right = match right {
        Object::Null => {
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message: format!("cannot do an operation with the null object"),
            })
        }
        Object::Arithmetic(arith) => Object::Arithmetic(-&arith),
        Object::Constants(c) => Object::Constants(-c.clone()),
        Object::Linear(linear) => Object::Linear(-&linear),
        Object::Integer(_rhs_integer) => {
            let left_int = left.integer();
            match left_int {
                Some(left_int) => return Ok(Object::Integer(left_int.sub(right, evaluator)?)),
                None => {
                    return Err(RuntimeErrorKind::UnstructuredError {
                        span: Default::default(),
                        message: format!("rhs is an integer, however the lhs is not"),
                    })
                }
            }
        }
        Object::Array(_right_arr) => {
            let left_arr = left.array();
            match left_arr {
                Some(left_arr) => {
                    return Ok(Object::Array(Array::sub(left_arr, _right_arr, evaluator)?))
                }
                None => {
                    return Err(RuntimeErrorKind::UnstructuredError {
                        span: Default::default(),
                        message: format!("rhs is an integer, however the lhs is not"),
                    })
                }
            }
        }
    };

    handle_add_op(left, negated_right, evaluator)
}
