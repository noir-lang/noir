use super::{object_to_wit_bits, GadgetCaller};
use crate::object::{Array, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OpCode;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::RuntimeError;

pub struct HashToFieldGadget;

impl GadgetCaller for HashToFieldGadget {
    fn name() -> OpCode {
        OpCode::HashToField
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeError> {
        let inputs = HashToFieldGadget::prepare_inputs(evaluator, env, call_expr)?;

        let res_witness = evaluator.add_witness_to_cs();
        let res_object = Object::from_witness(res_witness);

        let hash_to_field_gate = GadgetCall {
            name: HashToFieldGadget::name(),
            inputs,
            outputs: vec![res_witness],
        };

        evaluator.gates.push(Gate::GadgetCall(hash_to_field_gate));

        Ok(res_object)
    }
}

impl HashToFieldGadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeError> {
        let arr_expr = {
            // For HashToField, we expect a single input which should be an array
            assert_eq!(call_expr.arguments.len(), 1);
            call_expr.arguments.pop().unwrap()
        };

        // "HashToField should only take a single parameter, which is an array. This should have been caught by the compiler in the analysis phase";
        let arr = Array::from_expression(evaluator, env, &arr_expr)?;

        let mut inputs: Vec<GadgetInput> = Vec::with_capacity(arr.contents.len());

        for element in arr.contents.into_iter() {
            inputs.push(object_to_wit_bits(&element));
        }

        Ok(inputs)
    }
}
