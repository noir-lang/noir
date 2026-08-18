//! Tests for the flattened-size limit enforced on entry point parameters during
//! monomorphization. Inputs whose flattened field count exceeds the limit are rejected
//! here so that later stages (data-bus construction, Brillig array allocation) don't have
//! to cope with sizes that approach `u32::MAX`.

use crate::test_utils::{get_monomorphized, get_monomorphized_with_stdlib, stdlib_src};
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

/// The same limit applies to the circuit's output (return value), independent of whether the
/// program is ACIR or Brillig.
#[test]
fn rejects_large_array_return_from_main() {
    let src = r#"
    fn main() -> pub [Field; 4294967295] {
                     ^^^^^^^^^^^^^^^^^^^ The return value has 4294967295 elements which exceeds the limit of 16777216
        [0; 4294967295]
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn rejects_large_array_return_from_unconstrained_main() {
    let src = r#"
    unconstrained fn main() -> pub [Field; 4294967295] {
                                   ^^^^^^^^^^^^^^^^^^^ The return value has 4294967295 elements which exceeds the limit of 16777216
        [0; 4294967295]
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn accepts_array_return_at_the_limit() {
    let src = r#"
    unconstrained fn main() -> pub [Field; 16777216] {
        [0; 16777216]
    }
    "#;
    assert!(get_monomorphized(src).is_ok(), "a return of exactly the limit should be accepted");
}

/// The check only fires for the entry points that monomorphization marks as such: `main` and
/// fold functions. A generic `unconstrained` function called from ACIR becomes a Brillig entry
/// point per instantiation (here `mk_array::<10>` and `mk_array::<4294967295>`), with `N` only
/// known at monomorphization — yet it is a plain `brillig(inline)` function, not flagged as an
/// entry point, so its oversized return is NOT rejected here. It would fail later during codegen.
///
/// `zeroed()` produces the array without materializing a literal, so the test isolates the
/// return-type boundary from the separate interior-allocation gap.
#[test]
fn generic_unconstrained_entry_point_return_is_not_checked() {
    let src = r#"
    unconstrained fn mk_array<let N: u32>() -> [u64; N] {
        zeroed()
    }
    fn main() {
        // Safety: test
        let _a = unsafe { mk_array::<10>() };
        // Safety: test
        let _b = unsafe { mk_array::<4294967295>() };
    }
    "#;
    let result = get_monomorphized_with_stdlib(src, &[stdlib_src::ZEROED]);
    assert!(result.is_ok(), "generic Brillig entry point return is not rejected: {result:?}");
}

/// A non-entry Brillig function returning a huge array is an interior allocation, not a circuit
/// boundary, so it is NOT rejected by this check (it would fail later during Brillig codegen).
#[test]
fn non_entry_brillig_return_is_not_checked() {
    let src = r#"
    unconstrained fn helper() -> [u64; 4294967295] {
        zeroed()
    }
    unconstrained fn main() {
        let _ = helper();
    }
    "#;
    let result = get_monomorphized_with_stdlib(src, &[stdlib_src::ZEROED]);
    assert!(result.is_ok(), "non-entry Brillig return is not rejected: {result:?}");
}
