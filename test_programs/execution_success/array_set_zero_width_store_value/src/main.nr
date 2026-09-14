// Regression for https://github.com/noir-lang/noir-claude/issues/1733:
// assigning an empty array `[]` (a zero-flattened-width value) into a
// dynamically indexed tuple/struct element used to make ACIR generation
// initialize a memory block that no memory operation ever referenced, tripping
// the debug-only `assert_initialized_blocks_are_used` post-check. The zero-width
// member must appear in trailing position, so that its `array_set` is the last
// one in the read-modify-write chain lowered from the source.
fn main(x: Field, j: u32, k: u32, m: u32) -> pub Field {
    let cells: [(Field, [Field; 0]); 2] = [(x, []), (x + 1, [])];
    let mut d = cells;
    d[j].1 = [];
    let t = d[k].1;
    let mut e = cells;
    e[m].1 = t;
    e[m].0
}
