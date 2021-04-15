use noir_field::FieldElement;

use crate::{Evaluator, Object, RuntimeErrorKind};

pub fn handle_and_op<F: FieldElement>(
    left: Object<F>,
    right: Object<F>,
    evaluator: &mut Evaluator<F>,
) -> Result<Object<F>, RuntimeErrorKind> {
    match (left, right) {
        (Object::Integer(x), Object::Integer(y)) => Ok(Object::Integer(x.and(y, evaluator)?)),
        (x, y) => {
            let err=  RuntimeErrorKind::UnstructuredError{span : Default::default(), message : format!("currently we only support bitwise operations on ranged operations, found types {} and {}", x.r#type(), y.r#type())};
            Err(err)
        }
    }
}
