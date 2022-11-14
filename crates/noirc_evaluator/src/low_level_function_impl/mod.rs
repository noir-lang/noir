use crate::errors::RuntimeError;
use crate::interpreter::Interpreter;
// Functions that are in the low level standard library
// Low level std library methods are gadgets which are assumed to be present in the underlying proof system
// This means that the underlying PLONK library must have some way to deal with these methods.
// The standard library on the other hand, is a mixture of foreign and compiled functions.
use crate::{Environment, Object};
mod blake2s;
mod ecdsa_secp256k1;
mod fixed_based_scalar_mul;
mod hash_to_field;
mod merkle_membership;
mod pedersen;
mod schnorr;
mod sha256;

use acvm::acir::circuit::gate::GadgetInput;
use acvm::FieldElement;
use noirc_frontend::hir_def::expr::HirCallExpression;

pub trait GadgetCaller {
    fn name() -> acvm::acir::OPCODE;
    fn call(
        evaluator: &mut Interpreter,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeError>;
}
pub(crate) fn object_to_wit_bits(obj: &Object) -> GadgetInput {
    let (witness, num_bits) = match obj {
        Object::Integer(integer) => (integer.witness, integer.num_bits),
        Object::Linear(lin) => {
            if !lin.is_unit() {
                unimplemented!("Logic for non unit witnesses is currently not implemented")
            }
            (lin.witness, FieldElement::max_num_bits())
        }
        k => unimplemented!("logic for {:?} is not implemented yet", k),
    };

    GadgetInput { witness, num_bits }
}
