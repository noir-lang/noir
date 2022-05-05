use crate::{errors::RuntimeError, Environment, Evaluator, Object, RuntimeErrorKind};

mod arraysum;
use arraysum::ArraySum;
mod arrayprod;
use arrayprod::ArrayProd;
mod pred_eq;
use pred_eq::PredicateEq;
mod pow_const;
use pow_const::PowConst;
mod setpub;
use setpub::SetPub;

use noirc_errors::Span;
use noirc_frontend::hir_def::expr::HirCallExpression;

#[derive(Debug)]
enum BuiltInFunctions {
    ArraySum,
    ArrayProd,
    SetPub,
    PowConst,
    PredEq,
}

impl BuiltInFunctions {
    fn look_up_func_name(name: &str) -> Option<BuiltInFunctions> {
        match name {
            "arraysum" => Some(BuiltInFunctions::ArraySum),
            "arrayprod" => Some(BuiltInFunctions::ArrayProd),
            "set_pub" => Some(BuiltInFunctions::SetPub),
            "pow_const" => Some(BuiltInFunctions::PowConst),
            "predicate_equal" => Some(BuiltInFunctions::PredEq),
            _ => None,
        }
    }
}

pub trait BuiltInCaller {
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr_span: (HirCallExpression, Span),
    ) -> Result<Object, RuntimeError>;
}

pub fn call_builtin(
    evaluator: &mut Evaluator,
    env: &mut Environment,
    builtin_name: &str,
    call_expr_span: (HirCallExpression, Span),
) -> Result<Object, RuntimeError> {
    let (call_expr, span) = call_expr_span;
    let func = match BuiltInFunctions::look_up_func_name(builtin_name) {
        None => {
            let message =
                format!("cannot find a builtin function with the attribute name {}", builtin_name);
            return Err(RuntimeErrorKind::UnstructuredError { message }.add_span(span));
        }
        Some(func) => func,
    };

    match func {
        BuiltInFunctions::ArraySum => ArraySum::call(evaluator, env, (call_expr, span)),
        BuiltInFunctions::ArrayProd => ArrayProd::call(evaluator, env, (call_expr, span)),
        BuiltInFunctions::SetPub => SetPub::call(evaluator, env, (call_expr, span)),
        BuiltInFunctions::PowConst => PowConst::call(evaluator, env, (call_expr, span)),
        BuiltInFunctions::PredEq => PredicateEq::call(evaluator, env, (call_expr, span)),
    }
}
