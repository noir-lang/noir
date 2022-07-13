use noirc_errors::Span;
use noirc_frontend::hir_def::expr::HirCallExpression;

use super::BuiltInCaller;
use crate::object::Object;
use crate::FuncContext;
use crate::{Environment, Evaluator, RuntimeError, RuntimeErrorKind};
pub struct SetPub;

impl BuiltInCaller for SetPub {
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr_span: (HirCallExpression, Span),
    ) -> Result<Object, RuntimeError> {
        let (mut call_expr, span) = call_expr_span;

        assert_eq!(call_expr.arguments.len(), 1);
        let expr = call_expr.arguments.pop().unwrap();

        let object = evaluator.expression_to_object(env, &expr)?;

        // This can only be called in the main context
        if env.func_context != FuncContext::Main {
            let func_name =
                evaluator.context.def_interner.function_name(&call_expr.func_id).to_owned();

            return Err(RuntimeErrorKind::FunctionNonMainContext { func_name }.add_location(span));
        }

        let witness = object.witness().expect("expected a witness");

        evaluator.public_inputs.push(witness);

        Ok(object)
    }
}
