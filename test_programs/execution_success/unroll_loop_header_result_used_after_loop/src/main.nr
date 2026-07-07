// Regression test: a loop header whose *instruction* result (here the `println`
// type-descriptor array produced while evaluating the `while` condition) is also
// referenced by a block outside the loop after CSE deduplicates the two identical
// descriptors. Unrolling must redirect the exit block's reference to the final
// unrolled copy of the header instruction, otherwise the value is left dangling.
unconstrained fn main() {
    let a: [u8; 3] = [1, 2, 3];
    let mut i: u32 = 0;
    while cond(a) {
        if i == 0 {
            break;
        }
        i += 1;
    }
    println(a);
}

unconstrained fn cond(a: [u8; 3]) -> bool {
    println(a);
    true
}
