use super::sub::handle_sub_op;
use crate::{Environment, Evaluator, Gate, Polynomial};

/// XXX(med) : So at the moment, Equals is the same as SUB
/// Most likely we will need to check if it is a predicate equal or infix equal

/// This calls the sub op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_equal_op(
    left: Polynomial,
    right: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    let result = handle_sub_op(left, right);

    match result {
        Polynomial::Null => panic!("Constrain statement cannot output a null polynomial"),
        Polynomial::Constants(_) => panic!("Cannot constrain two constants"),
        Polynomial::Linear(linear) => evaluator.gates.push(Gate::Arithmetic(linear.into())),
        Polynomial::Arithmetic(arith) => evaluator.gates.push(Gate::Arithmetic(arith)),
    }
    Polynomial::Null
}
