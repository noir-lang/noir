use crate::{Arithmetic, Environment, Evaluator, FieldElement, Linear, Polynomial};

/// Dealing with multiplication
/// - Multiplying an arithmetic gate with anything else except a constant requires an intermediate variable
/// - We can safely multiply two linear polynomials

pub fn handle_mul_op(
    left: Polynomial,
    right: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    match (left, right) {
        (Polynomial::Null, _) => handle_null_mul(),
        (Polynomial::Arithmetic(arith), y) => handle_arithmetic_mul(arith, y, env, evaluator),
        (Polynomial::Constants(c), y) => handle_constant_mul(c, y),
        (Polynomial::Linear(lin), y) => handle_linear_mul(lin, y, env, evaluator),
    }
}

fn handle_null_mul() -> Polynomial {
    panic!("Cannot do an operation with the Null Polynomial")
}

fn handle_arithmetic_mul(
    arith: Arithmetic,
    polynomial: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    match polynomial.constant() {
        Some(constant) => return Polynomial::Arithmetic(&arith * &constant),
        None => {}
    };

    // Arriving here means that we do not have one of the operands as a constant
    // Create an intermediate variable for the arithmetic gate
    let intermediate_var = evaluator.create_intermediate_variable(env, arith);
    return handle_mul_op(intermediate_var, polynomial, env, evaluator);
}
fn handle_constant_mul(constant: FieldElement, polynomial: Polynomial) -> Polynomial {
    match polynomial {
        Polynomial::Arithmetic(arith) => Polynomial::Arithmetic(&arith * &constant),
        Polynomial::Linear(linear) => Polynomial::Linear(&linear * &constant),
        Polynomial::Constants(constant_rhs) => Polynomial::Constants(constant * constant_rhs),
        Polynomial::Null => handle_null_mul(),
    }
}
fn handle_linear_mul(
    linear: Linear,
    polynomial: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    match polynomial {
        Polynomial::Arithmetic(arith) => {
            return handle_arithmetic_mul(arith, Polynomial::Linear(linear), env, evaluator);
        }
        Polynomial::Linear(linear_rhs) => Polynomial::Arithmetic(&linear * &linear_rhs),
        Polynomial::Constants(constant) => Polynomial::Linear(&linear * &constant),
        Polynomial::Null => handle_null_mul(),
    }
}
