use noirc_errors::Span;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::BuiltInCaller;
use crate::binary_op::maybe_equal;
use crate::object::Object;
use crate::{Environment, Evaluator, RuntimeError};

/// Returns a 0 or 1, if the two elements are equal
pub struct PredicateEq;

impl BuiltInCaller for PredicateEq {
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr_span: (HirCallExpression, Span),
    ) -> Result<Object, RuntimeError> {
        let (mut call_expr, span) = call_expr_span;

        assert_eq!(call_expr.arguments.len(), 2);
        let rhs = call_expr.arguments.pop().unwrap();
        let lhs = call_expr.arguments.pop().unwrap();

        let lhs_obj = evaluator.expression_to_object(env, &lhs)?;
        let rhs_obj = evaluator.expression_to_object(env, &rhs)?;

        let pred = maybe_equal(lhs_obj, rhs_obj, evaluator).map_err(|kind| kind.add_span(span))?;
        Ok(Object::from_witness(pred.witness))
    }
}
