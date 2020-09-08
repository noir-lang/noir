use crate::{Environment, Evaluator, Polynomial};

pub fn handle_and_op(
    left: Polynomial,
    right: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    match (left, right) {
        (Polynomial::Integer(x), Polynomial::Integer(y)) => {
            Polynomial::Integer(x.and(y, env, evaluator))
        }
        (_, _) => panic!("Currently we only support bitwise operations on ranged operations"),
    }
}
