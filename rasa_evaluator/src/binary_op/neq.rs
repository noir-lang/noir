use super::{sub::handle_sub_op, mul::handle_mul_op, equal::handle_equal_op};
use crate::{Environment, Evaluator, FieldElement, Polynomial};


/// This calls the sub op under the hood
/// Then asserts that the result has an inverse
/// ie a != b => a-b has an inverse
pub fn handle_neq_op(left: Polynomial, right: Polynomial,    env: &mut Environment,
    evaluator: &mut Evaluator,) -> Polynomial {

    let result = handle_sub_op(left, right);

    // XXX: We need to create a better function for fresh variables
    let inter_var_name = format!(
        "{}{}",
        "inverse_",
        evaluator.get_unique_value(),
    );
    evaluator.store_witness(inter_var_name.clone());
    let inverse = evaluator.store_lone_variable(inter_var_name, env);

    let inv_mul_res = handle_mul_op(result, inverse, env, evaluator);

    handle_equal_op(inv_mul_res, Polynomial::Constants(FieldElement::one()))
}
