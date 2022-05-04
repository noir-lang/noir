use super::{object_to_wit_bits, GadgetCaller};
use crate::object::{Array, Integer, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::RuntimeError;

pub struct Sha256Gadget;

impl GadgetCaller for Sha256Gadget {
    fn name() -> OPCODE {
        OPCODE::SHA256
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeError> {
        let inputs = Sha256Gadget::prepare_inputs(evaluator, env, call_expr)?;

        // Create 32 fresh variables that will link to the SHA256 output

        let mut outputs = Vec::with_capacity(32);
        let mut contents = Vec::with_capacity(32);
        for _ in 0..32 {
            let witness = evaluator.add_witness_to_cs();
            let object = Object::Integer(Integer::from_witness_unconstrained(witness, 8));
            outputs.push(witness);
            contents.push(object);
        }

        let sha256_gate = GadgetCall { name: Sha256Gadget::name(), inputs, outputs };

        evaluator.gates.push(Gate::GadgetCall(sha256_gate));

        let arr = Array { length: contents.len() as u128, contents };

        Ok(Object::Array(arr))
    }
}

impl Sha256Gadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeError> {
        let arr_expr = {
            // For SHA256, we expect a single input which should be an array
            assert_eq!(call_expr.arguments.len(), 1);
            call_expr.arguments.pop().unwrap()
        };

        // "SHA256 should only take a single parameter, which is an array. This should have been caught by the compiler in the analysis phase";
        let arr = Array::from_expression(evaluator, env, &arr_expr)?;

        let mut inputs: Vec<GadgetInput> = Vec::with_capacity(arr.contents.len());

        for element in arr.contents.into_iter() {
            inputs.push(object_to_wit_bits(&element));
        }

        Ok(inputs)
    }
}
