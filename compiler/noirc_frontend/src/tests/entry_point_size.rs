//! Tests for the flattened-size limit enforced on entry point parameters during
//! monomorphization. Inputs whose flattened field count exceeds the limit are rejected
//! here so that later stages (data-bus construction, Brillig array allocation) don't have
//! to cope with sizes that approach `u32::MAX`.

use crate::test_utils::get_monomorphized;
use crate::tests::check_monomorphization_error;

#[test]
fn rejects_large_array_input_to_main() {
    let src = r#"
    fn main(_arr: [Field; 4294967295]) {}
            ^^^^ An input parameter has 4294967295 elements which exceeds the limit of 16777216
    "#;
    check_monomorphization_error(src);
}

#[test]
fn rejects_large_array_input_to_unconstrained_main() {
    let src = r#"
    unconstrained fn main(_arr: [Field; 4294967295]) {}
                          ^^^^ An input parameter has 4294967295 elements which exceeds the limit of 16777216
    "#;
    check_monomorphization_error(src);
}

/// A nested array whose element product overflows `u32` must produce a clean error rather
/// than panicking in the `u32`-based `Type::entry_point_field_count`.
#[test]
fn rejects_nested_array_input_overflowing_u32() {
    // 65536 * 65536 == 2^32 == 4294967296, which overflows u32.
    let src = r#"
    fn main(_arr: [[Field; 65536]; 65536]) {}
            ^^^^ An input parameter has 4294967296 elements which exceeds the limit of 16777216
    "#;
    check_monomorphization_error(src);
}

/// The limit applies to the whole flattened parameter, so an aggregate that individually
/// stays small but sums past the limit is still rejected.
#[test]
fn rejects_tuple_input_containing_large_array() {
    // 4294967295 elements for the array + 1 for the trailing Field.
    let src = r#"
    fn main(_input: ([Field; 4294967295], Field)) {}
            ^^^^^^ An input parameter has 4294967296 elements which exceeds the limit of 16777216
    "#;
    check_monomorphization_error(src);
}

#[test]
fn accepts_array_input_at_the_limit() {
    let src = r#"
    fn main(_arr: [Field; 16777216]) {}
    "#;
    assert!(get_monomorphized(src).is_ok(), "an input of exactly the limit should be accepted");
}

#[test]
fn rejects_array_input_just_over_the_limit() {
    let src = r#"
    fn main(_arr: [Field; 16777217]) {}
            ^^^^ An input parameter has 16777217 elements which exceeds the limit of 16777216
    "#;
    check_monomorphization_error(src);
}
