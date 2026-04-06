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
/// Suboptimal: both index results get `.clone()` because `handle_index` always
/// clones when the element type contains an array, regardless of whether the
/// collection has further uses. The second clone is unnecessary since `arr[1]`
/// is the last use of `arr`.
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
    // arr$l0[1].clone() is suboptimal — this is the last use of arr
    // arr$l0[0].clone() is arguably necessary since arr is used again,
    // but could be avoided if we knew the indexes don't alias (different constants)
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let arr$l0 = [[1, 2], [3, 4]];
        let _a$l1 = arr$l0[0].clone();
        let _b$l2 = arr$l0[1].clone()
    }
    ");
}

/// Nested array index: `arr[0][1]` on a 3D array. The outermost index result
/// gets cloned; the intermediate `arr[0]` does not because `handle_index`
/// processes the collection via `handle_reference_expression`.
///
/// Suboptimal: `handle_index` always clones when the element type contains an
/// array. Here `arr` has no further uses, so the clone is unnecessary.
#[test]
fn nested_array_double_index() {
    let src = "
    unconstrained fn main() {
        let arr = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];
        let _val = arr[0][1];
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // arr$l0[0][1].clone() is suboptimal — arr is not used again and the
    // intermediate arr[0] is a temporary
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let arr$l0 = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];
        let _val$l1 = arr$l0[0][1].clone()
    }
    ");
}
