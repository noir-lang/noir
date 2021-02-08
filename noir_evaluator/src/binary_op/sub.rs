use super::add::handle_add_op;
use crate::{Environment, Evaluator, Object, RuntimeErrorKind};

/// This calls the add op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_sub_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    let negated_right = match &right {
        Object::Null => {
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message: format!("cannot do an operation with the null object"),
            })
        }
        Object::Arithmetic(arith) => Object::Arithmetic(-arith),
        Object::Constants(c) => Object::Constants(-c.clone()),
        Object::Linear(linear) => Object::Linear(-linear),
        Object::Integer(_) => {
            let left_int = left.integer();
            if left_int.is_none() {
                return Err(RuntimeErrorKind::UnstructuredError {
                    span: Default::default(),
                    message: format!("rhs is an integer, however the lhs is not"),
                });
            } else {
                return Ok(Object::Integer(
                    left_int.unwrap().sub(right, env, evaluator)?,
                ));
            }
        }
        x => {
            return Err(RuntimeErrorKind::UnsupportedOp {
                span: Default::default(),
                op: "sub".to_owned(),
                first_type: left.r#type().to_owned(),
                second_type: right.r#type().to_owned(),
            })
        }
    };

    handle_add_op(left, negated_right, env, evaluator)
}
