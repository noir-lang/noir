//! Tests for skipping the comptime interpreter when there are elaboration errors for
//! comptime blocks, functions, or let statements.

use crate::tests::check_errors;

#[test]
fn evaluate_separate_comptime_block_with_preceding_failure() {
    // Guarantee that we run a comptime block even if a preceding comptime
    // block contains errors.
    let src = "
    fn main() {
        comptime {
            let i: u8 = 1;
            let _ = -i;
                    ^^ Cannot apply unary operator `-` to type `u8`
        }

        comptime {
            let x: i8 = 5 + 10;
            assert_eq(x, 15);
        }
    }
    ";
    check_errors(src);
}

#[test]
fn do_not_evaluate_comptime_block_with_failure() {
    let src = "
    fn main() {
        comptime {
            let x: i8 = 5 + 20;
            // We would expect this to trigger an interpreter error if it were not
            // for the type error below.
            assert_eq(x, 15);
            let i: u8 = 1;
            let _ = -i;
                    ^^ Cannot apply unary operator `-` to type `u8`
        }
    }
    ";
    check_errors(src);
}

#[test]
fn do_not_evaluate_comptime_block_with_preceding_failure() {
    let src = "
    fn main() {
        comptime {
            let i: u8 = 1;
            let _ = -i;
                    ^^ Cannot apply unary operator `-` to type `u8`
            let x: i8 = 5 + 20;
            // We would expect this to trigger an interpreter error if it were not
            // for the type error above.
            assert_eq(x, 15);
        }
    }
    ";
    check_errors(src);
}

#[test]
fn failing_comptime_function_not_run() {
    let src = "
    comptime mut global FLAG: bool = false; 
    
    fn main() {
        comptime {
            bad(); 
            // We expect the `FLAG` to remain `false`
            assert_eq(FLAG, false);
            // This comptime block will not be run at all as `bad` has errors
            assert_eq(FLAG, true);
        }
    }

    comptime fn bad() {
        // Type error here
        let _: i32 = 10_u32;
                     ^^^^^^ Expected type i32, found type u32
        FLAG = true;
    }
    ";
    check_errors(src);
}

#[test]
fn comptime_execution_stops_at_first_elaboration_error() {
    // This test demonstrates the incremental execution model for comptime code,
    // similar to Zig's comptime semantics. Execution proceeds by statement
    // in order until an error is encountered, then stops completely.
    //
    // Execution flow:
    // 1. COUNTER is incremented to 1 in the comptime block
    // 2. foo() is called, which increments COUNTER to 2
    // 3. The assertion `assert_eq(COUNTER, 2)` in foo() passes
    // 4. The call to bar() happens, but execution stops immediately when bar()
    //    encounters its type error during elaboration
    // 5. We never execute any statements in bar() at comptime
    //
    // Evidence that bar() never executes:
    // - The assertion `assert_eq(COUNTER, 0)` at the start of bar() succeeds,
    //   proving COUNTER is still 0 inside bar() (because bar() never
    //   actually runs). If it had run, it would see COUNTER = 2.
    // - bar() has a type error that prevents it from executing at comptime
    // - After bar() fails, we don't continue execution back in foo() or main()
    //
    // This test verifies that once we hit an error (the type error in bar()),
    // execution stops completely and we don't continue in the calling function.
    let src = "
    comptime mut global COUNTER: Field = 0;

    fn main() {
        comptime {
            COUNTER += 1;  // Would make COUNTER = 1
            let _x = foo();
            assert_eq(COUNTER, 0);
        }
    }

    comptime fn foo() -> u32 {
        COUNTER += 1;  // Would make COUNTER = 2
        assert_eq(COUNTER, 2);
        let result = bar();
        assert_eq(COUNTER, 0);
        result
    }

    comptime fn bar() -> u32 {
        assert_eq(COUNTER, 0);
        let _: u32 = true;
                     ^^^^ Expected type u32, found type bool
        assert_eq(COUNTER, 0);
        3
    }
    ";
    check_errors(src);
}

#[test]
fn comptime_execution_stops_at_assertion_failure() {
    // Companion test to `comptime_execution_stops_at_first_error` that demonstrates
    // execution stops at a runtime assertion failure (not just elaboration/type errors).
    //
    // This test shows that the incremental execution model applies to runtime errors:
    // execution proceeds until hitting an assertion failure, then stops completely.
    //
    // Execution flow:
    // 1. COUNTER is incremented to 1 in the comptime block
    // 2. foo() is called, which increments COUNTER to 2
    // 3. The assertion `assert_eq(COUNTER, 2)` in foo() passes
    // 4. bar() is called and begins execution
    // 5. The assertion `assert_eq(COUNTER, 99)` in bar() fails (COUNTER is 2)
    // 6. Execution stops - the second assertion `assert_eq(COUNTER, 0)` never runs
    // 7. Execution doesn't continue back in foo() (the assertion after bar() call
    //    never runs) or in main()
    let src = "
    comptime mut global COUNTER: Field = 0;

    fn main() {
        comptime {
            COUNTER += 1;  // Makes COUNTER = 1
            let _x = foo();
            assert_eq(COUNTER, 0);
        }
    }

    comptime fn foo() -> u32 {
        COUNTER += 1;  // Makes COUNTER = 2
        assert_eq(COUNTER, 2);
        let result = bar();
        assert_eq(COUNTER, 0);
        result
    }

    comptime fn bar() -> u32 {
        assert_eq(COUNTER, 99);
                  ^^^^^^^^^^^ Assertion failed
        assert_eq(COUNTER, 0);
        3
    }
    ";
    check_errors(src);
}

#[test]
fn function_with_error_called_from_comptime_global() {
    let src = "
        comptime fn bad() -> Field {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
            42
        }

        comptime global X: Field = bad();

        fn main() {
            let _ = X;
        }
        ";
    check_errors(src);
}

#[test]
fn function_with_error_called_from_comptime_function() {
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn bad() {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
            FLAG = true;
        }

        comptime fn caller() {
            bad();
            assert_eq(FLAG, false);
            // This comptime block will not be run at all
            assert_eq(FLAG, true);
        }

        fn main() {
            comptime {
                caller();
            }
        }
        ";
    check_errors(src);
}

#[test]
fn nested_function_calls_with_inner_error() {
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn inner() {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
            FLAG = true;
        }

        comptime fn outer() {
            inner();
        }

        fn main() {
            comptime {
                outer();
                assert_eq(FLAG, false);
                // This comptime block will not be run at all
                assert_eq(FLAG, true);
            }
        }
        ";
    check_errors(src);
}

#[test]
fn nested_function_calls_with_inner_post_call_mutation() {
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn inner() {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
        }

        comptime fn outer() {
            inner();
            FLAG = true;
        }

        fn main() {
            comptime {
                outer();
                assert_eq(FLAG, false);
                // This comptime block will not be run at all
                assert_eq(FLAG, true);
            }
        }
        ";
    check_errors(src);
}

#[test]
fn nested_function_calls_with_inner_error_pre_call_mutation() {
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn inner() {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
        }

        comptime fn outer() {
            FLAG = true;
            inner();
        }

        fn main() {
            comptime {
                outer();
                assert_eq(FLAG, false);
                // This comptime block will not be run at all
                assert_eq(FLAG, true);
            }
        }
        ";
    check_errors(src);
}

#[test]
fn nested_function_calls_with_inner_error_pre_call_mutation_decl_order() {
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn outer() {
            FLAG = true;
            inner();
        }

        fn main() {
            comptime {
                outer();
                assert_eq(FLAG, false);
                // This comptime block will not be run at all
                assert_eq(FLAG, true);
            }
        }

        comptime fn inner() {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
        }
        ";
    check_errors(src);
}

#[test]
fn function_with_error_in_comptime_variable() {
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn bad() -> Field {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
            FLAG = true;
            42
        }

        fn main() {
            comptime {
                let _x = bad();
                assert_eq(FLAG, false);
                // This comptime block will not be run at all
                assert_eq(FLAG, true);
            }
        }
        ";
    check_errors(src);
}

#[test]
fn function_with_error_in_local_comptime_variable() {
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn bad() -> Field {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
            // This will not be run at all
            FLAG = true;
            assert_eq(FLAG, false); 
            42
        }

        fn main() {
            comptime let _x = bad();
            assert_eq(FLAG, false); 
        }
        ";
    check_errors(src);
}

#[test]
fn multiple_functions_only_bad_one_skipped() {
    let src = "
        comptime mut global FLAG: bool = false;
        comptime mut global FLAG2: bool = false;

        comptime fn good() {
            FLAG = true;
        }

        comptime fn bad() {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
            FLAG2 = true;
        }

        fn main() {
            comptime {
                good();
                assert_eq(FLAG, true);
                bad();
                // This code is not run thus we do not see an assertion error
                assert_eq(FLAG, false);
                assert_eq(FLAG2, false);
            }
        }
        ";
    check_errors(src);
}

#[test]
fn function_with_multiple_errors_not_run() {
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn bad() {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
            let _: u8 = 300_u32;
                        ^^^^^^^ Expected type u8, found type u32
            FLAG = true;
        }

        fn main() {
            comptime {
                bad();
                assert_eq(FLAG, false);
            }
        }
        ";
    check_errors(src);
}

#[test]
fn attribute_function_with_error_not_run() {
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn my_attribute(_f: FunctionDefinition) -> Quoted {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
            FLAG = true;
            quote {}
        }

        #[my_attribute]
        fn some_function() { }

        fn main() {
            comptime {
                some_function();
                assert_eq(FLAG, false);
                assert_eq(FLAG, true);

            }
        }
        ";
    check_errors(src);
}

#[test]
fn attribute_with_error_prevents_function_execution() {
    // The annotated function should not be called when its attribute failed
    let src = "
        comptime mut global FLAG: bool = false;

        comptime fn my_attribute(_f: FunctionDefinition) -> Quoted {
            let _: i32 = 10_u32;
                         ^^^^^^ Expected type i32, found type u32
            FLAG = true;
            quote {}
        }

        #[my_attribute]
        comptime fn some_function() {
            FLAG = true;
        }

        fn main() {
            comptime {
                some_function();
                assert_eq(FLAG, false);
                assert_eq(FLAG, true);
            }
        }
        ";
    check_errors(src);
}

#[test]
fn regression_10686_0() {
    let src = "
    fn main() {
        comptime {
            let _ = i32 {};
                    ^^^ expected type got primitive type
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10686_1() {
    let src = "
    trait MyTrait {
        let N: u32;
    }
    struct Foo {}
    impl MyTrait for Foo {
        let N: u32 = 5;
    }
    fn main() {
        comptime {
            let x = 5 + <Foo as MyTrait>::M;
                                          ^ Trait `MyTrait` has no method named `M`
            assert_eq(x, 10);
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10686_2() {
    let src = "
    fn main() {
        comptime {
            let expr = quote { 1 + 1 };
            let _ = expr.foo();
                    ^^^^^^^^^^ No method named 'foo' found for type 'Quoted'
        }
    }
    ";
    check_errors(src);
}

// Regression for issue #10807 (https://github.com/noir-lang/noir/issues/10807)
#[test]
fn comptime_for_loop_with_end_value_out_of_range() {
    let src = "
    fn main() {
        comptime {
            let start: u32 = 1;
            // u128::MAX + 1
            let end: Field = 340282366920938463463374607431768211456;
            for i in start..end {
                            ^^^ Expected type u32, found type Field
                let _ = i;
            }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10807_1() {
    // Out of range end value with no type annotation
    let src = "
    fn main() {
        comptime {
            let start: u32 = 1;
            let end = 340282366920938463463374607431768211456;
                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ The value `340282366920938463463374607431768211456` cannot fit into `u32` which has range `0..=4294967295`
            for i in start..end {
                let _ = i;
            }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10861_0() {
    let src = "
    fn main() {
        comptime {
            // u32::MAX + 1
            let x: Field = 4294967296;
            let array = [1, 2, 3];
            assert_eq(array[x], 1);
                            ^ Indexing arrays and slices must be done with `u32`, not `Field`
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10861_1() {
    let src = "
    fn main() {
        comptime {
            // u64::MAX + 1
            let x: Field = 18446744073709551616;
            let array = [1, 2, 3];
            assert_eq(array[x], 1);
                            ^ Indexing arrays and slices must be done with `u32`, not `Field`
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10863() {
    let src = "
    fn main() {
        comptime {
            let x: i8 = -1;
            let array = [1, 2, 3];
            assert_eq(array[x], 1);
                            ^ Indexing arrays and slices must be done with `u32`, not `i8`
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10830() {
    let src = "
    struct Foo { }
    fn main() {
        comptime {
            let start: Foo = 1;
                             ^ Expected type Foo, found type Field
            let end: u32 = 5;
            for i in start..end {
                            ^^^ Expected type Foo, found type u32
                     ^^^^^ The type Foo cannot be used in a for loop
                let _ = i;
            }
        }
    }
    ";
    check_errors(src);
}

// Regression for issue #10819 (https://github.com/noir-lang/noir/issues/10819)
#[test]
fn comptime_trait_default_method_using_missing_associated_constant() {
    let src = "
    trait MyTrait {
        let N: u32;

        fn foo() {
            let _ = Self::N;
                          ^ Could not resolve 'N' in path
        }
    }
    struct Foo {}
    impl MyTrait for Foo { 
         ^^^^^^^ `MyTrait` is missing the associated type `N`
        // Leave this out to trigger errors
        // let N: u32 = 10;
    }
    fn main() {
        comptime {
            let _ = <Foo as MyTrait>::foo();
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regressoin_10829_0() {
    let src = "
    fn main() {
        comptime {
            let start: u32 = 1;
            let end: u32 = 2i8;
                           ^^^ Expected type u32, found type i8
            for i in start..end {
                let _ = i;
            }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regressoin_10829() {
    let src = "
    fn main() {
        comptime {
            let start: u32 = 1i8;
                             ^^^ Expected type u32, found type i8
            let end: u32 = 2;            
            for i in start..end {
                let _ = i;
            }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10688_0() {
    let src = "
    fn main() {
        comptime {
            for _ in 1_i8..2_u8 {}
                           ^^^^ Expected type i8, found type u8
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10688_1() {
    let src = "
    fn main() {
        comptime {
            for _ in 1_u8..2_i8 {}
                           ^^^^ Expected type u8, found type i8
        }
    }
    ";
    check_errors(src);
}

// Regression for issue #10831 (https://github.com/noir-lang/noir/issues/10831)
#[test]
fn oob_tuple_access() {
    let src = "
    fn main() {
        comptime {
            let mut x = (1, 2);
            // Check whether there is data corruption with the wrong index
            x.3 = 999;
              ^ Index 3 is out of bounds for this tuple (Field, Field) of length 2
            assert_eq(x.0, 999);
            assert_eq(x.0, 1);
            assert_eq(x.1, 999);
            assert_eq(x.1, 1);
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10832() {
    let src = "
    struct Foo {
        x: Field,
        y: Field,
    }
    fn main() {
        comptime {
            let mut f = Foo { x: 1, y: 2 };
            let Foo { x, y, undefined } = f;
                            ^^^^^^^^^ no such field undefined defined in struct Foo
            let _ = x;
            let _ = y;
            assert_eq(undefined, 999);
            assert_eq(undefined, 0);
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10855() {
    let src = "
    struct Foo {
        x: Field,
        y: Field,
    }
    fn main() {
        comptime {
            let mut f: Foo = Foo { x: 1, y: 2 };
            assert_eq(f.undefined, 999);
                        ^^^^^^^^^ Type Foo has no member named undefined
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_10865() {
    let src = "
    struct Foo {
        x: Field,
        y: Field,
    }
    fn main() {
        comptime {
            let mut f = Foo { x: 1, y: 2, undefined: 10 };
                                          ^^^^^^^^^ no such field undefined defined in struct Foo
            f.undefined = 999;
              ^^^^^^^^^ Type Foo has no member named undefined
            assert_eq(f.undefined, 999);
                        ^^^^^^^^^ Type Foo has no member named undefined
            assert_eq(f.undefined, 0);
                        ^^^^^^^^^ Type Foo has no member named undefined
        }
    }
    ";
    check_errors(src);
}
