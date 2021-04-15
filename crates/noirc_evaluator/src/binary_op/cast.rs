use noir_field::FieldElement;

use crate::{Evaluator, Integer, Linear, Object, RuntimeErrorKind, Type};

pub fn handle_cast_op<F: FieldElement>(
    evaluator: &mut Evaluator<F>,
    left: Object<F>,
    right: Type,
) -> Result<Object<F>, RuntimeErrorKind> {
    let num_bits = match right {
        Type::Integer(_, _sign, num_bits) => num_bits,
        _ => {
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message: "currently we do not support type casting to non integers".to_string(),
            })
        }
    };

    let casted_integer = match left {
        Object::Arithmetic(arith) => {
            let casted_integer = Integer::from_arithmetic(arith, num_bits, evaluator);
            casted_integer.constrain(evaluator)?;
            casted_integer
        }
        Object::Constants(_) => {
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message: "currently we do not support casting constants to a type".to_string(),
            })
        }
        Object::Linear(linear) => {
            let casted_integer = Integer::from_arithmetic(linear.into(), num_bits, evaluator);
            casted_integer.constrain(evaluator)?;
            casted_integer
        }
        Object::Integer(integer) => {
            // If we are casting a u8 to a u32, then this would require no extra constraints
            // Since all u8s can fit into u32
            // If we are casting a u32 to a u8, then this would require constraints

            let casted_integer =
                Integer::from_arithmetic(Linear::from(integer.witness).into(), num_bits, evaluator);

            let should_constrain = integer.num_bits > num_bits;
            if should_constrain {
                casted_integer.constrain(evaluator)?;
            };
            casted_integer
        }
        x => {
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message: format!("cannot cast {} to an integer", x.r#type()),
            })
        }
    };
    Ok(Object::Integer(casted_integer))
}
