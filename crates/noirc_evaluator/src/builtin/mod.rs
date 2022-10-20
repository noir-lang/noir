use crate::{
    errors::RuntimeError, interpreter::Interpreter, Environment, Object, RuntimeErrorKind,
};

mod arraysum;
use arraysum::ArraySum;
mod arrayprod;
use arrayprod::ArrayProd;
mod pred_eq;
use pred_eq::PredicateEq;

use noirc_errors::Location;
use noirc_frontend::hir_def::expr::HirCallExpression;

#[derive(Debug)]
enum BuiltInFunctions {
    ArraySum,
    ArrayProd,
    PredEq,
}

impl BuiltInFunctions {
    fn look_up_func_name(name: &str) -> Option<BuiltInFunctions> {
        match name {
            "arraysum" => Some(BuiltInFunctions::ArraySum),
            "arrayprod" => Some(BuiltInFunctions::ArrayProd),
            "predicate_equal" => Some(BuiltInFunctions::PredEq),
            _ => None,
        }
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
    evaluator: &mut Interpreter,
    env: &mut Environment,
    builtin_name: &str,
    call_expr: HirCallExpression,
    location: Location,
) -> Result<Object, RuntimeError> {
    let func = BuiltInFunctions::look_up_func_name(builtin_name).ok_or_else(|| {
        let message =
            format!("cannot find a builtin function with the attribute name {}", builtin_name);
        RuntimeErrorKind::UnstructuredError { message }.add_location(location)
    })?;

    match func {
        BuiltInFunctions::ArraySum => ArraySum::call(evaluator, env, call_expr, location),
        BuiltInFunctions::ArrayProd => ArrayProd::call(evaluator, env, call_expr, location),
        BuiltInFunctions::PredEq => PredicateEq::call(evaluator, env, call_expr, location),
    }
}
