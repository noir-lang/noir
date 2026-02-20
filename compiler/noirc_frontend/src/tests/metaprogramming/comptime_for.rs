//! Tests for `comptime for` loops, including the `comptime for` syntax sugar
//! which is equivalent to `comptime { for .. }`.

use crate::tests::assert_no_errors;

#[test]
fn comptime_for_loop_in_block() {
    let src = r#"
        fn main() {
            comptime {
                let mut sum = 0;
                for i in 0..3 {
                    sum += i;
                }
                assert_eq(sum, 3);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_for_sugar_basic() {
    let src = r#"
        fn main() {
            comptime for _i in 0..4 {}
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_for_sugar_with_assertion() {
    let src = r#"
        fn main() {
            comptime for i in 0..3 {
                assert(i < 3);
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_for_sugar_equivalent_to_comptime_block() {
    // `comptime for` should behave identically to `comptime { for .. }`.
    // Run both forms and assert they produce the same result.
    let src = r#"
        comptime mut global SUM_SUGAR: u32 = 0;
        comptime mut global SUM_BLOCK: u32 = 0;

        fn main() {
            comptime for i in 0..4 {
                SUM_SUGAR += i;
            }
            comptime {
                for i in 0..4 {
                    SUM_BLOCK += i;
                }
            }
            comptime {
                assert_eq(SUM_SUGAR, SUM_BLOCK);
            }
        }
    "#;
    assert_no_errors(src);
}
