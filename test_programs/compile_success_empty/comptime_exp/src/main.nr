fn main() {
    comptime
    {
        let expr = quote { foo(bar) }.as_expr().unwrap();
        let (_function, args) = expr.as_function_call().unwrap();
        assert_eq(args.len(), 1);
    }
}
