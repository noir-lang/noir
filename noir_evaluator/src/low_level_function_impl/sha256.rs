use super::GadgetCaller;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator, Type};
use acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acir::OPCODE;
use noirc_frontend::hir::lower::HirCallExpression;

use super::RuntimeErrorKind;

pub struct Sha256Gadget;

impl GadgetCaller for Sha256Gadget {
    fn name() -> OPCODE {
        OPCODE::SHA256
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeErrorKind> {
        let inputs = Sha256Gadget::prepare_inputs(evaluator, env, call_expr)?;

        // Create two fresh variables that will link to the SHA256 output

        let low_128_unique_name = evaluator.make_unique("low_128_sha256_");
        let low_128_witness = evaluator.add_witness_to_cs(); // XXX: usually the output of the function is public. To be conservative, lets make it private
        let low_128_object =
            evaluator.add_witness_to_env(low_128_unique_name, low_128_witness.clone(), env);

        let high_128_unique_name = evaluator.make_unique("high_128_sha256_");
        let high_128_witness = evaluator.add_witness_to_cs(); // XXX: usually the output of the function is public. To be conservative, lets make it private
        let high_128_object =
            evaluator.add_witness_to_env(high_128_unique_name, high_128_witness.clone(), env);

        let sha256_gate = GadgetCall {
            name: Sha256Gadget::name(),
            inputs,
            outputs: vec![low_128_witness, high_128_witness],
        };

        evaluator.gates.push(Gate::GadgetCall(sha256_gate));

        let arr = Array {
            length: 2,
            contents: vec![low_128_object, high_128_object],
        };

        Ok(Object::Array(arr))
    }
}

impl Sha256Gadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeErrorKind> {
        let arr_expr = {
            // For sha256, we expect a single input which should be an array
            assert_eq!(call_expr.arguments.len(), 1);
            call_expr.arguments.pop().unwrap()
        };

        // "Sha256 should only take a single parameter, which is an array. This should have been caught by the compiler in the analysis phase";
        let arr = Array::from_expression(evaluator, env, &arr_expr)?;

        let mut inputs: Vec<GadgetInput> = Vec::with_capacity(arr.contents.len());

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

        Ok(inputs)
    }
}
