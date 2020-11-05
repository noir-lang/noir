use super::sub::handle_sub_op;
use crate::{Environment, Evaluator, Gate, Linear, Object, EvaluatorError};

/// XXX(med) : So at the moment, Equals is the same as SUB
/// Most likely we will need to check if it is a predicate equal or infix equal

/// This calls the sub op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_equal_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Result<Object, EvaluatorError> {
    let result = handle_sub_op(left, right, env, evaluator)?;

    match result {
        Object::Null => panic!("Constrain statement cannot output a null polynomial"),
        Object::Constants(_) => panic!("Cannot constrain two constants"),
        Object::Linear(linear) => evaluator.gates.push(Gate::Arithmetic(linear.into())),
        Object::Arithmetic(arith) => evaluator.gates.push(Gate::Arithmetic(arith)),
        Object::Integer(integer) => {
            let witness_linear = Linear::from_witness(integer.witness);

            evaluator
                .gates
                .push(Gate::Arithmetic(witness_linear.into()))
        }
        x => {
            super::unsupported_error(vec![x]);
        }
    }
    Ok(Object::Null)
}
