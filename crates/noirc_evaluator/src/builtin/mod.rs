use crate::{
    errors::RuntimeError, interpreter::Interpreter, Environment, Object, RuntimeErrorKind,
};

use noirc_errors::Location;
use noirc_frontend::hir_def::expr::HirCallExpression;

#[derive(Debug)]
enum BuiltInFunctions {}

impl BuiltInFunctions {
    fn look_up_func_name(_name: &str) -> Option<BuiltInFunctions> {
        None
    }
}

pub trait BuiltInCaller {
    fn call(
        evaluator: &mut Interpreter,
        env: &mut Environment,
        call_expr: HirCallExpression,
        location: Location,
    ) -> Result<Object, RuntimeError>;
}

pub fn call_builtin(
    _evaluator: &mut Interpreter,
    _env: &mut Environment,
    builtin_name: &str,
    _call_expr: HirCallExpression,
    location: Location,
) -> Result<Object, RuntimeError> {
    let _func = BuiltInFunctions::look_up_func_name(builtin_name).ok_or_else(|| {
        let message =
            format!("cannot find a builtin function with the attribute name {}", builtin_name);
        RuntimeErrorKind::UnstructuredError { message }.add_location(location)
    })?;

    unreachable!("missing implementation for builtin function {}", builtin_name);
}
