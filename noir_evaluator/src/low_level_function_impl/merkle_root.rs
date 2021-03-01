use super::GadgetCaller;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
use noirc_frontend::hir::lower::expr::HirCallExpression;

use super::RuntimeErrorKind;

pub struct MerkleRootGadget;

impl GadgetCaller for MerkleRootGadget {
    fn name() -> OPCODE {
        OPCODE::MerkleRoot
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeErrorKind> {
        let inputs = MerkleRootGadget::prepare_inputs(evaluator, env, call_expr)?;

        // Create a fresh variable which will be the root

        let merkle_root_witness = evaluator.add_witness_to_cs();
        let merkle_root_object = Object::from_witness(merkle_root_witness);

        let merkle_root_gate = GadgetCall {
            name: MerkleRootGadget::name(),
            inputs,
            outputs: vec![merkle_root_witness],
        };

        evaluator.gates.push(Gate::GadgetCall(merkle_root_gate));

        Ok(merkle_root_object)
    }
}

impl MerkleRootGadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeErrorKind> {
        let arr_expr = {
            // For merkle root gadget, we expect a single input which should be an array
            assert_eq!(call_expr.arguments.len(), 1);
            call_expr.arguments.pop().unwrap()
        };

        let arr = Array::from_expression(evaluator, env, &arr_expr)?;

        // XXX: Instead of panics, return a user error here
        if arr.contents.len() < 2 {
            panic!("computing the merkle root requires more than one element")
        }
        // XXX: Instead of panics, return a user error here
        if arr.contents.len() % 2 != 0 {
            panic!("computing the merkle root requires you to have an even number of leaves")
        }

        let mut inputs: Vec<GadgetInput> = Vec::new();

        for element in arr.contents.into_iter() {
            let witness = match element {
                Object::Integer(integer) => (integer.witness),
                Object::Linear(lin) => {
                    if !lin.is_unit() {
                        unimplemented!(
                            "Merkle root Logic for non unit witnesses is currently not implemented"
                        )
                    }
                    lin.witness
                }
                k => unimplemented!("Merkle root logic for {:?} is not implemented yet", k),
            };

            inputs.push(GadgetInput {
                witness: witness,
                num_bits: noir_field::FieldElement::max_num_bits(),
            });
        }

        Ok(inputs)
    }
}
