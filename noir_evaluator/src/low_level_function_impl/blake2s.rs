use super::GadgetCaller;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
use noirc_frontend::hir::lower::HirCallExpression;

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

        // Create two fresh variables that will link to the Blake2s output

        let low_128_unique_name = evaluator.make_unique("low_128_Blake2s_");
        let low_128_witness = evaluator.add_witness_to_cs(); // XXX: usually the output of the function is public. To be conservative, lets make it private
        let low_128_object =
            evaluator.add_witness_to_env(low_128_unique_name, low_128_witness.clone(), env);

        let high_128_unique_name = evaluator.make_unique("high_128_Blake2s_");
        let high_128_witness = evaluator.add_witness_to_cs(); // XXX: usually the output of the function is public. To be conservative, lets make it private
        let high_128_object =
            evaluator.add_witness_to_env(high_128_unique_name, high_128_witness.clone(), env);

        let Blake2s_gate = GadgetCall {
            name: Blake2sGadget::name(),
            inputs,
            outputs: vec![low_128_witness, high_128_witness],
        };

        evaluator.gates.push(Gate::GadgetCall(Blake2s_gate));

        let arr = Array {
            length: 2,
            contents: vec![low_128_object, high_128_object],
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
            let (witness, num_bits) = match element {
                Object::Integer(integer) => (integer.witness, integer.num_bits),
                Object::Linear(lin) => {
                    if !lin.is_unit() {
                        unimplemented!(
                            "Blake2s Logic for non unit witnesses is currently not implemented"
                        )
                    }
                    (lin.witness, noir_field::FieldElement::max_num_bits())
                }
                k => unimplemented!("Blake2s logic for {:?} is not implemented yet", k),
            };

            inputs.push(GadgetInput {
                witness: witness,
                num_bits,
            });
        }

        Ok(inputs)
    }
}
