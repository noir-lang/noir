/// This module handles all of the binary operations between polynomials
pub mod add;
pub mod equal;
pub mod mul;
pub mod sub;
pub mod neq;
pub mod div;

pub use add::handle_add_op;
pub use equal::handle_equal_op;
pub use mul::handle_mul_op;
pub use sub::handle_sub_op;
pub use neq::handle_neq_op;
pub use div::handle_div_op;


use crate::{Environment, Evaluator, FieldElement, Polynomial, Gate};

/// Creates a new witness and constrains it to be the inverse of the polynomial passed in
pub fn invert(x: Polynomial,  env: &mut Environment,
    evaluator: &mut Evaluator) -> Polynomial {
        // Create a fresh witness
        // XXX: We need to create a better function for fresh variables
        let inter_var_name = format!(
            "{}{}",
            "inverse_",
            evaluator.get_unique_value(),
        );
        evaluator.store_witness(inter_var_name.clone());
        let x_inv = evaluator.store_lone_variable(inter_var_name, env);
    
        // Multiply inverse by original value
        let should_be_one = handle_mul_op(x, x_inv.clone(), env, evaluator);
    
        // Constrain x * x_inv = 1
        let _ = handle_equal_op(should_be_one, Polynomial::Constants(FieldElement::one()), env, evaluator);

        // Return inverse
        x_inv
    }