use super::GadgetCaller;
use crate::low_level_function_impl::object_to_wit_bits;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OpCode;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::RuntimeError;

pub struct EcdsaSecp256k1Gadget;

impl GadgetCaller for EcdsaSecp256k1Gadget {
    fn name() -> OpCode {
        OpCode::EcdsaSecp256k1
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeError> {
        let inputs = EcdsaSecp256k1Gadget::prepare_inputs(evaluator, env, call_expr)?;

        // Prepare output

        // Create a fresh variable which will be the root

        let _verify_witness = evaluator.add_witness_to_cs();
        let _verify_object = Object::from_witness(_verify_witness);

        let _verify_gate = GadgetCall {
            name: EcdsaSecp256k1Gadget::name(),
            inputs,
            outputs: vec![_verify_witness],
        };

        evaluator.gates.push(Gate::GadgetCall(_verify_gate));

        Ok(_verify_object)
    }
}

impl EcdsaSecp256k1Gadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeError> {
        assert_eq!(call_expr.arguments.len(), 4);

        let pub_key_y = call_expr.arguments.pop().unwrap();
        let pub_key_x = call_expr.arguments.pop().unwrap();
        let message = call_expr.arguments.pop().unwrap();
        let signature = call_expr.arguments.pop().unwrap();

        let signature = Array::from_expression(evaluator, env, &signature)?;
        let message = Array::from_expression(evaluator, env, &message)?;
        let pub_key_x = Array::from_expression(evaluator, env, &pub_key_x)?;
        let pub_key_y = Array::from_expression(evaluator, env, &pub_key_y)?;

        let mut inputs: Vec<GadgetInput> = Vec::new();

        for element in pub_key_x.contents.into_iter() {
            let gadget_inp = object_to_wit_bits(&element);
            assert_eq!(gadget_inp.num_bits, 8);

            inputs.push(gadget_inp);
        }

        for element in pub_key_y.contents.into_iter() {
            let gadget_inp = object_to_wit_bits(&element);
            assert_eq!(gadget_inp.num_bits, 8);

            inputs.push(gadget_inp);
        }

        for element in signature.contents.into_iter() {
            let gadget_inp = object_to_wit_bits(&element);
            assert_eq!(gadget_inp.num_bits, 8);

            inputs.push(gadget_inp);
        }
        for element in message.contents.into_iter() {
            let gadget_inp = object_to_wit_bits(&element);
            assert_eq!(gadget_inp.num_bits, 8);

            inputs.push(gadget_inp);
        }

        Ok(inputs)
    }
}
