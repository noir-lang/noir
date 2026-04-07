#![cfg(test)]
//! Tests documenting cases where the ownership pass inserts clones that are
//! technically unnecessary. These serve as a record of known suboptimal
//! behavior so that future optimizations can be validated against them —
//! the snapshots here should improve (fewer clones) as the pass gets smarter.
//!
//! Each test includes a comment explaining why the clone is safe to remove.

use crate::test_utils::get_monomorphized;

/// Two disjoint indexes into a nested array. Each index accesses a different
/// element, so they don't alias.
///
/// Suboptimal: `arr[0]` gets `.clone()` because the last-use analysis treats
/// both `arr[0]` and `arr[1]` as uses of the whole variable `arr`. If the
/// analysis tracked that the constant indexes 0 and 1 are disjoint, the clone
/// could be avoided.
#[test]
fn nested_array_two_disjoint_indexes() {
    let src = "
    unconstrained fn main() {
        let arr = [[1, 2], [3, 4]];
        let _a = arr[0];
        let _b = arr[1];
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // arr$l0[0].clone() is arguably necessary since arr is used again,
    // but could be avoided if we knew the indexes don't alias (different constants).
    // arr$l0[1] is now correctly moved — this is the last use of arr.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let arr$l0 = [[1, 2], [3, 4]];
        let _a$l1 = arr$l0[0].clone();
        let _b$l2 = arr$l0[1]
    }
    ");
}
