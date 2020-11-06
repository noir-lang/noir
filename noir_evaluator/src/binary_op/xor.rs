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
        (x, y) => return Err(EvaluatorError::UnstructuredError{span : Default::default(), message : format!("bitwise operations are only available on integers, found types : {} and {}", x.r#type(), y.r#type())}),
    }
}
