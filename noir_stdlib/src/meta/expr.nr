use crate::option::Option;

impl Expr {
    #[builtin(expr_as_function_call)]
    fn as_function_call(self) -> Option<(Expr, [Expr])> {}
}
