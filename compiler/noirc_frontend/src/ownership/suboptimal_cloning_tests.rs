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

/// Unconditional reassignment after a conditional break in a loop.
///
/// In this loop, `v` is unconditionally reassigned (`v = identity(v)`) on every
/// iteration that doesn't break. The `use_var(v)` after the reassignment always
/// sees a fresh value, so it should be moved, not cloned.
///
/// Suboptimal: `killed.clear()` on `break` is conservative — it wipes ALL pending
/// kills, including the kill from `v = identity(v)` which comes *after* the break
/// in forward order and thus always executes when `use_var(v)` is reached.
/// A more precise analysis would only clear kills for assignments that the
/// break/continue actually skips over.
#[test]
fn reassignment_after_break_in_loop() {
    let src = "
    unconstrained fn main(cond: bool) {
        let mut v = @[1, 2, 3];
        for _ in 0..5 {
            if cond { break; }
            v = identity(v);
            use_var(v);
        }
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    // use_var(v) inside the loop gets .clone() even though v is freshly assigned
    // on every iteration that reaches that point. The clone is safe to remove.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(cond$l0: bool) -> () {
        let mut v$l1 = @[1, 2, 3];
        for _$l2 in 0 .. 5 {
            if cond$l0 {
                break
            };
            v$l1 = identity$f1(v$l1);
            use_var$f2(v$l1.clone());
        };
        use_var$f2(v$l1);
    }
    unconstrained fn identity$f1(x$l3: [Field]) -> [Field] {
        x$l3
    }
    unconstrained fn use_var$f2(_x$l4: [Field]) -> () {
    }
    ");
}
