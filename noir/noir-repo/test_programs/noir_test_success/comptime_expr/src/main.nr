mod tests {
    use std::meta::op::UnaryOp;
    use std::meta::op::BinaryOp;

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
    fn test_expr_as_assign() {
        comptime
        {
            let expr = quote { { a = 1; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            let (_lhs, rhs) = exprs[0].as_assign().unwrap();
            assert_eq(rhs.as_integer().unwrap(), (1, false));
        }
    }

    #[test]
    fn test_expr_as_block() {
        comptime
        {
            let expr = quote { { 1; 4; 23 } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            assert_eq(exprs.len(), 3);
            assert_eq(exprs[0].as_integer().unwrap(), (1, false));
            assert_eq(exprs[1].as_integer().unwrap(), (4, false));
            assert_eq(exprs[2].as_integer().unwrap(), (23, false));

            assert(exprs[0].has_semicolon());
            assert(exprs[1].has_semicolon());
            assert(!exprs[2].has_semicolon());
        }
    }

    #[test]
    fn test_expr_as_method_call() {
        comptime
        {
            let expr = quote { foo.bar::<Field>(3, 4) }.as_expr().unwrap();
            let (_object, name, generics, arguments) = expr.as_method_call().unwrap();

            assert_eq(name, quote { bar });

            assert_eq(generics.len(), 1);
            assert(generics[0].is_field());

            assert_eq(arguments.len(), 2);
            assert_eq(arguments[0].as_integer().unwrap(), (3, false));
            assert_eq(arguments[1].as_integer().unwrap(), (4, false));
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
            assert(get_binary_op(quote { x < y }).is_less_than());
            assert(get_binary_op(quote { x <= y }).is_less_than_or_equal());
            assert(get_binary_op(quote { x > y }).is_greater_than());
            assert(get_binary_op(quote { x >= y }).is_greater_than_or_equal());
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
    fn test_expr_as_cast() {
        comptime
        {
            let expr = quote { 1 as Field }.as_expr().unwrap();
            let (expr, typ) = expr.as_cast().unwrap();
            assert_eq(expr.as_integer().unwrap(), (1, false));
            assert(typ.is_field());
        }
    }

    #[test]
    fn test_expr_as_comptime() {
        comptime
        {
            let expr = quote { comptime { 1; 4; 23 } }.as_expr().unwrap();
            let exprs = expr.as_comptime().unwrap();
            assert_eq(exprs.len(), 3);
        }
    }

    #[test]
    fn test_expr_as_comptime_as_statement() {
        comptime
        {
            let expr = quote { { comptime { 1; 4; 23 } } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            assert_eq(exprs.len(), 1);

            let exprs = exprs[0].as_comptime().unwrap();
            assert_eq(exprs.len(), 3);
        }
    }

    // This test can't only be around the comptime block since that will cause
    // `nargo fmt` to remove the comptime keyword.
    // docs:start:as_expr_example
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
    // docs:end:as_expr_example

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
    fn test_expr_as_member_access_with_an_lvalue() {
        comptime
        {
            let expr = quote { { foo.bar = 1; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            let (lhs, _rhs) = exprs[0].as_assign().unwrap();
            let (_, name) = lhs.as_member_access().unwrap();
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
    fn test_expr_as_unsafe() {
        comptime
        {
            let expr = quote { unsafe { 1; 4; 23 } }.as_expr().unwrap();
            let exprs = expr.as_unsafe().unwrap();
            assert_eq(exprs.len(), 3);
        }
    }

    #[test]
    fn test_expr_is_break() {
        comptime
        {
            let expr = quote { { break; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            assert(exprs[0].is_break());
        }
    }

    #[test]
    fn test_expr_is_continue() {
        comptime
        {
            let expr = quote { { continue; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            assert(exprs[0].is_continue());
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

fn main() {}
