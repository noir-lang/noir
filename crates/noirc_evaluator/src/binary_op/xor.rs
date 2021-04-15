use noir_field::FieldElement;

use crate::{Evaluator, Object, RuntimeErrorKind};

pub fn handle_xor_op<F: FieldElement>(
    left: Object<F>,
    right: Object<F>,
    evaluator: &mut Evaluator<F>,
) -> Result<Object<F>, RuntimeErrorKind> {
    match (left, right) {
        (Object::Integer(x), Object::Integer(y)) => Ok(Object::Integer(x.xor(y, evaluator)?)),
        (x, y) => {
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message: format!(
                    "bitwise operations are only available on integers, found types : {} and {}",
                    x.r#type(),
                    y.r#type()
                ),
            })
        }
    }
}
