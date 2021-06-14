use crate::{Arithmetic, Array, Evaluator, Linear, Object, RuntimeErrorKind};

///   Dealing with multiplication
/// - Multiplying an arithmetic gate with anything else except a constant requires an intermediate variable
/// - We can safely multiply two linear polynomials

pub fn handle_mul_op(
    left: Object,
    right: Object,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    let general_err = err_cannot_mul(left.r#type(), right.r#type());

    match (left, right) {
        (Object::Null, _) | (_, Object::Null) => Err(general_err),

        (Object::Array(_), Object::Array(_)) => Err(general_err),

        (Object::Arithmetic(x), y) | (y, Object::Arithmetic(x)) => {
            handle_arithmetic_mul(x, y, evaluator)
        }

        (Object::Constants(x), y) | (y, Object::Constants(x)) => {
            y.mul_constant(x).ok_or(general_err)
        }

        (Object::Linear(lin), y) | (y, Object::Linear(lin)) => handle_linear_mul(lin, y, evaluator),

        (Object::Integer(integer), y) | (y, Object::Integer(integer)) => {
            Ok(Object::Integer(integer.mul(y, evaluator)?))
        }
    }
}

fn err_cannot_mul(first_type: &'static str, second_type: &'static str) -> RuntimeErrorKind {
    RuntimeErrorKind::UnsupportedOp {
        op: "mul".to_owned(),
        first_type: first_type.to_owned(),
        second_type: second_type.to_owned(),
    }
}

fn handle_arithmetic_mul(
    arith: Arithmetic,
    polynomial: Object,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    if let Ok(constant) = polynomial.constant() {
        return Ok(Object::Arithmetic(&arith * &constant));
    };
    // Arriving here means that we do not have one of the operands as a constant
    // Create an intermediate variable for the arithmetic gate
    let (intermediate_var, _) = evaluator.create_intermediate_variable(arith);
    handle_mul_op(intermediate_var, polynomial, evaluator)
}

fn handle_linear_mul(
    linear: Linear,
    polynomial: Object,
    evaluator: &mut Evaluator,
) -> Result<Object, RuntimeErrorKind> {
    match polynomial {
        Object::Arithmetic(arith) => {
            handle_arithmetic_mul(arith, Object::Linear(linear), evaluator)
        }
        Object::Linear(linear_rhs) => Ok(Object::Arithmetic(linear * linear_rhs)),
        Object::Constants(constant) => Ok(Object::Linear(&linear * &constant)),
        Object::Integer(integer) => {
            let result = Linear::from_witness(integer.witness) * linear;
            Ok(Object::Arithmetic(result))
        }
        Object::Array(arr) => {
            let mut result = Vec::with_capacity(arr.length as usize);
            for element in arr.contents.into_iter() {
                result.push(handle_linear_mul(linear, element, evaluator)?);
            }

            Ok(Object::Array(Array {
                contents: result,
                length: arr.length,
            }))
        }
        Object::Null => Err(err_cannot_mul("()", "Witness")),
    }
}
