use super::GadgetCaller;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator, Type};
use acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acir::OPCODE;
use noirc_frontend::hir::lower::HirCallExpression;

use super::RuntimeErrorKind;

pub struct HashToFieldGadget;

impl GadgetCaller for HashToFieldGadget {
    fn name() -> OPCODE {
        OPCODE::HashToField
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeErrorKind> {
        let inputs = HashToFieldGadget::prepare_inputs(evaluator, env, call_expr)?;

        let res_name = evaluator.make_unique("hash_to_field_res");
        let res_witness = evaluator.add_witness_to_cs(); // XXX: usually the output of the function is public. To be conservative, lets make it private
        let res_object = evaluator.add_witness_to_env(res_name, res_witness.clone(), env);

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
    ) -> Result<Vec<GadgetInput>, RuntimeErrorKind> {
        let arr_expr = {
            // For HashToField, we expect a single input which should be an array
            assert_eq!(call_expr.arguments.len(), 1);
            call_expr.arguments.pop().unwrap()
        };

        // "HashToField should only take a single parameter, which is an array. This should have been caught by the compiler in the analysis phase";
        let arr = Array::from_expression(evaluator, env, &arr_expr)?;

        let mut inputs: Vec<GadgetInput> = Vec::with_capacity(arr.contents.len());

        for element in arr.contents.into_iter() {
            let (witness, num_bits) = match element {
                Object::Integer(integer) => (integer.witness, integer.num_bits),
                Object::Linear(lin) => {
                    if !lin.is_unit() {
                        unimplemented!(
                            "HashToField Logic for non unit witnesses is currently not implemented"
                        )
                    }
                    (lin.witness, noir_field::FieldElement::max_num_bits())
                }
                k => unimplemented!("HashToField logic for {:?} is not implemented yet", k),
            };

            inputs.push(GadgetInput { witness, num_bits });
        }

        Ok(inputs)
    }
}
