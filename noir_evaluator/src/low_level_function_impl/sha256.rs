use super::GadgetCaller;
use acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acir::OPCODE;
use crate::object::{Array, Object};
use crate::{CallExpression, Environment, Evaluator};


pub struct Sha256Gadget;

impl GadgetCaller for Sha256Gadget {

    fn name() -> OPCODE {
        OPCODE::SHA256
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: CallExpression,
    ) -> Object {
        let inputs = Sha256Gadget::prepare_inputs(evaluator, env, call_expr);

        // Create two fresh variables that will link to the SHA256 output
        let (low_128_witness, low_128_poly) =
            evaluator.create_fresh_witness("low_128_sha256_".to_string(), env);
        let (high_128_witness, high_128_poly) =
            evaluator.create_fresh_witness("high_128_sha256_".to_string(), env);

        let sha256_gate = GadgetCall {
            name: Sha256Gadget::name(),
            inputs: inputs,
            outputs: vec![low_128_witness.clone(), high_128_witness.clone()],
        };

        evaluator.gates.push(Gate::GadgetCall(sha256_gate));

        let arr = Array {
            length: 2,
            contents: vec![low_128_poly, high_128_poly],
        };

        Object::Array(arr)
    }
}

impl Sha256Gadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: CallExpression,
    ) -> Vec<GadgetInput> {

        // For sha256, we expect a single input which should be an array
        assert_eq!(call_expr.arguments.len(),1);

        let arr_expr = call_expr.arguments.pop().unwrap();
        let arr = match Array::from_expression(evaluator, env, arr_expr) {
            Some(arr) => arr,
            None => panic!("Sha256 should only take a single parameter, which is an array. This should have been caught by the compiler in the analysis phase")
        };

        let mut inputs: Vec<GadgetInput> = Vec::with_capacity(0);

        for element in arr.contents.into_iter() {
            let (witness, num_bits) = match element {
                Object::Integer(integer) => (integer.witness, integer.num_bits),
                Object::Linear(lin) => {
                    if !lin.is_unit() {
                        unimplemented!(
                            "SHA256 Logic for non unit witnesses is currently not implemented"
                        )
                    }
                    (lin.witness, noir_field::FieldElement::max_num_bits())
                }
                k => unimplemented!("SHA256 logic for {:?} is not implemented yet", k),
            };

            inputs.push(GadgetInput {
                witness: witness,
                num_bits,
            });
        }

        inputs
    }
}
