/// This module handles all of the binary operations between polynomials
pub mod add;
pub mod and;
pub mod bound_check;
pub mod cast;
pub mod div;
pub mod equal;
pub mod mul;
pub mod neq;
pub mod sub;
pub mod xor;

use std::ops::Neg;

use acvm::acir::{
    circuit::{gate::Directive, Gate},
    native_types::{Arithmetic, Linear},
};
pub use add::handle_add_op;
pub use and::handle_and_op;
pub use bound_check::handle_greater_than_equal_op;
pub use bound_check::handle_greater_than_op;
pub use bound_check::handle_less_than_equal_op;
pub use bound_check::handle_less_than_op;
pub use cast::handle_cast_op;
pub use div::handle_div_op;
pub use equal::handle_equal_op;
pub use mul::handle_mul_op;
pub use neq::handle_neq_op;
pub use sub::handle_sub_op;
pub use xor::handle_xor_op;

use crate::{object::Integer, Evaluator, FieldElement, Object, RuntimeErrorKind};

/// Creates a new witness and constrains it to be the inverse of the polynomial passed in
pub fn invert(x: Object, evaluator: &mut Evaluator) -> Result<Object, RuntimeErrorKind> {
    // Create a fresh witness

    let inverse_witness = evaluator.add_witness_to_cs();
    let inverse_obj = Object::from_witness(inverse_witness);

    // Multiply inverse by original value
    let should_be_one = handle_mul_op(x, inverse_obj.clone(), evaluator)?;

    // Constrain x * x_inv = 1
    let _ = handle_equal_op(
        should_be_one,
        Object::Constants(FieldElement::one()),
        evaluator,
    );

    // Return inverse
    Ok(inverse_obj)
}

/// Returns 1 if a == b else 0
pub fn maybe_equal(
    a: Object,
    b: Object,
    evaluator: &mut Evaluator,
) -> Result<Integer, RuntimeErrorKind> {
    const ICE_STR: &str = "ice: this method should only be called for arithmetic gates";
    let a_arith = a.into_arithmetic().expect(ICE_STR);
    let b_arith = b.into_arithmetic().expect(ICE_STR);

    // u = a - b
    let a_minus_b = &a_arith - &b_arith;
    let (_, u_wit) = evaluator.create_intermediate_variable(a_minus_b);

    // z = 1/u => uz = 1
    let z = evaluator.add_witness_to_cs();
    evaluator.gates.push(Gate::Directive(Directive::Invert {
        x: u_wit,
        result: z,
    }));

    // y = 1 -uz
    let uz: Arithmetic = (Linear::from_witness(u_wit) * Linear::from_witness(z)).into();
    let gate = uz.neg() + &FieldElement::one();
    let (_, y) = evaluator.create_intermediate_variable(gate);

    // yu = 0
    let gate: Arithmetic = (Linear::from_witness(u_wit) * Linear::from_witness(y)).into();
    evaluator.gates.push(Gate::Arithmetic(gate));

    // We know that y is a boolean
    let bool_y = Integer::from_witness(y, 1);

    Ok(bool_y)
}
