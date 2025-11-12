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

use crate::tests::assert_no_errors;
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
