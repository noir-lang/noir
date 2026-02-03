fn main() {
    comptime {
        let array = quote { [1, 2, 3] }.as_expr().unwrap();
        let expr1 = quote { [1, 2, 3]};
        let expr2 = quote { $array };
        assert_eq(expr1, expr2);
    }
}
