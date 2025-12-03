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
            // for the type error abbove.
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
            FLAG = true;
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
                bad();
                assert_eq(FLAG, true);
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
                assert_eq(FLAG, true);
            }
        }
        ";
    check_errors(src);
}
