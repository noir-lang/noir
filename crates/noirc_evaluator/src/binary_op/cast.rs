use crate::{Evaluator, Integer, Linear, Object, RuntimeErrorKind, Type};

pub fn handle_cast_op(
    evaluator: &mut Evaluator,
    left: Object,
    right: Type,
) -> Result<Object, RuntimeErrorKind> {
    let num_bits = match right {
        Type::Integer(_sign, num_bits) => num_bits,
        Type::FieldElement => {
            match left.to_arithmetic() {
                Some(arith) => {
                    // XXX: Create an intermediate variable for the arithmetic gate
                    // as we don't have full support for Arithmetic constraints in
                    // method object_to_wit_bits
                    let (object, _) = evaluator.create_intermediate_variable(arith);
                    return Ok(object);
                }
                None => {
                    return Err(RuntimeErrorKind::UnstructuredError {
                        message: "type cannot be casted to a Field element".to_string(),
                    })
                }
            };
        }
        x => {
            return Err(RuntimeErrorKind::UnstructuredError {
                message: format!("currently we do not support type casting to {}", x),
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
                message: format!("cannot cast {} to an integer", x.r#type()),
            })
        }
    };
    Ok(Object::Integer(casted_integer))
}
