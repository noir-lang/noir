use noirc_errors::Location;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::BuiltInCaller;
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::FuncContext;
use crate::{Environment, RuntimeError, RuntimeErrorKind};
pub struct SetPub;

impl BuiltInCaller for SetPub {
    fn call(
        evaluator: &mut Interpreter,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
        location: Location,
    ) -> Result<Object, RuntimeError> {
        assert_eq!(call_expr.arguments.len(), 1);
        let expr = call_expr.arguments.pop().unwrap();

        let object = evaluator.expression_to_object(env, &expr)?;

        // This can only be called in the main context
        if env.func_context != FuncContext::Main {
            let func_name =
                evaluator.context.def_interner.function_name(&call_expr.func_id).to_owned();

            return Err(
                RuntimeErrorKind::FunctionNonMainContext { func_name }.add_location(location)
            );
        }

        let witness = object.witness().expect("expected a witness");

        evaluator.push_public_input(witness);

        Ok(object)
    }
}
