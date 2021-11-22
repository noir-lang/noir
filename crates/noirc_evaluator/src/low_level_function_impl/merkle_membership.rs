use super::GadgetCaller;
use super::RuntimeError;
use crate::object::Object;
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
use noir_field::FieldElement;
use noirc_frontend::hir_def::expr::HirCallExpression;

pub struct MerkleMembershipGadget;

impl GadgetCaller for MerkleMembershipGadget {
    fn name() -> OPCODE {
        OPCODE::MerkleMembership
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeError> {
        let inputs = MerkleMembershipGadget::prepare_inputs(evaluator, env, call_expr)?;

        // Create a fresh variable which will be the boolean indicating
        // whether the item was in the tree or not

        let merkle_mem_witness = evaluator.add_witness_to_cs();
        let merkle_mem_object = Object::from_witness(merkle_mem_witness);

        let merkle_mem_gate = GadgetCall {
            name: MerkleMembershipGadget::name(),
            inputs,
            outputs: vec![merkle_mem_witness],
        };

        evaluator.gates.push(Gate::GadgetCall(merkle_mem_gate));

        Ok(merkle_mem_object)
    }
}

impl MerkleMembershipGadget {
    pub(super) fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeError> {
        assert_eq!(call_expr.arguments.len(), 3);

        let depth = call_expr.arguments.pop().unwrap();
        let leaf = call_expr.arguments.pop().unwrap();
        let root = call_expr.arguments.pop().unwrap();

        let depth = evaluator.expression_to_object(env, &depth)?;
        let leaf = evaluator.expression_to_object(env, &leaf)?;
        let root = evaluator.expression_to_object(env, &root)?;

        // TODO: change this to convert RuntimeErrorKind into RuntimeError
        let depth = depth
            .constant()
            .expect("expected depth to be a constant")
            .to_u128();
        let leaf_witness = leaf.witness().unwrap();
        let root_witness = root.witness().unwrap();

        let mut inputs: Vec<GadgetInput> = vec![GadgetInput {
            witness: root_witness,
            num_bits: noir_field::FieldElement::max_num_bits(),
        }];

        inputs.push(GadgetInput {
            witness: leaf_witness,
            num_bits: noir_field::FieldElement::max_num_bits(),
        });
        let index_witness = evaluator.add_witness_to_cs();
        inputs.push(GadgetInput {
            witness: index_witness,
            num_bits: noir_field::FieldElement::max_num_bits(),
        });

        // Add necessary amount of witnesses for the hashpath
        let arity = 2;
        let num_hash_items = arity * depth;
        for _ in 0..num_hash_items {
            inputs.push(GadgetInput {
                witness: evaluator.add_witness_to_cs(),
                num_bits: FieldElement::max_num_bits(),
            });
        }

        Ok(inputs)
    }
}
