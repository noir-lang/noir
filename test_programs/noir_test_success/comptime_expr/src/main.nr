mod tests {
    use std::meta::op::BinaryOp;
    use std::meta::op::UnaryOp;

    #[test]
    fn test_expr_as_array() {
        comptime {
            let expr = quote { [1, 2, 4] }.as_expr().unwrap();
            let elems = expr.as_array().unwrap();
            assert_eq(elems.len(), 3);
            assert_eq(elems[0].as_integer().unwrap(), (1, false));
            assert_eq(elems[1].as_integer().unwrap(), (2, false));
            assert_eq(elems[2].as_integer().unwrap(), (4, false));
        }
    }

    #[test]
    fn test_expr_modify_for_array() {
        comptime {
            let expr = quote { [1, 2, 4] }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let elems = expr.as_array().unwrap();
            assert_eq(elems.len(), 3);
            assert_eq(elems[0].as_integer().unwrap(), (2, false));
            assert_eq(elems[1].as_integer().unwrap(), (4, false));
            assert_eq(elems[2].as_integer().unwrap(), (8, false));
        }
    }

    #[test]
    fn test_expr_as_assert() {
        comptime {
            let expr = quote { assert(true) }.as_expr().unwrap();
            let (predicate, msg) = expr.as_assert().unwrap();
            assert_eq(predicate.as_bool().unwrap(), true);
            assert(msg.is_none());

            let expr = quote { assert(false, "oops") }.as_expr().unwrap();
            let (predicate, msg) = expr.as_assert().unwrap();
            assert_eq(predicate.as_bool().unwrap(), false);
            assert(msg.is_some());
        }
    }

    #[test]
    fn test_expr_modify_for_assert() {
        comptime {
            let expr = quote { assert(1) }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (predicate, msg) = expr.as_assert().unwrap();
            assert_eq(predicate.as_integer().unwrap(), (2, false));
            assert(msg.is_none());

            let expr = quote { assert(1, 2) }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (predicate, msg) = expr.as_assert().unwrap();
            assert_eq(predicate.as_integer().unwrap(), (2, false));
            assert_eq(msg.unwrap().as_integer().unwrap(), (4, false));
        }
    }

    #[test]
    fn test_expr_as_assert_eq() {
        comptime {
            let expr = quote { assert_eq(true, false) }.as_expr().unwrap();
            let (lhs, rhs, msg) = expr.as_assert_eq().unwrap();
            assert_eq(lhs.as_bool().unwrap(), true);
            assert_eq(rhs.as_bool().unwrap(), false);
            assert(msg.is_none());

            let expr = quote { assert_eq(false, true, "oops") }.as_expr().unwrap();
            let (lhs, rhs, msg) = expr.as_assert_eq().unwrap();
            assert_eq(lhs.as_bool().unwrap(), false);
            assert_eq(rhs.as_bool().unwrap(), true);
            assert(msg.is_some());
        }
    }

    #[test]
    fn test_expr_modify_for_assert_eq() {
        comptime {
            let expr = quote { assert_eq(1, 2) }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (lhs, rhs, msg) = expr.as_assert_eq().unwrap();
            assert_eq(lhs.as_integer().unwrap(), (2, false));
            assert_eq(rhs.as_integer().unwrap(), (4, false));
            assert(msg.is_none());

            let expr = quote { assert_eq(1, 2, 3) }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (lhs, rhs, msg) = expr.as_assert_eq().unwrap();
            assert_eq(lhs.as_integer().unwrap(), (2, false));
            assert_eq(rhs.as_integer().unwrap(), (4, false));
            assert_eq(msg.unwrap().as_integer().unwrap(), (6, false));
        }
    }

    #[test]
    fn test_expr_as_assign() {
        comptime {
            let expr = quote { { a = 1; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            let (_lhs, rhs) = exprs[0].as_assign().unwrap();
            assert_eq(rhs.as_integer().unwrap(), (1, false));
        }
    }

    #[test]
    fn test_expr_modify_for_assign() {
        comptime {
            let expr = quote { { a = 1; } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let exprs = expr.as_block().unwrap();
            let (_lhs, rhs) = exprs[0].as_assign().unwrap();
            assert_eq(rhs.as_integer().unwrap(), (2, false));
        }
    }

    #[test]
    fn test_expr_as_block() {
        comptime {
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
    fn test_expr_modify_for_block() {
        comptime {
            let expr = quote { { 1; 4; 23 } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let exprs = expr.as_block().unwrap();
            assert_eq(exprs.len(), 3);
            assert_eq(exprs[0].as_integer().unwrap(), (2, false));
            assert_eq(exprs[1].as_integer().unwrap(), (8, false));
            assert_eq(exprs[2].as_integer().unwrap(), (46, false));

            assert(exprs[0].has_semicolon());
            assert(exprs[1].has_semicolon());
            assert(!exprs[2].has_semicolon());
        }
    }

    #[test]
    fn test_expr_as_method_call() {
        comptime {
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
    fn test_expr_modify_for_method_call() {
        comptime {
            let expr = quote { foo.bar(3, 4) }.as_expr().unwrap();
            let expr = expr.modify(times_two);

            let (_object, name, generics, arguments) = expr.as_method_call().unwrap();

            assert_eq(name, quote { bar });

            assert_eq(generics.len(), 0);

            assert_eq(arguments.len(), 2);
            assert_eq(arguments[0].as_integer().unwrap(), (6, false));
            assert_eq(arguments[1].as_integer().unwrap(), (8, false));
        }
    }

    #[test]
    fn test_expr_as_integer() {
        comptime {
            let expr = quote { 1 }.as_expr().unwrap();
            assert_eq((1, false), expr.as_integer().unwrap());

            let expr = quote { -2 }.as_expr().unwrap();
            assert_eq((2, true), expr.as_integer().unwrap());
        }
    }

    #[test]
    fn test_expr_modify_for_integer() {
        comptime {
            let expr = quote { 1 }.as_expr().unwrap();
            let expr = expr.modify(times_two);

            assert_eq((2, false), expr.as_integer().unwrap());
        }
    }

    #[test]
    fn test_expr_as_binary_op() {
        comptime {
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
    fn test_expr_modify_for_binary_op() {
        comptime {
            let expr = quote { 3 + 4 }.as_expr().unwrap();
            let expr = expr.modify(times_two);

            let (lhs, op, rhs) = expr.as_binary_op().unwrap();
            assert_eq(lhs.as_integer().unwrap(), (6, false));
            assert(op.is_add());
            assert_eq(rhs.as_integer().unwrap(), (8, false));
        }
    }

    #[test]
    fn test_expr_as_bool() {
        comptime {
            let expr = quote { false }.as_expr().unwrap();
            assert(expr.as_bool().unwrap() == false);

            let expr = quote { true }.as_expr().unwrap();
            assert_eq(expr.as_bool().unwrap(), true);
        }
    }

    #[test]
    fn test_expr_as_cast() {
        comptime {
            let expr = quote { 1 as Field }.as_expr().unwrap();
            let (expr, typ) = expr.as_cast().unwrap();
            assert_eq(expr.as_integer().unwrap(), (1, false));
            assert(typ.is_field());
        }
    }

    #[test]
    fn test_expr_modify_for_cast() {
        comptime {
            let expr = quote { 1 as Field }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (expr, typ) = expr.as_cast().unwrap();
            assert_eq(expr.as_integer().unwrap(), (2, false));
            assert(typ.is_field());
        }
    }

    #[test]
    fn test_expr_as_comptime() {
        comptime {
            let expr = quote { comptime { 1; 4; 23 } }.as_expr().unwrap();
            let exprs = expr.as_comptime().unwrap();
            assert_eq(exprs.len(), 3);
        }
    }

    #[test]
    fn test_expr_modify_for_comptime() {
        comptime {
            let expr = quote { comptime { 1; 4; 23 } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let exprs = expr.as_comptime().unwrap();
            assert_eq(exprs.len(), 3);
            assert_eq(exprs[0].as_integer().unwrap(), (2, false));
        }
    }

    #[test]
    fn test_expr_as_comptime_as_statement() {
        comptime {
            let expr = quote { { comptime { 1; 4; 23 } } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            assert_eq(exprs.len(), 1);

            let exprs = exprs[0].as_comptime().unwrap();
            assert_eq(exprs.len(), 3);
        }
    }

    #[test]
    fn test_expr_as_constructor() {
        comptime {
            let expr = quote { Foo { a: 1, b: 2 } }.as_expr().unwrap();
            let (_typ, fields) = expr.as_constructor().unwrap();
            assert_eq(fields.len(), 2);
            assert_eq(fields[0].0, quote { a });
            assert_eq(fields[0].1.as_integer().unwrap(), (1, false));
            assert_eq(fields[1].0, quote { b });
            assert_eq(fields[1].1.as_integer().unwrap(), (2, false));
        }
    }

    #[test]
    fn test_expr_modify_for_constructor() {
        comptime {
            let expr = quote { foo::bar::Baz::<i32, i64> { a: 1, b: 2 } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (_typ, fields) = expr.as_constructor().unwrap();
            assert_eq(fields.len(), 2);
            assert_eq(fields[0].0, quote { a });
            assert_eq(fields[0].1.as_integer().unwrap(), (2, false));
            assert_eq(fields[1].0, quote { b });
            assert_eq(fields[1].1.as_integer().unwrap(), (4, false));
        }
    }

    // This test can't only be around the comptime block since that will cause
    // `nargo fmt` to remove the comptime keyword.
    // docs:start:as_expr_example
    #[test]
    fn test_expr_as_function_call() {
        comptime {
            let expr = quote { foo(42) }.as_expr().unwrap();
            let (_function, args) = expr.as_function_call().unwrap();
            assert_eq(args.len(), 1);
            assert_eq(args[0].as_integer().unwrap(), (42, false));
        }
    }
    // docs:end:as_expr_example

    #[test]
    fn test_expr_modify_for_function_call() {
        comptime {
            let expr = quote { foo(42) }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (_function, args) = expr.as_function_call().unwrap();
            assert_eq(args.len(), 1);
            assert_eq(args[0].as_integer().unwrap(), (84, false));
        }
    }

    #[test]
    fn test_expr_as_if() {
        comptime {
            let expr = quote { if 1 { 2 } }.as_expr().unwrap();
            let (_condition, _consequence, alternative) = expr.as_if().unwrap();
            assert(alternative.is_none());

            let expr = quote { if 1 { 2 } else { 3 } }.as_expr().unwrap();
            let (_condition, _consequence, alternative) = expr.as_if().unwrap();
            assert(alternative.is_some());
        }
    }

    #[test]
    fn test_expr_modify_for_if() {
        comptime {
            let expr = quote { if 1 { 2 } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (condition, consequence, alternative) = expr.as_if().unwrap();
            assert_eq(condition.as_integer().unwrap(), (2, false));
            let consequence = consequence.as_block().unwrap()[0].as_block().unwrap()[0];
            assert_eq(consequence.as_integer().unwrap(), (4, false));
            assert(alternative.is_none());

            let expr = quote { if 1 { 2 } else { 3 } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (condition, consequence, alternative) = expr.as_if().unwrap();
            assert_eq(condition.as_integer().unwrap(), (2, false));
            let consequence = consequence.as_block().unwrap()[0].as_block().unwrap()[0];
            assert_eq(consequence.as_integer().unwrap(), (4, false));
            let alternative = alternative.unwrap().as_block().unwrap()[0].as_block().unwrap()[0];
            assert_eq(alternative.as_integer().unwrap(), (6, false));
        }
    }

    #[test]
    fn test_expr_as_index() {
        comptime {
            let expr = quote { foo[bar] }.as_expr().unwrap();
            assert(expr.as_index().is_some());
        }
    }

    #[test]
    fn test_expr_modify_for_index() {
        comptime {
            let expr = quote { 1[2] }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (object, index) = expr.as_index().unwrap();
            assert_eq(object.as_integer().unwrap(), (2, false));
            assert_eq(index.as_integer().unwrap(), (4, false));
        }
    }

    #[test]
    fn test_expr_as_member_access() {
        comptime {
            let expr = quote { foo.bar }.as_expr().unwrap();
            let (_, name) = expr.as_member_access().unwrap();
            assert_eq(name, quote { bar });
        }
    }

    #[test]
    fn test_expr_modify_for_member_access() {
        comptime {
            let expr = quote { 1.bar }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (expr, name) = expr.as_member_access().unwrap();
            assert_eq(name, quote { bar });
            assert_eq(expr.as_integer().unwrap(), (2, false));
        }
    }

    #[test]
    fn test_expr_as_member_access_with_an_lvalue() {
        comptime {
            let expr = quote { { foo.bar = 1; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            let (lhs, _rhs) = exprs[0].as_assign().unwrap();
            let (_, name) = lhs.as_member_access().unwrap();
            assert_eq(name, quote { bar });
        }
    }

    #[test]
    fn test_expr_as_repeated_element_array() {
        comptime {
            let expr = quote { [1; 3] }.as_expr().unwrap();
            let (expr, length) = expr.as_repeated_element_array().unwrap();
            assert_eq(expr.as_integer().unwrap(), (1, false));
            assert_eq(length.as_integer().unwrap(), (3, false));
        }
    }

    #[test]
    fn test_expr_modify_for_repeated_element_array() {
        comptime {
            let expr = quote { [1; 3] }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (expr, length) = expr.as_repeated_element_array().unwrap();
            assert_eq(expr.as_integer().unwrap(), (2, false));
            assert_eq(length.as_integer().unwrap(), (6, false));
        }
    }

    #[test]
    fn test_expr_as_repeated_element_slice() {
        comptime {
            let expr = quote { &[1; 3] }.as_expr().unwrap();
            let (expr, length) = expr.as_repeated_element_slice().unwrap();
            assert_eq(expr.as_integer().unwrap(), (1, false));
            assert_eq(length.as_integer().unwrap(), (3, false));
        }
    }

    #[test]
    fn test_expr_modify_for_repeated_element_slice() {
        comptime {
            let expr = quote { &[1; 3] }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (expr, length) = expr.as_repeated_element_slice().unwrap();
            assert_eq(expr.as_integer().unwrap(), (2, false));
            assert_eq(length.as_integer().unwrap(), (6, false));
        }
    }

    #[test]
    fn test_expr_as_slice() {
        comptime {
            let expr = quote { &[1, 3, 5] }.as_expr().unwrap();
            let elems = expr.as_slice().unwrap();
            assert_eq(elems.len(), 3);
            assert_eq(elems[0].as_integer().unwrap(), (1, false));
            assert_eq(elems[1].as_integer().unwrap(), (3, false));
            assert_eq(elems[2].as_integer().unwrap(), (5, false));
        }
    }

    #[test]
    fn test_expr_modify_for_slice() {
        comptime {
            let expr = quote { &[1, 3, 5] }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let elems = expr.as_slice().unwrap();
            assert_eq(elems.len(), 3);
            assert_eq(elems[0].as_integer().unwrap(), (2, false));
            assert_eq(elems[1].as_integer().unwrap(), (6, false));
            assert_eq(elems[2].as_integer().unwrap(), (10, false));
        }
    }

    #[test]
    fn test_expr_as_tuple() {
        comptime {
            let expr = quote { (1, 2) }.as_expr().unwrap();
            let tuple_exprs = expr.as_tuple().unwrap();
            assert_eq(tuple_exprs.len(), 2);
        }
    }

    #[test]
    fn test_expr_modify_for_tuple() {
        comptime {
            let expr = quote { (1, 2) }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let tuple_exprs = expr.as_tuple().unwrap();
            assert_eq(tuple_exprs.len(), 2);
            assert_eq(tuple_exprs[0].as_integer().unwrap(), (2, false));
            assert_eq(tuple_exprs[1].as_integer().unwrap(), (4, false));
        }
    }

    #[test]
    fn test_expr_as_unary_op() {
        comptime {
            assert(get_unary_op(quote { -x }).is_minus());
            assert(get_unary_op(quote { !x }).is_not());
            assert(get_unary_op(quote { &mut x }).is_mutable_reference());
            assert(get_unary_op(quote { *x }).is_dereference());
        }
    }

    #[test]
    fn test_expr_modify_for_unary_op() {
        comptime {
            let expr = quote { -(1) }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (op, expr) = expr.as_unary_op().unwrap();
            assert(op.is_minus());
            assert_eq(expr.as_integer().unwrap(), (2, false));
        }
    }

    #[test]
    fn test_expr_as_unsafe() {
        comptime {
            let expr = quote {  
                /// Safety: test
                unsafe { 1; 4; 23 } 
                }
                .as_expr()
                .unwrap();
            let exprs = expr.as_unsafe().unwrap();
            assert_eq(exprs.len(), 3);
        }
    }

    #[test]
    fn test_expr_modify_for_unsafe() {
        comptime {
            let expr = quote { 
                /// Safety: test
                unsafe { 1; 4; 23 } 
            }
                .as_expr()
                .unwrap();
            let expr = expr.modify(times_two);
            let exprs = expr.as_unsafe().unwrap();
            assert_eq(exprs.len(), 3);
            assert_eq(exprs[0].as_integer().unwrap(), (2, false));
        }
    }

    #[test]
    fn test_expr_is_break() {
        comptime {
            let expr = quote { { break; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            assert(exprs[0].is_break());
        }
    }

    #[test]
    fn test_expr_is_continue() {
        comptime {
            let expr = quote { { continue; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            assert(exprs[0].is_continue());
        }
    }

    #[test]
    fn test_expr_as_lambda() {
        comptime {
            let expr = quote { |x: Field| -> Field { 1 } }.as_expr().unwrap();
            let (params, return_type, body) = expr.as_lambda().unwrap();
            assert_eq(params.len(), 1);
            assert(params[0].1.unwrap().is_field());
            assert(return_type.unwrap().is_field());
            assert_eq(body.as_block().unwrap()[0].as_integer().unwrap(), (1, false));

            let expr = quote { |x| { 1 } }.as_expr().unwrap();
            let (params, return_type, body) = expr.as_lambda().unwrap();
            assert_eq(params.len(), 1);
            assert(params[0].1.is_none());
            assert(return_type.is_none());
            assert_eq(body.as_block().unwrap()[0].as_integer().unwrap(), (1, false));
        }
    }

    #[test]
    fn test_expr_modify_lambda() {
        comptime {
            let expr = quote { |x: Field| -> Field { 1 } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (params, return_type, body) = expr.as_lambda().unwrap();
            assert_eq(params.len(), 1);
            assert(params[0].1.unwrap().is_field());
            assert(return_type.unwrap().is_field());
            assert_eq(
                body.as_block().unwrap()[0].as_block().unwrap()[0].as_integer().unwrap(),
                (2, false),
            );

            let expr = quote { |x| { 1 } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (params, return_type, body) = expr.as_lambda().unwrap();
            assert_eq(params.len(), 1);
            assert(params[0].1.is_none());
            assert(return_type.is_none());
            assert_eq(
                body.as_block().unwrap()[0].as_block().unwrap()[0].as_integer().unwrap(),
                (2, false),
            );
        }
    }

    #[test]
    fn test_expr_as_let() {
        comptime {
            let expr = quote { let x: Field = 1; }.as_expr().unwrap();
            let (_pattern, typ, expr) = expr.as_let().unwrap();
            assert(typ.unwrap().is_field());
            assert_eq(expr.as_integer().unwrap(), (1, false));
        }
    }

    #[test]
    fn test_expr_modify_for_let() {
        comptime {
            let expr = quote { let x : Field = 1; }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (_pattern, typ, expr) = expr.as_let().unwrap();
            assert(typ.unwrap().is_field());
            assert_eq(expr.as_integer().unwrap(), (2, false));
        }
    }

    #[test]
    fn test_expr_modify_for_let_without_type() {
        comptime {
            let expr = quote { let x = 1; }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (_pattern, typ, expr) = expr.as_let().unwrap();
            assert(typ.is_none());
            assert_eq(expr.as_integer().unwrap(), (2, false));
        }
    }

    #[test]
    fn test_expr_as_for_statement() {
        comptime {
            let expr = quote { for x in 2 { 3 } }.as_expr().unwrap();
            let (index, array, body) = expr.as_for().unwrap();
            assert_eq(index, quote { x });
            assert_eq(array.as_integer().unwrap(), (2, false));
            assert_eq(body.as_block().unwrap()[0].as_integer().unwrap(), (3, false));
        }
    }

    #[test]
    fn test_expr_modify_for_statement() {
        comptime {
            let expr = quote { for x in 2 { 3 } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (index, array, body) = expr.as_for().unwrap();
            assert_eq(index, quote { x });
            assert_eq(array.as_integer().unwrap(), (4, false));
            assert_eq(
                body.as_block().unwrap()[0].as_block().unwrap()[0].as_integer().unwrap(),
                (6, false),
            );
        }
    }

    #[test]
    fn test_expr_as_for_range_statement() {
        comptime {
            let expr = quote { for x in 2..3 { 4 } }.as_expr().unwrap();
            let (index, from, to, body) = expr.as_for_range().unwrap();
            assert_eq(index, quote { x });
            assert_eq(from.as_integer().unwrap(), (2, false));
            assert_eq(to.as_integer().unwrap(), (3, false));
            assert_eq(body.as_block().unwrap()[0].as_integer().unwrap(), (4, false));
        }
    }

    #[test]
    fn test_expr_modify_for_range_statement() {
        comptime {
            let expr = quote { for x in 2..3 { 4 } }.as_expr().unwrap();
            let expr = expr.modify(times_two);
            let (index, from, to, body) = expr.as_for_range().unwrap();
            assert_eq(index, quote { x });
            assert_eq(from.as_integer().unwrap(), (4, false));
            assert_eq(to.as_integer().unwrap(), (6, false));
            assert_eq(
                body.as_block().unwrap()[0].as_block().unwrap()[0].as_integer().unwrap(),
                (8, false),
            );
        }
    }

    #[test]
    fn test_automatically_unwraps_parenthesized_expression() {
        comptime {
            let expr = quote { ((if 1 { 2 })) }.as_expr().unwrap();
            assert(expr.as_if().is_some());
        }
    }

    #[test]
    fn test_resolve_to_function_definition() {
        comptime {
            let expr = quote { times_two }.as_expr().unwrap();
            let func = expr.resolve(Option::none()).as_function_definition().unwrap();
            assert_eq(func.name(), quote { times_two });
            assert_eq(func.parameters().len(), 1);
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

    comptime fn times_two(expr: Expr) -> Option<Expr> {
        expr.as_integer().and_then(|(value, _)| {
            let value = value * 2;
            quote { $value }.as_expr()
        })
    }
}

fn main() {}
