use super::GadgetCaller;
use super::MerkleMembershipGadget;
use super::RuntimeError;
use crate::object::Object;
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, Gate};
use acvm::acir::OPCODE;
use noirc_frontend::hir_def::expr::HirCallExpression;

pub struct InsertRegularMerkleGadget;

impl GadgetCaller for InsertRegularMerkleGadget {
    fn name() -> OPCODE {
        OPCODE::InsertRegularMerkle
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeError> {
        // The function to prepare inputs for insertion and checking membership are the same
        let inputs = MerkleMembershipGadget::prepare_inputs(evaluator, env, call_expr)?;

        // Create a fresh variable to store the new root

        let merkle_mem_witness = evaluator.add_witness_to_cs();
        let merkle_mem_object = Object::from_witness(merkle_mem_witness);

        let merkle_mem_gate = GadgetCall {
            name: InsertRegularMerkleGadget::name(),
            inputs,
            outputs: vec![merkle_mem_witness],
        };

        evaluator.gates.push(Gate::GadgetCall(merkle_mem_gate));

        Ok(merkle_mem_object)
    }
}
