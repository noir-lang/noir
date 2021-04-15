// Functions that are in the low level standard library
// Low level std library methods are gadgets which are assumed to be present in the underlying proof system
// This means that the underlying PLONK library must have some way to deal with these methods.
// The standard library on the other hand, is a mixture of foreign and compiled functions.
use crate::{Environment, Evaluator, Object};
mod blake2s;
mod ecdsa_secp256k1;
mod fixed_based_scalar_mul;
mod hash_to_field;
mod merkle_membership;
mod pedersen;
mod schnorr;
mod sha256;

use blake2s::Blake2sGadget;
use ecdsa_secp256k1::EcdsaSecp256k1Gadget;
use fixed_based_scalar_mul::FixedBaseScalarMulGadget;
use hash_to_field::HashToFieldGadget;
use merkle_membership::MerkleMembershipGadget;
use noir_field::FieldElement;
use pedersen::PedersenGadget;
use schnorr::SchnorrVerifyGadget;
use sha256::Sha256Gadget;

use noirc_frontend::hir_def::expr::HirCallExpression;

use super::RuntimeErrorKind;
use acvm::acir::OPCODE;

pub trait GadgetCaller<F: FieldElement> {
    fn name() -> acvm::acir::OPCODE;
    fn call(
        evaluator: &mut Evaluator<F>,
        env: &mut Environment<F>,
        call_expr: HirCallExpression,
    ) -> Result<Object<F>, RuntimeErrorKind>;
}

pub fn call_low_level<F: FieldElement>(
    evaluator: &mut Evaluator<F>,
    env: &mut Environment<F>,
    opcode_name: &str,
    call_expr: HirCallExpression,
) -> Result<Object<F>, RuntimeErrorKind> {
    let func = match OPCODE::lookup(opcode_name) {
        None => {
            let message = format!(
                "cannot find a low level opcode with the name {} in the IR",
                opcode_name
            );
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message,
            });
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
        k => {
            let message = format!("The OPCODE {} exists, however, currently the compiler does not have a concrete implementation for it", k);
            Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message,
            })
        }
    }
}
