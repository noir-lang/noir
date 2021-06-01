use super::GadgetCaller;
use crate::low_level_function_impl::object_to_wit_bits;
use crate::object::{Array, Integer, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::RuntimeErrorKind;

// XXX(TD): Reconcile all byte based hash functions in the C++ code, in the WASM binding and also in the compiler

pub struct Blake2sGadget;

impl GadgetCaller for Blake2sGadget {
    fn name() -> OPCODE {
        OPCODE::Blake2s
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeErrorKind> {
        let inputs = Blake2sGadget::prepare_inputs(evaluator, env, call_expr)?;

        // Create 32 fresh variables that will link to the Blake2s output
        let mut outputs = Vec::with_capacity(32);
        let mut contents = Vec::with_capacity(32);
        for _ in 0..32 {
            let witness = evaluator.add_witness_to_cs();
            let object = Object::Integer(Integer::from_witness(witness, 8));
            outputs.push(witness);
            contents.push(object);
        }

        let blake2s_gate = GadgetCall {
            name: Blake2sGadget::name(),
            inputs,
            outputs,
        };

        evaluator.gates.push(Gate::GadgetCall(blake2s_gate));

        let arr = Array {
            length: 2,
            contents,
        };

        Ok(Object::Array(arr))
    }
}

impl Blake2sGadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeErrorKind> {
        let arr_expr = {
            // For Blake2s, we expect a single input which should be an array
            assert_eq!(call_expr.arguments.len(), 1);
            call_expr.arguments.pop().unwrap()
        };

        // "Blake2s should only take a single parameter, which is an array. This should have been caught by the compiler in the analysis phase";
        let arr = Array::from_expression(evaluator, env, &arr_expr)?;

        let mut inputs: Vec<GadgetInput> = Vec::with_capacity(arr.contents.len());

        for element in arr.contents.into_iter() {
            inputs.push(object_to_wit_bits(&element));
        }

        Ok(inputs)
    }
}
