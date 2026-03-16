#![cfg(test)]
//! Tests documenting cases where the ownership pass inserts clones that are
//! technically unnecessary. These serve as a record of known suboptimal
//! behavior so that future optimizations can be validated against them —
//! the snapshots here should improve (fewer clones) as the pass gets smarter.
//!
//! Each test includes a comment explaining why the clone is safe to remove.

use crate::test_utils::get_monomorphized;

/// Extracting all fields of a tuple sequentially. Each `t.N` accesses a
/// distinct field, so no aliasing occurs — clones on intermediate extractions
/// are unnecessary.
///
/// Suboptimal: `t.0` gets `.clone()` even though `t.0`, `t.1`, and `t.2` are
/// disjoint fields. The clone is safe to remove because no two extractions
/// alias the same memory.
#[test]
fn tuple_mixed_array_non_array_extraction() {
    let src = "
    unconstrained fn main() {
        let arr1 = [1, 2];
        let arr2 = [3, 4];
        let t = (arr1, 42, arr2);
        let _a = t.0;
        let _b = t.1;
        let _c = t.2;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // t$l2.0.clone() is suboptimal — each field is extracted exactly once
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let arr1$l0 = [1, 2];
        let arr2$l1 = [3, 4];
        let t$l2 = (arr1$l0, 42, arr2$l1);
        let _a$l3 = t$l2.0.clone();
        let _b$l4 = t$l2.1;
        let _c$l5 = t$l2.2
    }
    ");
}

/// Extracting all fields of a struct. Like tuples, each field access is
/// independent — no two accesses alias. Clones are not needed.
///
/// Suboptimal: `s.data` (field 0) gets `.clone()` even though all three struct
/// fields are extracted independently. Safe to remove because structs are
/// lowered to tuples and each field is a distinct slot.
#[test]
fn struct_field_extraction() {
    let src = "
    struct MyStruct {
        data: [Field; 3],
        flag: bool,
        other: [Field; 2],
    }

    unconstrained fn main() {
        let s = MyStruct { data: [1, 2, 3], flag: true, other: [4, 5] };
        let _d = s.data;
        let _f = s.flag;
        let _o = s.other;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // s$l3.0.clone() is suboptimal — each struct field is extracted exactly once
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let s$l3 = {
            let data$l0 = [1, 2, 3];
            let flag$l1 = true;
            let other$l2 = [4, 5];
            (data$l0, flag$l1, other$l2)
        };
        let _d$l4 = s$l3.0.clone();
        let _f$l5 = s$l3.1;
        let _o$l6 = s$l3.2
    }
    ");
}

/// Nested extraction with non-overlapping paths: `x.0.0` and `x.0.1` reach
/// into distinct sub-fields of `x.0`. No aliasing, so no clone is needed.
///
/// Suboptimal: `x.0.0` gets `.clone()` even though it and `x.0.1` are sibling
/// fields within `x.0`. Safe to remove because the two paths diverge at the
/// second index — they never refer to the same data.
#[test]
fn nested_tuple_extraction_disjoint_subfields() {
    let src = "
    unconstrained fn main() {
        let x = (([1], [2]), ([3], [4]));
        let _a = x.0.0;
        let _b = x.0.1;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // x$l0.0.0.clone() is suboptimal — x.0.0 and x.0.1 are disjoint sub-fields
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let x$l0 = (([1], [2]), ([3], [4]));
        let _a$l1 = x$l0.0.0.clone();
        let _b$l2 = x$l0.0.1
    }
    ");
}

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

/// Mixed whole-variable use and field extractions. `foo(tuple)` uses the whole
/// variable so its clone is correct. But `tuple.0.0` is disjoint from all other
/// extraction paths and shouldn't need a clone.
///
/// Suboptimal: `tuple.0.0` gets `.clone()` even though it doesn't overlap with
/// any other extraction path. `tuple.1.1` correctly gets `.clone()` because it
/// overlaps with `tuple.1.1.1`.
#[test]
fn mixed_whole_variable_and_field_extractions() {
    let src = "
    unconstrained fn main() {
        let tuple = (([1], [2]), ([3], ([4], [5])));
        foo(tuple);
        foo(tuple.0.0);
        foo(tuple.1.1);
        foo(tuple.1.1.1);
    }

    fn foo<T>(_: T) {}
    ";

    let program = get_monomorphized(src).unwrap();
    // tuple$l0.0.0.clone() is suboptimal — disjoint from all other extraction paths
    // tuple$l0.clone() and tuple$l0.1.1.clone() are correct
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let tuple$l0 = (([1], [2]), ([3], ([4], [5])));
        foo$f1(tuple$l0.clone());;
        foo$f2(tuple$l0.0.0.clone());;
        foo$f3(tuple$l0.1.1.clone());;
        foo$f2(tuple$l0.1.1.1);
    }
    unconstrained fn foo$f1(_$l1: (([Field; 1], [Field; 1]), ([Field; 1], ([Field; 1], [Field; 1])))) -> () {
    }
    unconstrained fn foo$f2(_$l2: [Field; 1]) -> () {
    }
    unconstrained fn foo$f3(_$l3: ([Field; 1], [Field; 1])) -> () {
    }
    ");
}

/// Only field extractions, no whole-variable use. `tuple.0.0` is disjoint from
/// the other paths. `tuple.1.1` and `tuple.1.1.1` overlap (one is a prefix of
/// the other), so `tuple.1.1` correctly needs a clone.
///
/// Suboptimal: `tuple.0.0` gets `.clone()` even though it's completely disjoint
/// from `tuple.1.1` and `tuple.1.1.1`. Safe to remove because the paths don't
/// alias.
#[test]
fn extract_only_with_overlapping_and_disjoint_paths() {
    let src = "
    unconstrained fn main() {
        let tuple = (([1], [2]), ([3], ([4], [5])));
        foo(tuple.0.0);
        foo(tuple.1.1);
        foo(tuple.1.1.1);
    }

    fn foo<T>(_: T) {}
    ";

    let program = get_monomorphized(src).unwrap();
    // tuple$l0.0.0.clone() is suboptimal — disjoint from tuple.1.1 and tuple.1.1.1
    // tuple$l0.1.1.clone() is correct — overlaps with tuple.1.1.1
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let tuple$l0 = (([1], [2]), ([3], ([4], [5])));
        foo$f1(tuple$l0.0.0.clone());;
        foo$f2(tuple$l0.1.1.clone());;
        foo$f1(tuple$l0.1.1.1);
    }
    unconstrained fn foo$f1(_$l1: [Field; 1]) -> () {
    }
    unconstrained fn foo$f2(_$l2: ([Field; 1], [Field; 1])) -> () {
    }
    ");
}

/// `x.0.0` and `x.1` access completely disjoint top-level fields.
/// No aliasing — clone is unnecessary.
///
/// Suboptimal: `x.0.0` gets `.clone()` even though `x.0.0` and `x.1` diverge
/// at the very first index. Safe to remove because accessing a nested field of
/// `x.0` cannot alias `x.1`.
#[test]
fn disjoint_nested_and_shallow_extraction() {
    let src = "
    unconstrained fn main() {
        let x = (([1], [2]), [3]);
        let _a = x.0.0;
        let _b = x.1;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // x$l0.0.0.clone() is suboptimal — x.0.0 and x.1 are completely disjoint
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let x$l0 = (([1], [2]), [3]);
        let _a$l1 = x$l0.0.0.clone();
        let _b$l2 = x$l0.1
    }
    ");
}
