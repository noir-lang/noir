// Regression test for a constant folding bug where hoisting an instruction
// into a loop header and re-visiting it caused self-deduplication,
// orphaning result values.
//
// We have the following:
// - unconstrained (Brillig) function with a while loop
// - Nested if/else on the same boolean creating sibling blocks with
//   identical instructions (triggers hoisting to their common dominator)
// - A function call result used via array indexing (prevents simplification)
// - `a < ((b as u128) <= (b as u128))` produces an lt/cast chain that,
//   after simplification, creates a `not` matching the hoisted one
unconstrained fn main(b: Field) -> pub bool {
    let mut a = true;
    let mut running = true;
    while running {
        let _ = if a {
            foo()[(
                if a {
                    if a {
                        0
                    } else {
                        0
                    }
                } else {
                    if a {
                        0
                    } else {
                        0
                    }
                }
            )]
        } else {
            0
        };
        a = a < ((b as u128) <= (b as u128));
        running = false;
    }
    a
}

unconstrained fn foo() -> [u64; 1] {
    [0]
}
