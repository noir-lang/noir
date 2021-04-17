use super::GadgetCaller;
use crate::object::{Array, Integer, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
use noir_field::FieldElement;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::RuntimeErrorKind;

pub struct Sha256Gadget;

impl<F: FieldElement> GadgetCaller<F> for Sha256Gadget {
    fn name() -> OPCODE {
        OPCODE::SHA256
    }

    fn call(
        evaluator: &mut Evaluator<F>,
        env: &mut Environment<F>,
        call_expr: HirCallExpression,
    ) -> Result<Object<F>, RuntimeErrorKind> {
        let inputs = Sha256Gadget::prepare_inputs(evaluator, env, call_expr)?;

        // Create 32 fresh variables that will link to the SHA256 output

        let mut outputs = Vec::with_capacity(32);
        let mut contents = Vec::with_capacity(32);
        for _ in 0..32 {
            let witness = evaluator.add_witness_to_cs();
            let object = Object::Integer(Integer::from_witness(witness, 8));
            outputs.push(witness);
            contents.push(object);
        }

        let sha256_gate = GadgetCall {
            name: OPCODE::SHA256,
            inputs,
            outputs,
        };

        evaluator.gates.push(Gate::GadgetCall(sha256_gate));

        let arr = Array {
            length: 2,
            contents,
        };

        Ok(Object::Array(arr))
    }
}

impl Sha256Gadget {
    fn prepare_inputs<F: FieldElement>(
        evaluator: &mut Evaluator<F>,
        env: &mut Environment<F>,
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
                    (lin.witness, F::MAX_NUM_BITS)
                }
                k => unimplemented!("SHA256 logic for {:?} is not implemented yet", k),
            };

            inputs.push(GadgetInput { witness, num_bits });
        }

        Ok(inputs)
    }
}
