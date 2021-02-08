// Functions that are in the low level standard library
// Low level std library methods are gadgets which are assumed to be present in the underlying proof system
// This means that the underlying PLONK library must have some way to deal with these methods.
// The standard library on the other hand, is a mixture of foreign and compiled functions.
use crate::{Environment, Evaluator, Object};
mod blake2s;
mod merkle_membership;
mod merkle_root;
mod pedersen;
mod schnorr;
mod sha256;

use blake2s::Blake2sGadget;
use merkle_membership::MerkleMembershipGadget;
use merkle_root::MerkleRootGadget;
use pedersen::PedersenGadget;
use schnorr::SchnorrVerifyGadget;
use sha256::Sha256Gadget;

use noirc_frontend::hir::lower::HirCallExpression;

use super::RuntimeErrorKind;
use acir::OPCODE;

pub trait GadgetCaller {
    fn name() -> acir::OPCODE;
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeErrorKind>;
}

pub fn call_low_level(
    evaluator: &mut Evaluator,
    env: &mut Environment,
    opcode_name: &str,
    call_expr: HirCallExpression,
) -> Result<Object, RuntimeErrorKind> {
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
        OPCODE::MerkleRoot => MerkleRootGadget::call(evaluator, env, call_expr),
        OPCODE::MerkleMembership => MerkleMembershipGadget::call(evaluator, env, call_expr),
        OPCODE::SchnorrVerify => SchnorrVerifyGadget::call(evaluator, env, call_expr),
        OPCODE::Blake2s => Blake2sGadget::call(evaluator, env, call_expr),
        OPCODE::Pedersen => PedersenGadget::call(evaluator, env, call_expr),
        k => {
            let message = format!("The OPCODE {} exists, however, currently the compiler does not have a concrete implementation for it", k);
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message,
            });
        }
    }
}
