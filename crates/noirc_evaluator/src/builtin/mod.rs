use crate::{Environment, Evaluator, Object, RuntimeErrorKind};

mod arraysum;
use arraysum::ArraySum;
mod arrayprod;
use arrayprod::ArrayProd;
mod setpub;
use noir_field::FieldElement;
use noirc_frontend::hir_def::expr::HirCallExpression;
use setpub::SetPub;

#[derive(Debug)]
enum BuiltInFunctions {
    ArraySum,
    ArrayProd,
    SetPub,
}

impl BuiltInFunctions {
    fn look_up_func_name(name: &str) -> Option<BuiltInFunctions> {
        match name {
            "arraysum" => Some(BuiltInFunctions::ArraySum),
            "arrayprod" => Some(BuiltInFunctions::ArrayProd),
            "set_pub" => Some(BuiltInFunctions::SetPub),
            _ => None,
        }
    }
}

pub trait BuiltInCaller<F: FieldElement> {
    fn call(
        evaluator: &mut Evaluator<F>,
        env: &mut Environment<F>,
        call_expr: HirCallExpression,
    ) -> Result<Object<F>, RuntimeErrorKind>;
}

pub fn call_builtin<F: FieldElement>(
    evaluator: &mut Evaluator<F>,
    env: &mut Environment<F>,
    builtin_name: &str,
    call_expr: HirCallExpression,
) -> Result<Object<F>, RuntimeErrorKind> {
    let func = match BuiltInFunctions::look_up_func_name(builtin_name) {
        None => {
            let message = format!(
                "cannot find a builtin function with the attribute name {}",
                builtin_name
            );
            return Err(RuntimeErrorKind::UnstructuredError {
                span: Default::default(),
                message,
            });
        }
        Some(func) => func,
    };

    match func {
        BuiltInFunctions::ArraySum => ArraySum::call(evaluator, env, call_expr),
        BuiltInFunctions::ArrayProd => ArrayProd::call(evaluator, env, call_expr),
        BuiltInFunctions::SetPub => SetPub::call(evaluator, env, call_expr),
    }
}
