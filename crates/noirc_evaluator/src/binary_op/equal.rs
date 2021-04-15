use noir_field::FieldElement;

use super::sub::handle_sub_op;
use crate::{Evaluator, Gate, Linear, Object, RuntimeErrorKind};

/// XXX(med) : So at the moment, Equals is the same as SUB
/// Most likely we will need to check if it is a predicate equal or infix equal

/// This calls the sub op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_equal_op<F: FieldElement>(
    left: Object<F>,
    right: Object<F>,
    evaluator: &mut Evaluator<F>,
) -> Result<Object<F>, RuntimeErrorKind> {
    let result = handle_sub_op(left, right, evaluator)?;

    match result {
        Object::Null => {
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message: "constrain statement cannot output a null polynomial".to_string(),
            })
        } // XXX; This should be BUG  severity as sub should have caught it
        Object::Constants(_) => {
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message: "cannot constrain two constants".to_string(),
            })
        }
        Object::Linear(linear) => evaluator.gates.push(Gate::Arithmetic(linear.into())),
        Object::Arithmetic(arith) => evaluator.gates.push(Gate::Arithmetic(arith)),
        Object::Integer(integer) => {
            let witness_linear = Linear::from_witness(integer.witness);

            evaluator
                .gates
                .push(Gate::Arithmetic(witness_linear.into()))
        }
        Object::Array(arr) => arr.constrain_zero(evaluator),
    }
    Ok(Object::Null)
}
