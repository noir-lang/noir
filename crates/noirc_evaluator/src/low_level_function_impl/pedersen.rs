use super::GadgetCaller;
use super::RuntimeError;
use crate::low_level_function_impl::object_to_wit_bits;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OpCode;
use acvm::FieldElement;
use noirc_frontend::hir_def::expr::HirCallExpression;

pub struct PedersenGadget;

impl GadgetCaller for PedersenGadget {
    fn name() -> OpCode {
        OpCode::Pedersen
    }

    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeError> {
        let inputs = PedersenGadget::prepare_inputs(evaluator, env, call_expr)?;

        let pedersen_output_x = evaluator.add_witness_to_cs();
        let object_pedersen_x = Object::from_witness(pedersen_output_x);

        let pedersen_output_y = evaluator.add_witness_to_cs();
        let object_pedersen_y = Object::from_witness(pedersen_output_y);

        let pedersen_gate = GadgetCall {
            name: PedersenGadget::name(),
            inputs,
            outputs: vec![pedersen_output_x, pedersen_output_y],
        };

        evaluator.gates.push(Gate::GadgetCall(pedersen_gate));

        let arr = Array {
            length: 2,
            contents: vec![object_pedersen_x, object_pedersen_y],
        };

        Ok(Object::Array(arr))
    }
}

impl PedersenGadget {
    fn prepare_inputs(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeError> {
        let arr_expr = {
            // For pedersen gadget, we expect a single input which should be an array
            assert_eq!(call_expr.arguments.len(), 1);
            call_expr.arguments.pop().unwrap()
        };

        let arr = Array::from_expression(evaluator, env, &arr_expr)?;

        // XXX: Instead of panics, return a user error here
        if arr.contents.is_empty() {
            panic!("a pedersen hash requires at least one element to hash")
        }

        let mut inputs: Vec<GadgetInput> = Vec::new();

        for element in arr.contents.into_iter() {
            let gadget_inp = object_to_wit_bits(&element);
            assert_eq!(gadget_inp.num_bits, FieldElement::max_num_bits());

            inputs.push(gadget_inp);
        }

        Ok(inputs)
    }
}
