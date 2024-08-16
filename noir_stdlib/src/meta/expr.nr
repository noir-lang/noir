use crate::option::Option;
use crate::meta::op::UnaryOp;
use crate::meta::op::BinaryOp;

impl Expr {
    #[builtin(expr_as_integer)]
    fn as_integer(self) -> Option<(Field, bool)> {}

    #[builtin(expr_as_binary_op)]
    fn as_binary_op(self) -> Option<(Expr, BinaryOp, Expr)> {}

    #[builtin(expr_as_bool)]
    fn as_bool(self) -> Option<bool> {}

    #[builtin(expr_as_function_call)]
    fn as_function_call(self) -> Option<(Expr, [Expr])> {}

    #[builtin(expr_as_if)]
    fn as_if(self) -> Option<(Expr, Expr, Option<Expr>)> {}

    #[builtin(expr_as_index)]
    fn as_index(self) -> Option<(Expr, Expr)> {}

    #[builtin(expr_as_member_access)]
    fn as_member_access(self) -> Option<(Expr, Quoted)> {}

    #[builtin(expr_as_tuple)]
    fn as_tuple(self) -> Option<[Expr]> {}

    #[builtin(expr_as_unary_op)]
    fn as_unary_op(self) -> Option<(UnaryOp, Expr)> {}
}
