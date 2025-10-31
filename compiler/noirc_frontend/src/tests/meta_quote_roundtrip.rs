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
//!
//! ## Known Limitation
//!
//! TODO(https://github.com/noir-lang/noir/issues/10326):
//! Integer types (u8, u16, u32, u64, u128, i8, i16, i32, i64) do NOT preserve their type
//! through quote/unquote. The `Value::into_tokens()` implementation in value.rs strips
//! type suffixes, converting all integers to `Token::Int(value, None)`.
//! When parsed, these become Field by default. Only Field, Bool, and Unit types
//! correctly roundtrip.

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

// Primitive types

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Field values preserve correctly through quote/unquote
    /// Using u128 for simplicity.
    #[test]
    fn roundtrip_field_values(n in any::<u128>()) {
        let src = make_roundtrip_test("Field", n.to_string());
        assert_no_errors(&src);
    }
}

// TODO(https://github.com/noir-lang/noir/issues/10326): Integer types are not preserved. Extend the property tests for primitive types once that issue is resolved.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    /// into_tokens() strips type suffixes, so integers become a `Field` when unquoted.
    #[test]
    #[ignore = "integers don't preserve type through quote/unquote"]
    fn roundtrip_u8_values_fails(n in any::<u8>()) {
        let src = make_roundtrip_test("u8", n.to_string());
        assert_no_errors(&src);
    }
}

#[test]
fn roundtrip_zero_field() {
    let src = make_roundtrip_test("Field", "0".to_string());
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
