use crate::{Environment, Evaluator, Object, EvaluatorError};

pub fn handle_xor_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Result<Object, EvaluatorError> {
    match (left, right) {
        (Object::Integer(x), Object::Integer(y)) => {
            Ok(Object::Integer(x.xor(y, env, evaluator)?))
        }
        (_, _) => panic!("Currently we only support bitwise operations on ranged operations"),
    }
}
