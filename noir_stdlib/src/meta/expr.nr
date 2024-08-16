use crate::option::Option;
use crate::meta::op::UnaryOp;

impl Expr {
    #[builtin(expr_as_bool)]
    fn as_bool(self) -> Option<bool> {}

    #[builtin(expr_as_function_call)]
    fn as_function_call(self) -> Option<(Expr, [Expr])> {}

    #[builtin(expr_as_if)]
    fn as_if(self) -> Option<(Expr, Expr, Option<Expr>)> {}

    #[builtin(expr_as_index)]
    fn as_index(self) -> Option<(Expr, Expr)> {}

    #[builtin(expr_as_tuple)]
    fn as_tuple(self) -> Option<[Expr]> {}

    #[builtin(expr_as_unary_op)]
    fn as_unary_op(self) -> Option<(UnaryOp, Expr)> {}
}
