use super::sub::handle_sub_op;
use crate::{Environment, Evaluator, FieldElement, Integer, Linear, Polynomial};

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
    lower_bound: Polynomial,
    upper_bound: Polynomial,
    upper_bound_included: bool,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    let offset = if upper_bound_included {
        FieldElement::zero()
    } else {
        FieldElement::one()
    };

    let integer = match (lower_bound, upper_bound) {
        (lower_bound, Polynomial::Integer(y)) => {
            let max_bound_bits = y.num_bits;

            let x = &Linear::from_witness(y.witness) - &offset;

            let k = handle_sub_op(Polynomial::Linear(x), lower_bound, env, evaluator);
            Integer::from_polynomial(k, max_bound_bits, env, evaluator)
        }
        (lower_bound, Polynomial::Constants(y)) => {
            let max_bound_bits = y.num_bits();

            let k = handle_sub_op(
                Polynomial::Constants(y - offset),
                lower_bound,
                env,
                evaluator,
            );
            Integer::from_polynomial(k, max_bound_bits, env, evaluator)
        }
        (_, _) => {
            panic!("You can only apply the < op, if the upper bound is an integer or an Constant")
        }
    };
    integer.constrain(evaluator);
    Polynomial::Null
}

pub fn handle_less_than_op(
    left: Polynomial,
    right: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    bound_check(left, right, false, env, evaluator)
}
pub fn handle_less_than_equal_op(
    left: Polynomial,
    right: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    bound_check(left, right, true, env, evaluator)
}
pub fn handle_greater_than_op(
    left: Polynomial,
    right: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    bound_check(right, left, false, env, evaluator)
}
pub fn handle_greater_than_equal_op(
    left: Polynomial,
    right: Polynomial,
    env: &mut Environment,
    evaluator: &mut Evaluator,
) -> Polynomial {
    bound_check(right, left, true, env, evaluator)
}
