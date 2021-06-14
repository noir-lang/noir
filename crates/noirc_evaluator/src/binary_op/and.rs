use crate::{Evaluator, Object, RuntimeErrorKind};

pub fn handle_and_op(
    left: Object,
    right: Object,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    match (left, right) {
        (Object::Integer(x), Object::Integer(y)) => Ok(Object::Integer(x.and(y, evaluator)?)),
        (x, y) => {
            let err=  RuntimeErrorKind::UnstructuredError{ message : format!("currently we only support bitwise operations on ranged operations, found types {} and {}", x.r#type(), y.r#type())};
            Err(err)
        }
    }
}
