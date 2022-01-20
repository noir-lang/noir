use crate::errors::RuntimeError;
// Functions that are in the low level standard library
// Low level std library methods are gadgets which are assumed to be present in the underlying proof system
// This means that the underlying PLONK library must have some way to deal with these methods.
// The standard library on the other hand, is a mixture of foreign and compiled functions.
use crate::{Environment, Evaluator, Object};
mod blake2s;
mod ecdsa_secp256k1;
mod fixed_based_scalar_mul;
mod hash_to_field;
mod insert_regular_merkle;
mod merkle_membership;
mod pedersen;
mod schnorr;
mod sha256;

use super::RuntimeErrorKind;
use acvm::acir::circuit::gate::GadgetInput;
use acvm::acir::OPCODE;
use acvm::FieldElement;
use blake2s::Blake2sGadget;
use ecdsa_secp256k1::EcdsaSecp256k1Gadget;
use fixed_based_scalar_mul::FixedBaseScalarMulGadget;
use hash_to_field::HashToFieldGadget;
use insert_regular_merkle::InsertRegularMerkleGadget;
use merkle_membership::MerkleMembershipGadget;
use noirc_errors::Span;
use noirc_frontend::hir_def::expr::HirCallExpression;
use pedersen::PedersenGadget;
use schnorr::SchnorrVerifyGadget;
use sha256::Sha256Gadget;

pub trait GadgetCaller {
    fn name() -> acvm::acir::OPCODE;
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeError>;
}

pub fn call_low_level(
    evaluator: &mut Evaluator,
    env: &mut Environment,
    opcode_name: &str,
    call_expr_span: (HirCallExpression, Span),
) -> Result<Object, RuntimeError> {
    let (call_expr, span) = call_expr_span;
    let func = match OPCODE::lookup(opcode_name) {
        None => {
            let message = format!(
                "cannot find a low level opcode with the name {} in the IR",
                opcode_name
            );

            return Err(RuntimeErrorKind::UnstructuredError { message }.add_span(span));
        }

        Some(func) => func,
    };

    match func {
        OPCODE::SHA256 => Sha256Gadget::call(evaluator, env, call_expr),
        OPCODE::MerkleMembership => MerkleMembershipGadget::call(evaluator, env, call_expr),
        OPCODE::SchnorrVerify => SchnorrVerifyGadget::call(evaluator, env, call_expr),
        OPCODE::Blake2s => Blake2sGadget::call(evaluator, env, call_expr),
        OPCODE::Pedersen => PedersenGadget::call(evaluator, env, call_expr),
        OPCODE::EcdsaSecp256k1 => EcdsaSecp256k1Gadget::call(evaluator, env, call_expr),
        OPCODE::HashToField => HashToFieldGadget::call(evaluator, env, call_expr),
        OPCODE::FixedBaseScalarMul => FixedBaseScalarMulGadget::call(evaluator, env, call_expr),
        OPCODE::InsertRegularMerkle => InsertRegularMerkleGadget::call(evaluator, env, call_expr),
        k => {
            let message = format!("The OPCODE {} exists, however, currently the compiler does not have a concrete implementation for it", k);
            Err(RuntimeErrorKind::UnstructuredError { message }.add_span(span))
        }
    }
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
