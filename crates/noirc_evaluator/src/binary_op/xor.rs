use crate::{Evaluator, Object, RuntimeErrorKind};

pub fn handle_xor_op(
    left: Object,
    right: Object,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    match (left, right) {
        (Object::Integer(x), Object::Integer(y)) => Ok(Object::Integer(x.xor(y, evaluator)?)),
        (x, y) => Err(RuntimeErrorKind::UnstructuredError {
            message: format!(
                "bitwise operations are only available on integers, found types : {} and {}",
                x.r#type(),
                y.r#type()
            ),
        }),
    }
}
