use crate::{Arithmetic, Environment, Evaluator, FieldElement, Linear, Object, Type};

/// Dealing with multiplication
/// - Multiplying an arithmetic gate with anything else except a constant requires an intermediate variable
/// - We can safely multiply two linear polynomials

pub fn handle_mul_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    match (left, right) {
        (Object::Null, _) => handle_null_mul(),
        (Object::Arithmetic(arith), y) => handle_arithmetic_mul(arith, y, env, evaluator),
        (Object::Constants(c), y) => handle_constant_mul(c, y),
        (Object::Linear(lin), y) => handle_linear_mul(lin, y, env, evaluator),
        (Object::Integer(integer), y) => Object::Integer(integer.mul(y, env, evaluator)),
        (x, y) => super::unsupported_error(vec![x, y]),
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
) -> Object {
    match polynomial.constant() {
        Some(constant) => return Object::Arithmetic(&arith * &constant),
        None => {}
    };

    // Arriving here means that we do not have one of the operands as a constant
    // Create an intermediate variable for the arithmetic gate
    let (intermediate_var, _) = evaluator.create_intermediate_variable(env, arith, Type::Witness);
    return handle_mul_op(intermediate_var, polynomial, env, evaluator);
}
fn handle_constant_mul(constant: FieldElement, polynomial: Object) -> Object {
    match polynomial {
        Object::Arithmetic(arith) => Object::Arithmetic(&arith * &constant),
        Object::Linear(linear) => Object::Linear(&linear * &constant),
        Object::Constants(constant_rhs) => Object::Constants(constant * constant_rhs),
        Object::Null => handle_null_mul(),
        Object::Integer(_) => panic!("Can only mul an integer to an integer"),
        x => super::unsupported_error(vec![x]),
    }
}
fn handle_linear_mul(
    linear: Linear,
    polynomial: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    match polynomial {
        Object::Arithmetic(arith) => {
            return handle_arithmetic_mul(arith, Object::Linear(linear), env, evaluator);
        }
        Object::Linear(linear_rhs) => Object::Arithmetic(&linear * &linear_rhs),
        Object::Constants(constant) => Object::Linear(&linear * &constant),
        Object::Null => handle_null_mul(),
        Object::Integer(_) => panic!("Can only mul an integer to an integer"),
        x => super::unsupported_error(vec![x]),
    }
}
