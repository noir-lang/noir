// Regression test for https://github.com/noir-lang/noir-claude/issues/1068.
//
// `Point` declares its fields in non-alphabetical order (`y` before `x`). The
// match elaborator used to order a struct pattern's sub-patterns by field name
// while the branch variables follow declaration order, so each sub-pattern was
// tested against the wrong field. With two integer fields the arm is not dropped
// (the literals stay type-compatible), so there is no warning - it silently
// matches the wrong values. Runtime inputs keep the match from being
// constant-folded, exercising the generated decision tree.
struct Point {
    y: u8,
    x: u8,
}

fn main(a: u8, b: u8) {
    // a == 1, b == 2, so this is `Point { y: 1, x: 2 }`.
    let point = Point { y: a, x: b };
    let matched = match point {
        Point { y: 1, x: 2 } => true,
        _ => false,
    };
    // The arm matches only when `y == 1` and `x == 2`, which holds here.
    assert(matched);

    // Swapping the field values must not match the arm.
    let swapped_point = Point { y: b, x: a };
    let swapped = match swapped_point {
        Point { y: 1, x: 2 } => true,
        _ => false,
    };
    assert(!swapped);
}
