use crate::{CallExpression, Environment, Evaluator, Object};

mod arraysum;
use arraysum::ArraySum;
mod arrayprod;
use arrayprod::ArrayProd;

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
        call_expr: CallExpression,
    ) -> Object;
}

pub fn call_builtin(        
    evaluator: &mut Evaluator,
    env: &mut Environment,
    builtin_name: &str,
    call_expr: CallExpression) -> Object 
{
   
    let func = match BuiltInFunctions::look_up_func_name(builtin_name) {
        None => panic!("cannot find a builtin function with the attribute name {}", builtin_name),
        Some(func) => func
    };

    match func {
        BuiltInFunctions::ArraySum => ArraySum::call(evaluator, env, call_expr),
        BuiltInFunctions::ArrayProd => ArrayProd::call(evaluator, env, call_expr),
        k => panic!("The builtin function `{}(..)` exists, however, currently the compiler does not have a concrete implementation for it", &call_expr.func_name.0.contents),
    }
}