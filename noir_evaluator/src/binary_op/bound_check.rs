use super::sub::handle_sub_op;
use crate::{Environment, Evaluator, FieldElement, Integer, Linear, Object};

// There are three cases:
// a < b
// case 1 : b is a constant
//   For this, we can optimise the num_bits parameter, since the upper bound is known
// case 2 : b is a Witness
// This is not allowed. We cannot guess anything about the bounds and so it is not possible to apply the constraint.
// case 3 : b is an integer
// For this we can optimise since we know that b has been range constrained

// a <= b => b - a is always positive
// a < b => b - a - 1  is always positive

fn bound_check(
    lower_bound: Object,
    upper_bound: Object,
    upper_bound_included: bool,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    let offset = if upper_bound_included {
        FieldElement::zero()
    } else {
        FieldElement::one()
    };

    let integer = match (lower_bound, upper_bound) {
        (lower_bound, Object::Integer(y)) => {
            let max_bound_bits = y.num_bits;

            let x = &Linear::from_witness(y.witness) - &offset;

            // Convert lower bound to arithmetic struct
            // This is done because, if the lower bound is a integer, 
            // the compiler will complain as we cannot subtract an integer from a linear polynomial
            let lower_bound_as_arith = lower_bound.into_arithmetic();

            let k = handle_sub_op(Object::Linear(x), Object::Arithmetic(lower_bound_as_arith), env, evaluator);

            Integer::from_object(k, max_bound_bits, env, evaluator)
        }
        (lower_bound, Object::Constants(y)) => {
            let max_bound_bits = y.num_bits();

            let k = handle_sub_op(
                Object::Constants(y - offset),
                lower_bound,
                env,
                evaluator,
            );
            Integer::from_object(k, max_bound_bits, env, evaluator)
        }
        (_, _) => {
            panic!("You can only apply the < op, if the upper bound is an Integer or an Constant")
        }
    };
    integer.constrain(evaluator);
    Object::Null
}

pub fn handle_less_than_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    bound_check(left, right, false, env, evaluator)
}
pub fn handle_less_than_equal_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    bound_check(left, right, true, env, evaluator)
}
pub fn handle_greater_than_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    bound_check(right, left, false, env, evaluator)
}
pub fn handle_greater_than_equal_op(
    left: Object,
    right: Object,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Object {
    bound_check(right, left, true, env, evaluator)
}
