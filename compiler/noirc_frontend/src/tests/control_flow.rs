use crate::tests::{assert_no_errors, check_errors};

#[test]
fn resolve_for_expr() {
    let src = r#"
        fn main(x : u64) {
            for i in 1..20 {
                let _z = x + i;
            };
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_for_expr_incl() {
    let src = r#"
        fn main(x : u64) {
            for i in 1..=20 {
                let _z = x + i;
            };
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn for_loop_empty_range() {
    let src = r#"
    fn main() {
        let mut x = 0;
        for _i in 0..0 {
            x = 1;
        }
        assert(x == 0);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn for_loop_backwards_range() {
    let src = r#"
    fn main() {
        let mut x = 0;
        for _i in 10..5 {
            x = 1;
        }
        assert(x == 0);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn for_loop_single_elem_inclusive_max_value() {
    let src = r#"
    fn main() {
        let mut count = 0;
        for i in 4294967295..=4294967295 {
            count += 1;
            let _x: u32 = i;
        }
        assert(count == 1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn for_loop_mutate_induction_var() {
    let src = r#"
    fn main() {
        for i in 0..10 {
            i = 5;
            ^ Variable `i` must be mutable to be assigned to
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn break_and_continue_outside_loop() {
    let src = r#"
        pub unconstrained fn foo() {
            continue;
            ^^^^^^^^^ continue is only allowed within loops
        }
        pub unconstrained fn bar() {
            break;
            ^^^^^^ break is only allowed within loops
        }
    "#;
    check_errors(src);
}

#[test]
fn wrong_type_in_for_range() {
    let src = r#"
    pub fn foo() {
        for _ in true..false { 
                 ^^^^ The type bool cannot be used in a for loop
                 
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_if_without_else_type_mismatch() {
    let src = r#"
    fn main() {
        if true {
            1
            ^ Expected type Field, found type ()
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn if_else_type_mismatch() {
    let src = r#"
    fn main() {
        let _x = if true {
            let _ = 1;
        } else {
            2
            ^ Expected type (), found type Field
        };
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_empty_loop_no_break() {
    let src = r#"
    fn main() {
        // Safety: test
        unsafe {
            foo()
        }
    }

    unconstrained fn foo() {
        loop {}
        ^^^^ `loop` must have at least one `break` in it
        ~~~~ Infinite loops are disallowed
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_loop_without_break() {
    let src = r#"
    fn main() {
        // Safety: test
        unsafe {
            foo()
        }
    }

    unconstrained fn foo() {
        let mut x = 1;
        loop {
        ^^^^ `loop` must have at least one `break` in it
        ~~~~ Infinite loops are disallowed
            x += 1;
            bar(x);
        }
    }

    fn bar(_: Field) {}
    "#;
    check_errors(src);
}

#[test]
fn errors_on_loop_without_break_with_nested_loop() {
    let src = r#"
    fn main() {
        // Safety: test
        unsafe {
            foo()
        }
    }

    unconstrained fn foo() {
        let mut x = 1;
        loop {
        ^^^^ `loop` must have at least one `break` in it
        ~~~~ Infinite loops are disallowed
            x += 1;
            bar(x);
            loop {
                x += 2;
                break;
            }
        }
    }

    fn bar(_: Field) {}
    "#;
    check_errors(src);
}

#[test]
fn break_in_nested_and_outer_loops() {
    let src = r#"
    unconstrained fn main() {
        let mut x = 1;
        loop {
            x += 1;
            loop {
                x += 2;
                break; // Breaks from nested loop only
            }
            if x > 2 {
                break; // Breaks from outer loop
            }
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn continue_in_loop() {
    let src = r#"
    unconstrained fn main() {
        let mut x = 0;
        loop {
            x += 1;
            if x < 5 {
                continue;
            }
            break;
        }

        for i in 0..10 {
            if i == 5 {
                continue;
            }
        }

        while x > 0 {
            x -= 1;
            if x == 3 {
                continue;
            }
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_for_body_type_is_not_unit() {
    let src = r#"
    fn main() {
        for _ in 0..1 {
            1
            ^ Expected type (), found type Field
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_loop_body_type_is_not_unit() {
    let src = r#"
    unconstrained fn main() {
        loop {
            if false { break; }

            1
            ^ Expected type (), found type Field
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_while_body_type_is_not_unit() {
    let src = r#"
    unconstrained fn main() {
        while 1 == 1 {
            1
            ^ Expected type (), found type Field
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn overflowing_int_in_for_loop() {
    let src = r#"
    fn main() {
        for _ in -2..-1 {}
                 ^^ The value `-2` cannot fit into `u32` which has range `0..=4294967295`
                     ^^ The value `-1` cannot fit into `u32` which has range `0..=4294967295`
    }
    "#;
    check_errors(src);
}

#[test]
fn break_type_mismatch() {
    let src = r#"
    unconstrained fn main() {
        loop {
            if true {
                break;
            } else {
                5
                ^ Expected type (), found type Field
            };
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn continue_type_mismatch() {
    let src = r#"
    unconstrained fn main() {
        for _ in 0..1 {
            if true {
                continue;
            } else {
                5
                ^ Expected type (), found type Field
            }
        }
    }
    "#;
    check_errors(src);
}
