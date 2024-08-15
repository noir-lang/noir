fn main() {
    comptime
    {
        // Check Expr::as_function_call
        let expr = quote { foo(bar) }.as_expr().unwrap();
        let (_function, args) = expr.as_function_call().unwrap();
        assert_eq(args.len(), 1);

        // Check Expr::as_parenthesized
        let expr = quote { (1) }.as_expr().unwrap();
        let _ = expr.as_parenthesized().unwrap();

        // Check Expr::as_tuple
        let expr = quote { (1, 2) }.as_expr().unwrap();
        let tuple_exprs = expr.as_tuple().unwrap();
        assert_eq(tuple_exprs.len(), 2);
    }
}
