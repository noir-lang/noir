use noirc_errors::Location;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::BuiltInCaller;
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::{Environment, RuntimeError};

pub struct PowConst;

impl BuiltInCaller for PowConst {
    fn call(
        evaluator: &mut Interpreter,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
        location: Location,
    ) -> Result<Object, RuntimeError> {
        assert_eq!(call_expr.arguments.len(), 2);
        let exponent = call_expr.arguments.pop().unwrap();
        let base = call_expr.arguments.pop().unwrap();

        let base_object = evaluator.expression_to_object(env, &base)?;
        let exponent_object = evaluator.expression_to_object(env, &exponent)?;

        let base = base_object.constant().map_err(|kind| kind.add_location(location))?;
        let exp = exponent_object.constant().map_err(|kind| kind.add_location(location))?;

        let result = Object::Constants(base.pow(&exp));

        Ok(result)
    }
}
