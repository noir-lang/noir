use crate::{Environment, Evaluator, Object, RuntimeErrorKind};

mod arraysum;
use arraysum::ArraySum;
mod arrayprod;
use arrayprod::ArrayProd;
use noirc_frontend::hir::lower::HirCallExpression;

#[derive(Debug)]
enum BuiltInFunctions{
    ArraySum,
    ArrayProd
} 

impl BuiltInFunctions {
    fn look_up_func_name(name : &str) -> Option<BuiltInFunctions> {
        match name {
            "arraysum" => Some(BuiltInFunctions::ArraySum) ,
            "arrayprod" => Some(BuiltInFunctions::ArrayProd) ,
            _=> None
        }
    }
}

pub trait BuiltInCaller {
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeErrorKind>;
}

pub fn call_builtin(        
    evaluator: &mut Evaluator,
    env: &mut Environment,
    builtin_name: &str,
    call_expr: HirCallExpression) -> Result<Object, RuntimeErrorKind> 
{
   
    let func = match BuiltInFunctions::look_up_func_name(builtin_name) {
        None => {
            let message = format!("cannot find a builtin function with the attribute name {}", builtin_name);
            return Err(RuntimeErrorKind::UnstructuredError{span : Default::default(), message})
        }
        Some(func) => func
    };
    
    match func {
        BuiltInFunctions::ArraySum => ArraySum::call(evaluator, env, call_expr),
        BuiltInFunctions::ArrayProd => ArrayProd::call(evaluator, env, call_expr),
    }
}