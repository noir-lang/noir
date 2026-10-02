// Exercises the `remove_redundant_range_checks` SSA pass (issue #9463): range checks and
// unsigned less-than constraints implied by an earlier, dominating fact are elided.
fn main(a: u64, i: u32) -> pub u32 {
    // The first assert dominates and implies the second (16 <= 17): the second
    // comparison is removed.
    assert(a < 16);
    assert(a < 17);
    // Control with the direction reversed: `a < 16` does not imply `a < 10`,
    // so this comparison must be kept (and is semantically active).
    assert(a < 10);
    // The out-of-bounds check on `t[i % 3]` is implied by the `mod`. A composite
    // element type is used so the explicit SSA-level check is emitted (simple array
    // reads rely on ACIR's implicit memory check instead).
    let t = [(i, 1), (i, 2), (i, 3)];
    let (x, y) = t[i % 3];
    x + y + (a as u32)
}
