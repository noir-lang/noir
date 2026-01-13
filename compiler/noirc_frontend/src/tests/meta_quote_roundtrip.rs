//! Property-based tests for quote/unquote roundtrip preservation
//!
//! These tests verify the critical invariant that values can be quoted and unquoted
//! without loss of information. The roundtrip flow is:
//! 1. Value → into_tokens() via $value in quote { $value }
//! 2. Tokens stored in Quoted
//! 3. Tokens → parse → evaluate via unquote!(...)
//! 4. Assert resulting value equals original
//!
//! This tests the full metaprogramming pipeline that users rely on.

use crate::tests::{assert_no_errors, check_errors};
use proptest::prelude::*;

/// Helper to generate a roundtrip test program.
/// Tests that quoting then unquoting a value preserves its value.
fn make_roundtrip_test(type_annotation: &str, value_expr: String) -> String {
    format!(
        r#"
        fn main() {{
            comptime {{
                let original: {type_annotation} = {value_expr};
                let got: {type_annotation} = unquote!(quote {{ $original }});
                assert_eq(got, original);
            }}
        }}

        comptime fn unquote(code: Quoted) -> Quoted {{
            code
        }}
    "#
    )
}

// Primitive Types

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn roundtrip_field_values(n in any::<u128>()) {
        let src = make_roundtrip_test("Field", n.to_string());
        assert_no_errors(&src);
    }

    #[test]
    fn roundtrip_u8_values(n in any::<u8>()) {
        let src = make_roundtrip_test("u8", n.to_string());
        assert_no_errors(&src);
    }

    #[test]
    fn roundtrip_u16_values(n in any::<u16>()) {
        let src = make_roundtrip_test("u16", n.to_string());
        assert_no_errors(&src);
    }

    #[test]
    fn roundtrip_u32_values(n in any::<u32>()) {
        let src = make_roundtrip_test("u32", n.to_string());
        assert_no_errors(&src);
    }

    #[test]
    fn roundtrip_u64_values(n in any::<u64>()) {
        let src = make_roundtrip_test("u64", n.to_string());
        assert_no_errors(&src);
    }

    #[test]
    fn roundtrip_u128_values(n in any::<u128>()) {
        let src = make_roundtrip_test("u128", n.to_string());
        assert_no_errors(&src);
    }

    #[test]
    fn roundtrip_i8_values(n in any::<i8>()) {
        let src = make_roundtrip_test("i8", n.to_string());
        assert_no_errors(&src);
    }

    #[test]
    fn roundtrip_i16_values(n in any::<i16>()) {
        let src = make_roundtrip_test("i16", n.to_string());
        assert_no_errors(&src);
    }

    #[test]
    fn roundtrip_i32_values(n in any::<i32>()) {
        let src = make_roundtrip_test("i32", n.to_string());
        assert_no_errors(&src);
    }

    #[test]
    fn roundtrip_i64_values(n in any::<i64>()) {
        let src = make_roundtrip_test("i64", n.to_string());
        assert_no_errors(&src);
    }
}

#[test]
fn roundtrip_zero_field() {
    let src = make_roundtrip_test("Field", "0".to_string());
    assert_no_errors(&src);
}

#[test]
fn roundtrip_i64_min() {
    let src = make_roundtrip_test("i64", i64::MIN.to_string());
    assert_no_errors(&src);
}

/// Boolean values preserve correctly through quote/unquote
#[test]
fn roundtrip_false() {
    let src = make_roundtrip_test("bool", "false".to_string());
    assert_no_errors(&src);
}

#[test]
fn roundtrip_true() {
    let src = make_roundtrip_test("bool", "true".to_string());
    assert_no_errors(&src);
}

// Nested Quoting Tests

/// Test interpolating a quote which only contains literals declared within the inner quote
#[test]
fn nested_quote_basic() {
    let src = r#"
        fn main() {
            comptime {
                // Use a literal rather than splicing a value here
                let inner = quote { 3 };
                let nested = quote { $inner };
                let got = unquote!(nested);
                assert_eq(got, 3);
            }
        }

        comptime fn unquote(code: Quoted) -> Quoted {
            code
        }
    "#;
    assert_no_errors(src);
}

/// Tests interpolating a quote that itself contains value interpolation.
/// The value gets converted to tokens in the inner quote, then those tokens
/// are spliced into the outer quote, and finally parsed back to the original value.
#[test]
fn nested_quote_basic_with_type_annotations() {
    let src = r#"
        fn main() {
            comptime {
                let original: u8 = 5;
                // Splice a value into our initial quote
                let inner = quote { $original };
                let nested = quote { $inner };
                let got: u8 = unquote!(nested);
                assert_eq(got, original);
            }
        }

        comptime fn unquote(code: Quoted) -> Quoted {
            code
        }
    "#;
    assert_no_errors(src);
}

/// Type inference works correctly across quote boundaries
#[test]
fn nested_quote_basic_with_one_type_annotation() {
    let src = r#"
        fn main() {
            comptime {
                // Do not specify a type here
                let original = 5;
                // Splice a value into our initial quote
                let inner = quote { $original };
                let nested = quote { $inner };
                let got: u8 = unquote!(nested);
                // Note: This works because 'original' is inferred as u8.
                assert_eq(got, original);
            }
        }

        comptime fn unquote(code: Quoted) -> Quoted {
            code
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_quote_basic_no_type_annotation() {
    // Type inference is run before the comptime interpreter which determines
    // the types returned from quoted values. Thus, we can get some funky type mismatches
    // if types are not properly annotated.
    let src = r#"
        fn main() {
            comptime {
                // Do not specify a type here
                let original = 5;
                // Splice a value into our initial quote
                let inner = quote { $original };
                let nested = quote { $inner };
                let got: u8 = unquote!(nested);
                              ^^^^^^^^^^^^^^^^ Expected type u8, found type Field
                // `original` is inferred to be a Field as it has no type specified.
                // The literal `3` is inferred to be `u8` based off of the annotated
                // type on `got`. However, `got` has been found to be a `Field`. 
                // Thus, we get a type mismatch.
                assert_eq(got, 3);
                          ^^^^^^ No implementation for `Field` == `u8`
            }
        }

        comptime fn unquote(code: Quoted) -> Quoted {
            code
        }
    "#;
    check_errors(src);
}

/// Tests that an interpolated quote can be used in an expression
#[test]
fn nested_quote_with_add() {
    let src = r#"
        fn main() {
            comptime {
                let inner = quote { 3 };
                let nested = quote { $inner + 1 };
                let got = unquote!(nested);
                assert_eq(got, 4);
            }
        }

        comptime fn unquote(code: Quoted) -> Quoted {
            code
        }
    "#;
    assert_no_errors(src);
}

/// Tests that an interpolated quote can be used in an expression
#[test]
fn nested_quote_with_incorrect_add() {
    let src = r#"
        fn main() {
            comptime {
                let inner = quote { 3 };
                let nested = quote { $inner + 1 };
                let got = unquote!(nested);
                assert_eq(got, 5);
                          ^^^^^^ Assertion failed
            }
        }

        comptime fn unquote(code: Quoted) -> Quoted {
            code
        }
    "#;
    check_errors(src);
}

/// Multiple value interpolations at different nesting levels
#[test]
fn nested_quote_different_levels() {
    let src = r#"
        fn main() {
            comptime {
                let a = 10;
                let b = 20;
                let inner = quote { $a };
                let nested = quote { $inner + $b };
                let got = unquote!(nested);
                assert_eq(got, a + b);
            }
        }

        comptime fn unquote(code: Quoted) -> Quoted {
            code
        }
    "#;
    assert_no_errors(src);
}

/// Verify recursive token splicing through three levels of quote nesting.
/// Each level of nesting splices tokens from the previous level, and the final
/// unquote should recover the original value through all the layers.
#[test]
fn triple_nested_quote() {
    let src = r#"
        fn main() {
            comptime {
                let original = 1;
                let q1 = quote { $original };
                let q2 = quote { $q1 };
                let q3 = quote { $q2 };
                let got = unquote!(q3);
                assert_eq(got, original);
            }
        }

        comptime fn unquote(code: Quoted) -> Quoted {
            code
        }
    "#;
    assert_no_errors(src);
}
