use crate::option::Option;
use crate::meta::op::UnaryOp;
use crate::meta::op::BinaryOp;

impl Expr {
    #[builtin(expr_as_array)]
    fn as_array(self) -> Option<[Expr]> {}

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

    #[builtin(expr_as_repeated_element_array)]
    fn as_repeated_element_array(self) -> Option<(Expr, Expr)> {}

    #[builtin(expr_as_repeated_element_slice)]
    fn as_repeated_element_slice(self) -> Option<(Expr, Expr)> {}

    #[builtin(expr_as_slice)]
    fn as_slice(self) -> Option<[Expr]> {}

    #[builtin(expr_as_tuple)]
    fn as_tuple(self) -> Option<[Expr]> {}

    #[builtin(expr_as_unary_op)]
    fn as_unary_op(self) -> Option<(UnaryOp, Expr)> {}
}

mod tests {
    use crate::meta::op::UnaryOp;
    use crate::meta::op::BinaryOp;

    #[test]
    fn test_expr_as_array() {
        comptime
        {
            let expr = quote { [1, 2, 4] }.as_expr().unwrap();
            let elems = expr.as_array().unwrap();
            assert_eq(elems.len(), 3);
            assert_eq(elems[0].as_integer().unwrap(), (1, false));
            assert_eq(elems[1].as_integer().unwrap(), (2, false));
            assert_eq(elems[2].as_integer().unwrap(), (4, false));
        }
    }

    #[test]
    fn test_expr_as_integer() {
        comptime
        {
            let expr = quote { 1 }.as_expr().unwrap();
            assert_eq((1, false), expr.as_integer().unwrap());

            let expr = quote { -2 }.as_expr().unwrap();
            assert_eq((2, true), expr.as_integer().unwrap());
        }
    }

    #[test]
    fn test_expr_as_binary_op() {
        comptime
        {
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
        }
    }

    #[test]
    fn test_expr_as_bool() {
        comptime
        {
            let expr = quote { false }.as_expr().unwrap();
            assert(expr.as_bool().unwrap() == false);

            let expr = quote { true }.as_expr().unwrap();
            assert_eq(expr.as_bool().unwrap(), true);
        }
    }

    #[test]
    fn test_expr_as_function_call() {
        comptime
        {
            let expr = quote { foo(42) }.as_expr().unwrap();
            let (_function, args) = expr.as_function_call().unwrap();
            assert_eq(args.len(), 1);
            assert_eq(args[0].as_integer().unwrap(), (42, false));
        }
    }

    #[test]
    fn test_expr_as_if() {
        comptime
        {
            let expr = quote { if 1 { 2 } }.as_expr().unwrap();
            let (_condition, _consequence, alternative) = expr.as_if().unwrap();
            assert(alternative.is_none());

            let expr = quote { if 1 { 2 } else { 3 } }.as_expr().unwrap();
            let (_condition, _consequence, alternative) = expr.as_if().unwrap();
            assert(alternative.is_some());
        }
    }

    #[test]
    fn test_expr_as_index() {
        comptime
        {
            let expr = quote { foo[bar] }.as_expr().unwrap();
            assert(expr.as_index().is_some());
        }
    }

    #[test]
    fn test_expr_as_member_access() {
        comptime
        {
            let expr = quote { foo.bar }.as_expr().unwrap();
            let (_, name) = expr.as_member_access().unwrap();
            assert_eq(name, quote { bar });
        }
    }

    #[test]
    fn test_expr_as_repeated_element_array() {
        comptime
        {
            let expr = quote { [1; 3] }.as_expr().unwrap();
            let (expr, length) = expr.as_repeated_element_array().unwrap();
            assert_eq(expr.as_integer().unwrap(), (1, false));
            assert_eq(length.as_integer().unwrap(), (3, false));
        }
    }

    #[test]
    fn test_expr_as_repeated_element_slice() {
        comptime
        {
            let expr = quote { &[1; 3] }.as_expr().unwrap();
            let (expr, length) = expr.as_repeated_element_slice().unwrap();
            assert_eq(expr.as_integer().unwrap(), (1, false));
            assert_eq(length.as_integer().unwrap(), (3, false));
        }
    }

    #[test]
    fn test_expr_as_slice() {
        comptime
        {
            let expr = quote { &[1, 3, 5] }.as_expr().unwrap();
            let elems = expr.as_slice().unwrap();
            assert_eq(elems.len(), 3);
            assert_eq(elems[0].as_integer().unwrap(), (1, false));
            assert_eq(elems[1].as_integer().unwrap(), (3, false));
            assert_eq(elems[2].as_integer().unwrap(), (5, false));
        }
    }

    #[test]
    fn test_expr_as_tuple() {
        comptime
        {
            let expr = quote { (1, 2) }.as_expr().unwrap();
            let tuple_exprs = expr.as_tuple().unwrap();
            assert_eq(tuple_exprs.len(), 2);
        }
    }

    #[test]
    fn test_expr_as_unary_op() {
        comptime
        {
            assert(get_unary_op(quote { -x }).is_minus());
            assert(get_unary_op(quote { !x }).is_not());
            assert(get_unary_op(quote { &mut x }).is_mutable_reference());
            assert(get_unary_op(quote { *x }).is_dereference());
        }
    }

    #[test]
    fn test_automatically_unwraps_parenthesized_expression() {
        comptime
        {
            let expr = quote { ((if 1 { 2 })) }.as_expr().unwrap();
            assert(expr.as_if().is_some());
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
}
