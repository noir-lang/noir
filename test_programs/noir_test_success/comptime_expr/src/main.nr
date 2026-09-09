mod tests {
    use super::unresolved_type_to_string;
    use std::meta::op::BinaryOp;
    use std::meta::op::UnaryOp;

    #[test]
    fn test_expr_as_array() {
        comptime {
            let expr = quote { [1, 2, 4] }.as_expr().unwrap();
            let elems = expr.as_array().unwrap();
            assert_eq(elems.len(), 3);
            assert_eq(elems[0].as_integer().unwrap().value(), 1);
            assert_eq(elems[1].as_integer().unwrap().value(), 2);
            assert_eq(elems[2].as_integer().unwrap().value(), 4);
        }
    }

    #[test]
    fn test_expr_as_assert() {
        comptime {
            let expr = quote { assert(true) }.as_expr().unwrap();
            let assertion = expr.as_assert().unwrap();
            assert_eq(assertion.predicate().as_bool().unwrap(), true);
            assert(assertion.message().is_none());

            let expr = quote { assert(false, "oops") }.as_expr().unwrap();
            let assertion = expr.as_assert().unwrap();
            assert_eq(assertion.predicate().as_bool().unwrap(), false);
            assert(assertion.message().is_some());
        }
    }

    #[test]
    fn test_expr_as_assert_eq() {
        comptime {
            let expr = quote { assert_eq(true, false) }.as_expr().unwrap();
            let assertion = expr.as_assert_eq().unwrap();
            assert_eq(assertion.lhs().as_bool().unwrap(), true);
            assert_eq(assertion.rhs().as_bool().unwrap(), false);
            assert(assertion.message().is_none());

            let expr = quote { assert_eq(false, true, "oops") }.as_expr().unwrap();
            let assertion = expr.as_assert_eq().unwrap();
            assert_eq(assertion.lhs().as_bool().unwrap(), false);
            assert_eq(assertion.rhs().as_bool().unwrap(), true);
            assert(assertion.message().is_some());
        }
    }

    #[test]
    fn test_expr_as_assign() {
        comptime {
            let expr = quote { { a = 1; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            let assignment = exprs[0].as_assign().unwrap();
            assert_eq(assignment.lhs().quoted(), quote { a });
            assert_eq(assignment.rhs().as_integer().unwrap().value(), 1);
        }
    }

    #[test]
    fn test_expr_as_block() {
        comptime {
            let expr = quote {
                {
                    1;
                    4;
                    23
                }
            }
                .as_expr()
                .unwrap();
            let exprs = expr.as_block().unwrap();
            assert_eq(exprs.len(), 3);
            assert_eq(exprs[0].as_integer().unwrap().value(), 1);
            assert_eq(exprs[1].as_integer().unwrap().value(), 4);
            assert_eq(exprs[2].as_integer().unwrap().value(), 23);

            assert(exprs[0].has_semicolon());
            assert(exprs[1].has_semicolon());
            assert(!exprs[2].has_semicolon());
        }
    }

    #[test]
    fn test_expr_as_method_call() {
        comptime {
            let expr = quote { foo.bar::<Field>(3, 4) }.as_expr().unwrap();
            let method_call = expr.as_method_call().unwrap();

            assert_eq(method_call.receiver().quoted(), quote { foo });
            assert_eq(method_call.method_name(), quote { bar });

            let generics = method_call.generics();
            assert_eq(generics.len(), 1);
            assert_eq(unresolved_type_to_string!(generics[0]), "Field");

            let arguments = method_call.arguments();
            assert_eq(arguments.len(), 2);
            assert_eq(arguments[0].as_integer().unwrap().value(), 3);
            assert_eq(arguments[1].as_integer().unwrap().value(), 4);
        }
    }

    #[test]
    fn test_expr_as_integer() {
        comptime {
            let expr = quote { 1 }.as_expr().unwrap();
            let integer = expr.as_integer().unwrap();
            assert_eq(integer.value(), 1);

            let expr = quote { -2 }.as_expr().unwrap();
            let integer = expr.as_integer().unwrap();
            assert_eq(integer.value(), -2);

            // The type suffix shouldn't affect the value.
            let expr = quote { -2i8 }.as_expr().unwrap();
            let integer = expr.as_integer().unwrap();
            assert_eq(integer.value(), -2);
        }
    }

    #[test]
    fn test_expr_as_binary_op() {
        comptime {
            let expression = quote { x + y }.as_expr().unwrap().as_binary_op().unwrap();
            assert_eq(expression.lhs().quoted(), quote { x });
            assert_eq(expression.rhs().quoted(), quote { y });

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
            let cast = expr.as_cast().unwrap();
            assert_eq(cast.expression().as_integer().unwrap().value(), 1);
            assert_eq(unresolved_type_to_string!(cast.typ()), "Field");
        }
    }

    #[test]
    fn test_expr_as_comptime() {
        comptime {
            let expr = quote {
                comptime {
                    1;
                    4;
                    23
                }
            }
                .as_expr()
                .unwrap();
            let exprs = expr.as_comptime().unwrap();
            assert_eq(exprs.len(), 3);
        }
    }

    #[test]
    fn test_expr_as_comptime_as_statement() {
        comptime {
            let expr = quote {
                {
                    comptime {
                        1;
                        4;
                        23
                    }
                }
            }
                .as_expr()
                .unwrap();
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
            let constructor = expr.as_constructor().unwrap();
            assert_eq(unresolved_type_to_string!(constructor.typ()), "Foo");

            let fields = constructor.fields();
            assert_eq(fields.len(), 2);
            assert_eq(fields[0].name(), quote { a });
            assert_eq(fields[0].value().as_integer().unwrap().value(), 1);
            assert_eq(fields[1].name(), quote { b });
            assert_eq(fields[1].value().as_integer().unwrap().value(), 2);
        }
    }

    // This test can't only be around the comptime block since that will cause
    // `nargo fmt` to remove the comptime keyword.
    // docs:start:as_expr_example
    #[test]
    fn test_expr_as_function_call() {
        comptime {
            let expr = quote { foo(42) }.as_expr().unwrap();
            let function_call = expr.as_function_call().unwrap();
            assert_eq(function_call.function().quoted(), quote { foo });

            let args = function_call.arguments();
            assert_eq(args.len(), 1);
            assert_eq(args[0].as_integer().unwrap().value(), 42);
        }
    }
    // docs:end:as_expr_example

    #[test]
    fn test_expr_as_if() {
        comptime {
            let expr = quote { if 1 { 2 } }.as_expr().unwrap();
            let if_expression = expr.as_if().unwrap();
            assert_eq(if_expression.condition().as_integer().unwrap().value(), 1);
            assert_eq(
                if_expression.consequence().as_block().unwrap()[0].as_integer().unwrap().value(),
                2,
            );
            assert(if_expression.alternative().is_none());

            let expr = quote { if 1 { 2 } else { 3 } }.as_expr().unwrap();
            assert(expr.as_if().unwrap().alternative().is_some());
        }
    }

    #[test]
    fn test_expr_as_index() {
        comptime {
            let expr = quote { foo[bar] }.as_expr().unwrap();
            let index = expr.as_index().unwrap();
            assert_eq(index.collection().quoted(), quote { foo });
            assert_eq(index.index().quoted(), quote { bar });
        }
    }

    #[test]
    fn test_expr_as_member_access() {
        comptime {
            let expr = quote { foo.bar }.as_expr().unwrap();
            let member_access = expr.as_member_access().unwrap();
            assert_eq(member_access.object().quoted(), quote { foo });
            assert_eq(member_access.member(), quote { bar });
        }
    }

    #[test]
    fn test_expr_as_member_access_with_an_lvalue() {
        comptime {
            let expr = quote { { foo.bar = 1; } }.as_expr().unwrap();
            let exprs = expr.as_block().unwrap();
            let lhs = exprs[0].as_assign().unwrap().lhs();
            let member_access = lhs.as_member_access().unwrap();
            assert_eq(member_access.member(), quote { bar });
        }
    }

    #[test]
    fn test_expr_as_repeated_element_array() {
        comptime {
            let expr = quote { [1; 3] }.as_expr().unwrap();
            let array = expr.as_repeated_element_array().unwrap();
            assert_eq(array.element().as_integer().unwrap().value(), 1);
            assert_eq(array.length().as_integer().unwrap().value(), 3);
        }
    }

    #[test]
    fn test_expr_as_repeated_element_vector() {
        comptime {
            let expr = quote { @[1; 3] }.as_expr().unwrap();
            let vector = expr.as_repeated_element_vector().unwrap();
            assert_eq(vector.element().as_integer().unwrap().value(), 1);
            assert_eq(vector.length().as_integer().unwrap().value(), 3);
        }
    }

    #[test]
    fn test_expr_as_vector() {
        comptime {
            let expr = quote { @[1, 3, 5] }.as_expr().unwrap();
            let elems = expr.as_vector().unwrap();
            assert_eq(elems.len(), 3);
            assert_eq(elems[0].as_integer().unwrap().value(), 1);
            assert_eq(elems[1].as_integer().unwrap().value(), 3);
            assert_eq(elems[2].as_integer().unwrap().value(), 5);
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
    fn test_expr_as_unary_op() {
        comptime {
            let expression = quote { -x }.as_expr().unwrap().as_unary_op().unwrap();
            assert_eq(expression.rhs().quoted(), quote { x });

            assert(get_unary_op(quote { -x }).is_minus());
            assert(get_unary_op(quote { !x }).is_not());
            assert(get_unary_op(quote { &mut x }).is_mutable_reference());
            assert(get_unary_op(quote { *x }).is_dereference());
        }
    }

    #[test]
    fn test_expr_as_unsafe() {
        comptime {
            let expr = quote {
                // Safety: test
                unsafe {
                    1;
                    4;
                    23
                }
            }
                .as_expr()
                .unwrap();
            let exprs = expr.as_unsafe().unwrap();
            assert_eq(exprs.len(), 3);
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
            let lambda = expr.as_lambda().unwrap();
            let params = lambda.parameters();
            assert_eq(params.len(), 1);
            assert_eq(params[0].pattern().quoted(), quote { x });
            assert_eq(unresolved_type_to_string!(params[0].typ().unwrap()), "Field");
            assert_eq(unresolved_type_to_string!(lambda.return_type().unwrap()), "Field");
            assert_eq(lambda.body().as_block().unwrap()[0].as_integer().unwrap().value(), 1);

            let expr = quote { |x| { 1 } }.as_expr().unwrap();
            let lambda = expr.as_lambda().unwrap();
            let params = lambda.parameters();
            assert_eq(params.len(), 1);
            assert(params[0].typ().is_none());
            assert(lambda.return_type().is_none());
            assert_eq(lambda.body().as_block().unwrap()[0].as_integer().unwrap().value(), 1);
        }
    }

    #[test]
    fn test_expr_as_lambda_params_stored_type() {
        comptime {
            let expr = quote { |x| { 1 } }.as_expr().unwrap();
            let params = expr.as_lambda().unwrap().parameters();

            // The stored type of the parameter vector must match the type
            // advertised by `Expr::as_lambda`.
            let _: [std::meta::expr::LambdaParameter] = std::mem::checked_transmute(params);
        }
    }

    #[test]
    fn test_expr_as_let() {
        comptime {
            let expr = quote { let x: Field = 1; }.as_expr().unwrap();
            let let_expression = expr.as_let().unwrap();
            assert_eq(let_expression.pattern().quoted(), quote { x });
            assert_eq(unresolved_type_to_string!(let_expression.typ().unwrap()), "Field");
            assert_eq(let_expression.expression().as_integer().unwrap().value(), 1);
        }
    }

    #[test]
    fn test_expr_as_for_statement() {
        comptime {
            let expr = quote {
                for x in 2 {
                    3
                }
            }
                .as_expr()
                .unwrap();
            let for_expression = expr.as_for().unwrap();
            assert_eq(for_expression.identifier(), quote { x });
            assert_eq(for_expression.iterable().as_integer().unwrap().value(), 2);
            assert_eq(
                for_expression.body().as_block().unwrap()[0].as_integer().unwrap().value(),
                3,
            );
        }
    }

    #[test]
    fn test_expr_as_for_range_statement() {
        comptime {
            let expr = quote {
                for x in 2..3 {
                    4
                }
            }
                .as_expr()
                .unwrap();
            let for_range = expr.as_for_range().unwrap();
            assert_eq(for_range.identifier(), quote { x });
            assert_eq(for_range.start().as_integer().unwrap().value(), 2);
            assert_eq(for_range.end().as_integer().unwrap().value(), 3);
            assert(!for_range.is_inclusive());
            assert_eq(for_range.body().as_block().unwrap()[0].as_integer().unwrap().value(), 4);

            let expr = quote {
                for x in 2..=3 {
                    4
                }
            }
                .as_expr()
                .unwrap();
            let for_range = expr.as_for_range().unwrap();
            assert_eq(for_range.identifier(), quote { x });
            assert_eq(for_range.start().as_integer().unwrap().value(), 2);
            assert_eq(for_range.end().as_integer().unwrap().value(), 3);
            assert(for_range.is_inclusive());
            assert_eq(for_range.body().as_block().unwrap()[0].as_integer().unwrap().value(), 4);
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
        expr.as_unary_op().unwrap().operator()
    }

    comptime fn get_binary_op(quoted: Quoted) -> BinaryOp {
        let expr = quoted.as_expr().unwrap();
        expr.as_binary_op().unwrap().operator()
    }

    comptime fn times_two(expr: Expr) -> Option<Expr> {
        expr.as_integer().and_then(|value| {
            let value = value.value() * 2;
            quote { $value }.as_expr()
        })
    }
}

comptime fn unresolved_type_to_string(typ: UnresolvedType) -> Quoted {
    f"{typ}".as_quoted_str()
}

fn main() {}
