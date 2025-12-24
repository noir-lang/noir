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

// TODO(https://github.com/noir-lang/noir/issues/6247):
// add negative integer literal checks
#[test]
fn cast_negative_one_to_u8_size_checks() {
    let src = r#"
        fn main() {
            assert((-1) as u8 != 0);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn cast_signed_i8_to_field_must_error() {
    let src = r#"
        fn main() {
            assert((-1 as i8) as Field != 0);
                   ^^^^^^^^^^^^^^^^^^^ Only unsigned integer types may be casted to Field
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

#[test]
fn cast_numeric_to_bool() {
    let src = "
    fn main() {
        let x: u64 = 1;
        let _ = x as bool;
                ^^^^^^^^^ Cannot cast `u64` as `bool`
                ~~~~~~~~~ compare with zero instead: ` != 0`
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
                    ~~~~~~~~~ compare with zero instead: ` != 0`
        }
    }
    ";
    check_errors(src);
}
