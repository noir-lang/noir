use crate::option::Option;

impl Expr {
    #[builtin(expr_as_function_call)]
    fn as_function_call(self) -> Option<(Expr, [Expr])> {}

    #[builtin(expr_as_if)]
    fn as_if(self) -> Option<(Expr, Expr, Option<Expr>)> {}

    #[builtin(expr_as_index)]
    fn as_index(self) -> Option<(Expr, Expr)> {}

    #[builtin(expr_as_parenthesized)]
    fn as_parenthesized(self) -> Option<Expr> {}

    #[builtin(expr_as_tuple)]
    fn as_tuple(self) -> Option<[Expr]> {}
}
