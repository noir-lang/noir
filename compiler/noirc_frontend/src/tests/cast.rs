use crate::tests::{assert_no_errors, check_errors};

#[test]
fn cast_256_to_u8_size_checks() {
    let src = r#"
        fn main() {
            assert(256 as u8 == 0);
                   ^^^^^^^^^ Casting value of type Field to a smaller type (u8)
                   ~~~~~~~~~ casting untyped value (256) to a type with a maximum size (255) that's smaller than it
        }
    "#;
    check_errors(src);
}

#[test]
fn cast_negative_literal_to_integer_warns() {
    let src = r#"
        fn main() {
            let _ = -1 as i8;
                    ^^^^^^^^ Negative Field literal `-1` cast to `i8` evaluates to `0`
                    ~~~~~~~~ If this isn't desired, try `-1i8` instead or bind to a variable first to silence this warning
        }
    "#;
    check_errors(src);
}

#[test]
fn cast_suffixed_negative_literal_to_integer_does_not_warn() {
    let src = r#"
        fn main() {
            let _ = -1i8 as i8;
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn cast_signed_i8_to_field_must_error() {
    let src = r#"
        fn main() {
            assert(-1i8 as Field != 0);
                   ^^^^^^^^^^^^^ Only unsigned integer types may be casted to Field
        }
    "#;
    check_errors(src);
}

#[test]
fn cast_signed_i32_to_field_must_error() {
    let src = r#"
        fn main(x: i32) {
            assert(x as Field != 0);
                   ^^^^^^^^^^ Only unsigned integer types may be casted to Field
        }
    "#;
    check_errors(src);
}

#[test]
fn do_not_eagerly_error_on_cast_on_type_variable() {
    let src = r#"
    pub fn foo<T, U>(x: T, f: fn(T) -> U) -> U {
        f(x)
    }

    fn main() {
        let x: u8 = 1;
        let _: Field = foo(x, |x| x as Field);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn error_on_cast_over_type_variable() {
    let src = r#"
    pub fn foo<T, U>(f: fn(T) -> U, x: T, ) -> U {
        f(x)
    }

    fn main() {
        let x = "a";
        let _: Field = foo(|x| x as Field, x);
                                           ^ Expected type Field, found type str<1>
    }
    "#;
    check_errors(src);
}

// Regression test for https://github.com/noir-lang/noir-claude/issues/322
#[test]
fn cast_polymorphic_to_field_errors_when_later_constrained_to_signed() {
    let src = r#"
        fn main() {
            let x = 5;
            let _y = x as Field;
                     ^^^^^^^^^^ Only unsigned integer types may be casted to Field
            let _z: i8 = x;
        }
    "#;
    check_errors(src);
}

// Regression test for the bool-branch sibling of
// https://github.com/noir-lang/noir-claude/issues/322
#[test]
fn cast_polymorphic_to_bool_errors_when_later_constrained_to_numeric() {
    let src = r#"
        fn main() {
            let f = |x| {
                let _y = x as bool;
                         ^^^^^^^^^ Cannot cast `i32` as `bool`
                         ~~~~~~~~~ Compare with zero instead: ` != 0`
                let _z: i32 = x;
            };
            f(0);
        }
    "#;
    check_errors(src);
}

#[test]
fn cast_numeric_to_bool() {
    let src = "
    fn main() {
        let x: u64 = 1;
        let _ = x as bool;
                ^^^^^^^^^ Cannot cast `u64` as `bool`
                ~~~~~~~~~ Compare with zero instead: ` != 0`
    }
    ";
    check_errors(src);
}

#[test]
fn cast_field_and_integers_to_bool() {
    let src = "
    fn main() {
        let x = 1;
        let _ = x as bool;
                ^^^^^^^^^ Cannot cast `Field` as `bool`
                ~~~~~~~~~ Compare with zero instead: ` != 0`

        let x: i32 = 1;
        let _ = x as bool;
                ^^^^^^^^^ Cannot cast `i32` as `bool`
                ~~~~~~~~~ Compare with zero instead: ` != 0`

        let x: u64 = 1;
        let _ = x as bool;
                ^^^^^^^^^ Cannot cast `u64` as `bool`
                ~~~~~~~~~ Compare with zero instead: ` != 0`
    }
    ";
    check_errors(src);
}

#[test]
fn cast_numeric_to_bool_comptime() {
    let src = "
    fn main() {
        comptime {
            let x: u64 = 1;
            let _ = x as bool;
                    ^^^^^^^^^ Cannot cast `u64` as `bool`
                    ~~~~~~~~~ Compare with zero instead: ` != 0`
        }
    }
    ";
    check_errors(src);
}

#[test]
fn u1_type_is_removed() {
    let src = r#"
        fn main() {
            let _x: u1 = 0;
                    ^^ `u1` has been removed, use `bool` instead
        }
    "#;
    check_errors(src);
}
