use super::GadgetCaller;
use acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acir::OPCODE;
use noirc_frontend::hir::lower::HirCallExpression;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator, Type};

use super::RuntimeErrorKind;

pub struct PedersenGadget;

impl GadgetCaller for PedersenGadget {

    fn name() -> OPCODE {
        OPCODE::Pedersen
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeErrorKind> {
        let inputs = PedersenGadget::prepare_inputs(evaluator, env, call_expr)?;

        // Prepare output
        let pedersen_unique_name = evaluator.make_unique("pedersen_");
        let pedersen_witness = evaluator.add_witness_to_cs(pedersen_unique_name, Type::Witness); // XXX: usually the output of the function is public. To be conservative, lets make it private
        let pedersen_object = evaluator.add_witness_to_env(pedersen_witness.clone(), env);

        let pedersen_gate = GadgetCall {
            name: PedersenGadget::name(),
            inputs: inputs,
            outputs: vec![pedersen_witness],
        };

        evaluator.gates.push(Gate::GadgetCall(pedersen_gate));

        Ok(pedersen_object)
    }
}

impl PedersenGadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeErrorKind> {

        let arr_expr = {
            // For pedersen gadget, we expect a single input which should be an array
            assert_eq!(call_expr.arguments.len(),1);
            call_expr.arguments.pop().unwrap()
        };

        let arr = Array::from_expression(evaluator, env, &arr_expr)?;
        
        // XXX: Instead of panics, return a user error here
        if arr.contents.len() < 1 {
            panic!("a pedersen hash requires at least one element to hash")
        }

        let mut inputs: Vec<GadgetInput> = Vec::new();

        for element in arr.contents.into_iter() {
            let witness = match element {
                Object::Integer(integer) => (integer.witness),
                Object::Linear(lin) => {
                    if !lin.is_unit() {
                        unimplemented!(
                            "Pedersen logic for non unit witnesses is currently not implemented"
                        )
                    }
                    lin.witness
                }
                k => unimplemented!("Pedersen logic for {:?} is not implemented yet", k),
            };

            inputs.push(GadgetInput {
                witness: witness,
                num_bits : noir_field::FieldElement::max_num_bits(),
            });
        }

        Ok(inputs)
    }
}
