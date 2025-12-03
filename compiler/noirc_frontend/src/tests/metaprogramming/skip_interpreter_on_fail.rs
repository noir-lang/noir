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
fn regression_10686() {
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
