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
