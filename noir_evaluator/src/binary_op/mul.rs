use crate::{Arithmetic, Environment, Evaluator, FieldElement, Linear, Object, Type, EvaluatorError};

///   Dealing with multiplication
/// - Multiplying an arithmetic gate with anything else except a constant requires an intermediate variable
/// - We can safely multiply two linear polynomials

pub fn handle_mul_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Result<Object, EvaluatorError> {
    match (left, right) {
        (Object::Null, _) => Ok(handle_null_mul()),
        (Object::Arithmetic(arith), y) => handle_arithmetic_mul(arith, y, env, evaluator),
        (Object::Constants(c), y) => handle_constant_mul(c, y),
        (Object::Linear(lin), y) => handle_linear_mul(lin, y, env, evaluator),
        (Object::Integer(integer), y) => Ok(Object::Integer(integer.mul(y, env, evaluator)?)),
        (x, y) => return Err(EvaluatorError::UnsupportedOp{span : Default::default(), op : "mul".to_owned(), first_type : x.r#type().to_owned(), second_type :y.r#type().to_owned()})
    }
}

fn handle_null_mul() -> Object {
    panic!("Cannot do an operation with the Null Object")
}

fn handle_arithmetic_mul(
    arith: Arithmetic,
    polynomial: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Result<Object, EvaluatorError> {
    if let Ok(constant) = polynomial.constant() {
         return Ok(Object::Arithmetic(&arith * &constant))
    };

    // Arriving here means that we do not have one of the operands as a constant
    // Create an intermediate variable for the arithmetic gate
    let (intermediate_var, _) = evaluator.create_intermediate_variable(env, arith, Type::Witness);
    return handle_mul_op(intermediate_var, polynomial, env, evaluator);
}
fn handle_constant_mul(constant: FieldElement, polynomial: Object) -> Result<Object, EvaluatorError> {
    let result = match polynomial {
        Object::Arithmetic(arith) => Object::Arithmetic(&arith * &constant),
        Object::Linear(linear) => Object::Linear(&linear * &constant),
        Object::Constants(constant_rhs) => Object::Constants(constant * constant_rhs),
        Object::Null => handle_null_mul(),
        Object::Integer(_) => return Err(EvaluatorError::UnstructuredError{span : Default::default(), message : format!("currently you can only multiply an integer with an integer. Will be changed later to produce an Arith/Linear")}),
        x => return Err(EvaluatorError::UnsupportedOp{span : Default::default(), op : "mul".to_owned(), first_type : x.r#type().to_owned(), second_type :"constant".to_owned()})
    };
    Ok(result)
}
fn handle_linear_mul(
    linear: Linear,
    polynomial: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Result<Object, EvaluatorError> {
    match polynomial {
        Object::Arithmetic(arith) => {
            return handle_arithmetic_mul(arith, Object::Linear(linear), env, evaluator);
        }
        Object::Linear(linear_rhs) => Ok(Object::Arithmetic(&linear * &linear_rhs)),
        Object::Constants(constant) => Ok(Object::Linear(&linear * &constant)),
        Object::Null => Ok(handle_null_mul()),
        Object::Integer(_) => return Err(EvaluatorError::UnstructuredError{span : Default::default(), message : format!("currently you can only multiply an integer with an integer. Will be changed later to produce an Arith/Linear")}),
        x => return Err(EvaluatorError::UnsupportedOp{span : Default::default(), op : "mul".to_owned(), first_type : x.r#type().to_owned(), second_type :"linear".to_owned()}),
    }
}
