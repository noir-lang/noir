use std::meta::op::UnaryOp;
use std::meta::op::BinaryOp;

fn main() {
    comptime
    {
        // Check Expr::as_function_call
        let expr = quote { foo(bar) }.as_expr().unwrap();
        let (_function, args) = expr.as_function_call().unwrap();
        assert_eq(args.len(), 1);

        // Check Expr::as_index
        let expr = quote { foo[bar] }.as_expr().unwrap();
        let _ = expr.as_index().unwrap();

        // Check Expr::as_tuple
        let expr = quote { (1, 2) }.as_expr().unwrap();
        let tuple_exprs = expr.as_tuple().unwrap();
        assert_eq(tuple_exprs.len(), 2);

        // Check Expr::as_if
        let expr = quote { if 1 { 2 } }.as_expr().unwrap();
        let (_condition, _consequence, alternative) = expr.as_if().unwrap();
        assert(alternative.is_none());

        let expr = quote { if 1 { 2 } else { 3 } }.as_expr().unwrap();
        let (_condition, _consequence, alternative) = expr.as_if().unwrap();
        assert(alternative.is_some());

        // Check parenthesized expression is automatically unwrapped
        let expr = quote { ((if 1 { 2 })) }.as_expr().unwrap();
        assert(expr.as_if().is_some());

        // Check Expr::as_bool
        let expr = quote { false }.as_expr().unwrap();
        assert(expr.as_bool().unwrap() == false);

        let expr = quote { true }.as_expr().unwrap();
        assert_eq(expr.as_bool().unwrap(), true);

        // Check Expr::as_unary_op
        assert(get_unary_op(quote { -x }).is_minus());
        assert(get_unary_op(quote { !x }).is_not());
        assert(get_unary_op(quote { &mut x }).is_mutable_reference());
        assert(get_unary_op(quote { *x }).is_dereference());

        // Check Expr::as_binary_op
        assert(get_binary_op(quote { x + y }).is_add());
        assert(get_binary_op(quote { x - y }).is_subtract());
        assert(get_binary_op(quote { x * y }).is_multiply());
        assert(get_binary_op(quote { x / y }).is_divide());
        assert(get_binary_op(quote { x == y }).is_equal());
        assert(get_binary_op(quote { x != y }).is_not_equal());
        assert(get_binary_op(quote { x > y }).is_greater());
        assert(get_binary_op(quote { x >= y }).is_greater_or_equal());
        assert(get_binary_op(quote { x & y }).is_and());
        assert(get_binary_op(quote { x | y }).is_or());
        assert(get_binary_op(quote { x ^ y }).is_xor());
        assert(get_binary_op(quote { x >> y }).is_shift_right());
        assert(get_binary_op(quote { x << y }).is_shift_left());
        assert(get_binary_op(quote { x % y }).is_modulo());

        // Check Expr::as_integer
        let expr = quote { 1 }.as_expr().unwrap();
        assert_eq((1, false), expr.as_integer().unwrap());

        let expr = quote { -2 }.as_expr().unwrap();
        assert_eq((2, true), expr.as_integer().unwrap());

        // Check Expr::as_member_access
        let expr = quote { foo.bar }.as_expr().unwrap();
        let (_, name) = expr.as_member_access().unwrap();
        assert_eq(name, quote { bar });
    }
}

comptime fn get_unary_op(quoted: Quoted) -> UnaryOp {
    let expr = quoted.as_expr().unwrap();
    let (op, _) = expr.as_unary_op().unwrap();
    op
}

comptime fn get_binary_op(quoted: Quoted) -> BinaryOp {
    let expr = quoted.as_expr().unwrap();
    let (_, op, _) = expr.as_binary_op().unwrap();
    op
}
