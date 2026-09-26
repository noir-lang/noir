// Regression for https://github.com/noir-lang/noir-claude/issues/1513:
// two conditional writes to the same reference through a single pre-branch
// snapshot `old` used to compile to a circuit that returned `9` where the
// source computes `0`. `flatten_cfg::try_optimize_store_of_array_set`
// rebased the second write's per-element merge onto a fresh `Load` inserted
// after the first store had already landed, silently dropping the older
// store's semantics.
//
// Under `c && d`, both branches take their own `t = old` (the pre-branch
// snapshot); the second `*r = t` overwrites the first's write, leaving `*r`
// equal to `[old[0], 5]`. `(*r)[0]` is therefore `old[0] == 0`. The buggy
// circuit returned `9` — `t[0] = 9` from the first branch, incorrectly
// preserved through the second branch's per-element merge.
fn main(c: bool, d: bool, s: u32) -> pub Field {
    assert(s == 0);
    let mut a: [Field; 2] = [0, 0];
    let mut b: [Field; 2] = [7, 7];
    let refs: [&mut [Field; 2]; 2] = [&mut a, &mut b];
    let r = refs[s];
    let old = *r;
    if c {
        let mut t = old;
        t[0] = 9;
        *r = t;
    }
    if d {
        let mut t = old;
        t[1] = 5;
        *r = t;
    }
    (*r)[0]
}
