use super::sub::handle_sub_op;
use crate::{interpreter::Interpreter, Gate, Linear, Object, RuntimeErrorKind};

/// XXX(med) : So at the moment, Equals is the same as SUB
/// Most likely we will need to check if it is a predicate equal or infix equal

/// This calls the sub op under the hood
/// We negate the RHS and send it to the add op
pub fn handle_equal_op(
    left: Object,
    right: Object,
    interpreter: &mut Interpreter,
) -> Result<Object, RuntimeErrorKind> {
    let result = handle_sub_op(left, right, interpreter)?;

    match result {
        Object::Null => {
            return Err(RuntimeErrorKind::UnstructuredError {
                message: "constrain statement cannot output a null polynomial".to_string(),
            })
        } // XXX; This should be BUG  severity as sub should have caught it
        Object::Constants(_) => {
            return Err(RuntimeErrorKind::UnstructuredError {
                message: "cannot constrain two constants".to_string(),
            })
        }
        Object::Linear(linear) => interpreter.push_gate(Gate::Arithmetic(linear.into())),
        Object::Arithmetic(arith) => interpreter.push_gate(Gate::Arithmetic(arith)),
        Object::Integer(integer) => {
            let truncated = integer.truncate(interpreter).unwrap();
            let witness_linear = Linear::from_witness(truncated.witness);
            interpreter.push_gate(Gate::Arithmetic(witness_linear.into()))
        }
        Object::Array(arr) => arr.constrain_zero(interpreter),
    }
    Ok(Object::Null)
}
