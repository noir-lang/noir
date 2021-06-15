use super::GadgetCaller;
use crate::low_level_function_impl::object_to_wit_bits;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::RuntimeErrorKind;

pub struct SchnorrVerifyGadget;

impl GadgetCaller for SchnorrVerifyGadget {
    fn name() -> OPCODE {
        OPCODE::SchnorrVerify
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeErrorKind> {
        let inputs = SchnorrVerifyGadget::prepare_inputs(evaluator, env, call_expr)?;

        // Prepare output

        // Create a fresh variable which will be the root

        let schnorr_verify_witness = evaluator.add_witness_to_cs();
        let schnorr_verify_object = Object::from_witness(schnorr_verify_witness);

        let schnorr_verify_gate = GadgetCall {
            name: SchnorrVerifyGadget::name(),
            inputs,
            outputs: vec![schnorr_verify_witness],
        };

        evaluator.gates.push(Gate::GadgetCall(schnorr_verify_gate));

        Ok(schnorr_verify_object)
    }
}

impl SchnorrVerifyGadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeErrorKind> {
        assert_eq!(call_expr.arguments.len(), 4);

        let pub_key_y = call_expr.arguments.pop().unwrap();
        let pub_key_x = call_expr.arguments.pop().unwrap();
        let message = call_expr.arguments.pop().unwrap();
        let signature = call_expr.arguments.pop().unwrap();

        let signature =
            Array::from_expression(evaluator, env, &signature).map_err(|err| err.remove_span())?;
        let message =
            Array::from_expression(evaluator, env, &message).map_err(|err| err.remove_span())?;
        let pub_key_x = evaluator
            .expression_to_object(env, &pub_key_x)
            .map_err(|err| err.remove_span())?;
        let pub_key_y = evaluator
            .expression_to_object(env, &pub_key_y)
            .map_err(|err| err.remove_span())?;

        let pub_key_x_witness = pub_key_x.witness().unwrap();
        let pub_key_y_witness = pub_key_y.witness().unwrap();

        let mut inputs: Vec<GadgetInput> = vec![GadgetInput {
            witness: pub_key_x_witness,
            num_bits: noir_field::FieldElement::max_num_bits(),
        }];
        inputs.push(GadgetInput {
            witness: pub_key_y_witness,
            num_bits: noir_field::FieldElement::max_num_bits(),
        });

        // XXX: Technical debt: refactor so this functionality,
        // is not repeated across many gadgets
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
