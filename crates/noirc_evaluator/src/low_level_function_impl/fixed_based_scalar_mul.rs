use super::GadgetCaller;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator};
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
use noir_field::FieldElement;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::RuntimeErrorKind;

pub struct FixedBaseScalarMulGadget;

impl<F: FieldElement> GadgetCaller<F> for FixedBaseScalarMulGadget {
    fn name() -> OPCODE {
        OPCODE::FixedBaseScalarMul
    }

    fn call(
        evaluator: &mut Evaluator<F>,
        env: &mut Environment<F>,
        call_expr: HirCallExpression,
    ) -> Result<Object<F>, RuntimeErrorKind> {
        let inputs = FixedBaseScalarMulGadget::prepare_inputs(evaluator, env, call_expr)?;

        let witness_pubkey_x = evaluator.add_witness_to_cs();
        let object_pubkey_x = Object::from_witness(witness_pubkey_x);

        let witness_pubkey_y = evaluator.add_witness_to_cs();
        let object_pubkey_y = Object::from_witness(witness_pubkey_y);

        let fixed_base_gate = GadgetCall {
            name: OPCODE::FixedBaseScalarMul,
            inputs,
            outputs: vec![witness_pubkey_x, witness_pubkey_y],
        };

        evaluator.gates.push(Gate::GadgetCall(fixed_base_gate));

        let arr = Array {
            length: 2,
            contents: vec![object_pubkey_x, object_pubkey_y],
        };

        Ok(Object::Array(arr))
    }
}

impl FixedBaseScalarMulGadget {
    fn prepare_inputs<F: FieldElement>(
        evaluator: &mut Evaluator<F>,
        env: &mut Environment<F>,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeErrorKind> {
        let expr = {
            // we expect a single input which should be a Field
            assert_eq!(call_expr.arguments.len(), 1);
            call_expr.arguments.pop().unwrap()
        };

        let object = evaluator.expression_to_object(env, &expr)?;

        let (witness, num_bits) = match object {
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

        Ok(vec![GadgetInput { witness, num_bits }])
    }
}
