fn main() {
    comptime {
        let expr = quote { [1, 3] }.as_expr().unwrap().resolve(Option::none());
        assert_eq(expr.get_type().unwrap(), quote { [Field; 2] }.as_type());
    }
}
