use super::GadgetCaller;
use super::RuntimeErrorKind;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
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
    ) -> Result<Object, RuntimeErrorKind> {
        let inputs = MerkleMembershipGadget::prepare_inputs(evaluator, env, call_expr)?;

        // Create a fresh variable which will be the root

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
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeErrorKind> {
        assert_eq!(call_expr.arguments.len(), 4);

        let hash_path = call_expr.arguments.pop().unwrap();
        let index = call_expr.arguments.pop().unwrap();
        let leaf = call_expr.arguments.pop().unwrap();
        let root = call_expr.arguments.pop().unwrap();

        let hash_path = Array::from_expression(evaluator, env, &hash_path)?;
        let index = evaluator.expression_to_object(env, &index)?;
        let leaf = evaluator.expression_to_object(env, &leaf)?;
        let root = evaluator.expression_to_object(env, &root)?;

        let index_witness = index.witness().unwrap();
        let leaf_witness = leaf.witness().unwrap();
        let root_witness = root.witness().unwrap();

        // XXX: Instead of panics, return a user error here
        if hash_path.contents.is_empty() {
            panic!("the hash path must contain at least two items")
        }
        // XXX: Instead of panics, return a user error here
        if hash_path.contents.len() % 2 != 0 {
            panic!("the hash path is always an even number")
        }

        let mut inputs: Vec<GadgetInput> = vec![GadgetInput {
            witness: root_witness,
            num_bits: noir_field::FieldElement::max_num_bits(),
        }];

        inputs.push(GadgetInput {
            witness: leaf_witness,
            num_bits: noir_field::FieldElement::max_num_bits(),
        });
        inputs.push(GadgetInput {
            witness: index_witness,
            num_bits: noir_field::FieldElement::max_num_bits(),
        });

        for element in hash_path.contents.into_iter() {
            let witness = match element {
                Object::Integer(integer) => (integer.witness),
                Object::Linear(lin) => {
                    if !lin.is_unit() {
                        unimplemented!(
                            "Merkle membership Logic for non unit witnesses is currently not implemented"
                        )
                    }
                    lin.witness
                }
                k => unimplemented!("Merkle membership logic for {:?} is not implemented yet", k),
            };

            inputs.push(GadgetInput {
                witness,
                num_bits: noir_field::FieldElement::max_num_bits(),
            });
        }

        Ok(inputs)
    }
}
