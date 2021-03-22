use noirc_frontend::hir_def::expr::HirCallExpression;

use super::BuiltInCaller;
use crate::object::Object;
use crate::FuncContext;
use crate::{Environment, Evaluator, RuntimeErrorKind};
pub struct SetPub;

impl BuiltInCaller for SetPub {
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeErrorKind> {
        assert_eq!(call_expr.arguments.len(), 1);
        let expr = call_expr.arguments.pop().unwrap();

        let object = evaluator.expression_to_object(env, &expr)?;

        // This can only be called in the main context
        if env.func_context != FuncContext::Main {
            let span = evaluator.context.def_interner.expr_span(&expr);
            let func_name = evaluator
                .context
                .def_interner
                .function_meta(&call_expr.func_id)
                .name;
            return Err(RuntimeErrorKind::FunctionNonMainContext { func_name, span });
        }

        let witness = object.witness().expect("expected a witness");

        evaluator.public_inputs.push(witness);

        Ok(object)
    }
}
