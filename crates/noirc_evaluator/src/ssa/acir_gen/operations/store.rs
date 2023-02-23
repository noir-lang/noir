use acvm::{
    acir::circuit::opcodes::Opcode as AcirOpcode, acir::native_types::Expression, FieldElement,
};

use crate::{
    ssa::{
        acir_gen::{
            acir_mem::AcirMem,
            constraints::{self, subtract},
            expression_from_witness,
            internal_var_cache::InternalVarCache,
            operations::condition,
            InternalVar,
        },
        context::SsaContext,
        mem,
        node::{NodeId, Operation},
    },
    Evaluator,
};

fn unwrap_predicate(ctx: &SsaContext, predicate: Option<NodeId>) -> NodeId {
    if let Some(predicate) = predicate {
        if predicate.is_dummy() {
            ctx.one()
        } else {
            predicate
        }
    } else {
        ctx.one()
    }
}

pub(crate) fn evaluate(
    store: &Operation,
    acir_mem: &mut AcirMem,
    var_cache: &mut InternalVarCache,
    evaluator: &mut Evaluator,
    ctx: &SsaContext,
) -> Option<InternalVar> {
    if let Operation::Store { array_id, index, value, predicate } = *store {
        //maps the address to the rhs if address is known at compile time
        let index_var = var_cache.get_or_compute_internal_var_unwrap(index, evaluator, ctx);
        let value_var = var_cache.get_or_compute_internal_var_unwrap(value, evaluator, ctx);

        let pred = unwrap_predicate(ctx, predicate);
        if ctx.is_zero(pred) {
            return None;
        }
        let pred_var = var_cache.get_or_compute_internal_var_unwrap(pred, evaluator, ctx);

        match index_var.to_const() {
            Some(idx) => {
                let idx = mem::Memory::as_u32(idx);
                if ctx.is_one(pred) {
                    acir_mem.insert(array_id, idx, value_var, Expression::one());
                } else if let Some(dummy_load) =
                    acir_mem.load_array_element_constant_index(array_id, idx)
                {
                    let value_with_predicate = condition::evaluate_expression(
                        pred_var.expression(),
                        value_var.expression(),
                        dummy_load.expression(),
                        evaluator,
                    );
                    acir_mem.insert(array_id, idx, value_with_predicate.into(), Expression::one());
                } else {
                    let w = evaluator.add_witness_to_cs();
                    let w_expr = expression_from_witness(w);
                    let sub_p = constraints::subtract(
                        &Expression::one(),
                        FieldElement::one(),
                        pred_var.expression(),
                    );
                    let expr1 = constraints::mul_with_witness(evaluator, &sub_p, &w_expr);
                    let expr2 = constraints::mul_with_witness(
                        evaluator,
                        pred_var.expression(),
                        value_var.expression(),
                    );
                    let w1 = evaluator.create_intermediate_variable(expr1);
                    evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(
                        &w_expr,
                        FieldElement::one(),
                        &expression_from_witness(w1),
                    )));
                    let value_with_predicate =
                        constraints::add(&w_expr, FieldElement::one(), &expr2);
                    acir_mem.insert(
                        array_id,
                        idx,
                        value_with_predicate.into(),
                        pred_var.to_expression(),
                    );
                }
            }
            None => {
                let mut op = Expression::one();
                let mut val = value_var.to_expression();
                if let Some(predicate) = predicate {
                    if !predicate.is_dummy() && !ctx.is_one(predicate) {
                        let w = evaluator.add_witness_to_cs();
                        let pred =
                            var_cache.get_or_compute_internal_var_unwrap(predicate, evaluator, ctx);
                        op = pred.to_expression();
                        val = condition::evaluate_expression(
                            pred.expression(),
                            &value_var.to_expression(),
                            &expression_from_witness(w),
                            evaluator,
                        );
                    }
                }
                acir_mem.add_to_trace(&array_id, index_var.to_expression(), val, op);
            }
        }
    } else {
        unreachable!("Expected store, got {:?}", store.opcode());
    }
    //we do not generate constraint, so no output.
    None
}
