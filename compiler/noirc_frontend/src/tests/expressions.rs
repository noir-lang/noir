use crate::tests::{assert_no_errors, check_errors};

#[test]
fn resolve_unused_var() {
    let src = r#"
        fn main(x : Field) {
            let y = x + x;
                ^ unused variable y
                ~ unused variable
            assert(x == x);
        }
    "#;
    check_errors(src);
}

#[test]
fn resolve_unresolved_var() {
    let src = r#"
        fn main(x : Field) {
            let y = x + x;
            assert(y == z);
                        ^ cannot find `z` in this scope
                        ~ not found in this scope
        }
    "#;
    check_errors(src);
}

#[test]
fn unresolved_path() {
    let src = "
        fn main(x : Field) {
            let _z = some::path::to::a::func(x);
                     ^^^^ Could not resolve 'some' in path
        }
    ";
    check_errors(src);
}

#[test]
fn resolve_literal_expr() {
    let src = r#"
        fn main(x : Field) {
            let y = 5;
            assert(y == x);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_fmt_strings() {
    let src = r#"
        fn main() {
            let string = f"this is i: {i}";
                                       ^ cannot find `i` in this scope
                                       ~ not found in this scope
            println(string);
            ^^^^^^^^^^^^^^^ Unused expression result of type fmtstr<14, ()>

            let new_val = 10;
            println(f"random_string{new_val}{new_val}");
            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Unused expression result of type fmtstr<31, (Field, Field)>
        }
        fn println<T>(x : T) -> T {
            x
        }
    "#;
    check_errors(src);
}

#[test]
fn multiple_resolution_errors() {
    let src = r#"
        fn main(x : Field) {
           let y = foo::bar(x);
                   ^^^ Could not resolve 'foo' in path
           let z = y + a;
                       ^ cannot find `a` in this scope
                       ~ not found in this scope
               ^ unused variable z
               ~ unused variable
                       
        }
    "#;
    check_errors(src);
}

#[test]
fn bit_not_on_untyped_integer() {
    let src = r#"
    fn main() {
        let _: u32 = 3 & !1;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_prefix_expr() {
    let src = r#"
        fn main(x : Field) {
            let _y = -x;
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn cannot_use_prefix_minus_on_u32() {
    let src = r#"
    fn main() {
        let x: u32 = 1;
        let _ = -x;
                ^^ Cannot apply unary operator `-` to type `u32`
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_assign_to_module() {
    let src = r#"
    mod foo {}

    fn main() {
        foo = 1;
        ^^^ expected value got module
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_assign_to_nested_struct() {
    let src = r#"
    mod foo {
        pub struct bar {}
    }

    fn main() {
        foo::bar = 1;
        ^^^^^^^^ expected value got type
    }
    "#;
    check_errors(src);
}

#[test]
fn disallows_underscore_on_right_hand_side() {
    let src = r#"
        fn main() {
            let _ = 1;
            let _x = _;
                     ^ in expressions, `_` can only be used on the left-hand side of an assignment
                     ~ `_` not allowed here
        }
    "#;
    check_errors(src);
}

#[test]
fn does_not_error_on_return_values_after_block_expression() {
    // Regression test for https://github.com/noir-lang/noir/issues/4372
    let src = r#"
    fn case1() -> [Field] {
        if true {
        }
        &[1]
    }

    fn case2() -> [u8] {
        let mut var: u8 = 1;
        {
            var += 1;
        }
        &[var]
    }

    fn main() {
        let _ = case1();
        let _ = case2();
    }
    "#;
    assert_no_errors(src);
}
