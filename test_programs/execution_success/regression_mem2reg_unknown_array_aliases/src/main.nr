// Regression test for mem2reg incorrectly removing stores to references
// within arrays that have unknown combined aliases from block parameters.
// The inner func_1 call + if/else creates a block parameter with unknown
// aliases after inlining. The outer func_1's while loop then creates
// a pattern where a store to a reference inside a make_array with that
// block parameter is incorrectly removed.
unconstrained fn main() -> pub bool {
    let condition = *func_1([[&mut false], [&mut true]]);
    let row = if condition {
        [&mut false]
    } else {
        [&mut false]
    };
    *func_1([row, [&mut false]])
}

#[inline_always]
unconstrained fn func_1(mut b: [[&mut bool; 1]; 2]) -> &mut bool {
    let mut i: u32 = 0;
    while *b[1][0] {
        if i == 1 {
            break;
        }
        i += 1;
        let _j = b[0][0];
    }
    &mut true
}
