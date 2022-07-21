use noirc_errors::Location;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::BuiltInCaller;
use crate::binary_op;
use crate::object::{Array, Object};
use crate::{Environment, Evaluator, RuntimeError};

/// Sums all of the elements in an array
pub struct ArraySum;

impl BuiltInCaller for ArraySum {
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
        location: Location,
    ) -> Result<Object, RuntimeError> {
        let arr_expr = {
            assert_eq!(call_expr.arguments.len(), 1);
            call_expr.arguments.pop().unwrap()
        };

        // ArraySum should only take a single parameter, which is an array. This should have been caught by the compiler in the analysis phase
        let arr = Array::from_expression(evaluator, env, &arr_expr)?;

        let mut result = arr.get(0).map_err(|kind| kind.add_location(location))?;
        for i in 1..arr.contents.len() {
            result = binary_op::handle_add_op(
                result,
                arr.get(i as u128).map_err(|kind| kind.add_location(location))?,
                evaluator,
            )
            .map_err(|kind| kind.add_location(location))?;
        }

        Ok(result)
    }
}
