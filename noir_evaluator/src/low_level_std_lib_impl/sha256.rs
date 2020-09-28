use super::{GadgetCaller, LowLevelStandardLibrary, HashLibrary};
use crate::{
    circuit::gate::{GadgetCall, GadgetInput},
    Array, CallExpression, Environment, Evaluator, Gate, Polynomial
};

pub struct Sha256Gadget {}

impl GadgetCaller for Sha256Gadget {

    fn name() -> LowLevelStandardLibrary {
        LowLevelStandardLibrary::Hash(HashLibrary::SHA256)
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: CallExpression,
    ) -> Polynomial {
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

        Polynomial::Array(arr)
    }
}

impl Sha256Gadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: CallExpression,
    ) -> Vec<GadgetInput> {
        let mut inputs: Vec<GadgetInput> = Vec::with_capacity(call_expr.arguments.len());

        for arg_expr in call_expr.arguments.into_iter() {
            let arg_poly = evaluator.expression_to_polynomial(env, arg_expr);
            let (witness, num_bits) = match arg_poly {
                Polynomial::Integer(integer) => (integer.witness, integer.num_bits),
                Polynomial::Linear(lin) => {
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
