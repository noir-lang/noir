use super::add::handle_add_op;
use crate::{Environment, Evaluator, Object};

/// This calls the add op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_sub_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    let negated_right = match &right {
        Object::Null => panic!("Cannot do an operation with the Null Object"),
        Object::Arithmetic(arith) => Object::Arithmetic(-arith),
        Object::Constants(c) => Object::Constants(-c.clone()),
        Object::Linear(linear) => Object::Linear(-linear),
        Object::Integer(_) => {
            let left_int = left.integer();
            if left_int.is_none() {
                panic!("RHS is an integer, however the LHS is not ");
            } else {
                return Object::Integer(left_int.unwrap().sub(right, env, evaluator));
            }
        }
        x => super::unsupported_error(vec![x.clone()]),
    };

    handle_add_op(left, negated_right, env, evaluator)
}
