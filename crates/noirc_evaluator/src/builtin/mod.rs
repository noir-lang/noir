use crate::{errors::RuntimeError, interpreter::Interpreter, Environment, Object};

mod arrayprod;
mod arraysum;
mod pred_eq;

use noirc_errors::Location;
use noirc_frontend::hir_def::expr::HirCallExpression;

pub trait BuiltInCaller {
    fn call(
        evaluator: &mut Interpreter,
        env: &mut Environment,
        call_expr: HirCallExpression,
        location: Location,
    ) -> Result<Object, RuntimeError>;
}
