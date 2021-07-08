use noirc_errors::Span;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::BuiltInCaller;
use crate::binary_op;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator, RuntimeError};

pub struct PowConst;

impl BuiltInCaller for PowConst {
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr_span: (HirCallExpression, Span),
    ) -> Result<Object, RuntimeError> {
        let (mut call_expr, span) = call_expr_span;

        assert_eq!(call_expr.arguments.len(), 2);
        let exponent = call_expr.arguments.pop().unwrap();
        let base = call_expr.arguments.pop().unwrap();

        let base_object = evaluator.expression_to_object(env, &base)?;
        let exponent_object = evaluator.expression_to_object(env, &exponent)?;

        let base = base_object.constant().map_err(|kind| kind.add_span(span))?;
        let exp = exponent_object
            .constant()
            .map_err(|kind| kind.add_span(span))?;

        let result = Object::Constants(base.pow(&exp));

        Ok(result)
    }
}
