use super::BuiltInCaller;
use crate::object::{Array, Object};
use crate::binary_op;
use crate::{CallExpression, Environment, Evaluator};

/// Sums all of the elements in an array
pub struct ArrayProd;

impl BuiltInCaller for ArrayProd {
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: CallExpression,
    ) -> Object {

        assert_eq!(call_expr.arguments.len(),1);
        let arr_expr = call_expr.arguments.pop().unwrap();
        let arr = match Array::from_expression(evaluator, env, arr_expr) {
            Some(arr) => arr,
            None => panic!("ArrayProd should only take a single parameter, which is an array. This should have been caught by the compiler in the analysis phase")
        };

        let mut result = arr.get(0);
        for i in 1..arr.contents.len(){
            result = binary_op::handle_mul_op(result, arr.get(i as u128),env, evaluator);
        }

        result
    }
}