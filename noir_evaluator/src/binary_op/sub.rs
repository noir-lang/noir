use super::add::handle_add_op;
use crate::{Environment, Evaluator, Polynomial};

/// This calls the add op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_sub_op(
    left: Polynomial,
    right: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    let negated_right = match &right {
        Polynomial::Null => panic!("Cannot do an operation with the Null Polynomial"),
        Polynomial::Arithmetic(arith) => Polynomial::Arithmetic(-arith),
        Polynomial::Constants(c) => Polynomial::Constants(-c.clone()),
        Polynomial::Linear(linear) => Polynomial::Linear(-linear),
        Polynomial::Integer(_) => {
            let left_int = left.integer();
            if left_int.is_none() {
                panic!("RHS is an integer, however the LHS is not ");
            } else {
                return Polynomial::Integer(left_int.unwrap().sub(right, env, evaluator));
            }
        }
        x => super::unsupported_error(vec![x.clone()]),
    };

    handle_add_op(left, negated_right, env, evaluator)
}
